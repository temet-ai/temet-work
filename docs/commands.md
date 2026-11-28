---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 30de12a8-8bb8-4310-a59b-8d5267ce0b97
  - 26cd49d9-7565-4506-b988-a9b64ed0fa45
  - 7d9306de-2941-41bd-8a60-efad6ed38f2a
  - 3f18e0c3-9e38-42e7-8d05-5740c115b38d
  - 48c80237-558e-4a4a-a82d-bd96b37f1f46
  - 29b5659a-5fe0-4f00-806c-b1db18d91396
  - 198d20e8-7c5c-4a63-b1e6-5f43be8c6607
  - 0645e4e2-33bd-43f8-980f-503649da2f15
  - edd1e649-f07a-4d7c-b5c6-ff74b0021fbb
  - c14f6d16-7688-4e50-99d4-32021f0e29db
  - a8be342d-b7b2-4e65-bf6b-bcd753f14a12
  - ba3664be-ec7f-4b51-af6a-b1c20c80634d
---

# Commands

`temet-work` sends one command to the running daemon and prints its reply. People and
agents use the same commands; the brief each agent starts with names the one that
reports its step done.

| Command | Who runs it | Accepted when the issue is |
|---|---|---|
| `temet-work status [--json]` | anyone; `--json` prints the state as data (below) | any |
| `temet-work issue ISSUE` | anyone: the issue in full, with its text from its tracker's last sync (or one line saying why it was not read, or that it is empty), its plan, pull requests, fixes, review rounds and agents. The review rounds are every finished round in the order they finished: its number and whether it approved or blocked, and for a block each point with the author's reply to it, or "none on record", and whether the next round kept it open or resolved it, naming that round; a point no later round judged, or whose state an older log does not hold, says so. Each later line of a point or reply is indented under it. An arbiter's brief holds the latest of these rounds that fit in 48 KiB, each whole, and the last one always, with a line counting the earlier ones it leaves out; a last round that alone is over 48 KiB is cut there, with a line saying so. `temet-work issue` still holds every round whole. The text is the one the last sync that worked listed, so an edit in the tracker shows after the next sync (`sync_every`, 60 s unless set otherwise). An issue that is merging, deploying, being verified or waiting to be reopened after a failed verify keeps the text of the last sync that listed it | any |
| `temet-work screen AGENT` | anyone: what the agent's terminal shows now | any |
| `temet-work plan ISSUE FILE` | only the refiner working on ISSUE, with the plan as YAML; `-` reads it from standard input | refining |
| `temet-work ready ISSUE PULL`, or `ready ISSUE REPO=PULL ...`, with `--reply TEXT` for each point of the block it answers | only the author working on ISSUE, once the pull requests are open and the plan's checks hold; `PULL` is a number, needed the first time, and again in a repository where a person closed the pull request on record without merging, since a read in review or the merge then drops it; an issue that changes several repositories names each as `REPO=PULL`. The first ready after a block (from the review, or from an arbiter's fix ruling, where the replies answer the reviewer's last block) carries one `--reply` for each of the block's points, in their order: what the author did about it, or why not. A ready without exactly one reply for each point is refused, and the refusal lists the points and the form to use; any other ready takes no `--reply`. Each reply is kept on its point and shown with it by `temet-work issue` and `status --json`. A ready that starts a review round sets each of the plan's checks to running, since the author reports that every check holds; after a person's overturn has waived the review, the checks keep their state. A ready naming a pull request not on record, as after one is closed, ends that waiver and asks for the review again | working, or fixing |
| `temet-work review ISSUE approve` | only the reviewer working on ISSUE. Every check of the plan is marked passed, and every point of the round before resolved; `--failed` and `--open` are refused | in review |
| `temet-work review ISSUE block POINT... [--failed N ...] [--open N ...]` | only the reviewer working on ISSUE, one argument per thing to fix. `--failed N`, once for each check that does not hold, counts from 1 in the plan's order; every check left unnamed is marked passed. After round 1, `--open N`, once for each point of the round before that is still not dealt with, counts from 1 in that round's order: the point stays open and joins this block's points, so the author fixes it again and the next round judges it again. Every point left unnamed is resolved, and a block may consist of kept points alone. The reviewer's brief and each next-round message number the checks and the round before's points this way, each point with the author's reply. A number the plan or the round before does not have is refused, and so is `--open` in round 1 | in review |
| `temet-work retry ISSUE` | a person only | deploying or verifying: runs the deploy (or verify) again, at once unless a deploy or verify of the same project or `run_in` folder is running, and clears its failure count, its ten-minute wait and its reruns in `deploy-handoff.stale`, so a deploy that failed three times gets three fresh tries |
| `temet-work rule ISSUE approve REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work rule ISSUE fix REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work overturn DECISION OPTION REASON` | a person only; `DECISION` and `OPTION` are the numbers `status` lists (options count from 1), `REASON` is required | any, if the decision exists, the option is another than the one chosen, and the reason is not empty. What it changes depends on the issue: see below |
| `temet-work proposal RULE keep` | a person only: the rule moves to kept and ends every brief written from then on | the rule is proposed |
| `temet-work proposal RULE drop` | a person only: the rule moves to dropped and leaves status and every brief | the rule is proposed |
| `temet-work file PROJECT TITLE BODY [--milestone M]` | only the investigator of a group whose investigation is running, for a project or area its group takes issues from; `BODY` `-` reads it from standard input. The reply names the new issue and how a "Depends on" line names it | no issue: while the group's investigation runs. The milestone is the last in the group's list when none is named; one the group does not take is refused, and a group with no milestones files with none. A title an open issue of that project already has is refused, and so is the title of an issue a person closed in that project (one the loop did not close after verifying it, with none of its pull requests merged); the reply names the issue |
| `temet-work investigated GROUP` | only the investigator of GROUP whose investigation is running: the investigation ends and the investigator's session is stopped | no issue: while GROUP's investigation runs |
| `temet-work set TARGET [--role ROLE] KEY VALUE` | a person; a refiner for the task level of the issue it refines, except `runtime`, `runner`, `permission`, `review_rounds`, `ci`, `deploy`, `verify`, `run_in` and `investigate`, which only a person sets; a refiner's `model` is recorded for the author, and refused for another role. `--role` is for the settings that take a role (`docs/settings.md`, Roles) | any for a person; refining for a refiner |
| `temet-work unset TARGET [--role ROLE] KEY` | the same, `--role` included | any for a person; refining for a refiner |
| `temet-work settings TARGET [--role ROLE]` | anyone | any |

## Who may run a step

`plan`, `ready`, `review` and `rule` are steps. The daemon checks the sender before the
issue's stage, so a wrong sender is told whose step it is, not what stage the issue is at.

- An agent may run a step only if it has the role whose step it is (refiner for `plan`,
  author for `ready`, reviewer for `review`, arbiter for `rule`) and works on that same
  issue. An id the daemon did not start is refused too. The refusal names the step and
  its owner, for example: `plan for radar#44 is the refiner's: refiner-3 is working on it;
  you are author-2`.
- A person may run `rule` only. A ruling is a final judgement a person can take over, and
  it is recorded as a decision made by `person`. A person running `plan`, `ready` or
  `review` is refused with the owning role named: those would skip the work an agent is
  there to do. To get a stuck agent moving, a person rules or changes settings with `set`
  and `unset`.
- `status` (with or without `--json`), `issue`, `screen` and `settings` are open to
  anyone. `set` and `unset` keep their own rule, in the table.
- `file` and `investigated` belong to a group's running investigator, which works for its
  group and on no issue. A person, another agent, or an investigator whose investigation
  has ended is refused with the reason; a person files issues in the tracker itself.

## How the daemon knows who sent a command

The daemon starts every agent with two variables: `TEMET_WORK_AGENT`, the agent's id, and
`TEMET_WORK_AGENT_TOKEN`, a random 256-bit secret of the agent's own. `temet-work` and
`temet-work mcp` send both with every command. The event log keeps the secret's SHA-256
digest, so agents that were running before a restart keep working after it; the secret
itself is in no log, reply, status or issue text.

- A command that names an agent, a read included, is refused unless it carries that
  agent's secret. The refusal names the agent and says what was wrong: no secret, a wrong
  one, another agent's, or an id this daemon did not start. A command that names an agent
  whose session has ended is refused too. An agent started before agents had secrets has
  no digest in the log and is taken by its id until it ends.
- On Linux the daemon also reads the process that sent the command, through its socket
  (`SO_PEERCRED`), and that process's parents in `/proc`, and looks in each one's
  environment for a secret it issued. A command that names agent A from inside agent B's
  process tree is refused, even with A's own secret, so an agent that reads another
  agent's secret from `/proc` cannot use it from inside its own process tree. For a
  command that came through `temet-work relay` the tree read is the relay's own, on the
  daemon's machine under `sshd`, which says nothing of the machine the command came from.
  The relay's mark only adds refusals, so a process on the daemon's machine that sets it
  is still judged by its own tree.
- On Linux the sending process itself may be impossible to read: its program has no read
  bit, it called `prctl(PR_SET_DUMPABLE, 0)`, or it has ended. The daemon still reads its
  parents, so a secret found there refuses the command as above. A command that changes
  something and names an agent is refused from such a process even when nothing is found,
  and the refusal says why it could not be read. `status`, `issue`, `screen` and
  `settings` still answer it. An agent's own `temet-work` stays alive and readable while
  it waits for the reply.
- A command that names no agent is a person's. It may read from anywhere. A command that
  changes something (`rule`, `overturn`, `retry`, `proposal`, `set` and `unset`) is
  refused when it carries an agent's secret; when, on Linux, the sending process or one of
  its parents holds a secret this daemon issued, which still holds after the agent unsets
  both variables with `env -u`; when the sending process cannot be read; or when it came
  through `temet-work relay` from another machine, whose processes the daemon cannot see.
  A person runs these commands in a shell outside any agent's terminal, on the daemon's
  machine.
- A secret another daemon issued does not count, so a second daemon, and the tests, work
  from inside an agent's terminal.

Agents run as the same user as the daemon, so this does not stop an agent that:

- starts any process outside its process tree (`hc exec`, `hc run`, a double fork,
  `tmux new -d`, `systemd-run --user`, `at`, cron, `setsid -f`): with both variables
  dropped it passes as a person, and with another agent's id and secret it passes as
  that agent. `hc exec` and `hc run` start the program as herdr-core's child with
  herdr-core's environment, not the agent's, so it passes as a person even without
  `env -u`;
- edits the daemon's files or its event log;
- logs in to the daemon's machine over SSH and runs `temet-work` there, outside the relay;
- runs on another machine and reads another agent's secret from that machine's `/proc`:
  through the relay it passes as that agent, since the daemon cannot see the processes
  there;
- sends text to another agent's terminal through herdr-core (`hc send`, `hc keys`): it
  acts as that agent. A line starting with `!` runs as a shell command in that agent's
  process tree with its secret, so both the secret check and the tree check pass, and
  plain text is a prompt that agent acts on. `hc send` run from a terminal puts an
  `<hc from="…">` tag before the text, which unsetting `HC_TERMINAL_ID` leaves out.
  Terminals carry `group`, `issue` and `role` labels, so one such as the terminal of
  `reviewer-3` on `watch#12` is easy to find;
- types into a person's terminal.

On macOS the daemon checks secrets but not process trees, because `SO_PEERCRED` and
`/proc` are Linux only and refusing every command a person sends would leave a person no
way to run them. There an agent that unsets both variables passes as a person, and one
that has read another agent's secret passes as that agent.

An agent on another machine sends its secret through `temet-work` on that machine, so
that machine needs a build that sends it: install the same one there as on the daemon's
machine.

`ISSUE` is `project#key`, such as `radar#44`. A plan file looks like this:

```yaml
checks:
  - done_when: Signing in from radar stores a user token that outlives a restart
    proven_by: cargo nextest run -p radar-slack sign_in, then restart radar and open a channel
```

`TARGET` is `global`, an area or project id, a milestone as `project:Milestone name`, or an
issue. `ROLE` is refiner, author, reviewer, arbiter or investigator; without it a value is
for every role. `settings` lists every setting in effect there, each with the level that
set it (`task, set by refiner-3`, `built in`); `temet-work issue` shows the same for the
role at work on the issue. `KEY` is a setting's name, or `model.<runtime>`
(`model.codex`) for the model of one runtime's agents; a plain `model` is recorded as the
global runtime's `model.<runtime>` when it is set, and the reply names that key. A value of
the wrong kind, an unknown key, a runtime or runner the build lacks, a `model.<runtime>`
for such a runtime, a plain `model` where that scope and role run a runtime other than
the global one, or `set` of a setting at a level that never reads it (`sync_every` and
`investigate` on an issue or a milestone) is refused with the reason, and the command
exits 1. `unset` does not check the level, so a person can take away such a value that an
older log holds.

A refiner may add task-level settings to its plan, other than the nine only a person
sets: `runtime`, `runner`, `permission`, `review_rounds`, `ci`, `deploy`, `verify`,
`run_in` and `investigate` (`docs/settings.md`, "Setting them"), and other than
`sync_every`, which no task reads. What it sets is recorded as set by it. Its `model` or
`model.<runtime>`, from the plan or from its `set`, is for the author only: the models of
reviewers, arbiters and refiners are a person's to set, and a person's `set` on the issue
with no role overturns the refiner's value. A plan with settings:

```yaml
checks:
  - done_when: ...
    proven_by: ...
settings:
  model: opus
  starts: 5
```

A command for an issue at another step is refused, with the step the issue is at. The
command line exits 0 when the daemon did it, 1 when the daemon refused it, and 2 when the
command was not understood or no daemon answered. It also exits 2 when its
output cannot be written, and says why on standard error. A reader that stops reading
early, as `head` does, is no such failure: the command line says nothing and exits 0.

`temet-work status --json` sends the `view` command (`{"command":"view"}` on the socket)
and prints the reply as indented JSON: projects with the web addresses of their issues and
code, milestones, groups, the issues that are not done and the count of done ones, agents,
decisions and problems (`crates/core/src/view.rs`; what it carries is listed in
`docs/privacy.md`). Issue ids are `project#key`, each stage and agent state is a
fixed word with the text form's words beside it, each check's state (`passed`, `running`,
`waiting`, `failed`) and each review point's state (`open`, `resolved`) is a fixed word, and
decision options are numbered from 1.
Times, such as when an issue last changed, are milliseconds since 1970, and due dates are
`YYYY-MM-DD`; a value temet-work does not know, such as a due date the tracker does not
give, is `null`. `--json` on any other command is refused
with the usage; inside a value, a title, a body, a review point or a reason it is a word
like any other.

`temet-work mcp [--allow read|act]` serves the same views and commands as MCP tools over
standard input and output, for an agent's MCP client. At `read`, the default, it offers
`status`, `issue`, `screen` and `settings`; `act` adds `plan`, `ready`, `review`, `rule`,
`retry`, `overturn`, `proposal`, `file`, `investigated`, `set` and `unset`.

`temet-work parts` prints the version and the parts compiled in, without asking the
daemon.

`temet-work --version` prints one line, `temet-work <version> (<build id>)`, without
asking the daemon. The build id is the commit the binary was built from, as
`temet-workd --version` shows it.

A command an agent runs is recorded as that agent's (see "How the daemon knows who sent a
command"). A ruling is recorded as a decision, listed by `temet-work status` with its
numbered options, its cost to undo and its point: the issue's merge, after which
overturning it costs more. An approval costs high, since the loop does not take back a
merged change, and a send-back costs low, since the author's fix finishes either way and
the issue merges once the checks pass. `status --json` gives the cost as
`low`, `medium` or `high`, and the point as `merge` with whether the issue is past it,
which it is once the issue is deploying, verifying or done. It gives the point no time,
since nothing in temet-work knows when a merge will happen. A choice that is not a ruling
has neither, nor has a ruling recorded before rulings kept them.

## Overturning a decision

Agents decide and carry on; a person reads the decisions in `status` and can overturn one
with `overturn DECISION OPTION REASON`. An agent may not: agents are the ones overturned.
The daemon records who overturned it and why, and `status` shows the decision as
overturned with the reason. The loop then acts on the decision's issue, by what the issue
has reached:

- An arbiter's ruling that approved, before the issue has merged: the issue goes back to
  its author as a fix whose point is the reason.
- A ruling that sent it back, while it is being fixed or in review: the author's fix
  finishes, no review is asked for again, and the issue merges once the checks pass. In
  review, the reviewer is stopped, so each check still running goes back to waiting: no
  one judged it. A pull request on record that is found closed, or a ready naming one not
  on record, ends this: the change is reviewed again.
- After the merge nothing changes: the merge is the point after which overturning costs
  more, and the loop does not undo it. The overturn is recorded and the rule is still
  proposed.
- A choice that is not a ruling changes no stage.
- Overturning the same decision to the same option again changes nothing.

Every overturn also proposes a rule, once per decision: the decision's question, the
option now preferred and the reason. `status` lists proposed and kept rules under Rules,
each with its decision, issue, the agent that decided and who overturned it. `proposal
RULE keep` or `proposal RULE drop` answers it (`proposal`, because `rule` is the
arbiter's ruling). A kept rule ends every brief written afterwards, for every role, under
"Rules from overturned decisions". Agents already running keep their old brief: new
rules apply from the next start, so nothing waits on a person.

A review from an agent that has not yet heard of the round under review is refused: it
began on code that has since changed.

The daemon listens at `$TEMET_WORK_SOCKET`, else `$XDG_RUNTIME_DIR/temet-work/daemon.sock`,
else `~/.local/state/temet-work/daemon.sock`, one JSON request per line and one JSON reply
per line. The line `{"ask":"build"}` asks the daemon its build. It replies at once with
`{"reply":"done","text":"BUILD"}`, naming the build it is running, and the same connection
takes requests as before.

`TEMET_WORK_DAEMON` picks how a command reaches the daemon: unset or empty is the local
socket above; `ssh://[USER@]HOST[:PORT][/SOCKET/PATH]` runs `ssh -o BatchMode=yes
-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=2 HOST temet-work
relay --build BUILD --host NAME` (the client's build and machine name), with
`--socket SOCKET/PATH` when the address has a path. A hop gets 10 s to
connect, about 45 s of silence on an open link, and 60 s in all; past that ssh is killed
and the command fails with `ssh HOST: no answer within 60 s`. A host ssh cannot reach
fails with `ssh HOST: could not connect: ...`.
The hidden command `temet-work relay` copies one request line from standard input to the
local socket, marked as relayed, and the reply line back; it exits 4 when no daemon
answers. Before it sends the request, it asks the daemon its build on the same connection.
When `--build` is missing or names another build, it does not send the request and replies
with a refusal that names the daemon's build:
`temet-work on NAME is build X, the daemon is build Y: update it`. A daemon too old to
answer the question is not sent the request either, and the refusal says to restart it.
The hidden command `temet-work trust-folder RUNTIME PATH` marks PATH trusted for the
runtime in its user files on the machine it runs on, as the daemon does before an agent
starts in a folder on its own machine. For a folder on another machine the daemon runs it
on that machine through herdr-core. It exits 0 when the folder is trusted or already was,
and 1 with the reason on one line of standard error otherwise; a runtime that needs no
folder trusted, such as `codex`, exits 0 and writes nothing.
`docs/configuration.md` says how to set up a work folder on another machine.
