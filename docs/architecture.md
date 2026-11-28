---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 26cd49d9-7565-4506-b988-a9b64ed0fa45
  - 3f18e0c3-9e38-42e7-8d05-5740c115b38d
  - c27ea5f8-79a8-4f76-ae9d-52bda3729faa
  - d1ccda48-cbb5-44cc-9ea0-c03999c970e9
  - 0645e4e2-33bd-43f8-980f-503649da2f15
  - edd1e649-f07a-4d7c-b5c6-ff74b0021fbb
  - c14f6d16-7688-4e50-99d4-32021f0e29db
  - dbf1ab25-b936-427b-b27c-7106d8e0575d
  - fc025b57-8c0b-49dd-916e-cbc98c3f9cf3
  - ba3664be-ec7f-4b51-af6a-b1c20c80634d
  - 4fdc790b-4921-4d0a-9032-f3a83dbaf0ef
  - 3ee4a4e0-4c22-4ed9-ad8f-7d0dd362241c
  - 7d4c9131-a05c-43ac-ba6d-09406c95b2c6
---

# temet-work architecture

This document says how temet-work's code is divided, which rules keep the divisions in
place, how it is extended and how it is tested. Each rule that a test checks names that
test.

## What the system does

temet-work runs groups of coding-agent sessions through tickets: it refines a ticket,
works on it, coordinates the agents, reviews the change, checks that it is correct,
merges and deploys it, and takes the next ticket. It works across repositories, and
groups of sessions work side by side without waiting on each other. Agents decide with a
recorded default and carry on; a person reads those decisions afterwards and can overturn
them. Nothing in the loop waits on a person.

## Processes

| Process | Owns |
|---|---|
| `temet-workd`, the daemon | All state and all work. It keeps running when every window is closed. |
| herdr-core (tmux to come) | The terminals agents run in. They outlive the daemon too. |
| The desktop app (GPUI) | Nothing. It draws the daemon's view data and sends commands. |
| `temet-work`, the CLI, and its MCP server | Nothing. They send the same commands and read the same view data as the app, so an agent sees what a person sees. |

The app, the CLI and the MCP server reach the daemon through one transport interface
(`Transport` in `src/socket.rs`): a client sends a request and gets a reply, and has no
idea how it travelled. There are two implementations, the daemon's local socket and
SSH (`ssh HOST temet-work relay`, which copies one JSON line to the daemon's socket on
its machine, and refuses a client whose build is not the running daemon's).
`TEMET_WORK_DAEMON` picks one. The address carries the daemon's socket as its path
(`ssh://omit/run/user/1000/temet-work/daemon.sock`), because sshd passes no environment
and a second daemon on one machine has to be told apart; no path means the default
socket. A TCP transport with a token, on a private
network, can be added as a later issue without changing the callers. A command exists
once, in the daemon, and every front end uses it by name.

## Crates

The workspace has one small core. Every other crate depends on the core or on its family
crate, and on no other crate in the workspace. This table is the design; the one after it
says what each crate holds today.

| Crate | Depends on | Holds |
|---|---|---|
| `temet-work-core` | nothing in the workspace | The objects (area, project, milestone, issue, plan, check, group, agent, decision, rule), the settings chain (`docs/settings.md`), the work loop as a pure state machine (an event goes in, state and effects come out), commands, and view data with a text form. The `Part` contract. No I/O, no tool or service names, no UI types. |
| `temet-work-tracker` | core | The tracker contract, a fake, and the contract tests every tracker adapter runs. |
| `temet-work-tracker-github` | tracker only | GitHub. GitLab follows as its own crate. |
| `temet-work-tracker-jira` | tracker only | Jira Cloud (REST v3, `curl`). |
| `temet-work-runner` | core | The runner contract: start a labelled terminal, read it, send to it, wait on it, follow its events. A fake and the contract tests. |
| `temet-work-runner-herdr` | runner only | herdr-core. tmux follows as its own crate. |
| `temet-work-runtime` | core | The agent runtime contract: start a session with a brief and a model, tell working from idle. A fake and the contract tests. |
| `temet-work-runtime-claude` | runtime only | Claude Code. |
| `temet-work-runtime-codex` | runtime only | Codex. |
| `temet-work-codehost` | core | The code-host contract: read a pull request and its CI with each failing check's log, merge it, read the main pipeline after the merge, and give a repository's clone address and the ref of a pull request's head. One shared bare copy of each repository, beside the event log, that a pull request's change, a file and a search are read from without a checkout. A fake and the contract tests. A project's code host can differ from its tracker. |
| `temet-work-codehost-github` | codehost only | GitHub pull requests. |
| `temet-work-codehost-gitlab` | codehost only | GitLab merge requests. |
| `temet-work-deploy` | core | Runs a project's deploy and verify commands after a merge. |
| `temet-work-store` | core | SQLite and the event log. |
| `temet-work-style` | core | The style guide: tokens, the generated palette, the ban list as data, and the check used when reviewing UI work. |
| `temet-work-mike` | core | Optional. Reads capture, corrected turns and lessons from a mike proxy as input to the improvement loop. |
| `temet-work`, the root package | core, and each part behind its own feature | Wiring only: `src/lib.rs` registers the compiled-in parts. |

The desktop app will be the one front-end crate. It may depend on the core and on
`temet-work-style`, and it is the only crate allowed a UI library.

What exists today:

| Crate | State |
|---|---|
| `temet-work-core` | the objects, the work loop, the host that routes effects to parts, agents' briefs, the state as text and, beside it, as view data (`view.rs`) that the `view` command answers with, the daemon's reply, and the rule for where its socket is. The host applies each event at the time it logs it with, so the world knows when each issue last changed, and puts in the world each round the web addresses the tracker and code-host families give |
| `temet-work-runner`, `temet-work-runtime` | the contracts, a fake each, and contract tests that fail on a runner which types into a prompt or a runtime which drops the model. The runtime family turns a decision to start an agent into a program; the runner family starts it in a labelled terminal, reports the agent's state, holds text for an agent until its turn ends, and text about a paused issue until it is no longer paused, and types `continue` into an agent whose usage limit has reset. A terminal reported gone (its program ended by itself, or it was stopped or removed) or lost (its program was lost, as to a signal or a reboot) ends its agent in that poll, and the end records whether the program was lost. The daemon kills and removes the terminal, so nothing stays under the agent's labels, such as the shell herdr-core puts in place of a lost program. Any stop that fails is tried again a minute after the last try, until it works or the runner answers that the terminal is gone or unknown: a stop the loop asked for, the stop of an ended agent's terminal, and the stop after an agent's last reminder while the agent stays idle and its issue still waits on it. When the agent had not ended, the stop that works, or that answer, reports it ended. A gone or lost terminal of an agent that has already ended, as after a restart that lost a stop, is stopped once |
| `temet-work-runner-herdr` | the herdr-core socket client; puts each agent's terminal on the herdr app's screens by starting it with `herdr.space`, `herdr.tab` and `herdr.name`, one space per group, one tab per issue and one for the group's investigator; labels every terminal with `program`, the first word of its argv. An agent whose program was lost has ended even when herdr-core has put a shell in its place, as it does in a terminal the herdr app keeps: a listed terminal reads as lost while it is live with a `reason` (the shell is coming), or once its argv no longer starts with its `program` label (the shell is there). A terminal started before that label reads as lost when it is an agent's and its argv is one word, since no agent is started as one word. herdr-core's `lost` event reads as lost too; a `finished` terminal and a `finished` or `removed` event read as gone. A terminal missing from the first list after the client subscribes again, which follows a lost connection after which herdr-core may have restarted, reads as lost with the labels it had, so its agent ends as lost, not gone; one missing from a newer list on the same connection (after `missed`) was removed, and reads as gone. A call that sent nothing, because the event subscription or the control connection could not be opened or the request could not be written, fails as unreached, so a stop that never reached herdr-core is told apart from one whose kill may have landed (`rm` timing out or refusing, the connection closing after the kill). A request whose line cannot be written on a connection kept from before, as after herdr-core restarted, is sent once more on a new connection, so a stop after a restart reaches the new core. A terminal's latest status, which the runner family reads before the stop after an agent's last reminder, comes from the updates herdr-core has sent that the family has not taken yet, and reading it leaves them for the next poll; passes the runner contract against a live core (`TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-runner-herdr --features live --run-ignored only`) and opens nothing in other test runs |
| `temet-work-runtime-claude`, `temet-work-runtime-codex` | pass the runtime contract; the runtime family picks between them by the `runtime` setting. Claude runs `claude --permission-mode auto`, Codex `codex --approve-for-me` (workspace-write sandbox, automatic approval review); both pass on the one `PASSED_ON` list from `temet-work-runtime` |
| `temet-work-tracker` | the contract (issues, milestones with due dates, the address of a project's issues, comment, close, reopen, create), a fake, contract tests, and reading "Depends on" lines. The family syncs each project once a minute, reports only changes to issues and to the open milestones and their due dates, keeps each open issue's text from the last sync in memory for `temet-work issue`, and closes verified issues. A close in the tracker is not passed to the loop while the issue merges, deploys or is verified, since its pull request may have closed it. After a failed verify the family reopens the issue in its tracker, at once and again at each sync until a sync lists it as open, and passes no close of it until then; an issue being finished or reopened keeps the text of the last sync that listed it. A later close is a person's and finishes the issue |
| `temet-work-tracker-github` | issues and milestones through `gh`, issues filed with `gh issue create`, and closes and reopens with `gh issue close` and `gh issue reopen`; passes the tracker contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-tracker-jira` | issues, fix versions as milestones, comments, new issues, and transitions to Done and out of it through the Jira Cloud REST API with `curl`, signed in from `JIRA_URL`, `JIRA_USER` and `JIRA_TOKEN`; same contract, live test reads one project (`--features live`) |
| `temet-work-codehost` | the contract, a fake and contract tests. The family reads each pull request in review every 30 seconds, reports a conflict and each one a person closed, and otherwise reports CI finished once it has on all of an issue's pull requests, with the names of failing checks; a pull request in review that still shows no run at a later read, while its repository's `ci` is true, is listed as a problem until it has one; it merges them when the loop asks, in the plan's order, each after the previous repository's main pipeline passed, and sends a draft back to its author in place of merging it. A pull request a person closed without merging, found at a read in review or at the merge, is dropped from the issue and its CI no longer counts; the issue goes back to its author, or, already there, gets one more fix, and its running reviewer and arbiter are stopped, so the author reopens it if its branch still exists or opens another, and names that with `ready`. Each pull request's branch is reported the first time a read finds it and again when it changes, and an author's brief names it with how to check it out. It answers `change`, `show` and `search` from one bare copy of each repository (no working tree, no build folder), kept under `copies/` beside the event log and shared by every reader: the first read clones it from the address the adapter gives, a read at the default branch's head or a pull request's head (the ref the adapter names) fetches that head first, and a read at a commit the copy holds fetches nothing. Those reads run as work the daemon puts on a thread of its own, so the loop does not wait on a fetch. It answers `ci` from the adapter: each check's state and each failing one's log (`docs/commands.md`, "Reading code and CI") |
| `temet-work-codehost-github` | pull requests through `gh`, with whether each is a draft and its branch, merged with `--squash`; each check from `gh pr checks` and each failing job's log from `gh run view --job JOB --log-failed`; clones from `https://HOST/REPO.git` and reads a pull request's head at `refs/pull/N/head`; passes the contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-codehost-gitlab` | merge requests through `glab` and its sign-in, with each one's source branch, merged with `--squash`, `--rebase` or neither; a pipeline reads as failed once a job that may not fail has failed; each job of a merge request's pipeline, with each failed job's log; clones from `https://HOST/REPO.git` and reads a merge request's head at `refs/merge-requests/N/head`; passes the contract on recorded output and, read only, against GitLab (`--features live`) |
| `temet-work-deploy` | runs the deploy command after a merge and the verify command after a deploy, each on its own thread, one at a time per project and `run_in` folder; a failed deploy is listed as a problem, a failed verify sends the issue back to ready and lists it as a problem until it is done; a handoff line naming a build other than the daemon's runs the deploy again, up to three times, then waits for `temet-work retry ISSUE` |
| `temet-work-store` | the event log: one JSON line per event, numbered from 1, synced before it counts; a half-written last line is dropped on reading, a damaged earlier line is an error; replaying the log gives the same state as applying the events |
| `temet-work-style` | the design system's tokens as Rust constants, written by `scripts/sync-tokens`; the ban list; tests that hold the tokens to the design system's rules. `docs/ui.md` maps the app's parts to the design system |
| mike | a skeleton |
| `temet-work`, the root package | the parts, the configuration (`docs/configuration.md`), `temet-workd`, which syncs GitHub issues, starts agents and answers commands on its socket after reading who sent each, and the `temet-work` command line (`docs/commands.md`) |

Areas (`docs/areas.md`) group projects that work together; stages 1 and 2 are built, so an
issue can change several repositories, with a work folder and a pull request in each,
merged in its plan's order.

What does not work yet:

- A TCP transport with a token is not built: the daemon is reached over its local
  socket, or over SSH from another machine.

## Who coordinates

The daemon does, in code; agents do only what needs judgement: refining, writing code,
reviewing and arbitrating. The core's `World::apply` takes one event and returns the
effects for the daemon to carry out; after every event it takes the next issue for refining
and starts an author on a ready issue when a work folder is free in every repository it
changes (on the issue's `machine`, when that setting names one) and the issues it depends
on are done. Review runs for at most three rounds before an arbiter
rules. Every round the daemon also starts any agent an issue needs and does not have: one
whose session ended, or one it decided on and did not start before it stopped. After three
agents of one role have ended on one issue while it waited on them, it stops and lists the
issue as a problem. An agent whose program was lost, as on a reboot, does not count toward
those three. Such ends count separately, up to three times `starts` (nine by default), so a
program lost at the same point every run, as to an out-of-memory kill, is not started forever:
after that many the daemon stops starting that role and lists the issue as a problem with a line
of its own that says their programs were lost. Like the three, a lost end counts only while the
issue waited on the agent. The daemon
also asks again for a merge, deploy or verify that was lost; the parts that do them do not
repeat work already done or running; a deploy that restarts the daemon is recorded by
the new daemon, which finds the build in the handoff file (`docs/configuration.md`). An agent its
issue waits on that sits idle is reminded after ten minutes and again after twenty, and its session
is stopped after thirty so a fresh one takes over. A stop that fails, as when the terminal
service cannot be reached or a request times out, is tried again once a minute has passed since the
last try, with no further reminder, and is listed under Problems while it keeps failing; no fresh
agent starts while the old one runs. A stop answered with the terminal gone or unknown ends the
agent. Before that stop the runner family reads the latest status its runner holds for the agent's
terminal: when the runner already reports the program lost, as when the loss arrived after the
poll read its updates, the agent ends as lost and its terminal is still stopped. That status is
read again after a stop answered gone or unknown, which catches a loss the stop's own call brought
in from a herdr-core that restarted, and not after a stop that worked, whose kill caused any loss.
Only a loss the daemon did not cause ends an agent as lost. Once a refused stop may have reached
the runner, which may have carried out the kill before refusing (herdr-core's `rm` timing out, or
finding the terminal still live), a later report of the program lost, in a poll or before a retry,
is an ordinary end that uses a start. A refusal that sent nothing, such as a socket that refuses
the connection, leaves a later loss lost, unless an earlier try may have reached the runner. While the daemon has no terminal for an agent, as before its runner has reported any after a
restart, the agent is not ended either. Between tries an agent in a turn, at a prompt or waiting on
a limit, or one its issue no longer waits on, is not stopped, and the rules below say whether its
count is kept; the problem goes when the stop works or the count is dropped. An agent whose turn
ends in an error, such as a usage limit, is listed under Problems with the line its turn ended on:
the terminal's status message, or else the line on its screen that names the error, read once. When
that line gives a
reset time the runner family can read (`resets 5pm`, `resets 5:40pm` or `resets 17:40`, in the
daemon's time zone; am or pm may follow a space in any case, as in `resets 5:40 PM`), the daemon
types `continue` one minute after it, without starting a new agent, and text waiting for the agent
goes after that. A time up to an hour past counts as passed, whether it is yesterday's, today's or
tomorrow's, so a limit seen just after its reset is continued at once. The reset time is kept in the
event log, so a restart does not lose it. A time more than six hours ahead is not read, and the
agent is treated as idle: reminded, then stopped. The same happens when an error has no reset time
the daemon can read, or comes back with a reset time no later than the one already continued after.
A reminder starts a turn, which may end on the same error, or on another with no reset time to
wait for. When the daemon saw that turn working for at least `remind_after` (ten minutes by
default) before the error came back, the turn counts as recovery and a new count starts at that
error. A shorter turn counts toward the stop: the next reminder or the stop is still timed from
the last reminder. The count also starts again when such a turn ends between turns, when the
agent stops at a prompt or on a limit with a reset time, when it ends, and when its issue stops
waiting on it. A reviewer whose approval is in is not
waited on while the checks run: it is not reminded, no other is started in its place, and it stays
until the merge, so after a failed check a reviewer still running hears the next round. A refiner
stops once its plan is in and an arbiter once its ruling is in. The arbiter also stops when its
issue goes back to the author by a failed check, a conflict, a refused merge, a person's
overturn or a pull request a person closed, so a later arbitration on the issue starts a fresh
arbiter that hears that dispute. A pull request a person closed stops a running reviewer too, at
whatever stage the close is found, since a next round's message would not name the one that
replaces it; that reviewer ends while the issue is being fixed, so it does not count against
`starts`.
No second arbiter starts as long as one whose stop failed still runs. The daemon asks again each
round to stop one that has ruled, since nothing reminds it, and starts a fresh one once it has
ended. An approval by ruling is not replaced while the checks run. The author and reviewer stop once the issue's last pull request merges.

Any stop the loop asks for that fails, such as the author's and reviewer's after the merge, an
agent's on an issue closed in its tracker or a refiner's once its plan is in, is tried again by the
runner family a minute after the last try. It is tried until it works or the runner answers that
the terminal is gone or unknown, and the agent is then reported ended. As at the stop after a last
reminder, a loss reported once a refused try may have reached the runner is an ordinary end. The
tries stop when the agent's issue waits on it again, as on a reviewer whose stop failed after a
pull request was closed and that has since heard the next round: the loop has put it back to work.
When the loop asks for the same stop before that minute is up, the answer is the last try's reason
and nothing is tried. While the stop fails, Problems lists `stop AGENT`, with the reason the runner
family gave the last time the loop asked for that stop. The loop asks again each round for an agent
whose step is over for good (below), so for such an agent the reason is the latest try's. For any
other, such as a reviewer stopped for a closed pull request while its issue is being fixed, it
stays the reason of the first try while the runner family keeps trying once a minute. The line goes
once the agent has ended, whatever ended it.

A replay at start drops effects, so a stop that failed or was lost to a restart is asked for again:
each round the daemon asks to stop every running agent whose step is over for good. That is any
agent on an issue that is done, deploying or verifying, or back to new or ready after a reopen in
the tracker, a failed verify or a person's move; any agent started before a person last moved its
issue (`temet-work move`, `docs/commands.md`), whatever stage the issue has reached since, so a
fresh agent takes its role and the runner family types nothing more into it; any but the refiner
on an issue being refined; a refiner whose
issue's plan is in; an author whose issue holds other folders than the ones it held, as after a
failed verify while that author's stop kept failing; and an investigator whose investigation has
ended. It asks on a paused issue too, since a stop starts no work. Nothing waits on such an agent:
it is not reminded, and its end does not count against `starts`. It asks for none an issue still
needs: its author or reviewer while it is in review, being fixed or merging, its refiner while it
is refining, and the investigator of a running investigation. An author holds its issue's work
folders, one in each repository the issue changes. A merge or a close in the tracker takes them
from the issue, and no author is placed in one while an agent that held it still runs, since that
session may still change the folder. A refiner, reviewer, arbiter or investigator holds no folder.
It runs in an empty folder of the daemon's for its project, `readers/PROJECT` beside the event log,
which is made on the agent's machine before it starts. That folder is no checkout, and no author
is given it. Such an agent never waits for a work folder. An author holds its issue's folders all
through review, so a reviewer that waited for a free one would never start.

A group has no coordinator agent: in the research fleet the standing seat waited 25 of its 28
minutes, and in mike's larger runs such seats were where the token and turn costs came from
(`docs/findings/fleet-run.md`, `docs/findings/mike-fleets.md`).

A group takes for refining, and starts authors on, only issues its milestones, projects and
areas cover. Whether an issue it took is still covered is worked out from the issue and the group
each time and is not stored, so a log replays to the same state. An issue that leaves its group's
scope before any of its pull requests merged is paused (`World::paused`): `World::apply` drops the
effects that would start an agent, send a fix, start or announce a review round, or ask for a
merge, `World::missing` returns nothing for it, and `World::waiting_on` is false for an author
that has not heard its fixes, so it is not reminded into working. Agents already working finish
their step, since stopping one mid-turn throws away a plan half written or edits not yet
committed, and a milestone move is often undone. The issue keeps its folders and its idle author.
A paused ready issue goes to another group that covers it, since it holds no agent or folder; a
paused refining issue whose refiner has stopped goes back to new, so it holds no refining slot.
An issue with a pull request merged still merges the rest, deploys and is verified: stopping
between repositories would leave main branches that do not match, and a merged change has to
deploy to match main. When the issue is covered again, `World::missing` starts or tells the
agent its stage needs, or asks for the merge.

A person can hold the daemon (`temet-work hold`, or `temet-workd --hold` at start) and
resume it (`docs/commands.md`, "Holding the daemon"). The hold is an event, so a restart
keeps it. While it holds, `World::place` takes no issue and places no author, no
investigation starts, every issue with nothing merged is paused as one out of scope is
(`World::paused`), and nothing waits on an investigator or an agent of a paused issue
(`World::waiting_on`): no reminder is typed into its terminal and no idle session is
stopped, since the person may be talking to it there, and the runner family keeps any
message about a paused issue until it is no longer paused. Agents at work finish their
turn, for the reason above, and an issue with a pull request merged is not paused: it
merges, deploys and is verified, with the agents that takes.

When a group's work queue runs short, the daemon starts one investigator for it. It holds no
work folder and runs in the daemon's empty folder for the group's first repository. It reads
the group's projects and files the next issues through `temet-work file`
(`docs/configuration.md`). The daemon counts the queue, keeps to one investigator per group
and waits `investigate_every` between investigations; the clock comes into the core as the
time `Host::tick` takes, and each investigation's start and end times are on its events.

## How parts meet

A part never names another part. They meet through the core's events and effects, and
the host in `temet-work-core` routes them: it logs each event before applying it, then
offers each effect the event led to to every part in order until one carries it out. What
an effect leads to is handled before the next effect.

Starting an agent shows the path. The loop decides to start a refiner on an issue. The
runtime family asks the loop for the agent's id, folder and brief (`World::launch`),
makes the refiner's folder when it is on this machine, makes the program with its runtime,
and asks for it to run (`Effect::Run`). For a folder on another machine the runtime also
gives the program that trusts the folder there, which first makes the folder for an agent
that holds no work folder, such as this refiner. The runner family runs that program on the
machine first (`Runner::exec`). The runner family starts the
agent's program in a terminal labelled with the agent, group, issue and role, and reports
`AgentStarted`. From then on the runner family turns the terminal's updates into the
agent's state. Neither family names the other.

Each part is also polled every round: it returns what it has seen as events, and any
problem it has now, such as a tracker that cannot be reached. A problem stays listed
until a poll comes back without it.

## Who sent a command

The runner family makes each agent a random secret when it starts it, puts it in the
agent's environment, and records only its SHA-256 digest, in `AgentStarted`. For each
request the daemon hashes the secret the request carries and, on Linux, the secrets it
finds in the environments of the sending process and its parents (`src/peer.rs`, through
`SO_PEERCRED` and `/proc`), relayed or not; `temet-work relay` marks what it forwards
from another machine, and the mark only adds refusals. The core's `World::admit` compares those digests with the ones on record and
refuses a request it cannot take at its word, before the step checks run
(`docs/commands.md`). Making a secret and reading another process are I/O, so the core
does neither.

## Rules the tests enforce

| Rule | Test |
|---|---|
| No part depends on another part, and the core depends on no part. | `tests/parts.rs` `no_part_depends_on_another_and_core_depends_on_none` |
| An adapter depends on its family crate only. | `an_adapter_depends_on_its_family_only` |
| No part names another part's crate or id, and the core names none. | `no_part_names_another_and_core_names_none` |
| The core names no tool or service (GitHub, GitLab, Jira, Linear, herdr, tmux, Claude, Codex, mike, Slack). | `core_names_no_tool_or_service` |
| Only a front end uses a UI library. | `no_ui_library_outside_a_front_end` |
| Each part and adapter has one feature line in `Cargo.toml` and one `#[cfg]` block in `src/lib.rs`. | `the_root_registers_each_part_behind_its_feature` |
| Every mix that matters builds, and `parts()` returns exactly the parts compiled in. | `tests/mix.rs` through `scripts/check-mixes` |

The mixes are: no parts, each part alone, all parts, each adapter with its family, and
everything. A test of every combination is not run: the guard keeps parts from depending
on each other, so what two parts do together shows up in the all-parts build.

## Adding an adapter in this repository

1. Make `crates/<family>-<tool>/` with a crate `temet-work-<family>-<tool>` that depends
   on `temet-work-<family>` only.
2. Add it to `[workspace.dependencies]`, add the feature
   `<family>-<tool> = ["<family>", "dep:temet-work-<family>-<tool>"]` and the optional
   dependency in the root `Cargo.toml`.
3. Add it to its family inside the family's block in `src/lib.rs`, behind
   `#[cfg(feature = "<family>-<tool>")]`.
4. Run the family's contract tests against it, then `scripts/check-mixes`.

## Extending from outside the repository

Decided on 26 November 2026; the research and the reasons are in `docs/extensions.md`.
Trackers and the parsing half of checks are WebAssembly components with enforced limits;
runners and agent runtimes are subprocesses speaking JSON-RPC lines over stdio. Each
family has one contract, generated from its Rust types, and an outside extension runs the
same contract tests as an adapter in this repository.

In the crate layout each transport is an adapter of its family: a Wasm loader in the
tracker family (and in the check family once it exists), and a subprocess bridge in the
runner and runtime families. Wasmtime is compiled only into the Wasm loaders, behind their
features.

## Tests

| Rule | Where it is set |
|---|---|
| A test over 2 s is flagged slow and one over 20 s is killed. | `.config/nextest.toml` |
| A crate's whole suite stays under 10 s; CI fails when one goes over. | `scripts/test-budget`, run in CI |
| Most tests are pure core tests: an event goes in, state and effects are checked. They run in milliseconds. | |
| Adapters are tested through their family's contract tests, with fakes and recorded streams, never live services. | |
| Storage tests write a fresh log file per test under `CARGO_TARGET_TMPDIR`; there is no database tier. | |
| Tests against live services run nightly, behind a feature. | |
| A crate's tests test that crate with its own fixtures. A change that fails another crate's tests is a coupling fault to fix, not a snapshot to record again. | |
| While working, run only the changed crate: `cargo nextest run -p <crate>`. CI runs the workspace, the budget and the mixes. | `.github/workflows/ci.yml` |
| Every test runs sandboxed (`TEMET_WORK_SANDBOX=1`) and in UTC. | `.cargo/config.toml` |

## What comes next

In this order:

1. The settings chain (`docs/settings.md`): global, area, project, milestone and task
   levels, with roles inside each; runtimes and runners chosen by it; every family tested
   with two adapters at once. Done: the levels and the roles inside them are built, the
   `runtime` setting chooses between Claude Code and Codex and the `runner` setting chooses
   the runner, and the tracker, code-host, runtime and runner families are each tested with
   two adapters.
2. The app's theme generated from the design system's tokens, then the navbar with areas
   through review (`docs/ui.md`).
3. The desktop window on GPUI, drawing the daemon's view data.
4. Areas, stage 2: one issue across several repositories, merged in order
   (`docs/areas.md`).
5. A GitLab code host. Done: `temet-work-codehost-gitlab`. GitLab as a tracker is not built.
6. Overturning a decision by command, and rules proposed from overturns. Done: the loop applies an
   overturn itself, and a kept rule ends every brief written afterwards (`docs/commands.md`).
