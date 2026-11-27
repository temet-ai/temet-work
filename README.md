# temet-work

temet-work runs groups of coding-agent sessions through tickets: it refines a ticket,
works on it, reviews the change, checks it, merges and deploys it, and takes the next one,
across repositories. Agents decide and carry on; you read their decisions afterwards and
can overturn them.

The daemon, `temet-workd`, runs that loop today for projects on GitHub. It syncs their
issues, starts Claude Code agents in [herdr-core](https://herdr.dev) terminals, merges
pull requests through `gh` (GitLab merge requests through `glab`) and runs each project's
deploy command. The `temet-work` command line and its MCP server send it commands
(`docs/commands.md`). The desktop app (GPUI) and overturning a decision are not built yet;
`docs/architecture.md` lists what works today and what does not.

It is written in Rust. The research behind it is in `docs/research.md`.

## Licence

MIT. See `LICENSE`.
