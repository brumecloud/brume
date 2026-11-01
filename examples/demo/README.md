# brume demo: a guestbook

A small app to deploy on Brume: a static page served by nginx, an API, and Postgres. The page lists the latest messages and has a form to add one. The jalon 2 end-to-end test deploys it from public images.

```
browser ──> web (nginx :8080) ──/api/──> api (Go :8080) ──> db (postgres:17 :5432)
```

| Folder | What it is |
|---|---|
| [`api/`](api/) | The API, in Go. Static binary on `distroless/static`, runs as UID 65532. |
| [`web/`](web/) | `index.html` and its nginx config, on `nginx-unprivileged`, runs as UID 101. No external assets. |
| [`compose.yaml`](compose.yaml) | Local test only. Brume does not read it. |

## Run it locally

```sh
docker compose up --build
```

Open <http://127.0.0.1:8080>, or:

```sh
curl -X POST -H 'Content-Type: application/json' \
  -d '{"name":"ada","text":"hello"}' http://127.0.0.1:8080/api/messages
curl http://127.0.0.1:8080/api/messages
```

Compose runs the services at least as strictly as Brume does: one network, no published port but the local one, no start order, Brume's capabilities and `no-new-privileges` for `db`, and no capability at all and a read-only root filesystem for `api` and `web`. Postgres' first start can take up to half a minute; until then the API answers 503 and logs each attempt.

`docker compose down -v` removes everything, the database included.

## API

| Request | Answer |
|---|---|
| `GET /api/messages` | 200, the 50 latest messages, newest first: `[{"id","name","text","created_at"}]` |
| `POST /api/messages` with `{"name","text"}` | 201 and the message. 400 if `name` is not 1 to 64 characters, `text` not 1 to 500, a field is unknown, or the body is over 16 KiB. |
| `GET /healthz` | 200 `ok` once the table exists, 503 before |

Both trim the spaces around `name` and `text`. The guestbook is public: it keeps its last 1000 messages, and nginx accepts 2 posts a second overall (bursts of 20), answering 429 beyond. Until Postgres answers, `/api/` answers 503 `{"error":"the database is not ready yet"}`.

## Deploy it on Brume

Three services in one environment. Their slugs must be `db` and `api`: the API reaches `db` by that name, and nginx proxies `/api/` to `http://api:8080`. The frontend's slug is free; jalon 2 uses `web`.

| Service | Image | Variables | Port | Healthcheck | Volume |
|---|---|---|---|---|---|
| `db` | `postgres:17` | `POSTGRES_PASSWORD`: a shared secret | 5432 | the container keeps running | `/var/lib/postgresql/data` |
| `api` | `public.ecr.aws/c6e9e0x1/brume-demo-api:0.2` | `DATABASE_URL`: `postgres://postgres@db:5432/postgres?sslmode=disable`, a plain variable; `POSTGRES_PASSWORD`: the secret shared with `db` | 8080 | `GET /healthz` | none |
| `web` | `public.ecr.aws/c6e9e0x1/brume-demo-web:0.2` | none | 8080 | `GET /healthz` | none |

The domain goes on `web`. `api` and `db` stay inside the environment's network.

The API's `/healthz` turns 200 only once Postgres answers. If `db` starts much later than `api`, give the probe enough time, or deploy `db` first. nginx's `/healthz` does not depend on the API: `web` starts and stays healthy without it, and answers 502 `{"error":"the api is unreachable"}` on `/api/` meanwhile. nginx resolves `api` again every 10 seconds, so it follows a redeployed API.

### What the containers need

- **No capability.** Both run as non-root users and listen on 8080. They work with `--cap-drop ALL` and `--security-opt no-new-privileges`.
- **No writable root filesystem.** Both run with `--read-only`. The API writes nothing. nginx writes its pid file, and the temporary files it never needs in practice, to `/dev/shm`, the tmpfs Docker gives every container. It fails to start with `--ipc none`, which removes `/dev/shm`.
- **No volume, no tmpfs.**
- The API logs one line per request, and at startup `listening on :8080`, each attempt to reach Postgres, then `connected to postgres at db:5432/postgres, table messages ready`. It never logs `DATABASE_URL`. nginx logs one line per request, and its errors.

## Images

Public on Amazon ECR Public, for `linux/amd64` and `linux/arm64`, in the registry `public.ecr.aws/c6e9e0x1` of Brume's AWS account:

| Image | Digest of the multi-arch index |
|---|---|
| `public.ecr.aws/c6e9e0x1/brume-demo-api:0.2` | `sha256:69979983362420ee13f5757b07ecf51b5c65c289b01d1df8e805a7a0b4da2a46` |
| `public.ecr.aws/c6e9e0x1/brume-demo-web:0.2` | `sha256:59a09894f91a0d280fa07a02bbc10f89a43d2dd82b575faddd0c9f52c76c8abd` |

ECR Public throttles anonymous pulls, to about one per second per IP address. Several pulls in a row can get `toomanyrequests: Rate exceeded`; the next try a few seconds later passes.

To publish a new version, with the `assume-role` AWS profile:

```sh
aws ecr-public get-login-password --region us-east-1 --profile assume-role \
  | docker login --username AWS --password-stdin public.ecr.aws
docker buildx create --name brume-demo --driver docker-container --bootstrap
for s in api web; do
  docker buildx build --builder brume-demo --platform linux/amd64,linux/arm64 \
    --provenance=false --sbom=false \
    -t public.ecr.aws/c6e9e0x1/brume-demo-$s:<version> --push ./$s
done
docker logout public.ecr.aws
```

The API compiles for both platforms on the build machine's own, without emulation. The base images are pinned by digest in the Dockerfiles.
