#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BUN_BIN="${BUN_BIN:-$(command -v bun || true)}"
export BRUME_BIND="127.0.0.1:18080"
export BRUME_PUBLIC_URL="http://127.0.0.1:18080"
export BRUME_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/brume"
export BRUME_STORAGE_BACKEND="filesystem"
export BRUME_STORAGE_PATH="${ROOT_DIR}/.brume/e2e-storage"
export BRUME_GITHUB_CLIENT_ID="e2e-client"
export BRUME_GITHUB_CLIENT_SECRET="e2e-secret"

if [[ -z "${BUN_BIN}" ]]; then
  echo "error: Bun is required" >&2
  exit 1
fi

cd "${ROOT_DIR}"
docker compose up -d --wait postgres
mkdir -p "${ROOT_DIR}/.brume"
"${ROOT_DIR}/scripts/build-renderer.sh"
BRUME_RENDERER_DIST="${ROOT_DIR}/renderer/dist" cargo build --package brume-server --package brume-cli

"${ROOT_DIR}/target/debug/brume-server" serve >"${ROOT_DIR}/.brume/e2e-server.log" 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "${SERVER_PID}" 2>/dev/null || true
  wait "${SERVER_PID}" 2>/dev/null || true
}
trap cleanup EXIT

for _ in {1..60}; do
  if curl --fail --silent "${BRUME_PUBLIC_URL}/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl --fail --silent "${BRUME_PUBLIC_URL}/health" >/dev/null

TOKEN="$("${ROOT_DIR}/target/debug/brume-server" create-dev-token --github-id 1 --login e2e)"
BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_PUBLIC_URL}" \
  plan deploy "${ROOT_DIR}/fixtures/example-plan" \
  --slug e2e-plan \
  --visibility private

(
  cd "${ROOT_DIR}/bruno"
  bru run . \
    --env Local \
    --env-var "token=${TOKEN}" \
    --env-var "base_url=${BRUME_PUBLIC_URL}" \
    --env-var "plan=e2e-plan" \
    --bail
)
