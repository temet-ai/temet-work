# What temet-work stores and sends

<!-- Reference, not reassurance: one plain sentence per fact, no stacked "never"/"only". -->

## What it sends, and where

| To | What | When |
|---|---|---|
| GitHub, through `gh` | Reads issues, milestones, pull requests and their CI; comments on and closes issues; merges pull requests with `--squash` | Issues every 60 s; a pull request every 30 s while it is in review; comments, closes and merges when the loop calls for them |
| herdr-core, over its local socket | Starts, reads, sends text to and stops agents' terminals, with the environment each agent gets (`SECURITY.md`) | Every 2 s, and with each command |
| Anthropic, through the Claude Code sessions it starts | What Claude Code itself sends | While an agent runs |
| Nothing else | No telemetry. The daemon has no network client of its own. | |

## What it writes, and where

| Folder | What | Moved by |
|---|---|---|
| `~/.config/temet-work/` | `config.toml`, which temet-work reads at start and does not write | `XDG_CONFIG_HOME`, or `temet-workd --config FILE` |
| `~/.local/state/temet-work/` | `events.jsonl`, the event log, which includes the text `gh` returns when it refuses a merge and the output of each verify command | `XDG_STATE_HOME`, or `temet-workd --log FILE` |
| `$XDG_RUNTIME_DIR/temet-work/` | `daemon.sock`, the daemon's socket; it is in `~/.local/state/temet-work/` when `XDG_RUNTIME_DIR` is not set | `TEMET_WORK_SOCKET` |

The Mac uses the same paths.

## Removing it

Uninstall with `brew uninstall temet-ai/temet-work/temet-work`, or with
`mise unuse github:temet-ai/temet-work` for a mise install. Then delete
`~/.config/temet-work/` and `~/.local/state/temet-work/` to remove all its data. The agents
it started run in herdr-core terminals labelled `owner=temet-work` and keep running until
you stop them.
