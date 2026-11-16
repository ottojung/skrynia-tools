#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d /tmp/skrynia-real-ssh-XXXXXX)"
pid=""
cleanup() {
    if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi
    rm -rf "$tmp"
}
trap cleanup EXIT
chmod 700 "$tmp"
mkdir -p "$tmp/source/apps/demo" "$tmp/hosted" "$tmp/bin"
printf 'payload from real SSH\n' > "$tmp/source/payload.txt"
printf 'build:\n\tmkdir -p build/deep\n\tcp ../../payload.txt build/deep/artifact.bin\n\tchmod +x build/deep/artifact.bin\n' \
  > "$tmp/source/apps/demo/Makefile"
git -C "$tmp/source" init -q -b main
git -C "$tmp/source" add .
git -C "$tmp/source" -c user.name=Fixture -c user.email=fixture@example.invalid \
  commit -qm 'SSH deployment fixture'
sha="$(git -C "$tmp/source" rev-parse HEAD)"
printf '[url "file://%s/source"]\n  insteadOf = https://github.com/owner/repo\n' \
  "$tmp" > "$tmp/gitconfig"
if [ "${SKRYNIA_SSH_TEST_REAL_PODMAN:-0}" != 1 ]; then
    cat > "$tmp/bin/podman" <<'SH'
#!/usr/bin/env sh
if [ "$1" = "info" ]; then echo true; exit 0; fi
if [ "$1" = "container" ]; then exit 1; fi
while [ $# -gt 0 ]; do
    case "$1" in
        -v) src="${2%%:*}"; shift 2 ;;
        -w) dir="$2"; shift 2 ;;
        *) shift ;;
    esac
done
cd "$src/${dir#/workspace/}" || exit 1
exec make build
SH
    chmod 700 "$tmp/bin/podman"
fi
cargo build --locked --quiet --manifest-path "$repo/Cargo.toml"
ssh-keygen -q -t ed25519 -N '' -f "$tmp/id_ed25519"
ssh-keygen -q -t ed25519 -N '' -f "$tmp/host_ed25519"
port="$(python3 -c 'import socket; s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
{
    printf 'Port %s\nListenAddress 127.0.0.1\n' "$port"
    printf 'HostKey %s/host_ed25519\nPidFile %s/sshd.pid\n' "$tmp" "$tmp"
    printf 'AuthorizedKeysFile %s/authorized_keys\n' "$tmp"
    printf 'StrictModes no\nUsePAM no\nPasswordAuthentication no\nKbdInteractiveAuthentication no\n'
    printf 'PubkeyAuthentication yes\nPermitRootLogin no\n'
    printf 'AllowUsers %s\nLogLevel VERBOSE\nUseDNS no\nSubsystem sftp internal-sftp\n' "$(id -un)"
} > "$tmp/sshd_config"
server_bin="$(command -v sshd)"
"$server_bin" -t -f "$tmp/sshd_config"
forced="SKRYNIA_DEPLOY_ROOT=$tmp/hosted GIT_CONFIG_GLOBAL=$tmp/gitconfig PATH=$tmp/bin:$PATH $repo/target/debug/skrynia-tools app rpc --spec-stdin true"
printf 'restrict,command="%s" %s\n' "$forced" "$(cat "$tmp/id_ed25519.pub")" \
    > "$tmp/authorized_keys"
"$server_bin" -D -f "$tmp/sshd_config" -E "$tmp/sshd.log" &
pid="$!"
ready=0
for _ in $(seq 1 40); do
    if python3 -c 'import socket,sys;s=socket.socket();s.settimeout(.1);s.connect(("127.0.0.1",int(sys.argv[1])))' "$port" 2>/dev/null; then
        ready=1
        break
    fi
    if ! kill -0 "$pid" 2>/dev/null; then break; fi
    sleep .1
done
if [ "$ready" != 1 ]; then cat "$tmp/sshd.log"; exit 1; fi
export SKRYNIA_HOST=127.0.0.1 SKRYNIA_SSH_PORT="$port" SKRYNIA_USER="$(id -un)"
export SKRYNIA_SSH_KEY="$(cat "$tmp/id_ed25519")"
export SKRYNIA_KNOWN_HOSTS="[127.0.0.1]:$port $(cat "$tmp/host_ed25519.pub")"
export SKRYNIA_APP_NAME=demo SKRYNIA_APPS_DIR="$tmp/hosted" SKRYNIA_REPO=""
export SKRYNIA_SUBDIR=apps/demo SKRYNIA_COMMIT="" SKRYNIA_BUILDER_IMAGE="docker.io/library/rust:1.93-bookworm"
export GITHUB_REPOSITORY=owner/repo GITHUB_SHA="$sha" GITHUB_OUTPUT="$tmp/github-output"
bash "$repo/action/deploy/deploy.sh"
python3 - "$tmp" "$sha" <<'PY'
import json, pathlib, sys
root=pathlib.Path(sys.argv[1])
metadata=list((root/'hosted/metadata/demo').glob('*.json'))
assert len(metadata)==1, metadata
entry=json.loads(metadata[0].read_text())
assert entry['commit']==sys.argv[2]
assert (root/'hosted/apps/demo/deep/artifact.bin').read_text()=='payload from real SSH\n'
assert (root/'hosted/apps/demo/deep/artifact.bin').stat().st_mode & 0o111
output=(root/'github-output').read_text()
assert 'release='+entry['id'] in output
assert 'commit='+sys.argv[2] in output
assert 'path=/apps/demo/' in output
print('REAL SSH PASS: pinned host key, restricted deploy key, Git checkout, build, release outputs')
PY
ssh-keygen -q -t ed25519 -N '' -f "$tmp/evil_host"
export SKRYNIA_KNOWN_HOSTS="[127.0.0.1]:$port $(cat "$tmp/evil_host.pub")"
if bash "$repo/action/deploy/deploy.sh" > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: mismatched host key accepted' >&2
    exit 1
fi
export SKRYNIA_KNOWN_HOSTS="[127.0.0.1]:$port $(cat "$tmp/host_ed25519.pub")"
mkdir "$tmp/unauthorized"
export SKRYNIA_APPS_DIR="$tmp/unauthorized"
if bash "$repo/action/deploy/deploy.sh" > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: non-authorized deployment root accepted' >&2
    exit 1
fi
test ! -e "$tmp/unauthorized/apps"
echo 'REAL SSH REJECTION PASS: incorrect host key and out-of-scope deployment directory'
printf '%s\n' "$SKRYNIA_KNOWN_HOSTS" > "$tmp/known_hosts"
remote() {
    "$repo/target/debug/skrynia-tools" app "$@" \
      --ssh-host 127.0.0.1 --ssh-user "$(id -un)" --ssh-port "$port" \
      --ssh-key "$tmp/id_ed25519" --ssh-known-hosts "$tmp/known_hosts" \
      --apps-dir "$tmp/hosted"
}
first_id="$(python3 - "$tmp" <<'PYID'
import json,pathlib,sys
p=next((pathlib.Path(sys.argv[1])/'hosted/metadata/demo').glob('*.json'))
print(json.loads(p.read_text())['id'])
PYID
)"
remote list > "$tmp/apps.txt"
remote doctor --format json > "$tmp/doctor.json"
python3 -c 'import json,sys;assert json.load(open(sys.argv[1]))["prerequisites_ok"]' "$tmp/doctor.json"
remote versions --name demo > "$tmp/versions.txt"
remote current --name demo --format json > "$tmp/first.json"
grep -q "$first_id" "$tmp/versions.txt"
printf 'second artifact over remote CLI\n' > "$tmp/source/payload.txt"
git -C "$tmp/source" add payload.txt
git -C "$tmp/source" -c user.name=Fixture -c user.email=fixture@example.invalid \
  commit -qm 'second artifact'
second_sha="$(git -C "$tmp/source" rev-parse HEAD)"
if remote deploy --name demo --repo https://github.com/owner/repo \
  --subdir apps/demo --commit "$second_sha" --builder-image '--privileged' \
  > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: builder-image flag injection accepted' >&2
    exit 1
fi
grep -q 'invalid builder image reference' "$tmp/rejected.err"
remote deploy --name demo --repo https://github.com/owner/repo \
  --subdir apps/demo --commit "$second_sha" \
  --builder-image docker.io/library/rust:1.93-bookworm > "$tmp/second.json"
second_id="$(python3 - "$tmp/second.json" <<'PYID'
import json,sys
print(json.load(open(sys.argv[1]))['id'])
PYID
)"
remote compare --name demo --from "$first_id" --to "$second_id" --format json \
  > "$tmp/compare.json"
remote diff --name demo --from "$first_id" --to "$second_id" --format json > "$tmp/diff.json"
remote versions --name demo --format json > "$tmp/versions.json"
remote verify --name demo --format json > "$tmp/verification.json"
python3 - "$tmp" "$first_id" "$second_id" <<'PYID'
import json,pathlib,sys
root=pathlib.Path(sys.argv[1])
assert json.loads((root/'compare.json').read_text())['to']['id']==sys.argv[3]
diff=json.loads((root/'diff.json').read_text())
assert diff['from_verified'] is True and diff['to_verified'] is True
assert [v['path'] for v in diff['changes']['modified']]==['deep/artifact.bin']
assert diff['changes']['modified'][0]['before']['sha256'] != diff['changes']['modified'][0]['after']['sha256']
entries=json.loads((root/'versions.json').read_text())
assert len(entries)==2 and entries[0]['id']==sys.argv[3] and entries[0]['active']
assert entries[1]['id']==sys.argv[2] and not entries[1]['active']
assert (root/'hosted/apps/demo/deep/artifact.bin').read_text()=='second artifact over remote CLI\n'
verification=json.loads((root/'verification.json').read_text())
assert verification['verified'] is True
assert verification['expected_sha256']==verification['actual_sha256']
PYID
remote rollback --name demo --release "$first_id" > "$tmp/rollback.txt"
remote verify --name demo --release "$first_id" --format json > "$tmp/restored-verify.json"
python3 -c 'import json,sys;assert json.load(open(sys.argv[1]))["verified"] is True' "$tmp/restored-verify.json"
test "$(cat "$tmp/hosted/apps/demo/deep/artifact.bin")" = 'payload from real SSH'
grep -q "$first_id" "$tmp/rollback.txt"
remote audit --name demo --format json > "$tmp/audit-clean.json"
python3 -c 'import json,sys;assert json.load(open(sys.argv[1]))["clean"] is True' "$tmp/audit-clean.json"
stage="$tmp/hosted/releases/demo/.0123456789ab-0000000000000000.staging"
candidate="$tmp/hosted/apps/.demo.0123456789abcdef0123456789abcdef"
workspace="$tmp/hosted/.work/demo-0123456789abcdef0123456789abcdef"
mkdir "$stage" "$workspace"
ln -s "../releases/demo/$first_id" "$candidate"
if remote audit --name demo --format json > "$tmp/audit-incomplete.json" 2> "$tmp/rejected.err"; then
    echo 'ERROR: abandoned release staging was not reported' >&2
    exit 1
fi
python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));assert not d["clean"] and len(d["abandoned_staging"])==1 and len(d["abandoned_activation_links"])==1 and len(d["abandoned_workspaces"])==1' "$tmp/audit-incomplete.json"
test -d "$stage" && test -L "$candidate" && test -d "$workspace"
rmdir "$stage" "$workspace"
rm "$candidate"
echo 'REAL SSH RELEASE AUDIT PASS: abandoned staging, candidate links, workspaces retained'
echo 'REAL SSH MANAGEMENT PASS: remote list/versions/compare, remote deployment and constant-work rollback'
cp "$tmp/id_ed25519" "$tmp/insecure_key"
chmod 0644 "$tmp/insecure_key"
if "$repo/target/debug/skrynia-tools" app current --name demo \
    --apps-dir "$tmp/hosted" --ssh-host 127.0.0.1 --ssh-user "$(id -un)" \
    --ssh-port "$port" --ssh-key "$tmp/insecure_key" \
    --ssh-known-hosts "$tmp/known_hosts" > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: insecure SSH private key accepted' >&2
    exit 1
fi
if "$repo/target/debug/skrynia-tools" app current --name demo \
    --apps-dir "$tmp/hosted" --ssh-user "$(id -un)" \
    > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: partial remote options silently ran a local command' >&2
    exit 1
fi
if printf '{"action":"destroy","apps_dir":"%s"}' "$tmp/hosted" | \
  ssh -F /dev/null -T -i "$tmp/id_ed25519" -p "$port" \
    -o BatchMode=yes -o IdentitiesOnly=yes -o StrictHostKeyChecking=yes \
    -o UserKnownHostsFile="$tmp/known_hosts" \
    "$(id -un)@127.0.0.1" "ignored" > "$tmp/rejected.out" 2> "$tmp/rejected.err"; then
    echo 'ERROR: unauthorized RPC action accepted' >&2
    exit 1
fi
echo 'REAL SSH SECURITY PASS: private key mode, partial SSH arguments, RPC whitelist'

printf 'corrupted artifact\n' > "$tmp/hosted/releases/demo/$first_id/deep/artifact.bin"
if remote verify --name demo --release "$first_id" --format json \
  > "$tmp/tampered.json" 2> "$tmp/rejected.err"; then
    echo 'ERROR: corrupted release was reported as intact' >&2
    exit 1
fi
python3 -c 'import json,sys;assert json.load(open(sys.argv[1]))["verified"] is False' "$tmp/tampered.json"
if remote diff --name demo --from "$first_id" --to "$second_id" --format json \
  > "$tmp/invalid-diff.json" 2> "$tmp/rejected.err"; then
    echo 'ERROR: diff trusted a corrupted release' >&2
    exit 1
fi
grep -q 'release integrity mismatch' "$tmp/rejected.err"
echo 'REAL SSH INTEGRITY PASS: clean versions verified; modified historical artifact rejected'
