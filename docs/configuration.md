---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Configuring temet-workd

`temet-workd` reads one TOML file at start. It lists the projects whose issues it works
on, the work folders agents use, and the groups of agents.

| What | Where |
|---|---|
| Configuration | `$XDG_CONFIG_HOME/temet-work/config.toml` (`~/.config/temet-work/config.toml`), or `--config FILE` |
| Event log | `$XDG_STATE_HOME/temet-work/events.jsonl` (`~/.local/state/temet-work/events.jsonl`), or `--log FILE` |

The daemon works in rounds, one every two seconds, and prints each new problem to
stderr. `--once` stops after the first round and prints the state as text.

## Example

```toml
# A project: a repository and the tracker its issues come from.
[[project]]
id = "radar"              # short id, used in issue ids such as radar#44
tracker = "github"        # the tracker adapter
name = "temet-ai/radar"   # the tracker's name for it
# code_host = "github"    # where its pull requests are; the tracker when left out
# repo = "temet-ai/radar" # the code host's name for it; name when left out
# deploy = "make deploy"  # run after a merge; merged counts as deployed when left out
# verify = "make smoke"   # run after a deploy; deployed counts as verified when left out
# run_in = "/home/me/src/radar-deploy"   # where both run; the home folder when left out
# ci = false              # its pull requests run no CI; the plan's checks are left to the reviewer

# Settings (docs/settings.md): [settings] is global, [settings.reviewer] is for one role.
# [settings]
# runtime = "claude"
# model = "sonnet"
# [settings.reviewer]
# runtime = "codex"
#
# A project's, an area's and a milestone's own: under the project, the area or [[milestone]].
# [project.settings]
# model = "haiku"
#
# [[milestone]]
# of = "radar"                 # a project or an area
# name = "Slack sign-in rework"
# [milestone.settings]
# model = "opus"

# A work folder: a checkout an author works in, one issue at a time.
[[folder]]
id = "radar-1"
project = "radar"
path = "/home/me/src/radar-1"
# machine = "local"       # the default

[[folder]]
id = "radar-2"
project = "radar"
path = "/home/me/src/radar-2"

# An area: projects that work together (docs/areas.md). When one tracker project
# serves all of them, the area names it and its projects name no tracker of their own.
# [[area]]
# id = "guild"
# name = "Thieves' Guild"
# projects = ["guild-ui", "guild-api"]
# tracker = "jira"
# tracker_project = "GLD"

# A group: agents working through some projects' issues.
[[group]]
id = "group-1"
name = "Group 1"
projects = ["radar"]
# areas = ["guild"]                        # whole areas: their own issues and their projects'
milestones = ["Slack sign-in rework"]   # first to last; leave out for every issue
refining_at_once = 1                    # the default
```

A group takes its projects' open issues in milestone order, then in the tracker's order.
It refines `refining_at_once` issues at a time. It starts an author on a refined issue
when one of the project's folders is free and every issue in the issue's "Depends on"
lines is done. A project with two folders has at most two authors working at once.

Settings at every level, from global down to one task, are in `docs/settings.md`. The
tables `[settings]` (with `[settings.<role>]` inside it), `[project.settings]`,
`[area.settings]` and `[[milestone]]` become `SettingSet` events recorded as set by
`file`, when the daemon reads the file and again whenever it changes. A group's `model`
and a project's `ci`, `deploy`, `verify` and `run_in` are still accepted and are read as
settings at their level, so an existing file behaves as before. A key the daemon does not
know is an error naming the key. A value that names a runtime or runner the build does
not have stops the daemon at start, with the ones it has listed, and so does a build with
no default runtime or runner and no global value for it. What a person or an agent has
set since is not overwritten by the file; a value taken out of the file is unset.

Listing a folder under `[[folder]]` is your choice to let agents work in it. Before an
agent starts in a folder on this machine, the `claude` runtime marks the folder trusted
(`hasTrustDialogAccepted`) in Claude Code's own user file, `.claude.json` in the folder
`CLAUDE_CONFIG_DIR` names, else in your home folder. It changes only that folder's entry
and leaves a folder already trusted alone. A folder on another machine cannot be trusted
from here. An agent stopped at a prompt the daemon cannot answer is listed under Problems
with its name, its folder and the prompt, and the line goes when the agent goes on or ends.

Each project, folder and group in the file becomes an event in the log the first time the
daemon reads it, and again whenever it changes. Removing one from the file does not
remove it from the daemon yet.

## A work folder on another machine

An agent can run on another machine, in a checkout there. Give the folder that machine
and say how the machine reaches the daemon:

```toml
daemon_host = "omit"      # this machine, as the other machine's ssh names it

[[folder]]
id = "radar-mac"
project = "radar"
machine = "mac"           # a machine herdr-core knows (`hc machine add`)
path = "/Users/me/projects/radar"
```

The daemon starts the agent through herdr-core on `machine`, with `TEMET_WORK_DAEMON=ssh://omit` in its
environment (`ssh://omit/PATH/TO/daemon.sock` when this daemon was started with
`TEMET_WORK_SOCKET`, so a second daemon on the same machine is reached too; the path
after the host is the daemon's socket there, and none means its default). Every `temet-work` command the agent runs then goes over
`ssh -o BatchMode=yes omit temet-work relay [--socket PATH]`, which hands the request to the daemon's
local socket. No port is opened and no new secret is made. `BatchMode` makes a missing
key fail with ssh's own error instead of waiting for a password. With no `daemon_host`
the daemon starts no agent on another machine and lists a problem naming `machine` and
`daemon_host`, because the daemon's own name is often not one the other machine can
resolve.

The other machine needs four things, and each has a command that checks it:

| Needs | Check |
|---|---|
| SSH access from the daemon's machine | `ssh -o BatchMode=yes mac true` on the daemon's machine |
| SSH access back to the daemon's machine, under the name in `daemon_host` | `ssh -o BatchMode=yes omit true` on the other machine |
| `temet-work` on the `PATH` a non-interactive SSH session gets | `ssh mac temet-work parts`, and `ssh omit temet-work parts` from there |
| The checkout at `path`, and the agent runtime (such as `claude`) | `ssh mac 'cd /Users/me/projects/radar && git status -sb && command -v claude'` |

On a new machine, start `claude --permission-mode auto` once in the work folder and answer
"Allow this read outside the working directories?" with "Yes, and keep allowing". An unattended
agent stops at that prompt on its first read outside the folder.

A failed hop says which step failed in one line: `could not connect` (ssh, either
direction), `temet-work is not on the other machine's PATH`, or `no temet-work daemon
answers at ...` (the daemon is not running there). A refusal from the daemon is a
refusal, as for a local agent.

To send one command from your own shell the same way: `TEMET_WORK_DAEMON=ssh://omit temet-work status`.

## temet-work deploying itself

A deploy may restart the daemon that runs it, so the command does three things in this
order: build and install, record the handoff, start the restart outside the daemon. A
failed build stops the `&&` chain before anything is stopped, and its last lines are the
issue's problem. The daemon gives the command `TEMET_WORK_HANDOFF`, a file next to its log
(`deploy-handoff`); the command writes one line, `ISSUE BUILD`, into it. The old daemon
then does not record `Deployed`. The new daemon records it, once, when its own build
(`temet-workd --build`, the commit it was built from) is the one in the line, and the
verify command runs after that, once. Agents run in herdr-core terminals, so they keep
running across the restart and the new daemon starts no duplicates.

```toml
[[project]]
id = "temet-work"
tracker = "github"
name = "temet-ai/temet-work"
run_in = "/home/me/src/temet-work-deploy"   # a checkout of its own, never a work folder
deploy = '''
  git pull --ff-only &&
  cargo install --path . --locked --root ~/.local &&
  echo "$TEMET_WORK_ISSUE $(~/.local/bin/temet-workd --build)" >> "$TEMET_WORK_HANDOFF" &&
  systemd-run --user --on-active=3 systemctl --user restart temet-workd
'''
verify = '''
  test "$(~/.local/bin/temet-workd --build)" = "$(git describe --always --dirty)"
'''
```

`systemd-run` starts the restart as its own unit, so the restart is not in the daemon's
control group and is not killed with it. A deploy that handed off but was not followed by
a restart within ten minutes counts as failed and runs again; after three failures it
waits for a person, as any deploy does. If the daemon that starts is not the build the
line names, the issue is listed as a problem with both builds, and deleting its line from
`deploy-handoff` lets the deploy run again.
