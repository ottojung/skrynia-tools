#!/usr/bin/env python3
"""Isolated, disposable Skrynia user/grant revocation lifecycle smoke test."""
import json
import pathlib
import subprocess
import sys
import urllib.error
import urllib.request

executable, base, parent_file, root = sys.argv[1:]
root = pathlib.Path(root)
journal = root / "child-credentials.json"
tag = "1234567890abcdef1234567890abcdef"


def command(*args):
    result = subprocess.run([executable, "admin", *args], capture_output=True, text=True,
                            check=True, timeout=30)
    return json.loads(result.stdout)


created = command("create", "--name", "revokelive", "--url", base,
                  "--parent-token-file", parent_file, "--credentials-out", str(journal),
                  "--max-bytes", "100000", "--max-objects", "10", "--id", tag)
grant_id = created["grantId"]
record = json.loads(journal.read_text())
assert grant_id == record["grant"]["id"]
user = record["user"]["userToken"]
issuer = record["grant"]["issuerToken"]
key = f"a.revokelive.{tag}.original"
request = urllib.request.Request(base + "/store/" + key, data=b"retain-me",
                                 method="POST", headers={"Authorization": "Bearer " + user})
with urllib.request.urlopen(request, timeout=10) as reply:
    assert reply.status == 201

user_result = command("revoke-user", "--credentials-file", str(journal),
                      "--url", base, "--confirm-grant-id", grant_id)
assert user_result["state"] == "user_revoked"
assert "userToken" not in journal.read_text()
request = urllib.request.Request(base + "/store/" + f"a.revokelive.{tag}.after",
                                 data=b"must-be-rejected", method="POST",
                                 headers={"Authorization": "Bearer " + user})
try:
    urllib.request.urlopen(request, timeout=10)
except urllib.error.HTTPError as error:
    assert error.code == 403
else:
    raise AssertionError("revoked user could still create objects")

grant_result = command("revoke-grant", "--credentials-file", str(journal),
                       "--url", base, "--confirm-grant-id", grant_id)
assert grant_result["state"] == "grant_revoked"
assert json.loads(journal.read_text()) == {"state": "grant_revoked", "grantId": grant_id}
assert command("revoke-grant", "--credentials-file", str(journal),
               "--url", base, "--confirm-grant-id", grant_id)["state"] == "grant_revoked"
request = urllib.request.Request(base + "/grants/" + grant_id + "/users",
                                 data=b"", method="POST",
                                 headers={"Authorization": "Bearer " + issuer})
try:
    urllib.request.urlopen(request, timeout=10)
except urllib.error.HTTPError as error:
    assert error.code == 404
else:
    raise AssertionError("revoked child grant remained available")
with urllib.request.urlopen(base + "/store/" + key, timeout=10) as reply:
    assert reply.read() == b"retain-me"
print("Real Skrynia credential lifecycle: create, revoke user/grant, retain bytes — passed")
