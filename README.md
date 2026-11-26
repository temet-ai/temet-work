# temet-work

temet-work runs groups of coding-agent sessions through tickets: it refines a ticket,
works on it, reviews the change, checks it, merges and deploys it, and takes the next one,
across repositories. Agents decide and carry on; you read their decisions afterwards and
can overturn them.

Nothing runs yet. The workspace, its crate boundaries and the tests that enforce them
exist. The research behind it is in `docs/research.md`, and the design of the code in
`docs/architecture.md`.

It is written in Rust. Agents run in [herdr-core](https://herdr.dev) terminals or tmux,
and the desktop app will use GPUI.

## Licence

MIT. See `LICENSE`.
