#!/usr/bin/env python3
"""Inventory-driven differential parity harness for pinned Skrynia v2 baseline.

Modes:
  --self-test                 Harness unit checks (no network/server).
  --inventory PATH            Validate/load inventory and print generated plan.
  --generate-plan             Print generated case plan (with --inventory).
  --baseline-self-consistency Run generated scenarios against two fresh disposable
                              v2 instances of the same pinned build and compare
                              normalized observations.
  --baseline-candidate        Compare baseline vs candidate base URLs for selected
                              read-only public probes (or extended cases when safe).

Safety: never mutates production data. Disposable runs use temporary data dirs and
localhost ephemeral ports. This is a test harness script, not a product CLI.
"""
from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import random
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
PUBLIC_SAFE_IDS = {"health", "version", "push_vapid", "store_get", "store_head"}


@dataclass
class Response:
    status: int
    headers: dict[str, str]
    body: bytes
    error: str | None = None

    @property
    def content_type(self) -> str:
        return self.headers.get("content-type", "")

    def json_body(self) -> Any:
        return json.loads(self.body.decode("utf-8"))

    def text(self) -> str:
        return self.body.decode("utf-8", errors="replace")


@dataclass
class Observation:
    case_id: str
    status: int
    headers: dict[str, str]
    body: Any
    note: str = ""
    error: str | None = None


@dataclass
class PlanCase:
    endpoint_id: str
    case_id: str
    kind: str
    description: str
    requires: list[str] = field(default_factory=list)
    runnable_without_v3: bool = False
    runnable_baseline_self: bool = False


def _norm_headers(headers: Any) -> dict[str, str]:
    out: dict[str, str] = {}
    for k, v in headers.items():
        out[str(k).lower()] = str(v)
    return out


def load_inventory(path: str | Path) -> dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if data.get("schema") != "skrynia-tools.v2-compat-inventory":
        raise ValueError(f"unexpected schema: {data.get('schema')!r}")
    if not isinstance(data.get("endpoints"), list):
        raise ValueError("inventory missing endpoints list")
    return data


def endpoint_map(inv: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {e["id"]: e for e in inv["endpoints"] if isinstance(e, dict) and "id" in e}


def do_request(base: str, method: str, path: str, headers: dict[str, str] | None = None,
               body: bytes | None = None, timeout: float = 15.0) -> Response:
    url = base.rstrip("/") + path
    req = urllib.request.Request(url, data=body, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return Response(resp.status, _norm_headers(resp.headers), resp.read())
    except urllib.error.HTTPError as e:
        raw = e.read() if hasattr(e, "read") else b""
        return Response(int(e.code), _norm_headers(getattr(e, "headers", {}) or {}), raw)
    except Exception as e:  # noqa: BLE001
        return Response(0, {}, b"", error=f"{type(e).__name__}: {e}")


def body_cmp_value(body: bytes, content_type: str) -> Any:
    if "json" in content_type:
        try:
            return json.loads(body.decode("utf-8"))
        except Exception:
            return {"_json_parse_error": body.decode("utf-8", errors="replace")}
    try:
        return body.decode("utf-8")
    except Exception:
        return {"_bytes_hex": body.hex()}


def compare_observations(a: Observation, b: Observation, header_keys: list[str] | None = None) -> tuple[bool, str]:
    if a.error or b.error:
        ok = a.error == b.error and a.status == b.status and a.body == b.body
        return ok, f"transport/error mismatch {a.error!r} vs {b.error!r}"
    diffs: list[str] = []
    if a.status != b.status:
        diffs.append(f"status {a.status}!={b.status}")
    for hk in (header_keys or []):
        k = hk.lower()
        av, bv = a.headers.get(k), b.headers.get(k)
        if av != bv:
            diffs.append(f"header {k} {av!r}!={bv!r}")
    if a.body != b.body:
        diffs.append("body differs")
    if diffs:
        return False, "; ".join(diffs)
    return True, "match"


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def quote_etag(hexhash: str) -> str:
    return f'"{hexhash}"'


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


@dataclass
class DisposableV2:
    binary: Path
    data_dir: Path
    port: int
    proc: subprocess.Popen[bytes] | None = None
    base_url: str = ""
    root_id: str = ""
    root_issuer: str = ""

    @classmethod
    def start(cls, binary: str | Path, tmp_root: str | Path, label: str) -> "DisposableV2":
        binary = Path(binary).resolve()
        data_dir = Path(tmp_root) / f"data-{label}-{random.randint(1000,9999)}"
        data_dir.mkdir(parents=True, exist_ok=False)
        port = free_port()
        env = os.environ.copy()
        env["SKRYNIA_DATA_DIR"] = str(data_dir)
        env["SKRYNIA_HOST"] = "127.0.0.1"
        env["SKRYNIA_PORT"] = str(port)
        # keep defaults; avoid push delivery side effects in isolated tests
        env.setdefault("SKRYNIA_URL", f"http://127.0.0.1:{port}")
        proc = subprocess.Popen(
            [str(binary)],
            cwd=str(data_dir),
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        inst = cls(binary=binary, data_dir=data_dir, port=port, proc=proc,
                   base_url=f"http://127.0.0.1:{port}")
        inst._wait_ready()
        inst._load_root(env)
        return inst

    def _wait_ready(self, timeout: float = 20.0) -> None:
        deadline = time.time() + timeout
        last_err = None
        while time.time() < deadline:
            if self.proc and self.proc.poll() is not None:
                out = b""
                try:
                    out = self.proc.stdout.read() if self.proc.stdout else b""
                except Exception:
                    pass
                raise RuntimeError(f"server exited early rc={self.proc.returncode} out={out[:400]!r}")
            r = do_request(self.base_url, "GET", "/version", timeout=1.0)
            if r.status == 200 and r.error is None:
                return
            last_err = r.error or r.status
            time.sleep(0.1)
        raise RuntimeError(f"server not ready: {last_err}")

    def _load_root(self, env: dict[str, str]) -> None:
        # Prefer local operator subcommand on same data dir (authorized host context).
        cp = subprocess.run(
            [str(self.binary), "root-credentials"],
            cwd=str(self.data_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=10,
        )
        if cp.returncode != 0:
            raise RuntimeError(f"root-credentials failed rc={cp.returncode} err={cp.stderr[:300]!r}")
        creds = json.loads(cp.stdout.strip())
        self.root_id = creds["id"]
        self.root_issuer = creds["issuerToken"]

    def stop(self) -> None:
        if not self.proc:
            return
        self.proc.terminate()
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait(timeout=5)

    def request(self, method: str, path: str, **kw: Any) -> Response:
        return do_request(self.base_url, method, path, **kw)


def auth_headers(token: str) -> dict[str, str]:
    return {"Authorization": f"Bearer {token}"}


def issue_user(server: DisposableV2, grant_id: str, issuer_token: str) -> tuple[str, str]:
    r = server.request("POST", f"/grants/{grant_id}/users", headers=auth_headers(issuer_token))
    if r.status != 200:
        raise RuntimeError(f"issue_user status={r.status} body={r.text()[:200]}")
    j = r.json_body()
    return j["userId"], j["userToken"]


def create_child_grant(server: DisposableV2, parent_user_token: str, locations: str,
                       operations: list[str], max_bytes: int, max_objects: int) -> tuple[str, str]:
    body = json.dumps({
        "locations": locations,
        "operations": operations,
        "maxBytes": max_bytes,
        "maxObjects": max_objects,
    }).encode("utf-8")
    r = server.request("POST", "/grants", headers={**auth_headers(parent_user_token), "Content-Type": "application/json"}, body=body)
    if r.status != 200:
        raise RuntimeError(f"create_child_grant status={r.status} body={r.text()[:300]}")
    j = r.json_body()
    return j["id"], j["issuerToken"]


def normalize_obs(case_id: str, r: Response, *, keep_headers: list[str] | None = None,
                  normalize_json_paths: set[str] | None = None) -> Observation:
    if r.error:
        return Observation(case_id, r.status, {}, None, error=r.error)
    hdrs = {}
    for k in (keep_headers or []):
        lk = k.lower()
        if lk in r.headers:
            hdrs[lk] = r.headers[lk]
    body: Any
    ct = r.content_type
    if "json" in ct:
        try:
            body = r.json_body()
        except Exception:
            body = {"_raw": r.text()}
        body = normalize_json_value(body, normalize_json_paths or set(), path="$")
    elif r.body == b"":
        body = ""
    else:
        try:
            body = r.body.decode("utf-8")
        except Exception:
            body = {"_bytes_hex": r.body.hex()}
    return Observation(case_id, r.status, hdrs, body)


def normalize_json_value(val: Any, dyn_paths: set[str], path: str) -> Any:
    if path in dyn_paths:
        if isinstance(val, str):
            if HEX64_RE.match(val):
                return "<hex64>"
            return "<str>"
        return "<dynamic>"
    if isinstance(val, dict):
        return {k: normalize_json_value(v, dyn_paths, f"{path}.{k}") for k, v in sorted(val.items())}
    if isinstance(val, list):
        return [normalize_json_value(v, dyn_paths, f"{path}[]") for v in val]
    return val


# ---------- case plan generation ----------

def generate_plan(inv: dict[str, Any]) -> list[PlanCase]:
    em = endpoint_map(inv)
    plan: list[PlanCase] = []
    def add(eid: str, case_id: str, kind: str, desc: str, req: list[str] | None = None,
            no_v3: bool = True, self_ok: bool = True) -> None:
        plan.append(PlanCase(eid, case_id, kind, desc, req or [], no_v3, self_ok))
    # public/meta
    for eid in ["health", "version", "push_vapid"]:
        if eid in em:
            add(eid, f"{eid}.get.success", "success", f"GET {em[eid]['path']} returns documented success status")
    # store reads/conditionals/ranges
    if "store_get" in em:
        add("store_get", "store_get.missing.404", "negative", "GET missing object returns 404")
        add("store_get", "store_get.present.200", "success", "GET present object returns 200, body, etag/cache headers")
        add("store_get", "store_get.if_none_match.304", "conditional", "GET with matching If-None-Match returns 304")
        add("store_get", "store_get.range.206", "range", "GET with satisfiable single range returns 206 + Content-Range")
        add("store_get", "store_get.range.unsat.416", "range-negative", "GET with unsatisfiable range returns 416")
        add("store_get", "store_get.if_range.stale.full", "conditional", "GET with stale If-Range returns full object")
    if "store_head" in em:
        add("store_head", "store_head.missing.404", "negative", "HEAD missing object returns 404")
        add("store_head", "store_head.present.200", "success", "HEAD present object returns metadata without body")
        add("store_head", "store_head.if_none_match.304", "conditional", "HEAD with matching If-None-Match returns 304")
    # store mutations
    if "store_create" in em:
        add("store_create", "store_create.success.201", "success", "POST create absent object returns 201")
        add("store_create", "store_create.collision.409", "negative", "POST create existing object returns 409")
        add("store_create", "store_create.forbidden_precondition.400", "negative", "POST with forbidden precondition header returns 400 invalid_precondition")
        add("store_create", "store_create.path_conflict.409", "negative", "POST causing file/dir path conflict returns 409 path_conflict")
        add("store_create", "store_create.unauthorized.401_or_403", "auth", "POST without/wrong auth denied")
    if "store_replace" in em:
        add("store_replace", "store_replace.success.200", "success", "PUT existing object returns 200")
        add("store_replace", "store_replace.absent.404", "negative", "PUT absent object returns 404")
        add("store_replace", "store_replace.cas.match.200", "conditional", "PUT with matching X-Skrynia-If-SHA256 returns 200")
        add("store_replace", "store_replace.cas.mismatch.412", "negative", "PUT with mismatching X-Skrynia-If-SHA256 returns 412 content_mismatch")
        add("store_replace", "store_replace.http_conditional.400", "negative", "PUT with forbidden HTTP conditional header returns 400")
    if "store_delete" in em:
        add("store_delete", "store_delete.success.200", "success", "DELETE existing object returns 200")
        add("store_delete", "store_delete.absent.404", "negative", "DELETE absent object returns 404")
        add("store_delete", "store_delete.cas.mismatch.412", "negative", "DELETE with mismatching X-Skrynia-If-SHA256 returns 412")
    # listing
    if "store_list" in em:
        add("store_list", "store_list.success.200", "success", "GET /store?locations=... with list auth returns keys")
        add("store_list", "store_list.pagination", "success", "Listing pagination with limit/after and next cursor")
        add("store_list", "store_list.unauthorized", "auth", "Listing without token or missing list op denied")
        add("store_list", "store_list.invalid_matcher.400", "negative", "Invalid/missing/duplicated matcher rejected")
    # grants
    for eid, cid, kind, desc in [
        ("grant_create", "grant_create.success.200", "grant", "POST /grants creates child grant and returns issuer token"),
        ("grant_create", "grant_create.narrowing.403", "grant-negative", "Child grant cannot widen scope/ops/quotas"),
        ("grant_issue_user", "grant_issue_user.success.200", "grant", "POST /grants/{id}/users issues user token"),
        ("grant_get", "grant_get.success.200", "grant", "GET /grants/{id} with exact user token returns grant/usage"),
        ("grant_revoke_user", "grant_revoke_user.success.200", "grant", "DELETE user token revokes it; subsequent use denied"),
        ("grant_revoke_user", "grant_revoke_user.absent.404", "grant-negative", "DELETE absent user returns 404 user_not_found"),
        ("grant_revoke", "grant_revoke.success.200", "grant", "DELETE grant revokes subtree; objects preserved"),
        ("grant_revoke", "grant_revoke.absent.404", "grant-negative", "DELETE absent grant returns 404 grant_not_found"),
    ]:
        if eid in em:
            add(eid, cid, kind, desc)
    # quota
    add("quota", "quota.objects.exceeded", "quota", "Object quota exhaustion denies mutation with quota error", req=["grant_create", "store_create"])
    add("quota", "quota.bytes.exceeded", "quota", "Byte quota exhaustion denies mutation", req=["grant_create", "store_create"])
    # push
    for eid, cid, kind, desc in [
        ("push_subscribe", "push_subscribe.invalid.400", "push-negative", "Invalid push subscription body rejected"),
        ("push_subscribe", "push_subscribe.auth.403", "push-auth", "Subscribe without capability/scope denied"),
        ("push_update", "push_update.invalid_cap.403_or_404", "push-negative", "Update with wrong/missing capability denied"),
        ("push_delete", "push_delete.invalid_cap.403_or_404", "push-negative", "Delete with wrong/missing capability denied"),
    ]:
        if eid in em:
            add(eid, cid, kind, desc)
    # mark non-self-runnable items that need special external deps/behavior
    for c in plan:
        if c.endpoint_id.startswith("push_subscribe") and "invalid.400" not in c.case_id:
            c.runnable_baseline_self = False
            c.note = "requires HTTPS push endpoint/network semantics; partial validation only"
        if c.kind in ("quota",) and "exceeded" in c.case_id:
            c.runnable_baseline_self = True  # we can construct tiny quotas
    # full successful push subscribe needs valid HTTPS endpoint acceptance semantics and may need VAPID/keys behavior; treat carefully
    for c in plan:
        if c.endpoint_id == "push_subscribe" and c.kind == "push-negative":
            c.runnable_baseline_self = True
        if c.endpoint_id in ("push_update", "push_delete"):
            c.runnable_baseline_self = True  # can test invalid capability paths without real subscription if API allows, else reported
    return plan


# ---------- executable baseline scenarios ----------

def run_public_meta(server: DisposableV2) -> list[Observation]:
    obs = []
    for name, path in [("health", "/health"), ("version", "/version"), ("push_vapid", "/push/vapid")]:
        r = server.request("GET", path)
        dyn: set[str] = set()
        keep = ["content-type"]
        if name == "push_vapid":
            # VAPID key material is instance-generated; compare structure/status.
            dyn = {"$.publicKey", "$.public_key", "$.key"}
        o = normalize_obs(f"{name}.get.success", r, keep_headers=keep,
                          normalize_json_paths=dyn)
        obs.append(o)
    return obs


def run_store_scenarios(server: DisposableV2) -> list[Observation]:
    obs: list[Observation] = []
    # bootstrap user token with broad permissions for test prefix
    _, root_user = issue_user(server, server.root_id, server.root_issuer)
    test_loc = "parity/*"
    child_id, child_issuer = create_child_grant(server, root_user, test_loc,
                                               ["create", "modify", "delete", "list", "delegate", "subscribe"],
                                               10_000_000, 1000)
    _, user = issue_user(server, child_id, child_issuer)
    h_user = auth_headers(user)

    missing = "/store/parity/missing-object"
    r = server.request("GET", missing)
    obs.append(normalize_obs("store_get.missing.404", r, keep_headers=["content-type"]))
    r = server.request("HEAD", missing)
    obs.append(normalize_obs("store_head.missing.404", r))

    key = "/store/parity/hello.txt"
    body1 = b"hello parity\n"
    r = server.request("POST", key, headers=h_user, body=body1)
    obs.append(normalize_obs("store_create.success.201", r, keep_headers=[]))
    # collision
    r = server.request("POST", key, headers=h_user, body=b"other")
    obs.append(normalize_obs("store_create.collision.409", r, keep_headers=["content-type"]))
    # forbidden precondition
    r = server.request("POST", "/store/parity/forbidden-pre.txt", headers={**h_user, "X-Skrynia-If-SHA256": sha256_hex(b"x")}, body=b"x")
    obs.append(normalize_obs("store_create.forbidden_precondition.400", r, keep_headers=["content-type"]))
    # path conflict: create parent file-like path then child
    r = server.request("POST", "/store/parity/pathnode", headers=h_user, body=b"file")
    obs.append(Observation("store_create.path_conflict.setup", r.status, {}, r.text()[:120]))
    r = server.request("POST", "/store/parity/pathnode/child", headers=h_user, body=b"child")
    obs.append(normalize_obs("store_create.path_conflict.409", r, keep_headers=["content-type"]))
    # unauthorized create (no token)
    r = server.request("POST", "/store/parity/noauth.txt", body=b"x")
    obs.append(normalize_obs("store_create.unauthorized", r, keep_headers=["content-type"]))

    # GET present / conditionals / range
    r = server.request("GET", key)
    etag = r.headers.get("etag", "")
    obs.append(normalize_obs("store_get.present.200", r, keep_headers=["content-type", "cache-control", "x-content-type-options", "etag"],
                             normalize_json_paths=set()))
    r = server.request("GET", key, headers={"If-None-Match": etag})
    obs.append(normalize_obs("store_get.if_none_match.304", r, keep_headers=["etag"]))
    r = server.request("GET", key, headers={"Range": "bytes=0-4"})
    obs.append(normalize_obs("store_get.range.206", r, keep_headers=["content-range", "content-type"]))
    r = server.request("GET", key, headers={"Range": "bytes=999-1000"})
    obs.append(normalize_obs("store_get.range.unsat.416", r, keep_headers=["content-range", "content-type"]))
    # stale If-Range should return full object (200) per inventory/spec
    r = server.request("GET", key, headers={"Range": "bytes=0-4", "If-Range": '"0000"'})
    obs.append(normalize_obs("store_get.if_range.stale.full", r, keep_headers=["content-range", "content-type"]))

    # HEAD
    r = server.request("HEAD", key)
    obs.append(Observation("store_head.present.200", r.status, {k: r.headers.get(k, "") for k in ["content-type", "cache-control", "x-content-type-options", "etag"] if k in r.headers}, "" if r.body == b"" else r.text()))
    r = server.request("HEAD", key, headers={"If-None-Match": etag})
    obs.append(normalize_obs("store_head.if_none_match.304", r, keep_headers=["etag"]))

    # replace
    body2 = b"hello parity v2\n"
    r = server.request("PUT", "/store/parity/absent-repl", headers=h_user, body=body2)
    obs.append(normalize_obs("store_replace.absent.404", r, keep_headers=["content-type"]))
    r = server.request("PUT", key, headers=h_user, body=body2)
    obs.append(normalize_obs("store_replace.success.200", r, keep_headers=[]))
    # CAS match/mismatch (need current hash)
    rget = server.request("GET", key)
    cur_hash = sha256_hex(rget.body)
    r = server.request("PUT", key, headers={**h_user, "X-Skrynia-If-SHA256": cur_hash}, body=b"v3")
    obs.append(normalize_obs("store_replace.cas.match.200", r, keep_headers=[]))
    r = server.request("PUT", key, headers={**h_user, "X-Skrynia-If-SHA256": "0" * 64}, body=b"v4")
    obs.append(normalize_obs("store_replace.cas.mismatch.412", r, keep_headers=["content-type"]))
    r = server.request("PUT", key, headers={**h_user, "If-Match": "*"}, body=b"v5")
    obs.append(normalize_obs("store_replace.http_conditional.400", r, keep_headers=["content-type"]))

    # delete
    r = server.request("DELETE", "/store/parity/absent-del", headers=h_user)
    obs.append(normalize_obs("store_delete.absent.404", r, keep_headers=["content-type"]))
    # create temp for delete cases
    r = server.request("POST", "/store/parity/todel.txt", headers=h_user, body=b"bye")
    dh = sha256_hex(b"bye")
    r = server.request("DELETE", "/store/parity/todel.txt", headers={**h_user, "X-Skrynia-If-SHA256": "f" * 64})
    obs.append(normalize_obs("store_delete.cas.mismatch.412", r, keep_headers=["content-type"]))
    r = server.request("DELETE", "/store/parity/todel.txt", headers={**h_user, "X-Skrynia-If-SHA256": dh})
    obs.append(normalize_obs("store_delete.success.200", r, keep_headers=[]))

    # listing
    r = server.request("GET", "/store?locations=parity/*", headers=h_user)
    o = normalize_obs("store_list.success.200", r, keep_headers=["content-type"])
    # keys contain dynamic set? compare sorted keys content is fine if deterministic sequence; normalize not needed except ensure sorted
    obs.append(o)
    # pagination: create a few keys
    for i in range(3):
        server.request("POST", f"/store/parity/p/{i}.txt", headers=h_user, body=f"{i}".encode())
    r = server.request("GET", "/store?locations=parity/p/*&limit=1", headers=h_user)
    obs.append(normalize_obs("store_list.pagination.page1", r, keep_headers=["content-type"]))
    try:
        j = r.json_body()
        after = j.get("next")
        if after:
            r2 = server.request("GET", f"/store?locations=parity/p/*&limit=1&after={urllib.parse.quote(after, safe='')}", headers=h_user)
            obs.append(normalize_obs("store_list.pagination.page2", r2, keep_headers=["content-type"]))
    except Exception as e:
        obs.append(Observation("store_list.pagination.page2", 0, {}, None, error=str(e)))
    # unauthorized listing
    r = server.request("GET", "/store?locations=parity/*")
    obs.append(normalize_obs("store_list.unauthorized", r, keep_headers=["content-type"]))
    # invalid matcher
    r = server.request("GET", "/store", headers=h_user)
    obs.append(normalize_obs("store_list.invalid_matcher.400", r, keep_headers=["content-type"]))

    return obs

def run_grant_scenarios(server: DisposableV2) -> list[Observation]:
    obs: list[Observation] = []
    _, root_user = issue_user(server, server.root_id, server.root_issuer)
    # create child grant
    child_id, child_issuer = create_child_grant(server, root_user, "g/*", ["create", "modify", "delete", "list", "delegate"], 5000, 50)
    obs.append(Observation("grant_create.success.200", 200, {}, {"has_id": bool(child_id), "has_issuer": bool(child_issuer)}))
    child_uid, child_user = issue_user(server, child_id, child_issuer)
    obs.append(Observation("grant_issue_user.success.200", 200, {}, {"has_userId": bool(child_uid), "has_userToken": bool(child_user)}))
    # grant get by exact user
    r = server.request("GET", f"/grants/{child_id}", headers=auth_headers(child_user))
    o = normalize_obs("grant_get.success.200", r, keep_headers=["content-type"],
                      normalize_json_paths={"$.id", "$.parent", "$.rootIssuerToken"})
    obs.append(o)
    # narrowing violation: try to widen locations beyond parent - expect denial (usually 400/403)
    body = json.dumps({"locations": "*", "operations": ["create"], "maxBytes": 1, "maxObjects": 1}).encode()
    r = server.request("POST", "/grants", headers={**auth_headers(child_user), "Content-Type": "application/json"}, body=body)
    obs.append(normalize_obs("grant_create.narrowing.denied", r, keep_headers=["content-type"]))
    # revoke user
    r = server.request("DELETE", f"/grants/{child_id}/users/{child_uid}", headers=auth_headers(child_issuer))
    obs.append(normalize_obs("grant_revoke_user.success.200", r, keep_headers=["content-type"], normalize_json_paths={"$.grantId", "$.userId"}))
    # use revoked user token should fail
    r = server.request("GET", f"/grants/{child_id}", headers=auth_headers(child_user))
    obs.append(normalize_obs("grant_revoked_user.denied", r, keep_headers=["content-type"]))
    # revoke absent user
    r = server.request("DELETE", f"/grants/{child_id}/users/not-a-real-user", headers=auth_headers(child_issuer))
    obs.append(normalize_obs("grant_revoke_user.absent.404", r, keep_headers=["content-type"], normalize_json_paths={"$.grantId", "$.userId"}))
    # revoke grant
    r = server.request("DELETE", f"/grants/{child_id}", headers=auth_headers(child_issuer))
    obs.append(normalize_obs("grant_revoke.success.200", r, keep_headers=["content-type"], normalize_json_paths={"$.grantId"}))
    # revoke absent grant
    r = server.request("DELETE", f"/grants/{child_id}", headers=auth_headers(child_issuer))
    obs.append(normalize_obs("grant_revoke.absent.404", r, keep_headers=["content-type"], normalize_json_paths={"$.grantId"}))
    return obs


def run_quota_scenarios(server: DisposableV2) -> list[Observation]:
    obs: list[Observation] = []
    _, root_user = issue_user(server, server.root_id, server.root_issuer)
    # object quota: allow one stored object in scope
    gid, issuer = create_child_grant(server, root_user, "q/*", ["create", "modify", "delete", "list"], 100_000, 1)
    _, user = issue_user(server, gid, issuer)
    h = auth_headers(user)
    r = server.request("POST", "/store/q/a.txt", headers=h, body=b"a")
    obs.append(normalize_obs("quota.objects.first_ok", r))
    r = server.request("POST", "/store/q/b.txt", headers=h, body=b"b")
    obs.append(normalize_obs("quota.objects.exceeded", r, keep_headers=["content-type"]))
    # byte quota: minimum valid maxBytes is 2048; object accounting includes payload+512 overhead.
    gid2, issuer2 = create_child_grant(server, root_user, "qb/*", ["create", "modify", "delete", "list"], 2048, 10)
    _, user2 = issue_user(server, gid2, issuer2)
    h2 = auth_headers(user2)
    r = server.request("POST", "/store/qb/small.txt", headers=h2, body=b"x" * 100)  # ~612 accounted
    obs.append(normalize_obs("quota.bytes.first_ok", r))
    r = server.request("POST", "/store/qb/big.txt", headers=h2, body=b"y" * 1600)  # ~2112 accounted > 2048
    obs.append(normalize_obs("quota.bytes.exceeded", r, keep_headers=["content-type"]))
    return obs


def run_push_scenarios(server: DisposableV2) -> list[Observation]:
    obs: list[Observation] = []
    _, root_user = issue_user(server, server.root_id, server.root_issuer)
    gid, issuer = create_child_grant(server, root_user, "p/*", ["create", "modify", "delete", "list", "subscribe"], 100_000, 100)
    _, user = issue_user(server, gid, issuer)
    h = {**auth_headers(user), "Content-Type": "application/json"}
    # invalid body cases (safely isolated, no real push delivery endpoint needed for validation failures)
    r = server.request("POST", "/push/subscriptions", headers=h, body=b"{}")
    obs.append(normalize_obs("push_subscribe.invalid.400.empty", r, keep_headers=["content-type"]))
    r = server.request("POST", "/push/subscriptions", headers=h, body=json.dumps({"locations": "p/*", "events": ["bogus"], "endpoint": "https://example.invalid/x", "keys": {"p256dh": "x", "auth": "y"}}).encode())
    obs.append(normalize_obs("push_subscribe.invalid.400.events", r, keep_headers=["content-type"]))
    # unauthorized/no subscribe capability
    gid2, issuer2 = create_child_grant(server, root_user, "p2/*", ["create", "list"], 100_000, 100)
    _, user2 = issue_user(server, gid2, issuer2)
    h2 = {**auth_headers(user2), "Content-Type": "application/json"}
    r = server.request("POST", "/push/subscriptions", headers=h2, body=json.dumps({"locations": "p2/*", "events": ["create"], "endpoint": "https://example.invalid/x", "keys": {"p256dh": "x", "auth": "y"}}).encode())
    obs.append(normalize_obs("push_subscribe.auth.denied", r, keep_headers=["content-type"]))
    # invalid capability for update/delete (nonexistent id/capability)
    r = server.request("PUT", "/push/subscriptions/nonexistent", headers={"X-Skrynia-Capability": "bad", "Content-Type": "application/json"}, body=json.dumps({"endpoint": "https://example.invalid/y", "keys": {"p256dh": "a", "auth": "b"}}).encode())
    obs.append(normalize_obs("push_update.invalid_cap", r, keep_headers=["content-type"]))
    r = server.request("DELETE", "/push/subscriptions/nonexistent", headers={"X-Skrynia-Capability": "bad"})
    obs.append(normalize_obs("push_delete.invalid_cap", r, keep_headers=["content-type"]))
    # note: successful subscribe/update/delete with real HTTPS endpoint semantics may be environment/network dependent
    obs.append(Observation("push_subscribe.success.SKIP", 0, {}, "skipped: requires accepted HTTPS endpoint/keys and may involve outbound validation; not safely deterministic here", note="report-only"))
    return obs


SCENARIOS: list[tuple[str, Callable[[DisposableV2], list[Observation]]]] = [
    ("public_meta", run_public_meta),
    ("store_reads_writes_conditionals_ranges_auth", run_store_scenarios),
    ("grants", run_grant_scenarios),
    ("quota", run_quota_scenarios),
    ("push_validation", run_push_scenarios),
]


def compare_scenario_results(name: str, a: list[Observation], b: list[Observation]) -> list[str]:
    failures: list[str] = []
    if len(a) != len(b):
        failures.append(f"{name}: observation count {len(a)} != {len(b)}")
        n = min(len(a), len(b))
    else:
        n = len(a)
    for i in range(n):
        oa, ob = a[i], b[i]
        # for dynamic fields we normalized; compare key aspects
        ok = (oa.case_id == ob.case_id and oa.status == ob.status and oa.body == ob.body and oa.error == ob.error)
        # some headers may vary (date/server absent in normalized), compared where included
        if oa.headers != ob.headers:
            ok = False
        if not ok:
            failures.append(f"{name}/{oa.case_id}: A(status={oa.status},err={oa.error},body={oa.body!r},hdr={oa.headers!r}) != B(status={ob.status},err={ob.error},body={ob.body!r},hdr={ob.headers!r})")
    return failures


def run_baseline_self_consistency(binary: str, inv: dict[str, Any], scenarios: list[str] | None = None) -> int:
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-self-"))
    results: dict[str, Any] = {"binary": str(Path(binary).resolve()), "scenarios": {}, "failures": []}
    try:
        selected = SCENARIOS
        if scenarios:
            wanted = set(scenarios)
            selected = [s for s in SCENARIOS if s[0] in wanted]
        for name, fn in selected:
            print(f"SCENARIO {name}: starting disposable v2 instances A/B")
            a_srv = DisposableV2.start(binary, tmp, f"{name}-a")
            b_srv = DisposableV2.start(binary, tmp, f"{name}-b")
            try:
                obs_a = fn(a_srv)
                obs_b = fn(b_srv)
            finally:
                a_srv.stop()
                b_srv.stop()
            fails = compare_scenario_results(name, obs_a, obs_b)
            results["scenarios"][name] = {
                "observations": len(obs_a),
                "ok": len(fails) == 0,
                "cases": [o.case_id for o in obs_a],
            }
            if fails:
                results["failures"].extend(fails)
                for f in fails:
                    print("FAIL", f)
            else:
                print(f"PASS {name} observations={len(obs_a)}")
        ok = len(results["failures"]) == 0
        results["ok"] = ok
        print(json.dumps({"result": "pass" if ok else "fail", "scenarios": len(results["scenarios"]), "failures": len(results["failures"])}, sort_keys=True))
        return 0 if ok else 1
    finally:
        # keep tmp dir only on failure? clean to be tidy but preserve for debugging by printing path
        print(f"disposable_tmp={tmp}")
        # cleanup temporary data dirs/process artifacts best-effort
        shutil.rmtree(tmp, ignore_errors=True)


def print_plan(inv: dict[str, Any]) -> None:
    plan = generate_plan(inv)
    print(f"inventory_schema={inv.get('schema')} endpoints={len(inv.get('endpoints', []))} generated_cases={len(plan)}")
    by_kind: dict[str, int] = {}
    for c in plan:
        by_kind[c.kind] = by_kind.get(c.kind, 0) + 1
    print("kinds=" + json.dumps(by_kind, sort_keys=True))
    for c in plan:
        flags = []
        if c.runnable_baseline_self:
            flags.append("baseline-self")
        if c.runnable_without_v3:
            flags.append("no-v3")
        print(f"- {c.case_id} [{c.endpoint_id}] ({c.kind}) {' '.join(flags)} :: {c.description}")


def self_test() -> int:
    inv = load_inventory(Path(__file__).resolve().parents[2] / "tests/http/v2_compat_inventory.json") if (Path(__file__).resolve().parents[2] / "tests/http/v2_compat_inventory.json").exists() else None
    if inv is None:
        inv = {"schema": "skrynia-tools.v2-compat-inventory", "endpoints": [{"id": "health", "method": "GET", "path": "/health", "auth": "public", "success": [200], "negative": []}]}
    plan = generate_plan(inv)
    assert any(c.endpoint_id == "store_get" and "range.206" in c.case_id for c in plan)
    assert any(c.endpoint_id == "grant_create" for c in plan)
    assert any(c.endpoint_id == "quota" or c.case_id.startswith("quota.") for c in plan)
    assert sha256_hex(b"abc") == hashlib.sha256(b"abc").hexdigest()
    a = Observation("x", 404, {"content-type": "application/json"}, {"error": "missing"})
    b = Observation("x", 404, {"content-type": "application/json"}, {"error": "missing"})
    ok, _ = compare_observations(a, b, ["content-type"])
    assert ok
    b2 = Observation("x", 200, {"content-type": "application/json"}, {})
    ok2, _ = compare_observations(b2, a, ["content-type"])
    assert not ok2
    # normalization
    n = normalize_json_value({"id": "abcdef0123456789", "n": 1}, {"$.id"}, "$")
    assert n["id"] == "<str>" and n["n"] == 1
    print(f"self_test_ok generated_cases={len(plan)}")
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--inventory", help="Path to v2 compatibility inventory JSON")
    ap.add_argument("--baseline", default=os.environ.get("SKRYNIA_V2_BASE_URL", ""), help="Pinned v2 base URL")
    ap.add_argument("--candidate", default=os.environ.get("SKRYNIA_V3_FACADE_BASE_URL", ""), help="Candidate v3+facade base URL")
    ap.add_argument("--v2-binary", default=os.environ.get("SKRYNIA_V2_SERVER_BINARY", ""), help="Path to pinned v2 skrynia-server binary for disposable self-consistency")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--generate-plan", action="store_true")
    ap.add_argument("--baseline-self-consistency", action="store_true")
    ap.add_argument("--scenario", action="append", default=[], help="Limit to named scenario(s)")
    ap.add_argument("--ids", default="", help="Comma-separated endpoint ids for public probe comparison")
    ap.add_argument("--missing-key", default="__differential_parity_probe_missing__")
    ap.add_argument("--allow-unsafe", action="store_true", help="Reserved; mutation/targeted runs must still be explicitly isolated")
    args = ap.parse_args(argv)

    if args.self_test:
        # prefer repo inventory when path provided or default exists
        return self_test()

    inv: dict[str, Any] | None = None
    if args.inventory:
        inv = load_inventory(args.inventory)
        print(f"inventory_loaded endpoints={len(inv.get('endpoints', []))} schema={inv.get('schema')}")

    if args.generate_plan:
        if not inv:
            raise SystemExit("--generate-plan requires --inventory")
        print_plan(inv)
        return 0

    if args.baseline_self_consistency:
        if not inv:
            raise SystemExit("--baseline-self-consistency requires --inventory")
        if not args.v2_binary:
            raise SystemExit("--baseline-self-consistency requires --v2-binary (disposable pinned v2 build)")
        return run_baseline_self_consistency(args.v2_binary, inv, args.scenario or None)

    # compatibility path: baseline-vs-candidate public probes (no v3 candidate expected now)
    if args.baseline and args.candidate:
        selected = [s.strip() for s in args.ids.split(",") if s.strip()] if args.ids else sorted(PUBLIC_SAFE_IDS)
        results = []
        for cid in selected:
            if cid not in PUBLIC_SAFE_IDS:
                print(f"{cid}: SKIP requires auth/mutation/disposable setup or v3 parity context (not run)")
                results.append((cid, None))
                continue
            # inline simple public cases
            path = {"health": "/health", "version": "/version", "push_vapid": "/push/vapid", "store_get": f"/store/{args.missing_key}", "store_head": f"/store/{args.missing_key}"}[cid]
            method = "HEAD" if cid == "store_head" else "GET"
            a = do_request(args.baseline, method, path)
            b = do_request(args.candidate, method, path)
            oa = normalize_obs(cid, a, keep_headers=["content-type", "cache-control", "x-content-type-options"])
            ob = normalize_obs(cid, b, keep_headers=["content-type", "cache-control", "x-content-type-options"])
            ok, msg = compare_observations(oa, ob, ["content-type", "cache-control", "x-content-type-options"])
            print(f"{cid}: {'PASS' if ok else 'FAIL'} {msg} status={a.status}/{b.status}")
            results.append((cid, ok))
        failed = [cid for cid, ok in results if ok is False]
        print("result=" + ("fail" if failed else "pass"))
        return 1 if failed else 0

    if inv:
        print("mode=inventory-loaded")
        print("no_targets: provide --v2-binary for baseline self-consistency, or --baseline/--candidate for differential probes")
        print("v3_candidate=unavailable (not run)")
        return 0
    ap.print_help()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
