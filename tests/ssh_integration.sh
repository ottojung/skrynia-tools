#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/.." && pwd)"
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
printf 'build:\n\tmkdir -p build/deep\n\tcp ../../payload.txt build/deep/artifact.bin\n' \
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
forced="SKRYNIA_DEPLOY_ROOT=$tmp/hosted GIT_CONFIG_GLOBAL=$tmp/gitconfig PATH=$tmp/bin:$PATH $repo/target/debug/skrynia-tools app deploy --spec-stdin true"
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
