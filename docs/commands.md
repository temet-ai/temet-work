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
  - 3ee4a4e0-4c22-4ed9-ad8f-7d0dd362241c
---

# Commands

`temet-work` sends one command to the running daemon and prints its reply. People and
agents use the same commands; the brief each agent starts with names the one that
reports its step done.

| Command | Who runs it | Accepted when the issue is |
|---|---|---|
| `temet-work status [--json]` | anyone; `--json` prints the state as data (below) | any |
| `temet-work issue ISSUE` | anyone: the issue in full, with its text from its tracker's last sync (or one line saying why it was not read, or that it is empty), its plan, pull requests, fixes, review rounds and agents. The review rounds are every finished round in the order they finished: its number and whether it approved or blocked, and for a block each point with the author's reply to it, or "none on record", and whether the next round kept it open or resolved it, naming that round; a point no later round judged, or whose state an older log does not hold, says so. Each later line of a check, point to fix, point or reply is indented under it, and no control character is printed. An arbiter's brief holds the last of these rounds always, and as many of the earlier ones as fit in the room the rest of the brief leaves, with a line counting those it leaves out; a last round too long for the room is cut there, with a line saying so (see "What a brief holds"). `temet-work issue` still holds every check, point to fix, round, point and reply whole. The text is the one the last sync that worked listed, so an edit in the tracker shows after the next sync (`sync_every`, 60 s unless set otherwise). An issue that is merging, deploying, being verified or waiting to be reopened after a failed verify keeps the text of the last sync that listed it | any |
| `temet-work screen AGENT` | anyone: what the agent's terminal shows now | any |
| `temet-work plan ISSUE FILE` | only the refiner working on ISSUE, with the plan as YAML; `-` reads it from standard input | refining |
| `temet-work ready ISSUE PULL`, or `ready ISSUE REPO=PULL ...`, with `--reply TEXT` for each point of the block it answers | only the author working on ISSUE, once the pull requests are open and the plan's checks hold; `PULL` is a number, needed the first time, and again in a repository where a person closed the pull request on record without merging, since a read in review or the merge then drops it; an issue that changes several repositories names each as `REPO=PULL`. The first ready after a block (from the review, or from an arbiter's fix ruling, where the replies answer the reviewer's last block) carries one `--reply` for each of the block's points, in their order: what the author did about it, or why not. A ready without exactly one reply for each point is refused, and the refusal lists the points and the form to use; any other ready takes no `--reply`. Each reply is kept on its point and shown with it by `temet-work issue` and `status --json`. A ready that starts a review round sets each of the plan's checks to running, since the author reports that every check holds; after a person's overturn has waived the review, the checks keep their state. A ready naming a pull request not on record, as after one is closed, ends that waiver and asks for the review again | working, or fixing |
| `temet-work review ISSUE approve` | only the reviewer working on ISSUE. Every check of the plan is marked passed, and every point of the round before resolved; `--failed` and `--open` are refused | in review |
| `temet-work review ISSUE block POINT... [--failed N ...] [--open N ...]` | only the reviewer working on ISSUE, one argument per thing to fix. `--failed N`, once for each check that does not hold, counts from 1 in the plan's order; every check left unnamed is marked passed. After round 1, `--open N`, once for each point of the round before that is still not dealt with, counts from 1 in that round's order: the point stays open and joins this block's points, so the author fixes it again and the next round judges it again. Every point left unnamed is resolved, and a block may consist of kept points alone. A block takes at most 300 points, new and kept together: the author replies to each, and its brief lists one `--reply` for each. A block with more is refused. The reviewer's brief and each next-round message number the checks and the round before's points this way, each point with the author's reply. A number the plan or the round before does not have is refused, and so is `--open` in round 1 | in review |
| `temet-work retry ISSUE` | a person only | deploying or verifying: runs the deploy (or verify) again, at once unless a deploy or verify of the same project or `run_in` folder is running, and clears its failure count, its ten-minute wait and its reruns in `deploy-handoff.stale`, so a deploy that failed three times gets three fresh tries |
| `temet-work rule ISSUE approve REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work rule ISSUE fix REASON` | the arbiter working on ISSUE, or a person | with the arbiter |
| `temet-work choose ISSUE --question TEXT --option TEXT --option TEXT [--option TEXT ...] --took TEXT --why TEXT --cost low\|medium\|high` | any agent working on ISSUE, whatever its role; not a person, who decides through settings and overturns. It records a choice the agent made as a decision by that agent: the question, the options weighed (at least two, each once), the one it took, given as its text among the options, why, and what overturning it costs. The point is the issue's merge, the only point the loop has. Work carries on at once: the choice moves no stage, stops no agent and asks for no fix or merge. The reply names the decision, as `decision 1 is recorded`. A choice with no cost or another word for it is refused, naming low, medium and high, and so is one whose taken option is not among the options, which the refusal lists, one with an empty question or reason, and one with fewer than two options or an empty or repeated one. Each flag's text is the argument after it, whatever it says | any |
| `temet-work overturn DECISION OPTION REASON` | a person only; `DECISION` and `OPTION` are the numbers `status` lists (options count from 1), `REASON` is required | any, if the decision exists, the option is another than the one chosen, and the reason is not empty. What it changes depends on the issue: see below |
| `temet-work proposal RULE keep` | a person only: the rule moves to kept and ends every brief written from then on | the rule is proposed |
| `temet-work proposal RULE drop` | a person only: the rule moves to dropped and leaves status and every brief | the rule is proposed |
| `temet-work hold` | a person only: until a person resumes the daemon, no issue is taken, no investigation starts, and an issue with nothing merged gets no new agent, fix, review round or merge; agents at work finish their turn (see "Holding the daemon") | no issue: while the daemon is not held |
| `temet-work resume` | a person only: work starts again | no issue: while the daemon is held |
| `temet-work move ISSUE STAGE` | a person only. `STAGE` is one of the view's stage words: `new`, `refining`, `ready`, `working`, `in_review`, `fixing`, `arbitration`, `merging`, `deploying`, `verifying`, `done`; another word is refused with a list of them. Back to new the issue is refined again; back to ready a new author does it again from its plan (see "Moving an issue") | to new: refining, ready, working, in review, fixing or with the arbiter; to ready: working, in review, fixing or with the arbiter; either only while none of its pull requests has merged. Every other move is refused with its reason |
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
- `choose` is not a step: any agent working on the issue may record a choice on it. An
  agent working on another issue is refused with the issue it works on, and so is an id
  the daemon did not start. A person is refused too: a choice recorded as a person's would
  have that person overturn themselves.
- `file` and `investigated` belong to a group's running investigator, which works for its
  group and on no issue. A person, another agent, or an investigator whose investigation
  has ended is refused with the reason; a person files issues in the tracker itself.
- `hold`, `resume` and `move` are a person's alone, as `retry`, `overturn` and `proposal`
  are. An agent is refused: it finishes its step.
- After a person moves an issue, a step from an agent that started before the move is
  refused, and so are its `choose` and a refiner's `set`, with a reason that names the
  move: `plan for radar#44 is refused: a person moved radar#44 to new after refiner-3
  started, so its step is over and its session is being stopped; refiner-5 is working on
  it now`.

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
  changes something (`rule`, `overturn`, `retry`, `proposal`, `hold`, `resume`, `move`, `set`
  and `unset`) is
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
decision options are numbered from 1. Each issue lists the stages a person may move it to
(`moves`), worked out by the rule `move` is checked by, so a front end offers only moves the
daemon takes; an issue that may not be moved lists none.
Times, such as when an issue last changed, are milliseconds since 1970, and due dates are
`YYYY-MM-DD`; a value temet-work does not know, such as a due date the tracker does not
give, is `null`. `--json` on any other command is refused
with the usage; inside a value, a title, a body, a review point or a reason it is a word
like any other.

`temet-work mcp [--allow read|act]` serves the same views and commands as MCP tools over
standard input and output, for an agent's MCP client. At `read`, the default, it offers
`status`, `issue`, `screen` and `settings`; `act` adds `plan`, `ready`, `review`, `rule`,
`choose`, `retry`, `overturn`, `proposal`, `hold`, `resume`, `move`, `file`, `investigated`,
`set` and `unset`.

`temet-work parts` prints the version and the parts compiled in, without asking the
daemon.

`temet-work --version` prints one line, `temet-work <version> (<build id>)`, without
asking the daemon. The build id is the commit the binary was built from, as
`temet-workd --version` shows it.

A command an agent runs is recorded as that agent's (see "How the daemon knows who sent a
command"). A ruling and a choice recorded with `choose` are each a decision, listed by
`temet-work status` with its numbered options, the option taken, who decided and why, then
its cost to undo and its point: the issue's merge, after which overturning it costs more,
as in `cost medium, point merge`, and `point merge (passed)` once the issue is deploying,
verifying or done. A choice costs what its agent gave. An approval costs high, since the
loop does not take back a merged change, and a send-back costs low, since the author's fix
finishes either way and the issue merges once the checks pass. `status --json` gives the
cost as `low`, `medium` or `high`, and the point as `merge` with whether the issue is past
it. It gives the point no time, since nothing in temet-work knows when a merge will
happen. Only a ruling recorded before rulings kept them has neither, and `status` prints
none for it.

## Holding the daemon

`temet-work hold` holds the daemon until a person runs `temet-work resume`, and
`temet-workd --hold` starts it held (`docs/configuration.md`). While it is held:

- No issue is taken for refining, no author is placed in a work folder and no
  investigation starts. An investigation whose investigator never started gets none.
- Every issue is paused, as one that left its group's scope is: no agent starts on it, no
  fix is sent, no review round starts and nothing merges. An issue with a pull request
  merged is not paused: it still merges the rest, deploys and is verified, since stopping
  between repositories would leave main branches that do not match, and its agents work as
  without the hold, as when a refused merge sends it back to its author.
- Agents at work finish their turn, and their commands are still taken: an author's
  `ready` or a reviewer's verdict is recorded, and the step after it waits. Nothing waits
  on an investigator or an agent of a paused issue: none is reminded, none is stopped for
  sitting idle, one that ends does not count toward `starts`, and a message to one queued
  before the hold, such as a fix, waits until the resume. A person can then talk to such an
  agent in its terminal without the daemon typing into it. An agent stopped on a usage
  limit is still sent `continue` once the limit resets, since its turn is under way.
- The trackers still sync, deploys and verifies run, `retry` works, and stops the loop
  asks for, such as of an agent whose step is over, still go out.
- A person's `move` is taken: the issue's agents are stopped and its folders released at
  once, and no refiner or author starts on it until the resume (see "Moving an issue").

The hold is an event in the log, so a restart keeps it and the time it began. `status`
begins with a line saying the daemon is held, since when, and that `temet-work resume`
lifts it; `status --json` gives `held`, true or false, and `held_since`, the time it began
or `null`. A `hold` while the daemon is held, and a `resume` while it is not, are refused
with the reason. On `resume` the loop takes issues and places authors at once, and the
next round starts what each issue's stage needs.

## Moving an issue

`temet-work move ISSUE STAGE` moves an issue back to an earlier stage, as the window's
Move to does. A person runs it from a shell outside any agent's terminal; it is refused
from an agent, as `overturn` is (see "How the daemon knows who sent a command"). Two moves
are taken, both backwards, and only while none of the issue's pull requests has merged:

- Back to new, from refining, ready, working, in review, fixing or with the arbiter: the
  issue is refined again from nothing. It has no group and no plan, and is taken for
  refining again in its turn, by milestone and tracker order, once its group has a
  refining slot free, by a fresh refiner.
- Back to ready, from working, in review, fixing or with the arbiter: a new author does it
  again from its plan. It keeps its plan and its group, and each of the plan's checks goes
  back to waiting. A new author, never the old one, starts once a work folder is free in
  each repository it changes and the issues it depends on are done.

Either move:

- Stops every agent running on the issue: its refiner, author, reviewer and arbiter. Each
  one's step is over for good, also once a fresh agent works on the issue at the same
  stage. The loop asks its stop again each round until it has ended, after a restart too.
  It is not reminded, nothing more is typed into it (neither a fix or round queued for it
  before the move, nor `continue` after a usage limit), its end does not count toward
  `starts`, and its `plan`, `ready`, `review`, `rule`, `choose` or `set` is refused with a
  reason that names the move.
- Releases every work folder the issue held. No author is placed in one while an agent
  that held it runs, so the new author takes another free folder, or waits until the old
  author has ended.
- Drops the issue's pull requests, their branches and the notes of any a person closed,
  its points to fix, its approval, its check results, a waived review and its review
  rounds, so its next change starts at round 1. The pull requests stay open on the code
  host, for the person to close or reuse.
- Keeps its decisions, its issue-level settings, and what a failed verify left: the line
  under Problems and the reopen in its tracker.

A move is taken while the daemon is held, since it starts nothing: the issue's agents are
stopped and its folders released at once, and no refiner or author starts on it until a
person resumes the daemon.

The reply says the stage the issue is at now, which is the next one when the loop took it
at once: `radar#44 is new`, `radar#44 is refining` when a refining slot was free, or
`radar#44 is working` when a folder was free. It names each pull request the move dropped:
`radar#44 is ready; the move dropped its pull request 57, which is left open on the code
host for you to close or reuse`.

Every other move is refused with its reason, and nothing is logged:

| Move | Reason given |
|---|---|
| To the stage the issue is at | it is already there |
| To ready from new or refining | it has no plan yet: its refiner's plan moves it to ready |
| To refining | the loop takes an issue at new for refining when its group has a refining slot free: move it to new |
| To working | the loop starts an author on a ready issue once a work folder is free: move it to ready |
| To in_review | its author's `ready` puts it in review |
| To fixing | a reviewer's block, an arbiter's ruling, a failed check or a conflict sends it back to its author |
| To arbitration | the loop hands it to an arbiter when the review rounds run out with the reviewer still blocking |
| To merging | the loop merges it once its review and its checks have passed |
| To deploying or verifying | the loop deploys and verifies it after its merge |
| Any move of an issue merging, deploying or verifying, or with a pull request merged | the loop cannot undo a merge: the issue finishes, and once it is done a person reopens it in its tracker to have it worked again |
| To done | close it in its tracker, and the loop takes it as done at the next sync |
| Out of done | reopen it in its tracker |
| A stage word this build does not know | the reply lists the words |

The move is an event in the log, with the issue and the stage, so a restart keeps it.

## What a brief holds

Every brief, next-round message and fix message is held to 100 KiB (102,400 bytes). A
brief is one argument of the command that starts its agent, and Linux refuses one over
128 KiB; the two messages reach a running agent through herdr-core and add to its context.

A plan check (its done_when and proven_by together), kept rule, review point, point to fix
or author's reply of more than 1,000 characters is printed as its first 1,000 characters,
followed by `(cut here; temet-work issue ISSUE prints it whole)`, or for a rule
`(cut here; temet-work status prints it whole, under Rules)`. One of 1,000 characters or
fewer is printed whole. Each later line of one is indented under it, so that none reads as
another item of its list.

The parts of a brief take the room in this order:

1. The role's own text, its commands and the pull requests, whole.
2. The checks and the points it judges or answers, with the author's replies. When they
   do not fit at 1,000 characters each, each is cut shorter, to no less than 200 bytes;
   when they do not fit at that, the last are left out, and a line counts them and names
   `temet-work issue`.
3. For an arbiter, the latest review round, cut with a line saying so when it alone is too
   long. The checks before it leave it at least 16 KiB.
4. The latest kept rules that fit, after a line counting the others and naming
   `temet-work status`, which lists every kept rule whole under Rules.
5. For an arbiter, the latest earlier review rounds that fit, each whole, after a line
   counting the others and naming `temet-work issue`.

A next-round or fix message takes its room the same way, without rules. The `ready` an
author's brief and fix message end with lists one `--reply` for each point of the block,
and a block takes at most 300 points, so that list stays under 10 KB; for a block of more
from an older log, it gives the first and the last `--reply` and the number of points.
When the plain parts alone would take a text over the limit, as a long list of the issues
a person closed can in an investigator's brief, the longest is cut, with a line saying so.
The command the agent reports with is never cut.

`plan`, `review`, `ready`, `rule`, `choose` and `overturn` store their text with every
control character dropped but the line feed and the tab: ESC, NUL, VT, FF, BS, DEL and the C1
controls among them. A carriage return, alone or before a line feed, NEL (U+0085) and the
line and paragraph separators (U+2028 and U+2029) are stored as a line feed. Briefs,
messages and `temet-work issue` print text the same way when it was logged before this,
and the text a code host gives a point to fix. A choice keeps its question and each option
on one line, each run of spaces and line breaks in them stored as one space, so that
`status` lists the decision on one line and `--took` matches an option however it was
wrapped.

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
"Rules from overturned decisions"; when not all fit, it holds the latest that do (see
"What a brief holds"). Agents already running keep their old brief: new
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
