# Working on temet-work

temet-work runs groups of coding-agent sessions through tickets, from refining to merged
and deployed code. How the code is divided and why: `docs/architecture.md`. Read it
before you change code.

## Commands

| What | Command |
|---|---|
| Test the crate you changed | `cargo nextest run -p <crate>` |
| Test the workspace | `cargo nextest run --workspace --all-features` |
| Check every mix of parts | `scripts/check-mixes` |
| Check the test time budget | `cargo nextest run --workspace --all-features --profile ci && scripts/test-budget` |
| Lint | `cargo clippy --workspace --all-features --all-targets -- -D warnings` |
| Licences and advisories | `cargo deny check` |
| Check the desktop app, after a change to `crates/core` or `crates/style` | `(cd app && cargo clippy --all-targets -- -D warnings && cargo nextest run)` |

Test the crate you changed as you work. Before you commit, run the workspace tests,
`scripts/check-mixes` and `scripts/test-budget`. A change to `crates/core`
(temet-work-core) or `crates/style` (temet-work-style) also runs
`(cd app && cargo clippy --all-targets -- -D warnings && cargo nextest run)`: the app is a
workspace of its own that uses both crates, so the workspace tests do not build it, and CI's
`app` job fails the pull request when the app breaks. When that run changes
`app/Cargo.lock`, commit it, since the job builds with `--locked`.

## Rules

- This repository has been public since its first commit, so no private material goes
  into it. Turn the hooks on once per checkout: `git config core.hooksPath scripts/hooks`.
- Test data uses made-up names, never a real organisation, person, project or ticket.
  Take them from Terry Pratchett's Discworld (`Sam Vimes`, `ankh-morpork/watch`,
  `WATCH-12`); hosts are `example.com` or `.invalid`.
- The core names no tool or service and does no I/O. A part depends on the core only; an
  adapter depends on its family crate only. Only a front end uses a UI library.
  `tests/parts.rs` checks each of these.
- A test that fails in a crate you did not touch means your change coupled the two. Fix
  the coupling; never re-record the other crate's snapshots.
- A test takes milliseconds. Over 2 s is flagged, over 20 s is killed, and a crate's suite
  over 10 s fails CI.
- Nothing waits on a person. When a choice stops you, take the default you would defend,
  record it as a decision with your reason, and finish your step. Do not stop to ask for
  approval or wait for an answer.
- A real run goes on a second `temet-workd` started from your branch. Its `--config`,
  `--log`, `TEMET_WORK_SOCKET` and every folder it makes, its work folders too, go under
  `target/real-run/` in the checkout or work folder you work in; remove that folder before
  you start one. Git ignores `target/`, so a real run is no change that keeps a reader's
  checkout, and it goes when the checkout is removed. The second daemon's own reader
  checkouts, which it puts beside its work folders, end up inside it too. Never touch the
  live `temet-workd` service or `~/.config/temet-work/config.toml`.
- After a rebase, push your own branch with `git push --force-with-lease`.
- A change to what temet-work stores or sends updates `docs/privacy.md` in the same
  commit.
