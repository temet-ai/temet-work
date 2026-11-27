---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 30de12a8-8bb8-4310-a59b-8d5267ce0b97
  - 26cd49d9-7565-4506-b988-a9b64ed0fa45
---

# Commands

`temet-work` sends one command to the running daemon and prints its reply. People and
agents use the same commands; the brief each agent starts with names the one that
reports its step done.

| Command | Who runs it | Accepted when the issue is |
|---|---|---|
| `temet-work status` | anyone | any |
| `temet-work issue ISSUE` | anyone: the issue in full, with its plan, pull requests, fixes and agents | any |
| `temet-work screen AGENT` | anyone: what the agent's terminal shows now | any |
| `temet-work plan ISSUE FILE` | only the refiner working on ISSUE, with the plan as YAML; `-` reads it from standard input | refining |
| `temet-work ready ISSUE PULL`, or `ready ISSUE REPO=PULL ...` | only the author working on ISSUE, once the pull requests are open and the plan's checks hold; `PULL` is a number, needed the first time; an issue that changes several repositories names each as `REPO=PULL` | working, or fixing |
| `temet-work review ISSUE approve` | only the reviewer working on ISSUE | in review |
| `temet-work review ISSUE block POINT...` | only the reviewer working on ISSUE, one argument per thing to fix | in review |
| `temet-work retry ISSUE` | a person only | deploying or verifying: runs the deploy (or verify) again, at once unless a deploy or verify of the same project or `run_in` folder is running, and clears its failure count, its ten-minute wait and its reruns in `deploy-handoff.stale`, so a deploy that failed three times gets three fresh tries |
| `temet-work rule ISSUE approve REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work rule ISSUE fix REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work overturn DECISION OPTION REASON` | a person only; `DECISION` and `OPTION` are the numbers `status` lists (options count from 1), `REASON` is required | any, if the decision exists, the option is another than the one chosen, and the reason is not empty. What it changes depends on the issue: see below |
| `temet-work proposal RULE keep` | a person only: the rule moves to kept and ends every brief written from then on | the rule is proposed |
| `temet-work proposal RULE drop` | a person only: the rule moves to dropped and leaves status and every brief | the rule is proposed |
| `temet-work set TARGET [--role ROLE] KEY VALUE` | a person; a refiner for the task level of the issue it refines, except `deploy`, `verify`, `run_in`, `permission` and `ci`, which only a person sets; a refiner's `model` and `runtime` are recorded for the author, and refused for another role | any for a person; refining for a refiner |
| `temet-work unset TARGET [--role ROLE] KEY` | the same | any for a person; refining for a refiner |
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
- `status`, `issue`, `screen` and `settings` are open to anyone. `set` and `unset` keep
  their own rule, in the table.

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

- starts a process that leaves its process tree (`setsid -f`): with both variables
  dropped it passes as a person, and with another agent's id and secret it passes as
  that agent;
- edits the daemon's files or its event log;
- logs in to the daemon's machine over SSH and runs `temet-work` there, outside the relay;
- runs on another machine and reads another agent's secret from that machine's `/proc`:
  through the relay it passes as that agent, since the daemon cannot see the processes
  there;
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
role at work on the issue. A value of the wrong kind, an unknown key or a runtime or
runner the build lacks is refused with the reason, and the command exits 1.

A refiner may add task-level settings to its plan, other than the five only a person sets
(`docs/settings.md`, "Setting them"); they are recorded as set by it. Its `model` and
`runtime`, from the plan or from its `set`, are for the author only: the models and
runtimes of reviewers, arbiters and refiners are a person's to set, and a person's `set`
on the issue with no role overturns the refiner's value. A plan with settings:

```yaml
checks:
  - done_when: ...
    proven_by: ...
settings:
  model: opus
  review_rounds: 5
```

A command for an issue at another step is refused, with the step the issue is at. The
command line exits 0 when the daemon did it, 1 when the daemon refused it, and 2 when the
command was not understood or no daemon answered.

`temet-work mcp [--allow read|act]` serves the same views and commands as MCP tools over
standard input and output, for an agent's MCP client. At `read`, the default, it offers
`status`, `issue`, `screen` and `settings`; `act` adds `plan`, `ready`, `review`, `rule`,
`retry`, `overturn`, `proposal`, `set` and `unset`.

`temet-work parts` prints the version and the parts compiled in, without asking the
daemon.

A command an agent runs is recorded as that agent's (see "How the daemon knows who sent a
command"). A ruling is recorded as a decision, listed by `temet-work status` with its
numbered options.

## Overturning a decision

Agents decide and carry on; a person reads the decisions in `status` and can overturn one
with `overturn DECISION OPTION REASON`. An agent may not: agents are the ones overturned.
The daemon records who overturned it and why, and `status` shows the decision as
overturned with the reason. The loop then acts on the decision's issue, by what the issue
has reached:

- An arbiter's ruling that approved, before the issue has merged: the issue goes back to
  its author as a fix whose point is the reason.
- A ruling that sent it back, while it is being fixed or in review: the author's fix
  finishes, no review is asked for again, and the issue merges once the checks pass.
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
per line.

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
answers. When `--build` is missing or is not its own build, it does not contact the daemon and replies with a
refusal: `temet-work on NAME is build X, the daemon is build Y: update it`.
The hidden command `temet-work trust-folder RUNTIME PATH` marks PATH trusted for the
runtime in its user files on the machine it runs on, as the daemon does before an agent
starts in a folder on its own machine. For a folder on another machine the daemon runs it
on that machine through herdr-core. It exits 0 when the folder is trusted or already was,
and 1 with the reason on one line of standard error otherwise; a runtime that needs no
folder trusted, such as `codex`, exits 0 and writes nothing.
`docs/configuration.md` says how to set up a work folder on another machine.
