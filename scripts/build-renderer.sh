#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_TARGET="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
BUN_BIN="${BUN_BIN:-$(command -v bun || true)}"

if [[ -z "${BUN_BIN}" ]]; then
  echo "error: Bun 1.3.14 or newer is required to build the Brume renderer" >&2
  exit 1
fi

case "${RUST_TARGET}" in
  aarch64-apple-darwin) BUN_TARGET="bun-darwin-arm64" ;;
  x86_64-apple-darwin) BUN_TARGET="bun-darwin-x64" ;;
  x86_64-unknown-linux-gnu) BUN_TARGET="bun-linux-x64" ;;
  aarch64-unknown-linux-gnu) BUN_TARGET="bun-linux-arm64" ;;
  x86_64-pc-windows-msvc) BUN_TARGET="bun-windows-x64" ;;
  *)
    echo "error: unsupported renderer target ${RUST_TARGET}" >&2
    exit 1
    ;;
esac

cd "${ROOT_DIR}/renderer"
"${BUN_BIN}" install --frozen-lockfile
"${BUN_BIN}" run typecheck
"${BUN_BIN}" run build:web

OUTPUT_DIR="${ROOT_DIR}/renderer/dist/${RUST_TARGET}"
mkdir -p "${OUTPUT_DIR}"
OUTPUT_FILE="${OUTPUT_DIR}/brume-renderer"
if [[ "${RUST_TARGET}" == "x86_64-pc-windows-msvc" ]]; then
  OUTPUT_FILE="${OUTPUT_FILE}.exe"
fi

"${BUN_BIN}" build \
  --compile \
  --minify \
  --no-compile-autoload-dotenv \
  --no-compile-autoload-bunfig \
  --target="${BUN_TARGET}" \
  src/worker/main.ts \
  --outfile "${OUTPUT_FILE}"

echo "Renderer built for ${RUST_TARGET}: ${OUTPUT_FILE}"

