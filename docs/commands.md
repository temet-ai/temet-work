---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
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
| `temet-work retry ISSUE` | a person only | deploying or verifying: runs the deploy (or verify) again at once and clears its failure count and ten-minute wait, so a deploy that failed three times gets three fresh tries |
| `temet-work rule ISSUE approve REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work rule ISSUE fix REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work overturn DECISION OPTION REASON` | a person only; `DECISION` and `OPTION` are the numbers `status` lists (options count from 1), `REASON` is required | any, if the decision exists, the option is another than the one chosen, and the reason is not empty. What it changes depends on the issue: see below |
| `temet-work proposal RULE keep` | a person only: the rule moves to kept and ends every brief written from then on | the rule is proposed |
| `temet-work proposal RULE drop` | a person only: the rule moves to dropped and leaves status and every brief | the rule is proposed |
| `temet-work set TARGET [--role ROLE] KEY VALUE` | a person; a refiner for the task level of the issue it refines | any |
| `temet-work unset TARGET [--role ROLE] KEY` | the same | any |
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

A refiner may add task-level settings to its plan; they are recorded as set by it:

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

The daemon starts every agent with `TEMET_WORK_AGENT` set to the agent's id, so a command
an agent runs is recorded as that agent's. A ruling is recorded as a decision, listed by
`temet-work status` with its numbered options.

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
socket above; `ssh://[USER@]HOST[:PORT][/SOCKET/PATH]` runs `ssh -o BatchMode=yes HOST temet-work relay`, with `--socket SOCKET/PATH` when the address has a path.
The hidden command `temet-work relay` copies one request line from standard input to the
local socket and the reply line back; it exits 4 when no daemon answers.
`docs/configuration.md` says how to set up a work folder on another machine.
