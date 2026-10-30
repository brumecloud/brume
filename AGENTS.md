# Brume

## Rust

- Follow the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with the Brume adaptations in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). The security rules in that file take precedence.
- Ignore the Chromium, Android and Fuchsia specific rules. Brume uses Cargo, tokio and `#[test]` / `#[tokio::test]`.
- Format with `cargo +nightly fmt` (`rustfmt.toml` uses unstable options). Build on stable.
- `cargo clippy -- -D warnings` must pass.

## Docs

- The public docs live in [`docs/`](docs/AGENTS.md), a Fumadocs site. Read `docs/AGENTS.md` before writing a page.
- A change to `archi/base/` updates the docs pages that list it in their `sources`, in the same PR.
