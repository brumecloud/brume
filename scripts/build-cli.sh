#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "${ROOT_DIR}/scripts/build-metadata.sh"
brume_prepare_build "${ROOT_DIR}"

PROFILE="debug"
TARGET=""
EXPLICIT_TARGET="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) PROFILE="release"; shift ;;
    --target) TARGET="$2"; EXPLICIT_TARGET="true"; shift 2 ;;
    *) echo "error: unknown argument $1" >&2; exit 1 ;;
  esac
done

if [[ -z "${TARGET}" ]]; then
  TARGET="$(rustc -vV | sed -n 's/^host: //p')"
fi

"${ROOT_DIR}/scripts/build-renderer.sh" "${TARGET}"

CARGO_ARGS=(build --locked --package brume-cli)
if [[ "${EXPLICIT_TARGET}" == "true" ]]; then
  CARGO_ARGS+=(--target "${TARGET}")
fi
if [[ "${PROFILE}" == "release" ]]; then
  CARGO_ARGS+=(--release)
fi

BRUME_RENDERER_DIST="${ROOT_DIR}/renderer/dist" cargo "${CARGO_ARGS[@]}"
