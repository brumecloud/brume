#!/usr/bin/env bash
# Checks, in Docker, what scripts/release/build.sh wrote in dist/:
#   - brume --version runs in debian:12 and ubuntu:24.04, and ldd finds libduckdb.so beside it;
#   - install.sh installs from file:// in ubuntu:24.04, then hands over to `brume install`;
#   - install.sh refuses an archive changed by one byte, a failed download and a non-root user.
#
# Usage: scripts/release/test.sh <version> [arm64|amd64|all]   (default: all)
# amd64 runs under emulation (--platform linux/amd64).
set -euo pipefail

repo=$(cd "$(dirname "$0")/../.." && pwd)
dist=$repo/dist
version=${1:?usage: test.sh <version> [arm64|amd64|all]}
case ${2:-all} in
  arm64 | amd64) archs=("$2") ;;
  all) archs=(arm64 amd64) ;;
  *) echo "test.sh: unknown architecture: $2" >&2 && exit 1 ;;
esac

failures=0
check() {
  local name=$1 output=$2
  shift 2
  if printf '%s\n' "$output" | "$@"; then
    echo "ok    $name"
  else
    echo "FAIL  $name"
    printf '%s\n' "$output" | sed 's/^/      | /'
    failures=$((failures + 1))
  fi
}
# Every pattern must match a line of stdin.
has() {
  local input
  input=$(cat)
  for pattern in "$@"; do
    grep -qE -- "$pattern" <<<"$input" || return 1
  done
}
# No line of stdin matches the pattern.
lacks() {
  ! grep -qE -- "$1"
}

for arch in "${archs[@]}"; do
  archive=brume-$version-linux-$arch.tar.gz
  [[ -f $dist/$archive ]] || { echo "test.sh: $dist/$archive is missing" >&2 && exit 1; }

  for image in debian:12 ubuntu:24.04; do
    out=$(docker run --rm --platform "linux/$arch" --volume "$dist:/dist:ro" "$image" sh -c "
      mkdir /opt/release && tar -xzf /dist/$archive -C /opt/release
      ls /opt/release/brume
      /opt/release/brume/brume --version
      ldd /opt/release/brume/brume" 2>&1) || true
    check "$arch $image: brume --version, libduckdb.so beside it" "$out" has \
      "^brume $version\$" \
      'libduckdb\.so => /opt/release/brume/libduckdb\.so' \
      '^libduckdb\.so$'
    check "$arch $image: ldd resolves every library" "$out" lacks 'not found'
  done

  # ubuntu:24.04 with curl, which a server that ran `curl … | sh` has.
  test_image=brume-release-test:ubuntu-24.04-$arch
  docker build --quiet --platform "linux/$arch" --tag "$test_image" - >/dev/null <<'EOF'
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
EOF
  run() {
    docker run --rm --platform "linux/$arch" --volume "$dist:/dist:ro" "$@" 2>&1 || true
  }

  # Piped into sh, as `curl … | sh` does. brume install cannot finish in a container: until it
  # exists it is an unknown subcommand; after, it stops at its checks (no systemd).
  for tty in '' --tty; do
    out=$(run $tty --env BRUME_DOWNLOAD_BASE=file:///dist "$test_image" sh -c '
      cat /dist/install.sh | sh -s -- --admin ssh-tunnel --yes
      echo "install.sh exit=$?"
      echo "link=$(readlink /usr/local/bin/brume)"
      echo "lib=$(ls /usr/local/lib/brume | tr "\n" " ")"
      echo "tmp=$(ls -A /tmp | tr "\n" " ")"
      brume --version' | tr -d '\r')
    check "$arch install.sh ${tty:-without tty}: installs, then runs brume install" "$out" has \
      'Running brume install' \
      '^link=/usr/local/lib/brume/brume$' \
      '^lib=LICENSE-duckdb\.txt LICENSE\.md brume libduckdb\.so $' \
      '^tmp=$' \
      "^brume $version\$"
    printf '%s\n' "$out" | grep -A 3 'Running brume install' | sed 's/^/      | /'
  done

  out=$(run "$test_image" sh -c "
    mkdir /tmp/bad && cp /dist/* /tmp/bad/
    printf X | dd of=/tmp/bad/$archive bs=1 seek=1000 conv=notrunc 2>/dev/null
    cmp -s /dist/$archive /tmp/bad/$archive && echo 'the byte did not change'
    BRUME_DOWNLOAD_BASE=file:///tmp/bad sh /tmp/bad/install.sh
    echo \"install.sh exit=\$?\"
    ls /usr/local/lib/brume /usr/local/bin/brume 2>&1
    rm -r /tmp/bad
    echo \"tmp=\$(ls -A /tmp | tr '\n' ' ')\"")
  check "$arch install.sh refuses an archive changed by one byte" "$out" has \
    'does not match the SHA-256' '^install.sh exit=1$' \
    "cannot access '/usr/local/lib/brume'" "cannot access '/usr/local/bin/brume'" '^tmp=$'

  out=$(run --env BRUME_DOWNLOAD_BASE=file:///nowhere "$test_image" sh -c '
    sh /dist/install.sh; echo "install.sh exit=$?"')
  check "$arch install.sh stops on a failed download" "$out" has \
    "could not download file:///nowhere/$archive" '^install.sh exit=1$'

  out=$(run --user 1000 "$test_image" sh -c 'sh /dist/install.sh; echo "install.sh exit=$?"')
  check "$arch install.sh refuses a non-root user" "$out" has \
    'run it as root' '^install.sh exit=1$'
done

# `--platform linux/amd64` replaced the local tags with their amd64 images: bring back the native
# ones, which other tools on this machine expect.
for image in debian:12 ubuntu:24.04; do
  docker pull --quiet "$image" >/dev/null
done

if ((failures > 0)); then
  echo "$failures check(s) failed"
  exit 1
fi
echo "All checks passed"
