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

## Jira

A project whose issues are in Jira Cloud names the tracker `jira` and its project key:

```toml
[[project]]
id = "eng"
tracker = "jira"
name = "ENG"              # the Jira project key
repo = "temet-ai/eng"     # Jira holds no code: name the code host's repository
code_host = "github"
```

An area can name a Jira project the same way with `tracker = "jira"` and `tracker_project = "ENG"`.

The adapter calls the Jira Cloud REST API v3 with `curl`. The site and the sign-in come from
the daemon's environment, never from this file:

| Variable | Holds |
|---|---|
| `JIRA_URL` | the site, `https://your-site.atlassian.net` (it must be `https`) |
| `JIRA_USER` | the email of the account that owns the token |
| `JIRA_TOKEN` | an Atlassian API token |

A missing variable is a problem on the project that names it. Open issues are the project's
issues whose status category is not Done. An issue's first fix version is its milestone, so a
group's `milestones = ["Sign-in rework"]` names a version in Jira, and the milestones the
project offers are its unreleased, unarchived versions. An issue depends on the issues its text
names on a line starting "Depends on" and on those that block it with an "is blocked by" link.
Closing an issue takes the transition that leads to a Done status.

## GitLab code host

A project whose merge requests are on GitLab names the code host `gitlab`, whatever its tracker:

```toml
[[project]]
id = "rota"
tracker = "github"
name = "example-org/rota"
code_host = "gitlab"
repo = "example-group/sub/rota"   # the full group path, with every namespace
```

`repo` is the full path with its namespaces. The adapter runs `glab`, so `glab` must be
signed in (`glab auth login`) for the daemon's user, and no token goes in this file. It
reaches the host `glab` uses by default; for another one, set `GITLAB_HOST` for the daemon.
A pipeline reads as failed as soon as a job that may not fail has failed. A merge request is
merged at once, in the way the `merge` setting says (`squash`, `merge` or `rebase`), and
`glab` is not asked to remove its source branch. After one repository of an area's issue
merges, the next waits for the newest pipeline of that commit on the target branch; one held
at a manual job has not passed and is listed as a problem until someone runs the job.

## Example

```toml
# A project: a repository and the tracker its issues come from.
[[project]]
id = "radar"              # short id, used in issue ids such as radar#44
tracker = "github"        # the tracker adapter
name = "temet-ai/radar"   # the tracker's name for it
# code_host = "github"    # where its pull requests are ("github" or "gitlab"); the tracker when left out
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
when one of the project's folders is free (on the issue's `machine`, when that setting
names one) and every issue in the issue's "Depends on" lines is done. A project with two
folders has at most two authors working at once.

A project's `deploy` and `verify` commands run through `sh -c` in its `run_in` folder (your
home folder when left out), with `TEMET_WORK_ISSUE` (the issue, such as `radar#44`),
`TEMET_WORK_PROJECT` (the project's id) and `TEMET_WORK_REPO` (the code host's name for its
repository) set. The daemon reaches herdr-core at `HC_TERMINAL_SOCKET`, else at
`$XDG_RUNTIME_DIR/herdr-core/default.sock`.

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

Listing a folder under `[[folder]]` is your choice to let agents work in it. Before each
agent starts in a folder, the `claude` runtime marks the folder trusted
(`hasTrustDialogAccepted`) in Claude Code's own user file, `.claude.json` in the folder
`CLAUDE_CONFIG_DIR` names, else in your home folder. It changes only that folder's entry
and leaves a folder already trusted alone. For a folder on another machine this happens
on that machine, in its user file ("A work folder on another machine", below). An agent
stopped at a prompt the daemon cannot answer is listed under Problems with its name, its
folder and the prompt, and the line goes when the agent goes on or ends.

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
`ssh -o BatchMode=yes omit temet-work relay --build BUILD --host NAME [--socket PATH]`,
which hands the request to the daemon's local socket when `BUILD`, the build of the agent's
`temet-work`, is the build of the one on the daemon's machine. No port is opened and no new
secret is made. `BatchMode` makes a missing
key fail with ssh's own error instead of waiting for a password. With no `daemon_host`
the daemon starts no agent on another machine and lists a problem naming `machine` and
`daemon_host`, because the daemon's own name is often not one the other machine can
resolve.

The other machine needs five things, and each has a command that checks it:

| Needs | Check |
|---|---|
| SSH access from the daemon's machine | `ssh -o BatchMode=yes mac true` on the daemon's machine |
| SSH access back to the daemon's machine, under the name in `daemon_host` | `ssh -o BatchMode=yes omit true` on the other machine |
| `temet-work` on the `PATH` a non-interactive SSH session gets, and on the one herdr-core runs programs with there | `ssh mac temet-work parts` and `hc exec --on mac temet-work parts`, and `ssh omit temet-work parts` from the other machine |
| `temet-work` at the daemon's build | `ssh mac 'TEMET_WORK_DAEMON=ssh://omit temet-work status'`: at another build it is refused, naming both builds |
| The checkout at `path`, and the agent runtime (such as `claude`) | `ssh mac 'cd /Users/me/projects/radar && git status -sb && command -v claude'` |

The build is what `git describe --always --dirty` said where `temet-work` was built, and
`temet-workd --build` prints the daemon's. `temet-work` has no flag that prints its own, so
the check is that `temet-work relay` on the daemon's machine does not refuse it. The
refusal is one line,
`refused: temet-work on mac is build X, the daemon is build Y: update it`. A `temet-work`
too old to send its build is refused as build `unknown` on `the other machine`. Two
clones of one commit can describe it differently (other tags, or a longer short hash),
and those count as different builds.

Before each agent starts in a folder on another machine, the daemon trusts the folder
there: herdr-core runs `temet-work trust-folder claude PATH` on that machine, with no
shell, and with `CLAUDE_CONFIG_DIR` when the daemon has it. That command marks the folder
trusted in the machine's own `.claude.json`, as the daemon does for a folder on its own
machine, and writes nothing when the folder already is. The daemon's log has a line for
each step, in this order:

```text
trusted: /Users/me/projects/radar on mac, for author-1
started: author-1 on mac in /Users/me/projects/radar (terminal mac/t3)
```

When the step fails, no terminal is started, and the start is listed under Problems in one
line naming the folder, the machine and the reason, such as the command missing from the
`PATH` herdr-core uses there:

```text
run author-1: cannot trust /Users/me/projects/radar on mac: temet-work is not on mac's PATH for programs herdr-core runs there; `hc exec --on mac which temet-work` checks it
```

A nonzero exit ends with the last line the command wrote, such as
`temet-work exited 1 on mac: /Users/me/.claude.json is not a JSON object`. The daemon tries
the start again each round, and the line goes once it succeeds.

The daemon does not answer Claude Code's other first-run prompt, "Allow this read outside
the working directories?". On a new machine, start `claude --permission-mode auto` once in
the work folder and answer it with "Yes, and keep allowing": an unattended agent stops at
that prompt on its first read outside the folder.

A failed hop says which step failed in one line: `could not connect` (ssh, either
direction), `no answer within 60 s`, `temet-work is not on the other machine's PATH`,
or `no temet-work daemon answers at ...` (the daemon is not running there). A refusal
from the daemon is a refusal, as for a local agent.

A hop has three time limits:

| What stalls | Limit | The message |
|---|---|---|
| Connecting to the host (`ConnectTimeout=10`) | 10 s | `ssh HOST: could not connect: ...` |
| An open link that goes silent (`ServerAliveInterval=15`, `ServerAliveCountMax=2`) | about 45 s of silence | `ssh HOST: could not connect: Timeout, server ... not responding.` |
| Anything else, from the start of the hop to its end | 60 s | `ssh HOST: no answer within 60 s` |

The 60 s limit counts the whole hop, so a link that drops more than about 15 s into a
request ends with that message. temet-work passes the three `-o` options on the command
line, so they win over `~/.ssh/config`. After `no answer within 60 s`, or a link that
dropped in the middle of a request, the daemon may already have applied the command:
run `temet-work issue ISSUE` before you send it again.

To send one command from your own shell the same way: `TEMET_WORK_DAEMON=ssh://omit temet-work status`.

## Sending work to one machine

The `machine` setting (`docs/settings.md`) names the machine whose work folders an issue's
agents may be given. This project has a checkout on the daemon's own machine (`local`) and
one on a Mac. Agents for the issues in the milestone run on the Mac, and their reviewers
stay on the daemon's machine:

```toml
daemon_host = "omit"

[[project]]
id = "radar"
tracker = "github"
name = "temet-ai/radar"

[[folder]]
id = "radar-1"
project = "radar"
machine = "local"
path = "/home/me/src/radar-1"

[[folder]]
id = "radar-mac"
project = "radar"
machine = "mac"
path = "/Users/me/projects/radar"

[[milestone]]
of = "radar"
name = "Slack sign-in rework"

[milestone.settings]
machine = "mac"

[milestone.settings.reviewer]
machine = "local"

[[group]]
id = "group-1"
projects = ["radar"]
```

When no folder on that machine is free, the issue stays ready, even while a folder on
another machine is free, and its author starts when a folder on that machine is released.
`temet-work status` lists it as `radar#44 waiting for a work folder on mac`.

`machine` picks which `[[folder]]` an agent is given, and so which machine it runs on. The
folder's own `machine` field still says where that folder is. The daemon's own machine is
`local`, whatever its host name.

## temet-work deploying itself

Self-deploy updates only the daemon's machine. Every other machine that has a work folder
needs the same install, of the same commit; until then the commands its agents send are
refused ("A work folder on another machine", above).

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
a restart within ten minutes counts as a failed try, and runs again ten minutes after
that. The tries of a handed-off deploy add up across rounds until the third; then it waits
for a person, as any deploy does. A deploy that fails after writing its handoff line has
the line removed and runs again in ten minutes, unless that was its third try. If the
daemon that starts is not the build the line names, the issue is listed as a problem with
both builds, and `temet-work retry ISSUE` deploys it again.
