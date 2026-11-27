# Contributing to temet-work

Thank you for helping. The most useful contributions right now are bug reports from real
use, and small fixes. For anything larger, open an issue first and say what you have in
mind, so we can agree on the shape before you spend time on it.

## Licence

temet-work is MIT licensed. By sending a pull request you agree that your contribution is
licensed under the MIT licence too. There is no contributor licence agreement to sign.

## Building and testing

You need Rust 1.98 or later, [cargo-nextest](https://nexte.st) and, for the licence check,
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny). On Linux the build links with
mold (`pacman -S mold` or `apt install mold`). From the repository root:

```bash
cargo build
cargo nextest run -p <crate>                 # the tests of the crate you changed
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check                             # licences and advisories
```

Tests never touch your desktop or accounts: cargo sets `TEMET_WORK_SANDBOX`, so temet-work opens no
browser, copies nothing, shows no notification and starts no real third-party program
during a test.

## Rules every change keeps

The rules in `AGENTS.md` hold for every change, whether a person or an agent makes it,
including the one on test data: made-up names only, taken from Terry Pratchett's
Discworld.

## Pull requests

Keep a pull request to one change, with tests for it. Say in the description what it does
for someone using temet-work and how you checked it. If a snapshot changes, read its diff
before accepting it.
