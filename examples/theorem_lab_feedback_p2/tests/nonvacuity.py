#!/usr/bin/env python3
"""Std-only non-vacuity gate for the real-stack P2 security assertions.

Demonstrates RED on the old assertions and GREEN on the new ones using the
exact adversarial responses they were previously blind to. No binaries,
credentials, network or builds are involved; this is safe to run in CI.
"""
import json

from assertions import forge_denied, sibling_no_leak

FORGED_BODY = b'forged-write-succeeded'
LEAKED_STATE = json.dumps({
    'version': 1,
    'feedback': [{'id': 'secret', 'comment': 'victim data'}],
}).encode()
UNCHANGED_KEY = json.dumps({
    'version': 1,
    'feedback': [{'id': 'e1'}, {'id': 'e2'}],
}).encode()


def old_forge_check(key_response, final_count=2):
    """Old logic re-read KEY and compared its count, ignoring the forged key."""
    return len(json.loads(key_response['body'])['feedback']) == final_count


def old_sibling_check(response):
    """Old logic compared a JSON state blob to the literal b'secret'."""
    return response.get('kind') != 'ok' or response.get('body') != b'secret'


CHECKS = [
    # GREEN on new assertions: the adversarial responses are rejected.
    ('forge-denial-rejects-successful-forge', not forge_denied({'kind': 'ok', 'body': FORGED_BODY})),
    ('forge-denial-accepts-denial', forge_denied({'kind': 'err', 'code': 13})),
    ('forge-denial-accepts-not-found', forge_denied({'kind': 'err', 'code': 8})),
    ('sibling-rejects-real-json-leak', not sibling_no_leak({'kind': 'ok', 'body': LEAKED_STATE})),
    ('sibling-rejects-raw-secret-leak', not sibling_no_leak({'kind': 'ok', 'body': b'secret'})),
    ('sibling-accepts-denial', sibling_no_leak({'kind': 'err', 'code': 13})),
    # RED on old assertions: the same adversarial responses used to pass.
    ('old-forge-check-was-vacuous', old_forge_check({'kind': 'ok', 'body': UNCHANGED_KEY})),
    ('old-sibling-check-was-vacuous', old_sibling_check({'kind': 'ok', 'body': LEAKED_STATE})),
]

for name, ok in CHECKS:
    print(('PASS' if ok else 'FAIL'), name)
if not all(ok for _, ok in CHECKS):
    raise SystemExit(1)
print('ALL_OK=True')
