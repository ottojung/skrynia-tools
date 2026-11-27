#!/usr/bin/env python3
"""HISTORICAL model of the retired publication-root CAS staging arena.

This modelled the pre-3.1 `stage_claim.rs` protocol: one `cas1` occupancy byte
per 8 KiB slot (0 -> 1 claim, 1 -> 0 release), no modulo counter, no shrink.
ABI 3.1 replaced that arena with private linear-memory buffers, so this model
is retained only as historical evidence and is NOT an acceptance gate.
"""
from __future__ import annotations

import concurrent.futures
import threading

SLOTS = 256
ERR_IO = -5
ERR_BOUNDS = -7


class Ram:
    """One unpartitioned shared RAM; occupancy bytes live at offsets 0..SLOTS."""

    def __init__(self, sized: bool = True) -> None:
        self.bytes = bytearray(SLOTS) if sized else bytearray()
        self.lock = threading.Lock()
        self.shrinks = 0
        self.grows = 0
        self.staged: dict[int, bytes] = {}

    def cas1(self, off: int, exp: int, des: int) -> int:
        with self.lock:
            if off >= len(self.bytes):
                return ERR_BOUNDS
            seen = self.bytes[off]
            if seen == exp:
                self.bytes[off] = des
            return seen

    def truncate(self, size: int) -> int:
        with self.lock:
            if size < len(self.bytes):
                self.shrinks += 1
                return ERR_IO
            if size > len(self.bytes):
                self.grows += 1
                self.bytes.extend(bytes(size - len(self.bytes)))
            return 0


def ready(ram: Ram) -> int:
    """Claim a free slot; size RAM only when it is still empty, never shrink."""
    if ram.cas1(0, 0, 0) == ERR_BOUNDS:
        if ram.truncate(SLOTS) != 0:
            raise RuntimeError("truncate_failed")
        if ram.cas1(0, 0, 0) == ERR_BOUNDS:
            raise RuntimeError("still_uninitialized")
    for byte in range(SLOTS):
        if ram.cas1(byte, 0, 1) == 0:
            return byte
    raise RuntimeError(ERR_IO)


def release(ram: Ram, slot: int) -> None:
    """Release on every ordinary return; a peer cannot release this byte."""
    if ram.cas1(slot, 1, 0) != 1:
        raise RuntimeError("release_of_free_slot")


def stage(ram: Ram, slot: int, payload: bytes) -> None:
    """Each claimant stages only into its own disjoint 8 KiB slot."""
    with ram.lock:
        ram.staged[slot] = payload


def check(name: str, ok: bool, detail: object = "") -> bool:
    print(("PASS" if ok else "FAIL"), name, detail, flush=True)
    return ok


def overlap_with_one_active(total: int) -> bool:
    ram = Ram()
    held = ready(ram)
    stage(ram, held, b"ACTIVE-UNTOUCHED")
    seen: list[int] = []
    errors: list[str] = []

    def cycle(index: int) -> None:
        try:
            slot = ready(ram)
            stage(ram, slot, f"REQ-{index}".encode())
            release(ram, slot)
            seen.append(slot)
        except RuntimeError as exc:
            errors.append(str(exc))

    with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
        list(pool.map(cycle, range(total)))
    ok = check(
        f"overlap_{total}_invocations_one_active",
        total > SLOTS and not errors and len(seen) == total and held not in seen,
        f"held={held} reused={held in seen} errors={errors[:4]}",
    )
    ok &= check("active_slot_byte_still_claimed", ram.bytes[held] == 1, ram.bytes[held])
    ok &= check("active_staged_bytes_intact", ram.staged[held] == b"ACTIVE-UNTOUCHED")
    release(ram, held)
    ok &= check("release_returns_slot_to_free",
                ram.cas1(held, 0, 0) == 0 and ram.cas1(held, 1, 0) == 0)
    reused = ready(ram)
    ok &= check("freed_slot_can_be_reclaimed", reused == held, (reused, held))
    release(ram, reused)
    return ok


def capacity_fail_closed() -> bool:
    ram = Ram()
    slots = [ready(ram) for _ in range(SLOTS)]
    ok = check("256_distinct_concurrent_slots", len(set(slots)) == SLOTS, len(set(slots)))
    try:
        ready(ram)
        ok &= check("257th_claim_fails_closed", False, "slot reused while active")
    except RuntimeError as exc:
        ok &= check("257th_claim_fails_closed", str(exc) == str(ERR_IO), str(exc))
    for slot in slots:
        release(ram, slot)
    ok &= check("all_bytes_free_after_release", all(b == 0 for b in ram.bytes))
    return ok


def no_shrink() -> bool:
    empty = Ram(sized=False)
    slot = ready(empty)
    ok = check("empty_ram_grown_once", empty.grows == 1 and empty.shrinks == 0,
               (empty.grows, empty.shrinks))
    release(empty, slot)
    for _ in range(SLOTS * 2):
        s = ready(empty)
        release(empty, s)
    ok &= check("no_shrink_on_any_invocation", empty.shrinks == 0, empty.shrinks)
    return ok


def main() -> int:
    ok = overlap_with_one_active(300)
    ok &= capacity_fail_closed()
    ok &= no_shrink()
    print("ALL_OK=" + str(bool(ok)), flush=True)
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
