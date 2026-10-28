# Brume

## Rust

- Follow the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with the Brume adaptations in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). The security rules in that file take precedence.
- Ignore the Chromium, Android and Fuchsia specific rules. Brume uses Cargo, tokio and `#[test]` / `#[tokio::test]`.
- Format with `cargo +nightly fmt` (`rustfmt.toml` uses unstable options). Build on stable.
- `cargo clippy -- -D warnings` must pass.
