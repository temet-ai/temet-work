# Working on temet-work

temet-work runs groups of coding-agent sessions through tickets, from refining to merged
and deployed code. How the code is divided and why: `docs/architecture.md`. Read it
before adding a crate.

## Commands

| What | Command |
|---|---|
| Test the crate you changed | `cargo nextest run -p <crate>` |
| Test the workspace | `cargo nextest run --workspace --all-features` |
| Check every mix of parts | `scripts/check-mixes` |
| Check the test time budget | `cargo nextest run --workspace --all-features --profile ci && scripts/test-budget` |
| Lint | `cargo clippy --workspace --all-features --all-targets -- -D warnings` |
| Licences and advisories | `cargo deny check` |

Test the crate you changed as you work. Before you commit, run the workspace tests,
`scripts/check-mixes` and `scripts/test-budget`.

## Rules

- The core names no tool or service and does no I/O. A part depends on the core only; an
  adapter depends on its family crate only. Only a front end uses a UI library.
  `tests/parts.rs` checks each of these.
- A test that fails in a crate you did not touch means your change coupled the two. Fix
  the coupling; never re-record the other crate's snapshots.
- A test takes milliseconds. Over 2 s is flagged, over 20 s is killed, and a crate's suite
  over 10 s fails CI.
- Turn the hooks on once per checkout: `git config core.hooksPath scripts/hooks`. Every
  commit here is public.
