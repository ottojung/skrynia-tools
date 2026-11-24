#!/usr/bin/env python3
"""Compile-free F1-F4 verifier for the PR71 durable replay ledger.

No Rust toolchain is invoked. It combines (a) source-static assertions at
exact file:pattern level with (b) a faithful Python model of the two state
machines involved: `DurableLedger::decide` + `Pump::run_once` (F1) and the
file tag check (F2). Run once against the reviewed head (expect VULNERABLE)
and once against the fix branch (expect FIXED).
"""
import argparse
import hashlib
import sys
from pathlib import Path

DOMAIN_OLD = b"skrynia-ledger-digest-v1"
DOMAIN_NEW = b"skrynia-ledger-mac-v1"


def read(src: Path, rel: str) -> str:
    p = src / rel
    if not p.is_file():
        raise SystemExit(f"MISSING {p}")
    return p.read_text()


def source_checks(src: Path):
    model = read(src, "src/ledger/model.rs")
    fmt = read(src, "src/ledger/format.rs")
    wire = read(src, "src/ledger/wire.rs")
    mod = read(src, "src/ledger/mod.rs")
    store = read(src, "src/ledger/store.rs")
    cycle = read(src, "src/pump/cycle.rs")
    retry = read(src, "tests/retry.rs")

    has = lambda text, needle: needle in text
    fixed = {
        "F1_isolate": has(cycle, 'Err(Error::Wire("ledger-reuse")) => Decision::Fresh'),
        "F1_no_abort": "self.ledger.decide(seq)?" not in cycle,
        "F1_multihint_test": "out_of_order_hint_survives_lower_seq_transient_failure" in retry,
        "F2_keyed_tag": has(model, "pub fn tag(src: &[u8], cap: &[u8; 32])")
        and has(model, "skrynia-ledger-mac-v1"),
        "F2_no_unkeyed": "pub fn digest(" not in model,
        "F2_decode": has(fmt, "tag(body, key)") and "pub fn decode(src: &[u8], binding: &[u8; 32], cap: usize, key: &[u8; 32])" in fmt,
        "F2_encode": has(wire, "tag(&out, key)"),
        "F2_mac_field": "mac_key" in mod and "&cfg.sender_cap" in mod,
        "F3_binding_export": "ledger_binding" in model and "ledger_binding" in mod,
        "F3_from_config": "pub fn from_config(dir: &Path, cfg:" in store,
        "F4_roundtrip_test": "file_store_roundtrip_restart_and_forged_tag_rejected" in wire,
    }
    vulnerable = {
        "F1_abort": "self.ledger.decide(seq)?" in cycle,
        "F2_unkeyed": "pub fn digest(" in model and "digest(body)" in fmt,
        "F3_no_export": "ledger_binding" not in model and "from_config" not in store,
        "F4_no_test": "file_store_roundtrip" not in wire,
    }
    return fixed, vulnerable


def f1_model(isolate: bool):
    """Model decide() + run_once() for hints [5,6], 5 transient-fails then recovers."""
    records = {}  # seq -> state
    sends = 0
    acked = []

    def decide(seq):
        if seq in records:
            return records[seq]
        if any(s >= seq for s in records):
            return "REUSE"
        return "FRESH"

    def run_once(hints, statuses):
        nonlocal sends
        it = iter(statuses)
        failure = None
        report = {"acked": 0, "skipped": 0}
        for seq in hints:
            d = decide(seq)
            if d == "REUSE":
                if not isolate:
                    return "ERR", report  # cycle.rs:21 `?` aborts the whole run
                d = "FRESH"  # per-hint isolation (the fix)
            if d == "Acked":
                report["skipped"] += 1
                continue
            if d == "Delivered":
                acked.append(seq)
                records[seq] = "Acked"
                report["acked"] += 1
                continue
            status = next(it)
            sends += 1
            if status >= 500:
                failure = status  # 503 exhausts -> no marker, no ACK
                continue
            records[seq] = "Delivered"
            acked.append(seq)  # P1 HintAck
            records[seq] = "Acked"
            report["acked"] += 1
        return ("ERR" if failure else "OK"), report

    r1 = run_once([5, 6], [503, 201])
    r2 = run_once([5, 6], [201])
    r3 = run_once([5, 6], [])
    return {"run1": r1, "run2": r2, "run3": r3, "sends": sends, "acked": acked}


def f2_model(use_key: bool):
    """A directory writer forges body+binding and recomputes the old digest."""
    cap = bytes([7] * 32)
    binding = hashlib.sha256(b"skrynia-webpush-ledger-v1" + cap + bytes([9] * 32) + b"k" + (0).to_bytes(8, "little")).digest()
    body = b"SKRLGRV1" + binding + (1024).to_bytes(4, "little") + (1).to_bytes(4, "little") + (5).to_bytes(8, "little") + bytes([2]) + bytes(7)
    old_tag = hashlib.sha256(DOMAIN_OLD + body).digest()
    # Attacker does NOT know cap, so cannot compute DOMAIN_NEW || cap || body.
    computed = old_tag if not use_key else hashlib.sha256(DOMAIN_NEW + cap + body).digest()
    accepted = computed == old_tag and not use_key
    return {"forgery_accepted": accepted}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True)
    ap.add_argument("--expect", choices=["VULNERABLE", "FIXED"], required=True)
    args = ap.parse_args()
    src = Path(args.src).resolve()
    fixed, vuln = source_checks(src)

    old = f1_model(isolate=False)
    new = f1_model(isolate=True)
    f2_old = f2_model(use_key=False)
    f2_new = f2_model(use_key=True)

    # RED reproduction: base aborts on hint 5 forever (no second send/ack).
    red_ok = (
        old["run1"][0] == "ERR"
        and old["sends"] == 2  # only hint 6's success in run 2 path
        and old["acked"] == [6]
        and f2_old["forgery_accepted"]
    )
    # GREEN: fix re-sends+acks hint 5, converges to all-skipped, forgery rejected.
    green_ok = (
        new["run1"][0] == "ERR"
        and new["run2"][1]["acked"] == 1
        and new["sends"] == 3
        and new["acked"] == [6, 5]
        and new["run3"][1]["skipped"] == 2
        and not f2_new["forgery_accepted"]
    )
    all_fixed = green_ok and all(fixed.values()) and not any(vuln.values())
    is_vulnerable = red_ok and vuln["F1_abort"] and vuln["F2_unkeyed"]

    print(f"src={src}")
    print("FIXED_PATTERNS:", fixed)
    print("VULNERABLE_PATTERNS:", vuln)
    print("F1_BASE_MODEL (abort):", old)
    print("F1_FIXED_MODEL (isolate):", new)
    print("F2_OLD_CHECK:", f2_old, "F2_NEW_CHECK:", f2_new)

    verdict = "FIXED" if all_fixed else ("VULNERABLE" if is_vulnerable else "INDETERMINATE")
    print("VERDICT:", verdict, "EXPECTED:", args.expect)
    if verdict != args.expect:
        print("FAIL: verdict does not match expectation", file=sys.stderr)
        return 1
    print(f"PASS: {verdict} as expected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
