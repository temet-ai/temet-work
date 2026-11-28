# Working on temet-work

temet-work runs groups of coding-agent sessions through tickets, from refining to merged
and deployed code. How the code is divided and why: `docs/architecture.md`. Read it
before you change code.

## Commands

| What | Command |
|---|---|
| Test the crate you changed | `cargo nextest run -p <crate>` |
| Test the workspace and its time budget | `cargo nextest run --workspace --all-features --profile ci && scripts/test-budget` |
| Check every mix of parts | `scripts/check-mixes` |
| Lint | `cargo clippy --workspace --all-features --all-targets -- -D warnings` |
| Licences and advisories | `cargo deny check` |
| Check the desktop app, a workspace of its own in `app/` | `(cd app && cargo clippy --all-targets -- -D warnings && cargo nextest run)` |
| See what the desktop app draws, offscreen | `(cd app && cargo run --example shots --features shots -- target/shots)` |

Test the crate you changed as you work. CI runs the rest of the table, apart from the
shots, on every pull request. It builds with `--locked`, so commit a `Cargo.lock` that
changed.

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
- What review finds is fixed in the pull request under review, while the fix is cheap.
  Never file it as a new issue: issues filed on work just done keep the work from ever
  finishing.
- Nothing waits on a person. When a choice stops you, take the default you would defend,
  record it as a decision with your reason, and finish your step. Do not stop to ask for
  approval or wait for an answer.
- A real run goes on a second `temet-workd` started from your branch, with its own
  `--config`, `--log` and `TEMET_WORK_SOCKET`. Never touch the live `temet-workd` service
  or `~/.config/temet-work/config.toml`.
- Remove a worktree or checkout you made once its branch is merged or abandoned, with its
  build folder.
- After a rebase, push your own branch with `git push --force-with-lease`.
- A change to what temet-work stores or sends updates `docs/privacy.md` in the same
  commit.
