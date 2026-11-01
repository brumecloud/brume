# Brume

## Rust

- Follow the [Google Rust style guide](https://google.github.io/styleguide/rustguide.html), with the Brume adaptations in [`archi/base/10-developpement.md`](archi/base/10-developpement.md#style-rust). The security rules in that file take precedence.
- Ignore the Chromium, Android and Fuchsia specific rules. Brume uses Cargo, tokio and `#[test]` / `#[tokio::test]`.
- Format with `cargo +nightly fmt` (`rustfmt.toml` uses unstable options). Build on stable.
- `cargo clippy -- -D warnings` must pass.

## Docs

- The public docs live in [`docs/`](docs/AGENTS.md), a Fumadocs site. Read `docs/AGENTS.md` before writing a page.
- A change to `archi/base/` updates the docs pages that list it in their `sources`, in the same PR.

## Agent workflow

Building jalon 1 took 8 h 11, of which about 3 h waiting on builds and tests and 2 h on reviews that blocked the next task. These rules cut both.

### Plans

- Ask every question in one batch, before the plan is approved. After that, decide and keep going.
- Keep the first, shared task as thin as possible: nothing runs in parallel until it lands.
- Start a task as soon as its real dependencies land, not when the previous phase ends. If a task's first steps do not need the previous task, split them off and start them earlier.
- Aim for delegated tasks of 40 minutes or less.
- At most two Rust tracks at a time on a 16 GB machine, plus the web track.

### Reviews

- Each task gets a review in its own thread, and the branch gets a final review before merge.
- A whole-track review is for risky tracks only, such as the engine.
- Never wait on a review. Start the next task, run the review alongside, and apply its fixes after.

### Builds and tests

- A new worktree builds from scratch, and sccache serves the dependencies. It is enabled in the user's `~/.cargo/config.toml`, not in the repo. With a warm cache, `cargo test --workspace --no-run` plus `cargo clippy` take about 30 s. Do not copy a `target` folder into a worktree: measured, it is slower than the cache.
- Leave `CARGO_BUILD_JOBS` at its default: a full build peaks at about 3.5 GB of RAM.
- While iterating, run only the affected tests: `cargo test -p <crate> <test>`, `vp test run <file>`. Run the full gate once, at the end of a task: `cargo +nightly fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `pnpm run ci` in `web/`.
- End-to-end tests run once, at the end of the milestone, before the final review: `cargo test -p brume --test e2e -- --ignored`, the Docker tests of `brume-agent`, and the walk through the panel in a browser. A task that writes one runs only that one.
- Run a command that can take minutes in the background and wait for its notification, never in a `sleep` or `until` loop. Do not pipe it into `tail`: it hides the progress.
