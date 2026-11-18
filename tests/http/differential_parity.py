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
import signal
import socket
import statistics
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
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


def issue_user(server: Any, grant_id: str, issuer_token: str) -> tuple[str, str]:
    r = server.request("POST", f"/grants/{grant_id}/users", headers=auth_headers(issuer_token))
    if r.status != 200:
        raise RuntimeError(f"issue_user status={r.status} body={r.text()[:200]}")
    j = r.json_body()
    return j["userId"], j["userToken"]


def create_child_grant(server: Any, parent_user_token: str, locations: str,
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


# ---------- server handle abstraction ----------

@dataclass
class ServerHandle:
    base_url: str
    root_id: str = ""
    root_issuer: str = ""
    label: str = "server"

    def request(self, method: str, path: str, **kw: Any) -> Response:
        return do_request(self.base_url, method, path, **kw)

    def stop(self) -> None:
        return None


def handle_from_disposable(s: DisposableV2, label: str = "") -> ServerHandle:
    return ServerHandle(base_url=s.base_url, root_id=s.root_id, root_issuer=s.root_issuer, label=label or s.base_url)


def candidate_handle_from_args(base_url: str, root_id: str = "", root_issuer: str = "") -> ServerHandle:
    return ServerHandle(base_url=base_url.rstrip('/'), root_id=root_id, root_issuer=root_issuer, label="candidate")


# ---------- executable baseline scenarios ----------

def run_public_meta(server: DisposableV2 | ServerHandle) -> list[Observation]:
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


def run_store_scenarios(server: DisposableV2 | ServerHandle) -> list[Observation]:
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

def run_grant_scenarios(server: DisposableV2 | ServerHandle) -> list[Observation]:
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


def run_quota_scenarios(server: DisposableV2 | ServerHandle) -> list[Observation]:
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


def run_push_scenarios(server: DisposableV2 | ServerHandle) -> list[Observation]:
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



# ---------- concurrency, crash, security, latency ----------

def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    if len(s) == 1:
        return s[0]
    k = (len(s) - 1) * (p / 100.0)
    f = int(k)
    c = min(f + 1, len(s) - 1)
    if f == c:
        return s[f]
    return s[f] + (k - f) * (s[c] - s[f])


def run_concurrency_checks(binary: str, workers: int = 8, rounds: int = 3) -> dict[str, Any]:
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-conc-"))
    results: dict[str, Any] = {"ok": True, "checks": [], "workers": workers, "rounds": rounds}
    srv = None
    try:
        srv = DisposableV2.start(binary, tmp, "conc")
        _, root_user = issue_user(srv, srv.root_id, srv.root_issuer)
        # conflicting creates: same key, many parallel POSTs -> exactly one 201, rest 409
        gid, issuer = create_child_grant(srv, root_user, "conc/*", ["create", "modify", "delete", "list"], 10_000_000, 10_000)
        _, user = issue_user(srv, gid, issuer)
        h = auth_headers(user)
        for r_i in range(rounds):
            key = f"/store/conc/create-{r_i}-{random.randint(0,1_000_000)}"
            statuses: list[int] = []
            def do_create(_: int) -> int:
                resp = srv.request("POST", key, headers=h, body=b"x")
                return resp.status
            with ThreadPoolExecutor(max_workers=workers) as ex:
                futs = [ex.submit(do_create, i) for i in range(workers)]
                for fu in as_completed(futs):
                    statuses.append(fu.result())
            c201 = statuses.count(201)
            c409 = statuses.count(409)
            ok = c201 == 1 and c409 == workers - 1 and all(s in (201, 409) for s in statuses)
            results["checks"].append({"name": f"concurrent_create_round_{r_i}", "ok": ok, "statuses": sorted(statuses), "201": c201, "409": c409})
            if not ok:
                results["ok"] = False
            # verify readable and consistent
            g = srv.request("GET", key)
            if g.status != 200 or g.body != b"x":
                results["checks"].append({"name": f"concurrent_create_readback_{r_i}", "ok": False, "status": g.status, "len": len(g.body)})
                results["ok"] = False

        # CAS replace races: initialize object then parallel PUTs with same old hash
        key2 = "/store/conc/cas-race"
        srv.request("POST", key2, headers=h, body=b"v0")
        cur = sha256_hex(b"v0")
        statuses = []
        def do_put(_: int) -> int:
            resp = srv.request("PUT", key2, headers={**h, "X-Skrynia-If-SHA256": cur}, body=f"next-{random.randint(0,999999)}".encode())
            return resp.status
        with ThreadPoolExecutor(max_workers=workers) as ex:
            futs = [ex.submit(do_put, i) for i in range(workers)]
            for fu in as_completed(futs):
                statuses.append(fu.result())
        c200 = statuses.count(200)
        c412 = statuses.count(412)
        ok = c200 == 1 and c412 == workers - 1 and all(s in (200, 412) for s in statuses)
        results["checks"].append({"name": "concurrent_cas_replace", "ok": ok, "statuses": sorted(statuses), "200": c200, "412": c412})
        if not ok:
            results["ok"] = False

        # concurrent grant user issuance/revocation shouldn't corrupt server; basic liveness
        gid2, issuer2 = create_child_grant(srv, root_user, "conc2/*", ["list"], 100_000, 100)
        issued: list[tuple[str, str]] = []
        lock = threading.Lock()
        def issue_one(_: int) -> tuple[int, str]:
            try:
                uid, utok = issue_user(srv, gid2, issuer2)
                with lock:
                    issued.append((uid, utok))
                return 200, uid
            except Exception as e:  # noqa: BLE001
                return 0, str(e)
        with ThreadPoolExecutor(max_workers=min(workers, 6)) as ex:
            futs = [ex.submit(issue_one, i) for i in range(min(workers, 6))]
            issue_statuses = [fu.result()[0] for fu in as_completed(futs)]
        ok_issue = all(s == 200 for s in issue_statuses) and len(issued) >= 1
        results["checks"].append({"name": "concurrent_issue_users", "ok": ok_issue, "statuses": sorted(issue_statuses), "count": len(issued)})
        if not ok_issue:
            results["ok"] = False
        # revoke all issued concurrently
        def revoke_one(pair: tuple[str, str]) -> int:
            uid, _ = pair
            r = srv.request("DELETE", f"/grants/{gid2}/users/{uid}", headers=auth_headers(issuer2))
            return r.status
        with ThreadPoolExecutor(max_workers=min(workers, 6)) as ex:
            futs = [ex.submit(revoke_one, p) for p in list(issued)]
            rev_statuses = [fu.result() for fu in as_completed(futs)]
        # each should be revoked once; duplicates shouldn't happen but tolerate 200/404 patterns only if consistent; expect 200s
        ok_rev = all(s == 200 for s in rev_statuses)
        results["checks"].append({"name": "concurrent_revoke_users", "ok": ok_rev, "statuses": sorted(rev_statuses)})
        if not ok_rev:
            results["ok"] = False

        # health after stress
        hr = srv.request("GET", "/health")
        results["checks"].append({"name": "post_concurrency_health", "ok": hr.status == 200, "status": hr.status})
        if hr.status != 200:
            results["ok"] = False
    finally:
        if srv:
            srv.stop()
        results["disposable_tmp"] = str(tmp)
        shutil.rmtree(tmp, ignore_errors=True)
    return results


def run_crash_restart_checks(binary: str) -> dict[str, Any]:
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-crash-"))
    results: dict[str, Any] = {"ok": True, "checks": []}
    srv = None
    try:
        srv = DisposableV2.start(binary, tmp, "crash")
        _, root_user = issue_user(srv, srv.root_id, srv.root_issuer)
        gid, issuer = create_child_grant(srv, root_user, "crash/*", ["create", "modify", "delete", "list"], 10_000_000, 10_000)
        _, user = issue_user(srv, gid, issuer)
        h = auth_headers(user)

        # committed write then crash
        r = srv.request("POST", "/store/crash/committed.txt", headers=h, body=b"durable")
        results["checks"].append({"name": "write_before_crash", "ok": r.status == 201, "status": r.status})
        if r.status != 201:
            results["ok"] = False
        # give a moment for any async flush paths
        time.sleep(0.2)
        # crash
        if srv.proc:
            srv.proc.kill()
            srv.proc.wait(timeout=5)
        # restart same data dir
        env = os.environ.copy()
        env["SKRYNIA_DATA_DIR"] = str(srv.data_dir)
        env["SKRYNIA_HOST"] = "127.0.0.1"
        env["SKRYNIA_PORT"] = str(srv.port)
        env.setdefault("SKRYNIA_URL", srv.base_url)
        proc2 = subprocess.Popen([str(binary)], cwd=str(srv.data_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        srv.proc = proc2
        srv._wait_ready(timeout=20.0)
        # verify committed object readable
        g = srv.request("GET", "/store/crash/committed.txt")
        ok_read = g.status == 200 and g.body == b"durable"
        results["checks"].append({"name": "read_after_restart", "ok": ok_read, "status": g.status, "body": g.text()[:50]})
        if not ok_read:
            results["ok"] = False
        # verify can continue writing
        r2 = srv.request("POST", "/store/crash/after-restart.txt", headers=h, body=b"ok")
        results["checks"].append({"name": "write_after_restart", "ok": r2.status == 201, "status": r2.status})
        if r2.status != 201:
            results["ok"] = False

        # crash during burst of writes: best-effort, acknowledge uncertainty
        keys_written = []
        for i in range(20):
            rr = srv.request("POST", f"/store/crash/burst-{i}.txt", headers=h, body=f"b{i}".encode())
            if rr.status == 201:
                keys_written.append(f"/store/crash/burst-{i}.txt")
        if srv.proc:
            srv.proc.kill()
            srv.proc.wait(timeout=5)
        proc3 = subprocess.Popen([str(binary)], cwd=str(srv.data_dir), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        srv.proc = proc3
        srv._wait_ready(timeout=20.0)
        readable = 0
        for k in keys_written:
            gr = srv.request("GET", k)
            if gr.status == 200:
                readable += 1
        # acknowledged writes should be durable in a correct implementation; record observed count
        ok_burst = readable == len(keys_written)
        results["checks"].append({"name": "burst_acknowledged_reads_after_restart", "ok": ok_burst, "acknowledged": len(keys_written), "readable": readable, "note": "acknowledged 201 writes expected durable"})
        if not ok_burst:
            results["ok"] = False
        hr = srv.request("GET", "/health")
        results["checks"].append({"name": "post_crash_health", "ok": hr.status == 200, "status": hr.status})
        if hr.status != 200:
            results["ok"] = False
    finally:
        if srv:
            srv.stop()
        results["disposable_tmp"] = str(tmp)
        shutil.rmtree(tmp, ignore_errors=True)
    return results


def run_security_checks(binary: str | None = None, candidate: str | None = None, candidate_root_id: str = "", candidate_root_issuer: str = "") -> dict[str, Any]:
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-sec-"))
    results: dict[str, Any] = {"ok": True, "checks": []}
    srv = None
    handle: Any = None
    own = False
    try:
        if binary:
            srv = DisposableV2.start(binary, tmp, "sec")
            handle = handle_from_disposable(srv)
            own = True
        elif candidate:
            handle = candidate_handle_from_args(candidate, candidate_root_id, candidate_root_issuer)
        else:
            raise ValueError("security checks require --v2-binary or --candidate")

        # traversal / malformed path probes (public read surface and store operations)
        traversal_paths = [
            "/store/../etc/passwd",
            "/store/%2e%2e/%2e%2e/etc/passwd",
            "/store/..%2f..%2fetc/passwd",
            "/store/parity/../../secret",
            "/store/.//../x",
        ]
        for p in traversal_paths:
            r = handle.request("GET", p)
            # must not return 200 with sensitive host content; acceptable: 400/403/404
            body_text = r.text().lower()
            leaked = ("root:" in body_text and "/bin" in body_text)
            ok = (r.status in (400, 403, 404)) and not leaked
            results["checks"].append({"name": f"traversal_get:{p}", "ok": ok, "status": r.status, "leaked": leaked})
            if not ok:
                results["ok"] = False

        # malformed/missing auth should not leak credentials and should deny
        for name, headers in [
            ("missing_auth_list", {}),
            ("bad_bearer", {"Authorization": "Bearer definitely-not-valid"}),
            ("empty_bearer", {"Authorization": "Bearer "}),
            ("weird_auth_scheme", {"Authorization": "Basic abc"}),
        ]:
            r = handle.request("GET", "/store?locations=*", headers=headers)
            body = r.text()
            ok = r.status in (400, 401, 403) and "definitely-not-valid" not in body
            results["checks"].append({"name": name, "ok": ok, "status": r.status})
            if not ok:
                results["ok"] = False

        # malformed JSON bodies on auth endpoints (if creds available)
        if handle.root_id and handle.root_issuer:
            _, root_user = issue_user(handle, handle.root_id, handle.root_issuer)
            h = {**auth_headers(root_user), "Content-Type": "application/json"}
            for name, body in [
                ("grant_malformed_json", b"{"),
                ("grant_wrong_type", b'"string"'),
                ("grant_empty_object", b"{}"),
            ]:
                r = handle.request("POST", "/grants", headers=h, body=body)
                ok = r.status in (400, 422)
                results["checks"].append({"name": name, "ok": ok, "status": r.status})
                if not ok:
                    results["ok"] = False
            # create scoped grant and test issuer/user separation and scope confusion
            gid, issuer = create_child_grant(handle, root_user, "sec/*", ["create", "list"], 100_000, 100)
            uid, utok = issue_user(handle, gid, issuer)
            # issuer token must not be usable as user token for store writes
            r = handle.request("POST", "/store/sec/issuer-write.txt", headers=auth_headers(issuer), body=b"x")
            ok = r.status in (401, 403)
            results["checks"].append({"name": "issuer_cannot_write_store", "ok": ok, "status": r.status})
            if not ok:
                results["ok"] = False
            # user token must not issue users
            r = handle.request("POST", f"/grants/{gid}/users", headers=auth_headers(utok))
            ok = r.status in (401, 403)
            results["checks"].append({"name": "user_cannot_issue_users", "ok": ok, "status": r.status})
            if not ok:
                results["ok"] = False
            # out-of-scope write denied
            r = handle.request("POST", "/store/other/outside.txt", headers=auth_headers(utok), body=b"x")
            ok = r.status in (401, 403, 404)
            results["checks"].append({"name": "out_of_scope_write_denied", "ok": ok, "status": r.status})
            if not ok:
                results["ok"] = False
            # ensure error responses don't echo bearer tokens
            bad_tok = "Bearer skrynia-secret-should-not-echo"
            r = handle.request("GET", f"/grants/{gid}", headers={"Authorization": bad_tok})
            leaked_tok = "skrynia-secret-should-not-echo" in r.text()
            results["checks"].append({"name": "auth_error_no_token_echo", "ok": (not leaked_tok and r.status in (401,403,404)), "status": r.status, "leaked": leaked_tok})
            if leaked_tok or r.status not in (401, 403, 404):
                results["ok"] = False
        else:
            results["checks"].append({"name": "auth_mutation_security", "ok": True, "skipped": True, "reason": "no root credentials for candidate; ran public/malformed-auth probes only"})

    finally:
        if srv:
            srv.stop()
        results["disposable_tmp"] = str(tmp)
        shutil.rmtree(tmp, ignore_errors=True)
    return results


def run_latency_writes(binary: str | None = None, candidate: str | None = None,
                       candidate_root_id: str = "", candidate_root_issuer: str = "",
                       count: int = 200, warmup: int = 20, payload_bytes: int = 128) -> dict[str, Any]:
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-lat-"))
    results: dict[str, Any] = {"ok": True, "count": count, "warmup": warmup, "payload_bytes": payload_bytes, "samples_ms": []}
    srv = None
    handle: Any = None
    try:
        if binary:
            srv = DisposableV2.start(binary, tmp, "lat")
            handle = handle_from_disposable(srv)
        elif candidate:
            handle = candidate_handle_from_args(candidate, candidate_root_id, candidate_root_issuer)
        else:
            raise ValueError("latency requires --v2-binary or --candidate")
        if not (handle.root_id and handle.root_issuer):
            raise ValueError("latency requires root credentials (auto for v2 binary, or --candidate-root-id/--candidate-root-issuer)")
        _, root_user = issue_user(handle, handle.root_id, handle.root_issuer)
        gid, issuer = create_child_grant(handle, root_user, "lat/*", ["create", "modify", "delete", "list"], 100_000_000, 100_000)
        _, user = issue_user(handle, gid, issuer)
        h = auth_headers(user)
        payload = b"a" * payload_bytes

        def do_write(i: int) -> tuple[int, float]:
            t0 = time.perf_counter()
            r = handle.request("POST", f"/store/lat/w-{i}-{random.randint(0,10**9)}.bin", headers=h, body=payload)
            t1 = time.perf_counter()
            return r.status, (t1 - t0) * 1000.0

        # warmup
        for i in range(warmup):
            st, _ = do_write(i)
            if st != 201:
                results["ok"] = False
                results["warmup_error"] = {"i": i, "status": st}
                break
        samples = []
        if results["ok"]:
            for i in range(count):
                st, ms = do_write(warmup + i)
                if st != 201:
                    results["ok"] = False
                    results["error"] = {"i": i, "status": st}
                    break
                samples.append(ms)
        results["samples_ms"] = samples
        if samples:
            results["p50_ms"] = percentile(samples, 50)
            results["p95_ms"] = percentile(samples, 95)
            results["p99_ms"] = percentile(samples, 99)
            results["min_ms"] = min(samples)
            results["max_ms"] = max(samples)
            results["mean_ms"] = statistics.mean(samples)
        results["methodology"] = {
            "operation": "POST create simple write (create-if-absent), unique keys",
            "payload_bytes": payload_bytes,
            "warmup": warmup,
            "count": len(samples),
            "concurrency": 1,
            "storage": "disposable local filesystem data dir",
            "notes": "warm server, localhost HTTP; includes request/response overhead; durability semantics per server defaults",
        }
    finally:
        if srv:
            srv.stop()
        results["disposable_tmp"] = str(tmp)
        shutil.rmtree(tmp, ignore_errors=True)
    return results


SCENARIOS: list[tuple[str, Callable[[Any], list[Observation]]]] = [
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


def run_candidate_differential(binary: str, candidate: str, inv: dict[str, Any], scenarios: list[str] | None = None,
                              candidate_root_id: str = "", candidate_root_issuer: str = "") -> int:
    """Run implemented executable scenarios against disposable v2 and candidate base URL, compare normalized observations.

    Candidate is treated as an externally managed disposable instance. For auth/mutation scenarios, candidate root
    credentials must be supplied; public-only subset is compared when credentials are absent.
    """
    tmp = Path(tempfile.mkdtemp(prefix="skrynia-parity-diff-"))
    results: dict[str, Any] = {"candidate": candidate, "scenarios": {}, "failures": [], "ok": False}
    try:
        selected = SCENARIOS
        if scenarios:
            wanted = set(scenarios)
            selected = [s for s in SCENARIOS if s[0] in wanted]
        cand = candidate_handle_from_args(candidate, candidate_root_id, candidate_root_issuer)
        for name, fn in selected:
            needs_auth = name != "public_meta"
            if needs_auth and not (cand.root_id and cand.root_issuer):
                msg = f"SKIP {name}: candidate root credentials required (--candidate-root-id/--candidate-root-issuer)"
                print(msg)
                results["scenarios"][name] = {"ok": None, "skipped": True, "reason": msg}
                continue
            print(f"DIFFERENTIAL SCENARIO {name}: disposable v2 vs candidate")
            v2 = DisposableV2.start(binary, tmp, f"diff-{name}-v2")
            try:
                obs_v2 = fn(v2)
                obs_cand = fn(cand)
            finally:
                v2.stop()
            fails = compare_scenario_results(f"{name}(v2-vs-candidate)", obs_v2, obs_cand)
            results["scenarios"][name] = {"ok": len(fails) == 0, "observations": len(obs_v2)}
            if fails:
                results["failures"].extend(fails)
                for f in fails:
                    print("FAIL", f)
            else:
                print(f"PASS {name} observations={len(obs_v2)}")
        # pass if no executed scenario failed; skipped-only is not proof
        executed = [v for v in results["scenarios"].values() if v.get("ok") is not None]
        results["ok"] = bool(executed) and all(v.get("ok") for v in executed) and len(results["failures"]) == 0
        results["executed_scenarios"] = len(executed)
        results["skipped_scenarios"] = sum(1 for v in results["scenarios"].values() if v.get("skipped"))
        print(json.dumps({"result": "pass" if results["ok"] else "fail", "executed": results["executed_scenarios"], "skipped": results["skipped_scenarios"], "failures": len(results["failures"])}, sort_keys=True))
        return 0 if results["ok"] else 1
    finally:
        print(f"disposable_tmp={tmp}")
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--inventory", help="Path to v2 compatibility inventory JSON")
    ap.add_argument("--baseline", default=os.environ.get("SKRYNIA_V2_BASE_URL", ""), help="Pinned v2 base URL")
    ap.add_argument("--candidate", default=os.environ.get("SKRYNIA_V3_FACADE_BASE_URL", ""), help="Candidate v3+facade base URL")
    ap.add_argument("--candidate-root-id", default=os.environ.get("SKRYNIA_CANDIDATE_ROOT_ID", ""), help="Candidate root grant id (for auth scenarios)")
    ap.add_argument("--candidate-root-issuer", default=os.environ.get("SKRYNIA_CANDIDATE_ROOT_ISSUER", ""), help="Candidate root issuer token (for auth scenarios)")
    ap.add_argument("--v2-binary", default=os.environ.get("SKRYNIA_V2_SERVER_BINARY", ""), help="Path to pinned v2 skrynia-server binary for disposable runs")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--generate-plan", action="store_true")
    ap.add_argument("--baseline-self-consistency", action="store_true")
    ap.add_argument("--differential-scenarios", action="store_true", help="Run executable scenario suite: disposable v2 vs candidate (candidate must be externally managed disposable instance)")
    ap.add_argument("--concurrency", action="store_true", help="Run executable concurrency/race checks on disposable v2")
    ap.add_argument("--crash-restart", action="store_true", help="Run executable crash/restart durability checks on disposable v2")
    ap.add_argument("--security", action="store_true", help="Run executable security regression probes on disposable v2 and/or candidate")
    ap.add_argument("--latency-writes", action="store_true", help="Measure warm simple write latency (p50/p95/p99)")
    ap.add_argument("--latency-count", type=int, default=200)
    ap.add_argument("--latency-warmup", type=int, default=20)
    ap.add_argument("--latency-payload-bytes", type=int, default=128)
    ap.add_argument("--concurrency-workers", type=int, default=8)
    ap.add_argument("--concurrency-rounds", type=int, default=3)
    ap.add_argument("--scenario", action="append", default=[], help="Limit to named scenario(s)")
    ap.add_argument("--ids", default="", help="Comma-separated endpoint ids for public probe comparison")
    ap.add_argument("--missing-key", default="__differential_parity_probe_missing__")
    ap.add_argument("--allow-unsafe", action="store_true", help="Reserved; mutation/targeted runs must still be explicitly isolated")
    ap.add_argument("--json-out", help="Write JSON result to path for machine-readable evidence")
    args = ap.parse_args(argv)

    if args.self_test:
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

    if args.differential_scenarios:
        if not inv:
            raise SystemExit("--differential-scenarios requires --inventory")
        if not args.v2_binary:
            raise SystemExit("--differential-scenarios requires --v2-binary (disposable pinned v2 baseline)")
        if not args.candidate:
            raise SystemExit("--differential-scenarios requires --candidate (externally managed disposable v3+facade base URL)")
        return run_candidate_differential(args.v2_binary, args.candidate, inv, args.scenario or None, args.candidate_root_id, args.candidate_root_issuer)

    if args.concurrency:
        if not args.v2_binary:
            raise SystemExit("--concurrency requires --v2-binary (disposable pinned v2 build)")
        res = run_concurrency_checks(args.v2_binary, workers=args.concurrency_workers, rounds=args.concurrency_rounds)
        print(json.dumps(res, sort_keys=True, default=str))
        if args.json_out:
            Path(args.json_out).write_text(json.dumps(res, indent=2, sort_keys=True, default=str) + "\n", encoding="utf-8")
        return 0 if res.get("ok") else 1

    if args.crash_restart:
        if not args.v2_binary:
            raise SystemExit("--crash-restart requires --v2-binary (disposable pinned v2 build)")
        res = run_crash_restart_checks(args.v2_binary)
        print(json.dumps(res, sort_keys=True, default=str))
        if args.json_out:
            Path(args.json_out).write_text(json.dumps(res, indent=2, sort_keys=True, default=str) + "\n", encoding="utf-8")
        return 0 if res.get("ok") else 1

    if args.security:
        if not args.v2_binary and not args.candidate:
            raise SystemExit("--security requires --v2-binary and/or --candidate")
        res = run_security_checks(binary=args.v2_binary or None, candidate=args.candidate or None,
                                  candidate_root_id=args.candidate_root_id, candidate_root_issuer=args.candidate_root_issuer)
        print(json.dumps(res, sort_keys=True, default=str))
        if args.json_out:
            Path(args.json_out).write_text(json.dumps(res, indent=2, sort_keys=True, default=str) + "\n", encoding="utf-8")
        return 0 if res.get("ok") else 1

    if args.latency_writes:
        if not args.v2_binary and not args.candidate:
            raise SystemExit("--latency-writes requires --v2-binary and/or --candidate")
        res = run_latency_writes(binary=args.v2_binary or None, candidate=args.candidate or None,
                                 candidate_root_id=args.candidate_root_id, candidate_root_issuer=args.candidate_root_issuer,
                                 count=args.latency_count, warmup=args.latency_warmup, payload_bytes=args.latency_payload_bytes)
        print(json.dumps(res, sort_keys=True, default=str))
        if args.json_out:
            Path(args.json_out).write_text(json.dumps(res, indent=2, sort_keys=True, default=str) + "\n", encoding="utf-8")
        return 0 if res.get("ok") else 1

    # compatibility path: baseline-vs-candidate public probes
    if args.baseline and args.candidate:
        selected = [s.strip() for s in args.ids.split(",") if s.strip()] if args.ids else sorted(PUBLIC_SAFE_IDS)
        results = []
        for cid in selected:
            if cid not in PUBLIC_SAFE_IDS:
                print(f"{cid}: SKIP requires auth/mutation/disposable setup or v3 parity context (not run)")
                results.append((cid, None))
                continue
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
        print("no_targets: provide --v2-binary for baseline self-consistency/concurrency/crash-restart/security/latency, or --baseline/--candidate for probes, or --differential-scenarios with candidate")
        print("v3_candidate=unavailable unless --candidate provided (not run)")
        return 0
    ap.print_help()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
