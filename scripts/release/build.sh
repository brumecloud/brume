#!/usr/bin/env bash
# Builds the Linux release of Brume in dist/:
#   brume-<version>-linux-<arch>.tar.gz  a brume/ folder with the binary and libduckdb.so
#   install.sh                           the installer, with the version and SHA-256s pinned
#   SHA256SUMS                           the archives and install.sh
#
# Usage: scripts/release/build.sh <version> [arm64|amd64|all]   (default: all)
#
# The panel is built first, then embedded in the binary. The binary is compiled in Docker
# against the glibc of Debian 12. See scripts/release/README.md.
set -euo pipefail

# The official libduckdb, matching libduckdb-sys in Cargo.lock: crate 1.MMmmpp.x is DuckDB
# MM.mm.pp. On a DuckDB upgrade, update the three values from the GitHub release.
DUCKDB_VERSION=2.0.0
DUCKDB_SHA256_ARM64=c60eecf91143ca6bbac55c4f4d40dc22520488b0ac257c86e0fe9a2b451cf2b7
DUCKDB_SHA256_AMD64=993ad89e7dfca66cd6da994c2cad090c2fc160ddf9f550498a098ab4bb13cfb1

IMAGE=brume-release-build:rust-1.92-duckdb-$DUCKDB_VERSION
# Named volumes, so a rebuild is incremental.
REGISTRY_VOLUME=brume-release-cargo-registry
TARGET_VOLUME=brume-release-target

repo=$(cd "$(dirname "$0")/../.." && pwd)
dist=$repo/dist

die() {
  echo "build.sh: $*" >&2
  exit 1
}

step() {
  printf '\n==> %s\n' "$*"
}

sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$@"; else shasum -a 256 "$@"; fi
}

version=${1:-}
target=${2:-all}
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]] ||
  die "usage: build.sh <version> [arm64|amd64|all]"
case $target in
  arm64 | amd64) archs=("$target") ;;
  all) archs=(arm64 amd64) ;;
  *) die "unknown architecture: $target (arm64, amd64 or all)" ;;
esac

# `brume --version` prints the workspace version: the archive must not say otherwise.
workspace_version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo/Cargo.toml" | head -n 1)
[[ $version == "$workspace_version" ]] ||
  die "version $version differs from the workspace version in Cargo.toml ($workspace_version)"

encoded=$(grep -A 1 '^name = "libduckdb-sys"$' "$repo/Cargo.lock" |
  sed -n 's/^version = "1\.\([0-9]*\)\.[0-9]*"$/\1/p')
locked=$((10#$encoded / 10000)).$((10#$encoded / 100 % 100)).$((10#$encoded % 100))
[[ $locked == "$DUCKDB_VERSION" ]] ||
  die "Cargo.lock wants DuckDB $locked, this script pins $DUCKDB_VERSION: update DUCKDB_*"

start=$SECONDS

step "Building the panel (web/dist)"
cd "$repo/web"
# This machine's pnpm may fail to switch to the pinned version: fall back to npx.
if pnpm --version >/dev/null 2>&1; then
  pnpm=(pnpm)
else
  pnpm=(npx -y "pnpm@$(sed -n 's/.*"packageManager": "pnpm@\(.*\)".*/\1/p' package.json)")
fi
"${pnpm[@]}" install --frozen-lockfile
./node_modules/.bin/vp build
cd "$repo"

step "Preparing the build image ($IMAGE)"
docker build --tag "$IMAGE" --file "$repo/scripts/release/Dockerfile.build" \
  --build-arg "DUCKDB_VERSION=$DUCKDB_VERSION" \
  --build-arg "DUCKDB_SHA256_ARM64=$DUCKDB_SHA256_ARM64" \
  --build-arg "DUCKDB_SHA256_AMD64=$DUCKDB_SHA256_AMD64" \
  "$repo/scripts/release"

# A release is built from a commit: the archive must match the source it is dated from.
if [ "${BRUME_ALLOW_DIRTY:-}" != 1 ] && ! git -C "$repo" diff --quiet HEAD -- crates web Cargo.toml Cargo.lock; then
  die "uncommitted changes in crates/, web/ or the manifests: commit them, or set BRUME_ALLOW_DIRTY=1 for a test build"
fi
mkdir -p "$dist"
source_date=$(git -C "$repo" log -1 --format=%ct)
for arch in "${archs[@]}"; do
  case $arch in
    arm64) triple=aarch64-unknown-linux-gnu ;;
    amd64) triple=x86_64-unknown-linux-gnu ;;
  esac
  archive=brume-$version-linux-$arch.tar.gz
  step "Compiling $archive"
  rm -f "$dist/$archive"
  # The rpath $ORIGIN makes brume load the libduckdb.so beside it.
  docker run --rm \
    --volume "$repo:/src:ro" \
    --volume "$dist:/dist" \
    --volume "$REGISTRY_VOLUME:/usr/local/cargo/registry" \
    --volume "$TARGET_VOLUME:/target" \
    --env CARGO_TARGET_DIR=/target \
    --env "DUCKDB_LIB_DIR=/opt/duckdb/$arch" \
    --env "DUCKDB_INCLUDE_DIR=/opt/duckdb/$arch" \
    --env "CARGO_TARGET_$(tr 'a-z-' 'A-Z_' <<<"$triple")_RUSTFLAGS=-C link-arg=-Wl,-rpath,\$ORIGIN" \
    --env "TRIPLE=$triple" --env "ARCH=$arch" --env "ARCHIVE=$archive" \
    --env "SOURCE_DATE_EPOCH=$source_date" \
    --workdir /src \
    "$IMAGE" bash -euo pipefail -c '
      cargo build --release --locked --package brume --target "$TRIPLE"
      stage=$(mktemp -d)
      mkdir "$stage/brume"
      install -m 0755 "/target/$TRIPLE/release/brume" "$stage/brume/brume"
      install -m 0644 "/opt/duckdb/$ARCH/libduckdb.so" "$stage/brume/libduckdb.so"
      tar --create --sort=name --owner=0 --group=0 --numeric-owner \
        --mtime="@$SOURCE_DATE_EPOCH" --directory "$stage" brume |
        gzip -9 --no-name >"/dist/$ARCHIVE"
    '
done

step "Writing install.sh and SHA256SUMS"
cd "$dist"
# Every archive of this version in dist/, so arm64 and amd64 can be built one after the other.
archives=(brume-"$version"-linux-*.tar.gz)
sum_of() {
  if [[ -f $1 ]]; then sha256 "$1" | cut -d ' ' -f 1; fi
}
sed -e "s/@VERSION@/$version/g" \
  -e "s/@SHA256_ARM64@/$(sum_of "brume-$version-linux-arm64.tar.gz")/g" \
  -e "s/@SHA256_AMD64@/$(sum_of "brume-$version-linux-amd64.tar.gz")/g" \
  "$repo/scripts/release/install.sh.in" >install.sh
chmod 0755 install.sh
! grep -n '@[A-Z0-9_]*@' install.sh || die "install.sh still has a placeholder"
sh -n install.sh
sha256 "${archives[@]}" install.sh >SHA256SUMS

step "Done in $((SECONDS - start)) s"
ls -l "$dist"
cat SHA256SUMS
echo "Test it: scripts/release/test.sh $version"
