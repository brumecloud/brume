#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "${ROOT_DIR}/scripts/build-metadata.sh"
brume_prepare_build "${ROOT_DIR}"

PROFILE="debug"
TARGET=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) PROFILE="release"; shift ;;
    --target) TARGET="$2"; shift 2 ;;
    *) echo "error: unknown argument $1" >&2; exit 1 ;;
  esac
done

cd "${ROOT_DIR}/renderer"
"${BUN_BIN:-bun}" install --frozen-lockfile
"${BUN_BIN:-bun}" run typecheck
"${BUN_BIN:-bun}" run build:web

CARGO_ARGS=(build --locked --package brume-server)
[[ "${PROFILE}" == "release" ]] && CARGO_ARGS+=(--release)
[[ -n "${TARGET}" ]] && CARGO_ARGS+=(--target "${TARGET}")
BRUME_RENDERER_DIST="${ROOT_DIR}/renderer/dist" cargo "${CARGO_ARGS[@]}"
