# What temet-work stores and sends

<!-- Reference, not reassurance: one plain sentence per fact, no stacked "never"/"only". -->

## Where it sends data

| To | What | When |
|---|---|---|
| GitHub, through `gh` | Reads issues, milestones, pull requests and their CI, and the workflow runs of a merge commit; comments on and closes issues; merges pull requests by the `merge` setting (`--squash` unless set otherwise) | Issues every 60 s; a pull request every 30 s while it is in review, and a merge commit's runs every 30 s while the next repository waits for them; comments, closes and merges when the loop calls for them |
| GitLab, through `glab`, for a project whose `code_host` is `gitlab` | Reads merge requests, their pipelines and jobs, and the pipelines of a merged commit; merges merge requests | A merge request every 30 s while it is in review, and a merged commit's pipelines every 30 s while the next repository waits for them; merges when the loop calls for them |
| Jira Cloud, through `curl`, for a project or area whose tracker is `jira` | Searches the open issues (summary, description, fix versions, links, status) and checks the status of the issues they depend on; comments on issues; runs the transition to Done. It signs in over HTTPS with `JIRA_USER` and `JIRA_TOKEN` from the daemon's environment, gives the token to `curl` on standard input, and does not store either | Issues every 60 s; comments and transitions when the loop calls for them |
| herdr-core, over its local socket | Starts, reads, sends text to and stops agents' terminals, on this machine or on another machine herdr-core knows, with each agent's brief and environment (`SECURITY.md`) | Every 2 s, and with each command |
| Anthropic or OpenAI, through the Claude Code or Codex sessions it starts | What Claude Code or Codex itself sends, including the agent's brief. A kept rule (`docs/commands.md`) is in every brief on every project | While an agent runs |
| The daemon's machine, over your own `ssh`, from an agent on another machine | Each `temet-work` command the agent runs (a plan, a review, a ruling) goes to the daemon's socket through `temet-work relay`, with that machine's `temet-work` build and its name (`uname -n`), and the reply comes back. Nothing listens on a port | While an agent on that machine runs |
| A project's `deploy` and `verify` commands | Whatever those commands send; temet-work runs them with `sh -c` | After a merge, and after a deploy |
| Nothing else | No telemetry. The daemon has no network client of its own. | |

Each agent gets `TEMET_WORK_AGENT`, `TEMET_WORK_DAEMON` when it is on another machine, and
these from the daemon's environment when they are set: `CLAUDE_CONFIG_DIR`, `HTTPS_PROXY`,
`NO_PROXY`, `NODE_EXTRA_CA_CERTS`, `GITHUB_TOKEN` and `GH_TOKEN` (`SECURITY.md`).

## Where it writes files

| Where | What | Moved by |
|---|---|---|
| `~/.config/temet-work/` | `config.toml`, which temet-work reads at start and does not write | `XDG_CONFIG_HOME`, or `temet-workd --config FILE` |
| `~/.local/state/temet-work/` | `events.jsonl`, the event log. It holds the issues, plans, review points, rulings, a person's overturn reasons and the rules made from them, the text `gh` or `glab` returns when it refuses a merge, and the output of each verify command. `deploy-handoff` holds the issue and build of a deploy that restarts the daemon | `XDG_STATE_HOME`, or `temet-workd --log FILE` |
| `$XDG_RUNTIME_DIR/temet-work/` | `daemon.sock`, the daemon's socket. It is in `~/.local/state/temet-work/` when `XDG_RUNTIME_DIR` is not set, as on a Mac | `TEMET_WORK_SOCKET` |
| Standard error | A line per request, `request: by=AGENT COMMAND -> done` or `-> refused: REASON`, and a line per new problem. Under systemd it goes to the journal | |
| `~/.claude.json`, Claude Code's user file | `projects.<folder>.hasTrustDialogAccepted` for each work folder on this machine, so a Claude Code agent does not stop at the folder-trust question. Nothing else in the file changes | `CLAUDE_CONFIG_DIR` |

## Removing it

Uninstall with `cargo uninstall temet-work`. Then delete `~/.config/temet-work/` and
`~/.local/state/temet-work/` to remove its data. The trust entries in `~/.claude.json` stay
until you remove them. The agents it started run in herdr-core terminals labelled
`owner=temet-work` and keep running until you stop them.
