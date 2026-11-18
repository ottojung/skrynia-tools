#!/usr/bin/env bash
set -euo pipefail
# Pinned v2 behavioral fixture/differential harness scaffold.
# Default mode validates the inventory artifact; optional probe mode runs
# black-box checks against a target v2-compatible HTTP endpoint.
# Safety: never target production data. Mutation probes require an explicit
# disposable target and SKRYNIA_V2_ALLOW_MUTATIONS=1.

repo="$(cd "$(dirname "$0")/../.." && pwd)"
inv="$repo/tests/http/v2_compat_inventory.json"
doc="$repo/docs/v2-compat-inventory.md"
mode="${1:-check}"
base="${SKRYNIA_V2_BASE_URL:-}"

fail() { printf 'v2_compat_harness: %s\n' "$*" >&2; exit 1; }
need_file() { [ -f "$1" ] || fail "missing $1"; }

check_inventory() {
  need_file "$inv"
  need_file "$doc"
  python3 - "$inv" "$doc" <<'PY'
import json,sys
inv=json.load(open(sys.argv[1],encoding='utf8'))
doc=open(sys.argv[2],encoding='utf8').read()
assert inv.get('schema')=='skrynia-tools.v2-compat-inventory'
p=inv.get('pinned',{})
assert p.get('skrynia_tools_commit'), 'missing skrynia_tools_commit'
assert p.get('v2_server_commit'), 'missing pinned v2_server_commit'
eps=inv.get('endpoints',[])
assert eps, 'no endpoints'
ids=set()
for e in eps:
  for k in ('id','method','path','auth','success','negative'):
    assert k in e, f'missing {k} in {e.get("id")}'
  assert e['id'] not in ids, f'duplicate id {e["id"]}'
  ids.add(e['id'])
need={'health','version','push_vapid','store_get','store_head','store_list','store_create','store_replace','store_delete','grant_create','grant_get','grant_issue_user','grant_revoke_user','grant_revoke','push_subscribe','push_update','push_delete'}
missing=sorted(need-ids)
assert not missing, f'missing endpoint ids: {missing}'
assert 'a33336e29c8781115489a3b4adc270573bc458fc' in doc
print(f'inventory_ok endpoints={len(eps)} corpus={len(inv.get("reference_corpus",[]))}')
PY
}

probe_public() {
  [ -n "$base" ] || fail "SKRYNIA_V2_BASE_URL is required for probe mode"
  command -v curl >/dev/null || fail "curl required"
  printf 'probing public endpoints at %s\n' "$base"
  for path in /health /version /push/vapid; do
    code="$(curl -sS -o /tmp/v2-probe-body.$$ -w '%{http_code}' "$base$path" || true)"
    printf 'GET %s -> %s\n' "$path" "$code"
  done
  miss="$base/store/__v2_compat_probe_missing__$(date +%s)"
  code="$(curl -sS -o /dev/null -w '%{http_code}' "$miss" || true)"
  printf 'GET missing store object -> %s (expected 404)\n' "$code"
  [ "$code" = "404" ] || printf 'warning: unexpected status for missing object\n' >&2
  rm -f /tmp/v2-probe-body.$$
}

case "$mode" in
  check) check_inventory ;;
  probe)
    check_inventory
    probe_public
    if [ "${SKRYNIA_V2_ALLOW_MUTATIONS:-0}" = "1" ]; then
      printf 'mutation probes: scaffold only (implement isolated grant/object/push differential cases here)\n'
    else
      printf 'mutation probes skipped (set SKRYNIA_V2_ALLOW_MUTATIONS=1 for disposable-target differential cases)\n'
    fi
    ;;
  *)
    fail "unknown mode: $mode (use check|probe)"
    ;;
esac
