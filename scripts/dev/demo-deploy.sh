#!/usr/bin/env bash
# Deploys the guestbook of examples/demo on a Brume host through its API, as the panel does,
# then checks it end to end:
#   - the shared secret POSTGRES_PASSWORD, and the services db, api and web;
#   - the three services online;
#   - a message written and read back through web, from a container on the environment's
#     network;
#   - db redeployed: the message is still there (its volume survived);
#   - the logs of the three services, read from the panel's live log stream.
#
# Usage, on the host, as root (it reads the containers with docker):
#   sudo scripts/dev/demo-deploy.sh [<login link or token>] [--clean]
#     [--domain <subdomain> | --no-domain]
#
# Without a link, it asks `brume admin login` for one. A second run reuses what the first one
# created and deploys only what differs. --clean deletes the three services and the shared
# secret first, the database with them. --domain gives web the public domain
# <subdomain>.<zone>, which needs `brume install --domain <zone>`; --no-domain removes it.
# Without either, web's domain is left as it is. Needs curl and jq (`apt-get install -y jq`).
set -euo pipefail
# A failure inside $(…) stops the script too.
shopt -s inherit_errexit

API=http://127.0.0.1:7878
DB_IMAGE=postgres:17
API_IMAGE=public.ecr.aws/c6e9e0x1/brume-demo-api:0.2
WEB_IMAGE=public.ecr.aws/c6e9e0x1/brume-demo-web:0.2
DATABASE_URL='postgres://postgres@db:5432/postgres?sslmode=disable'
CURL_IMAGE=curlimages/curl:8.17.0
# How long the three services get to be online after a deploy, in seconds.
ONLINE_TIMEOUT=300

die() {
  echo "demo-deploy: $*" >&2
  exit 1
}

step() {
  printf '\n==> %s\n' "$*"
}

command -v jq >/dev/null || die "jq is missing: apt-get install -y jq"

clean=0
link=
# web's domain setting, as JSON: a subdomain, "" to remove it; unset to leave it.
domain=
while [[ $# -gt 0 ]]; do
  case $1 in
    --clean) clean=1 ;;
    --domain)
      [[ $# -ge 2 ]] || die "--domain needs a subdomain, such as --domain demo"
      domain=$(jq -n --arg sub "$2" '$sub')
      shift
      ;;
    --domain=*) domain=$(jq -n --arg sub "${1#--domain=}" '$sub') ;;
    --no-domain) domain='""' ;;
    -*) die "unknown option: $1" ;;
    *) link=$1 ;;
  esac
  shift
done
[[ $(id -u) == 0 ]] || die "run it as root, with sudo: it reads the containers with docker"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
jar=$tmp/cookies
start=$SECONDS

# api METHOD PATH [BODY_FILE]: calls the API with the session, prints the body, fails on a
# status other than 2xx. The body goes in a file, never on the command line: it may hold a
# secret.
api() {
  local method=$1 path=$2 body=${3-}
  local args=(-sS -o "$tmp/answer" -w '%{http_code}' -X "$method" -b "$jar" -c "$jar"
    -H 'X-Brume: 1')
  if [[ -n $body ]]; then
    args+=(-H 'Content-Type: application/json' --data-binary "@$body")
  fi
  local status
  status=$(curl "${args[@]}" "$API$path") || die "$method $path: the API is unreachable"
  [[ $status == 2* ]] || die "$method $path answered $status: $(cat "$tmp/answer")"
  cat "$tmp/answer"
}

# post PATH JSON: a POST whose body is not secret.
post() {
  printf '%s' "$2" >"$tmp/body.json"
  api POST "$1" "$tmp/body.json"
}

step "Logging in"
if [[ -z $link ]]; then
  link=$(brume admin login) || die "brume admin login failed"
fi
token=${link##*token=}
token=${token%%[[:space:]]*}
[[ -n $token ]] || die "no token in the login link"
jq -n --arg token "$token" '{token: $token}' >"$tmp/login.json"
api POST /api/session "$tmp/login.json" >/dev/null
echo "Logged in as $(api GET /api/session | jq -r .admin)"

# The environment of a previous run: the one with a live service db, api or web. Else an
# environment of a project without any service, as --clean leaves it. Else a new project.
find_environment() {
  local services environments
  services=$(api GET /api/services)
  environments=$(api GET /api/environments)
  jq -rn --argjson services "$services" --argjson environments "$environments" '
    ([$services[] | select(.slug == "db" or .slug == "api" or .slug == "web")
      | .environmentId] | first) //
    ([$environments[] | select(.name == "production")
      | .projectId as $project
      | [$environments[] | select(.projectId == $project) | .id] as $ids
      | select(all($services[]; .environmentId as $env | all($ids[]; . != $env)))
      | .id] | first) // empty'
}

service_id() {
  api GET /api/services |
    jq -r --arg env "$environment" --arg slug "$1" \
      'first(.[] | select(.environmentId == $env and .slug == $slug) | .id) // empty'
}

shared_secret_id() {
  api GET /api/variables |
    jq -r --arg env "$environment" 'first(.[] | select(.level == "environment"
      and .ownerId == $env and .name == "POSTGRES_PASSWORD") | .id) // empty'
}

environment=$(find_environment)

if [[ $clean == 1 && -n $environment ]]; then
  step "Deleting the services and the secret of the previous run"
  for slug in web api db; do
    id=$(service_id "$slug")
    if [[ -n $id ]]; then
      api DELETE "/api/services/$id" >/dev/null
      echo "Deleted $slug"
    fi
  done
  secret=$(shared_secret_id)
  if [[ -n $secret ]]; then
    api DELETE "/api/variables/$secret" >/dev/null
    echo "Deleted the shared secret POSTGRES_PASSWORD"
  fi
fi

if [[ -z $environment ]]; then
  step "Creating the project"
  project=$(post /api/projects '{"kind":"empty"}' | jq -r .projectId)
  environment=$(api GET /api/environments |
    jq -r --arg project "$project" \
      'first(.[] | select(.projectId == $project and .name == "production") | .id)')
  echo "Project $project, environment production $environment"
else
  echo "Environment $environment"
fi

step "The shared secret POSTGRES_PASSWORD"
if [[ -n $(shared_secret_id) ]]; then
  # Postgres reads it at its first start only: a new value would lock the API out.
  echo "Already set: kept"
else
  (
    umask 077
    jq -n --arg env "$environment" --arg value "$(head -c 24 /dev/urandom | base64 | tr -d '+/=')" \
      '{level: "environment", ownerId: $env, name: "POSTGRES_PASSWORD", kind: "secret",
        value: $value}' >"$tmp/secret.json"
  )
  api POST /api/variables "$tmp/secret.json" >/dev/null
  rm -f "$tmp/secret.json"
  echo "Set, random"
fi

step "The services"
# ensure_service SLUG IMAGE: creates the service if the environment lacks it. Creating it
# deploys it at once.
ensure_service() {
  local id
  id=$(service_id "$1")
  if [[ -z $id ]]; then
    id=$(post "/api/environments/$environment/services" \
      "$(jq -nc --arg image "$2" --arg name "$1" '{image: $image, name: $name}')" | jq -r .id)
    echo "Created $1 ($2)" >&2
  fi
  echo "$id"
}
db=$(ensure_service db "$DB_IMAGE")
api_service=$(ensure_service api "$API_IMAGE")
web=$(ensure_service web "$WEB_IMAGE")

# stage_setting SERVICE FIELD VALUE_JSON: stages the setting if it differs. An empty text
# removes a setting, which then reads null.
stage_setting() {
  local current
  current=$(api GET "/api/services/$1/settings" | jq -c --arg field "$2" \
    '.[$field] | if . == null then "" else . end')
  if [[ $current != "$3" ]]; then
    post /api/changes "$(jq -nc --arg service "$1" --arg field "$2" --argjson value "$3" \
      '{type: "setting", serviceId: $service, field: $field, value: $value}')" >/dev/null
    echo "Staged $2 = $3"
  fi
}

# stage_variable SERVICE NAME KIND VALUE_JSON: stages the service variable if it is missing
# or differs. A null secret references the shared secret of the same name.
stage_variable() {
  local current
  current=$(api GET /api/variables | jq -c --arg service "$1" --arg name "$2" \
    'first(.[] | select(.level == "service" and .ownerId == $service and .name == $name)
      | {kind, value, reference}) // null')
  local wanted
  wanted=$(jq -nc --arg kind "$3" --argjson value "$4" \
    '{kind: $kind, value: $value, reference: ($kind == "secret" and $value == null)}')
  if [[ $current != "$wanted" ]]; then
    post /api/changes "$(jq -nc --arg service "$1" --arg name "$2" --arg kind "$3" \
      --argjson value "$4" \
      '{type: "variable", serviceId: $service, name: $name, kind: $kind, value: $value}')" \
      >/dev/null
    echo "Staged the $3 variable $2"
  fi
}

stage_setting "$db" port 5432
stage_variable "$db" POSTGRES_PASSWORD secret null
stage_setting "$api_service" port 8080
stage_setting "$api_service" healthcheckPath '"/healthz"'
stage_variable "$api_service" DATABASE_URL plain "$(jq -n --arg url "$DATABASE_URL" '$url')"
stage_variable "$api_service" POSTGRES_PASSWORD secret null
stage_setting "$web" port 8080
stage_setting "$web" healthcheckPath '"/healthz"'
if [[ -n $domain ]]; then
  stage_setting "$web" domain "$domain"
fi

staged=$(api GET /api/changes | jq --arg env "$environment" '[.[] | select(.environmentId == $env)] | length')
if [[ $staged -gt 0 ]]; then
  # The engine refuses a deploy while one of the environment's services is deploying, as the
  # services just created are.
  step "Waiting for the deployments in progress"
  deadline=$((SECONDS + ONLINE_TIMEOUT))
  while api GET /api/deployments | jq -e --arg ids "$db $api_service $web" \
    'any(.[]; (.serviceId as $id | $ids | contains($id))
      and (.status == "building" or .status == "deploying"))' >/dev/null; do
    [[ $SECONDS -lt $deadline ]] || die "a deployment is still in progress"
    sleep 2
  done
  step "Deploying $staged staged changes"
  post "/api/environments/$environment/changes/deploy" '{}' | jq -c .
fi

# wait_online: waits until db, api and web are online on their latest deployment.
wait_online() {
  local deadline=$((SECONDS + ONLINE_TIMEOUT)) last='' now
  while :; do
    now=$(jq -rn --argjson services "$(api GET /api/services)" \
      --argjson deployments "$(api GET /api/deployments)" \
      --arg ids "$db $api_service $web" '
      [$ids | split(" ")[] as $id
        | ($services[] | select(.id == $id)) as $service
        | ([$deployments[] | select(.serviceId == $id)] | sort_by(.createdAt) | last) as $last
        | "\($service.slug)=\($service.status)/\($last.status // "none")"] | join(" ")')
    if [[ $now != "$last" ]]; then
      printf '[%3ds] %s\n' $((SECONDS - start)) "$now"
      last=$now
    fi
    if [[ $now != *failed* && $(grep -o '=online/active' <<<"$now" | wc -l) == 3 ]]; then
      return 0
    fi
    if [[ $now == *failed* || $SECONDS -ge $deadline ]]; then
      echo "The last deployments of the three services:" >&2
      api GET /api/deployments | jq -r --arg ids "$db $api_service $web" '
        group_by(.serviceId)[] | select(.[0].serviceId as $id | $ids | contains($id))
        | sort_by(.createdAt) | last | "\(.serviceId) \(.id) \(.status)"' >&2
      return 1
    fi
    sleep 2
  done
}

step "Waiting for db, api and web to be online"
wait_online || die "the services are not online"

# The environment's network, from the web container.
web_container=$(docker ps --filter "label=brume.service=$web" --format '{{.Names}}' | head -n 1)
[[ -n $web_container ]] || die "no container for web"
network=$(docker inspect --format '{{range $name, $_ := .NetworkSettings.Networks}}{{$name}} {{end}}' \
  "$web_container" | tr ' ' '\n' | grep '^brume-env-' | head -n 1)
[[ -n $network ]] || die "the web container is on no brume-env network"

# guestbook ARGS...: curl in a throwaway container on the environment's network.
guestbook() {
  docker run --rm --network "$network" --cap-drop ALL --security-opt no-new-privileges \
    "$CURL_IMAGE" -sS --max-time 10 "$@"
}

step "Writing and reading a message through web ($network)"
docker pull -q "$CURL_IMAGE" >/dev/null
message="hello from demo-deploy $(date -u +%Y-%m-%dT%H:%M:%SZ)"
body=$(jq -nc --arg text "$message" '{name: "demo-deploy", text: $text}')
guestbook -f -X POST -H 'Content-Type: application/json' -d "$body" http://web:8080/api/messages |
  jq -c . || die "the POST through web failed"
guestbook -f http://web:8080/api/messages | jq -e --arg text "$message" \
  'any(.[]; .text == $text)' >/dev/null || die "the message is not in the guestbook"
echo "Read back: $message"

step "Redeploying db: the message must survive"
post "/api/services/$db/redeploy" '{}' | jq -c '{id, status}'
wait_online || die "the services are not online after the redeploy of db"
# The API reconnects to the new db on its next request.
deadline=$((SECONDS + 60))
until guestbook -f http://web:8080/api/messages 2>/dev/null |
  jq -e --arg text "$message" 'any(.[]; .text == $text)' >/dev/null; do
  [[ $SECONDS -lt $deadline ]] || die "the message is lost after the redeploy of db"
  sleep 2
done
echo "Still there: $message"

step "The logs of the three services, from the panel's live stream"
deadline=$((SECONDS + 60))
while :; do
  # The stream sends the history, then follows: curl reads it for a few seconds, then gives up.
  status=$(curl -s -N --max-time 5 -o "$tmp/logs" -w '%{http_code}' -b "$jar" \
    -H 'Accept: text/event-stream' "$API/api/environments/$environment/logs") || true
  [[ $status == 200 ]] || die "the log stream answered $status: $(cat "$tmp/logs")"
  sed -n 's/^data: //p' "$tmp/logs" | jq -s 'add // []' >"$tmp/lines.json"
  counts=$(jq -r --arg ids "$db $api_service $web" '
    . as $lines | [$ids | split(" ")[] as $id | [$lines[] | select(.serviceId == $id)] | length]
    | join(" ")' "$tmp/lines.json")
  read -r db_lines api_lines web_lines <<<"$counts"
  if [[ $db_lines -gt 0 && $api_lines -gt 0 && $web_lines -gt 0 ]]; then
    break
  fi
  [[ $SECONDS -lt $deadline ]] ||
    die "missing logs: db $db_lines lines, api $api_lines, web $web_lines"
  sleep 3
done
for pair in "db:$db" "api:$api_service" "web:$web"; do
  slug=${pair%%:*}
  id=${pair#*:}
  echo "-- $slug: $(jq --arg id "$id" '[.[] | select(.serviceId == $id)] | length' "$tmp/lines.json") lines, the last ones:"
  jq -r --arg id "$id" '[.[] | select(.serviceId == $id)] | .[-3:][]
    | "   \(.timestamp) \(.level) \(.message)"' "$tmp/lines.json"
done

printf '\nThe demo runs, checked in %ds.\n' $((SECONDS - start))
sub=$(api GET "/api/services/$web/settings" | jq -r '.domain // empty')
if [[ -n $sub ]]; then
  echo "Public: https://$sub.$(api GET /api/ingress | jq -r .zone)/"
fi
