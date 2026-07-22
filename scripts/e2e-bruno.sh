#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BUN_BIN="${BUN_BIN:-$(command -v bun || true)}"
export BRUME_BIND="127.0.0.1:18080"
export BRUME_API_PUBLIC_URL="http://api.localhost:18080"
export BRUME_AUTH_PUBLIC_URL="http://auth.localhost:18080"
export BRUME_PLAN_PUBLIC_URL="http://plan.localhost:18080"
export BRUME_PUBLIC_DOMAIN="localhost"
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

PIDS=()
cleanup() {
  for pid in "${PIDS[@]}"; do
    kill "${pid}" 2>/dev/null || true
  done
  for pid in "${PIDS[@]}"; do
    wait "${pid}" 2>/dev/null || true
  done
}
trap cleanup EXIT

"${ROOT_DIR}/target/debug/brume-server" serve >"${ROOT_DIR}/.brume/e2e-server.log" 2>&1 &
SERVER_PID=$!
PIDS+=("${SERVER_PID}")

for _ in {1..60}; do
  if curl --fail --silent "${BRUME_API_PUBLIC_URL}/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl --fail --silent "${BRUME_API_PUBLIC_URL}/health" >/dev/null

TOKEN_PAIR="$("${ROOT_DIR}/target/debug/brume-server" create-dev-token --github-id 1 --login e2e)"
TOKEN="$(TOKEN_PAIR="${TOKEN_PAIR}" "${BUN_BIN}" -e 'console.log(JSON.parse(Bun.env.TOKEN_PAIR).access_token)')"
REFRESH_TOKEN="$(TOKEN_PAIR="${TOKEN_PAIR}" "${BUN_BIN}" -e 'console.log(JSON.parse(Bun.env.TOKEN_PAIR).refresh_token)')"
REFRESH_TEST_PAIR="$("${ROOT_DIR}/target/debug/brume-server" create-dev-token --github-id 2 --login refresh-e2e)"
REFRESH_TEST_TOKEN="$(TOKEN_PAIR="${REFRESH_TEST_PAIR}" "${BUN_BIN}" -e 'console.log(JSON.parse(Bun.env.TOKEN_PAIR).refresh_token)')"
BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  plan deploy "${ROOT_DIR}/fixtures/example-plan" \
  --slug e2e-plan \
  --visibility private
BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  deploy "${ROOT_DIR}/fixtures/example-deployment" \
  --url static-e2e \
  --spa

GENERATED_DEPLOYMENT_OUTPUT="$(BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  deploy "${ROOT_DIR}/fixtures/example-deployment")"
GENERATED_DEPLOYMENT_URL="${GENERATED_DEPLOYMENT_OUTPUT#Deployed }"
GENERATED_DEPLOYMENT_URL="${GENERATED_DEPLOYMENT_URL%/}"

BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" plan list >/dev/null

if BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" tunnel 18081 --url static-e2e \
  >"${ROOT_DIR}/.brume/e2e-tunnel-deployment-conflict.log" 2>&1; then
  echo "error: tunnel unexpectedly claimed an existing deployment URL" >&2
  exit 1
fi

TUNNEL_FIXTURE_PORT=18081 TUNNEL_FIXTURE_NAME=first \
  "${BUN_BIN}" "${ROOT_DIR}/scripts/tunnel-fixture.ts" \
  >"${ROOT_DIR}/.brume/e2e-tunnel-fixture-first.log" 2>&1 &
FIRST_FIXTURE_PID=$!
PIDS+=("${FIRST_FIXTURE_PID}")
TUNNEL_FIXTURE_PORT=18082 TUNNEL_FIXTURE_NAME=second \
  "${BUN_BIN}" "${ROOT_DIR}/scripts/tunnel-fixture.ts" \
  >"${ROOT_DIR}/.brume/e2e-tunnel-fixture-second.log" 2>&1 &
SECOND_FIXTURE_PID=$!
PIDS+=("${SECOND_FIXTURE_PID}")

for _ in {1..60}; do
  if curl --fail --silent "http://127.0.0.1:18081/inspect" >/dev/null \
    && curl --fail --silent "http://127.0.0.1:18082/inspect" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl --fail --silent "http://127.0.0.1:18081/inspect" >/dev/null
curl --fail --silent "http://127.0.0.1:18082/inspect" >/dev/null

BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  tunnel 18081 --url tunnel-e2e \
  >"${ROOT_DIR}/.brume/e2e-tunnel-first.log" 2>&1 &
FIRST_TUNNEL_PID=$!
PIDS+=("${FIRST_TUNNEL_PID}")

for _ in {1..60}; do
  BODY="$(curl --silent "http://tunnel-e2e-e2e.localhost:18080/inspect" || true)"
  if [[ "${BODY}" == *'"instance":"first"'* ]]; then
    break
  fi
  sleep 0.25
done
[[ "${BODY}" == *'"instance":"first"'* ]]

BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  tunnel 18082 --url tunnel-e2e \
  >"${ROOT_DIR}/.brume/e2e-tunnel-second.log" 2>&1 &
SECOND_TUNNEL_PID=$!
PIDS+=("${SECOND_TUNNEL_PID}")

for _ in {1..60}; do
  BODY="$(curl --silent "http://tunnel-e2e-e2e.localhost:18080/inspect" || true)"
  if [[ "${BODY}" == *'"instance":"second"'* ]]; then
    break
  fi
  sleep 0.25
done
[[ "${BODY}" == *'"instance":"second"'* ]]

for _ in {1..60}; do
  if ! kill -0 "${FIRST_TUNNEL_PID}" 2>/dev/null; then
    break
  fi
  sleep 0.25
done
if kill -0 "${FIRST_TUNNEL_PID}" 2>/dev/null; then
  echo "error: replaced tunnel did not exit" >&2
  exit 1
fi

BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" tunnel 18081 \
  >"${ROOT_DIR}/.brume/e2e-tunnel-generated.log" 2>&1 &
GENERATED_TUNNEL_PID=$!
PIDS+=("${GENERATED_TUNNEL_PID}")
for _ in {1..60}; do
  GENERATED_TUNNEL_URL="$(sed -n 's/^Forwarding \(http[^ ]*\) ->.*$/\1/p' \
    "${ROOT_DIR}/.brume/e2e-tunnel-generated.log" | head -n 1)"
  if [[ -n "${GENERATED_TUNNEL_URL}" ]]; then
    break
  fi
  sleep 0.25
done
[[ "${GENERATED_TUNNEL_URL}" =~ ^http://[a-z2-7]{16}-e2e\.localhost:18080$ ]]

if BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  deploy "${ROOT_DIR}/fixtures/example-deployment" --url tunnel-e2e \
  >"${ROOT_DIR}/.brume/e2e-deployment-tunnel-conflict.log" 2>&1; then
  echo "error: deployment unexpectedly claimed an active tunnel URL" >&2
  exit 1
fi

BRUME_TOKEN="${TOKEN}" "${ROOT_DIR}/target/debug/brume" \
  --base-url "${BRUME_API_PUBLIC_URL}" \
  tunnel 18083 --url tunnel-failure \
  >"${ROOT_DIR}/.brume/e2e-tunnel-failure.log" 2>&1 &
FAILED_TUNNEL_PID=$!
PIDS+=("${FAILED_TUNNEL_PID}")

for _ in {1..60}; do
  STATUS="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    "http://tunnel-failure-e2e.localhost:18080/inspect")"
  if [[ "${STATUS}" == "502" ]]; then
    break
  fi
  sleep 0.25
done
[[ "${STATUS}" == "502" ]]

(
  cd "${ROOT_DIR}/bruno"
  bru run . \
    --env Local \
    --env-var "token=${TOKEN}" \
    --env-var "refresh_token=${REFRESH_TOKEN}" \
    --env-var "refresh_test_token=${REFRESH_TEST_TOKEN}" \
    --env-var "base_url=${BRUME_API_PUBLIC_URL}" \
    --env-var "auth_url=${BRUME_AUTH_PUBLIC_URL}" \
    --env-var "plan_url=${BRUME_PLAN_PUBLIC_URL}" \
    --env-var "tunnel_url=http://tunnel-e2e-e2e.localhost:18080" \
    --env-var "failed_tunnel_url=http://tunnel-failure-e2e.localhost:18080" \
    --env-var "offline_url=http://not-running-e2e.localhost:18080" \
    --env-var "deployment_url=http://static-e2e-e2e.localhost:18080" \
    --env-var "generated_deployment_url=${GENERATED_DEPLOYMENT_URL}" \
    --env-var "generated_tunnel_url=${GENERATED_TUNNEL_URL}" \
    --env-var "plan=e2e-plan" \
    --bail
)

"${BUN_BIN}" "${ROOT_DIR}/scripts/check-tunnel-websocket.ts" \
  "ws://tunnel-e2e-e2e.localhost:18080/ws"

kill "${SECOND_TUNNEL_PID}" 2>/dev/null || true
wait "${SECOND_TUNNEL_PID}" 2>/dev/null || true
for _ in {1..60}; do
  STATUS="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    "http://tunnel-e2e-e2e.localhost:18080/inspect")"
  if [[ "${STATUS}" == "404" ]]; then
    break
  fi
  sleep 0.25
done
[[ "${STATUS}" == "404" ]]

echo "Brume backend Bruno suite passed"
