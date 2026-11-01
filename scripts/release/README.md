# Release builds

`build.sh` compiles Brume for Linux and writes the release files in `dist/` (gitignored):

| File | What |
|---|---|
| `brume-<version>-linux-<arch>.tar.gz` | A `brume/` folder: the `brume` binary, with the panel embedded, and `libduckdb.so` |
| `install.sh` | The installer, generated from `install.sh.in`, with the version and the archives' SHA-256s pinned |
| `SHA256SUMS` | The archives and `install.sh` |

## Build

```sh
scripts/release/build.sh 0.1.0-alpha.1 all     # or arm64, or amd64
scripts/release/test.sh 0.1.0-alpha.1 all
```

- The version must be the workspace version of `Cargo.toml`: `brume --version` prints it.
- Needs Docker and Node. The script builds the panel (`pnpm install`, `vp build`), then compiles in `rust:1.92-bookworm`, so the binary needs only the glibc of Debian 12 and runs on Debian 12 and 13 and Ubuntu 24.04 and 26.04.
- The machine's own architecture compiles natively, the other one with a cross gcc. On a Mac with Apple Silicon, amd64 is the cross build.
- `libduckdb.so` is the official build of the DuckDB version that `libduckdb-sys` in `Cargo.lock` binds, checked against the SHA-256s pinned in `build.sh`. The binary finds it beside itself (rpath `$ORIGIN`). After a DuckDB upgrade, `build.sh` refuses to run until its `DUCKDB_*` values match.
- The cargo registry and target folder are the Docker volumes `brume-release-cargo-registry` and `brume-release-target`: a rebuild only compiles what changed. `docker volume rm` them to start from scratch.
- `install.sh` pins every archive of this version present in `dist/`, so arm64 and amd64 can be built one after the other. For a release, build `all`.
- A build refuses uncommitted changes in `crates/`, `web/` or the manifests, so an archive matches the commit it is dated from. `BRUME_ALLOW_DIRTY=1` lifts that for a test build, never for a release.
- The base image is pinned by digest in `Dockerfile.build`.
- The archive also holds `LICENSE.md` (Brume) and `LICENSE-duckdb.txt` (DuckDB's MIT license, for `libduckdb.so`).

## Test `install.sh` locally

`test.sh` runs every check in Docker: `brume --version` and `ldd` in `debian:12` and `ubuntu:24.04`, then `install.sh` in `ubuntu:24.04`, from `dist/` (`BRUME_DOWNLOAD_BASE=file:///dist`), with and without a terminal. It also checks that `install.sh` refuses an archive changed by one byte, a failed download and a non-root user. amd64 runs under emulation.

By hand, in a container:

```sh
docker run --rm -it -v "$PWD/dist:/dist:ro" ubuntu:24.04 sh -c '
  apt-get update -qq && apt-get install -yqq curl >/dev/null
  BRUME_DOWNLOAD_BASE=file:///dist sh /dist/install.sh --admin ssh-tunnel'
```

`install.sh` takes the same arguments as `brume install` and passes them on. In a container, `brume install` stops at its checks: there is no systemd. To go further, use a Linux VM such as the OrbStack machine `brume-e2e`, with `dist/` copied into it.
