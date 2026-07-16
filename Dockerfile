FROM oven/bun:1.3.14 AS renderer

WORKDIR /app/renderer
COPY renderer/package.json renderer/bun.lock ./
RUN --mount=type=cache,target=/root/.bun/install/cache bun install --frozen-lockfile
COPY renderer/ ./
RUN bun run typecheck && bun run build:web

FROM rust:1.97.0-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/
COPY --from=renderer /app/renderer/dist/web/ renderer/dist/web/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release --package brume-server \
    && cp /app/target/release/brume-server /app/brume-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/brume-server /usr/local/bin/brume-server

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/brume-server"]
CMD ["serve"]
