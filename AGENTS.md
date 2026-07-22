# Repository Agent Instructions

## Versioning

- Every new commit must increment the workspace version in `[workspace.package]` in `Cargo.toml`.
- Choose the SemVer increment according to the change: PATCH for backward-compatible fixes, MINOR for backward-compatible functionality, and MAJOR for incompatible API changes.
- Keep every workspace package entry in `Cargo.lock` synchronized with the workspace version.
- Never create a commit whose version is invalid SemVer, unchanged from its parent commit, or lower than its parent commit.

## Build provenance

- Every CLI and server build must embed metadata from the exact latest commit being built.
- The embedded metadata must include the full commit SHA, the commit title from the subject line, and the commit message body.
- Local build scripts must validate the workspace version before building and must obtain provenance directly from Git.
- Builds without Git metadata must receive equivalent provenance through explicit build environment variables.
- Expose the provenance through both `brume version` and the server `/health` response.
