---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 3f18e0c3-9e38-42e7-8d05-5740c115b38d
  - 48c80237-558e-4a4a-a82d-bd96b37f1f46
  - 198d20e8-7c5c-4a63-b1e6-5f43be8c6607
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

# Settings (docs/settings.md): [settings] is global, [settings.reviewer] is for one role,
# and takes only the settings that take a role (docs/settings.md, Roles).
# A model belongs to a runtime, so a `model` table names the runtime each one is for.
# [settings]
# runtime = "claude"
# [settings.model]
# claude = "sonnet"
# [settings.reviewer]
# runtime = "codex"
#
# A project's, an area's and a milestone's own: under the project, the area or [[milestone]].
# [project.settings.model]
# claude = "haiku"
#
# [[milestone]]
# of = "radar"                 # a project or an area
# name = "Slack sign-in rework"
# [milestone.settings.model]
# claude = "opus"

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
# queue_low = 4                         # starts an investigator below this many issues in its queue
# investigate_every = 120               # minutes from one investigation's end to the next; the default
```

A group takes its projects' open issues in milestone order, then in the tracker's order.
It refines `refining_at_once` issues at a time. It starts an author on a refined issue
when one of the project's folders is free (on the issue's `machine`, when that setting
names one) and every issue in the issue's "Depends on" lines is done. A project with two
folders has at most two authors working at once.

A merged pull request named in a "Depends on" line counts as done. In such a line, `#491` is
number 491 of the same project. An issue of another project or area has that project's or
area's id, or its tracker's name for it, before the number: `klatch#4` or `owner/repo#4`.
For a dependency the daemon has not synced, a sync asks the tracker of its project or area
whether it is closed. When no configured tracker can be asked about it, or the tracker answers
with an error, `temet-work status` lists the dependency under Problems with the reason, and
the issue waits. The dependency is also listed under Problems when the tracker says it is open
but a sync of its project or area, once one has worked, does not list it, as GitHub lists no
pull request among the issues. The daemon then never syncs it, and the issue waits on it.

An issue leaves a group's scope when the group no longer lists its milestone, project or
area, because the issue moved or the group changed. An agent already working on it
finishes its step: a refiner its plan, an author its change or the fixes it has heard, a
reviewer its round, an arbiter its ruling. After that the group does not start an agent on
it, send a fix or a new review round, or merge an approved change. The issue keeps its
work folders and its idle author, which hold its branch and what the author knew.
`temet-work status` and `temet-work issue` show its stage followed by "out of scope".
Three cases go differently:

- A refined issue no author has started on holds no agent or folder, so another group
  whose lists cover it takes it, without refining it again.
- A refining issue whose refiner stops before its plan is in goes back to the new issues.
  It holds no refining slot while it is out, and is refined from the start when a group
  takes it.
- An issue with one of its pull requests merged, or deploying or verifying, finishes: the
  rest merge in the plan's order, then it deploys and is verified.

When the issue is back in its group's lists it carries on from the stage it reached: the
agent that stage needs starts or is told, or the merge goes ahead. An issue with a plan is
not refined again.

A group's work queue is the refining, ready and new issues in its scope (its milestones,
projects and areas): those it is refining or has refined and not started, and those no group
has taken yet. An issue moved out of them does not hold off an investigator, and counts again
once it is back. Refining and new issues count whatever they depend on. A ready issue counts
only while it would start once a work folder is free, even in a project with none yet, decided
by the same rule that starts its author: its group covers it and every issue in its "Depends
on" lines is done.
`temet-work status` lists each ready issue that would not start under Waiting, with a line for
each reason:

- `radar#44 waiting on radar#51, never synced`: the daemon holds no issue by that name, in the
  issue's own project or another, so nothing says it is done.
- `radar#44 waiting on radar#51, out of scope (Later is not a milestone of core)`: the
  dependency is open and in a milestone the group does not take, even when another group takes
  it. The brackets say `no group covers it` when no group covers the dependency at all. A
  dependency on a repository that no project or area matches is one no group covers: no tracker
  is asked about it, and the line shows it as the issue's text writes it, such as `nowhere#5`.
- `radar#44 waiting on radar#51, not done (working)`: the dependency is open and in scope, at
  the stage in the brackets. It counts in its own place while it is new, refining or ready.
- `radar#44 waiting, out of scope (Later is not a milestone of core)`: the group no longer
  covers the issue itself, and no other group does. The brackets name what the group no longer
  lists: the milestone, the project or area, or, for an issue with no milestone, `it has no
  milestone`.

An open dependency in scope is followed to its own dependencies, and the line names the one
where that walk ends: one whose own open dependencies all lead back to it, as in a cycle,
where each of two issues names the other. `through radar#47` names the issues between when
radar#44 reaches it through radar#47. The issue counts again once nothing holds it: the
dependency is done or back in scope, its "Depends on" line is gone, or the issue is back in its
group's lists.

When the queue holds fewer than `queue_low` issues, the daemon starts one investigator for
the group. Left out, `queue_low` is twice the work folders of the group's projects and of
the projects in its areas, or 2 when they have none; 0 starts no investigator. The
investigator holds no work folder and changes no file: it runs in a free folder of the
group's first project, or in one an author holds when none is free. It reads each project's
docs for planned work that is not built, the open issues, and the pull requests merged since
the group's last finished investigation started: one is finished when its investigator ran
`temet-work investigated`. While none has finished, it reads those merged in the 7 days
before the group's first investigation started. It files an issue for each planned item no
open issue covers and for each defect it can show (`temet-work file`, `docs/commands.md`).
It reads docs and code at the current head of each repository's default branch, not as a
checkout's files stand: it fetches, then reads with `git show` or in a separate checkout
outside the checkouts its brief names and the folder it runs in. Each issue it files goes in
the last milestone of the group's list unless it names another the group takes, and is in
the loop as soon as it is filed. A project's `investigate` setting adds lines to the
investigator's brief (`docs/settings.md`). Closing an issue is how a person says no to it:
the brief lists the issues a person closed in the group's projects and areas, and the
investigator files none of them again. The list holds issues the daemon saw open while
syncing. A close in the tracker counts as a person's unless the daemon made it after
verifying the issue, or one of the issue's pull requests had merged before it, since a pull
request can close its issue as it merges. For the same reason the daemon ignores a close
while the issue merges, deploys or is verified. Reopening a done issue clears the merge. A
person who wants a listed issue built reopens it.

When the verify check fails, the issue goes back to its author to be fixed, and the daemon
reopens it in its tracker, since the merge may have closed it there: GitHub through
`gh issue reopen`, Jira through the first transition out of the Done status category. It
posts no comment. It syncs the issue's project at once to do this, and tries again at each
sync until a sync lists the issue as open. A failed reopen, such as a Jira workflow with no
transition out of Done, is listed under Problems with the tracker's error, and the daemon
ignores a close of the issue until the reopen happens. Meanwhile `temet-work issue` shows the
text from the last sync that listed the issue. A sync that lists the issue as open counts as
the reopen, whoever reopened it. From then on a close in the tracker is a person's: it
finishes the issue, stops its agents and frees its work folders, and the investigator does not
file it again. While the fix merges, deploys or is verified, and once one of its pull
requests has merged, the daemon ignores a close. Until the issue is done,
`temet-work status` and the app list it under Problems with what a close does now. Where the
daemon ignores one, the line gives the issue's stage and says that a close is not applied.

The investigation ends when the investigator runs `temet-work investigated GROUP` or its
session ends. An idle investigator is reminded and then stopped after `remind_after` and
`reminders`, like any agent the loop waits on. The next one starts once the queue is short
again and `investigate_every` minutes have passed since the last one ended: 120 when left
out, and at least 1. A group never investigated may start one at once. `temet-work status`
shows how many issues the group's queue holds, the count compared with `queue_low`, beside
`queue_low` itself, and `investigating: investigator-N` while one runs.

A project's `deploy` and `verify` commands run through `sh -c` in its `run_in` folder (your
home folder when left out), with `TEMET_WORK_ISSUE` (the issue, such as `radar#44`),
`TEMET_WORK_PROJECT` (the project's id) and `TEMET_WORK_REPO` (the code host's name for its
repository) set. One deploy or verify runs per project and `run_in` folder at a time: two
deploys would both pull and build in one checkout, and a verify compares what is installed
with that checkout. One asked for while another of the same project or folder runs, or
while a deploy there waits for its restart (at most ten minutes), starts in a later round.
Projects with different `run_in` folders deploy side by side. The daemon reaches herdr-core at `HC_TERMINAL_SOCKET`, else at
`$XDG_RUNTIME_DIR/herdr-core/default.sock`.

Settings at every level, from global down to one task, are in `docs/settings.md`. The
tables `[settings]` (with `[settings.<role>]` inside it), `[project.settings]`,
`[area.settings]` and `[[milestone]]` become `SettingSet` events recorded as set by
`file`. The daemon reads the file when it starts, so a change to it takes effect at the
next start. A `model` table in any of them, or in a role's table, gives a model per
runtime: `[settings.model]` with `claude = "sonnet"` is the setting `model.claude`. A
plain `model = "..."` is recorded as `model.<runtime>` of the file's global runtime (its
`[settings]` `runtime`, else the build's default), and stops the daemon at start, naming
`model.<runtime>`, in a table whose scope and role run another runtime. A runtime a person
set by command counts for that as well as one the file gives, except where it is the
global runtime a person moved. A group's `model` and a project's `ci`, `deploy`, `verify`
and `run_in` are still accepted and are read as settings at their level, so an existing
file behaves as before. A group's `model` is bound to the file's global runtime and never
stops the start; under a project on another runtime, that project's agents run their
runtime's default. A key the
daemon does not know is an error naming the key, and so is a key in a role's table that
takes no role (`docs/settings.md`, Roles). `sync_every` or `investigate` in a
`[[milestone]]`'s settings stops the daemon at start, naming the levels that read it
(`docs/settings.md`, "The settings"). A value that names a runtime or runner the
build does not have, or a model for such a runtime, stops the daemon at start, with the
ones it has listed, and so does a build with no default runtime or runner and no global
value for it. What a person or an agent has set since is not overwritten by the file; a
value taken out of the file is unset.

Listing a folder under `[[folder]]` is your choice to let agents work in it. Before each
agent starts in a folder, the `claude` runtime marks the folder trusted
(`hasTrustDialogAccepted`) in Claude Code's own user file, `.claude.json` in the folder
`CLAUDE_CONFIG_DIR` names, else in your home folder. It changes only that folder's entry
and leaves a folder already trusted alone. For a folder on another machine this happens
on that machine, in its user file ("A work folder on another machine", below). An agent
stopped at a prompt the daemon cannot answer is listed under Problems with its name, its
folder and the prompt, and the line goes when the agent goes on or ends. An agent whose
turn ended in an error, such as a usage limit, is listed the same way with the line its
turn ended on (`docs/architecture.md`, "Who coordinates").

Each project, folder and group in the file becomes an event in the log at the first start
that reads it, and again at a start that finds it changed. Removing one from the file does
not remove it from the daemon yet.

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
`temet-work`, is the build of the running daemon. No port is opened and no new
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

The build is what `git describe --always --dirty` said where the program was built.
`temet-workd --build` prints the build of the installed binary, which is the running
daemon's only until the next install. `temet-work` has no flag that prints its own, so
the check is that `temet-work relay` on the daemon's machine does not refuse it. The relay
asks the running daemon its build, so the build of the `temet-work` that runs the relay
makes no difference. The refusal is one line that names the running daemon's build,
`refused: temet-work on mac is build X, the daemon is build Y: update it`. A `temet-work`
too old to send its build is refused as build `unknown` on `the other machine`. Two
clones of one commit can describe it differently (other tags, or a longer short hash),
and those count as different builds.

So the other machine's `temet-work` must be at the daemon's build. The relay checks that
only once an agent runs there. A `temet-work` older than the trust step below fails at
that step, before any agent starts, so the relay's refusal is never reached.

A daemon started from a `temet-work` too old to say its build gets no command from another
machine until it is restarted. The command is refused with
`refused: the daemon at PATH does not say its build: it was started from an older temet-work; restart it`.

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

A `temet-work` there with no `trust-folder` command is reported as older than the daemon,
and the line names the daemon's build, the one to update it to:

```text
run author-1: cannot trust /Users/me/projects/radar on mac: temet-work on mac is older than this daemon (build Y): update it
```

Any other nonzero exit ends with the first line the command wrote to standard error, such as
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
daemon that starts is not the build the line names, and it has no handoff of its own
waiting for that issue, it runs the deploy again. It first moves the line to
`deploy-handoff.stale`, next to `deploy-handoff`, adding its own build, so the file shows
which build each restart brought up. Those lines count the reruns across the restarts
they cause: after three for one issue, the issue is listed as a problem naming both
builds and waits for `temet-work retry ISSUE`. A retry, a recorded `Deployed`, or the
issue leaving deploying clears its lines.
