---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 26cd49d9-7565-4506-b988-a9b64ed0fa45
  - 3f18e0c3-9e38-42e7-8d05-5740c115b38d
  - c27ea5f8-79a8-4f76-ae9d-52bda3729faa
  - d1ccda48-cbb5-44cc-9ea0-c03999c970e9
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
| `temet-work-codehost` | core | The code-host contract: read a pull request and its CI, merge it, read the main pipeline after the merge. A fake and the contract tests. A project's code host can differ from its tracker. |
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
| `temet-work-core` | the objects, the work loop, the host that routes effects to parts, agents' briefs, and the state as text |
| `temet-work-runner`, `temet-work-runtime` | the contracts, a fake each, and contract tests that fail on a runner which types into a prompt or a runtime which drops the model. The runtime family turns a decision to start an agent into a program; the runner family starts it in a labelled terminal, reports the agent's state, holds text for an agent until its turn ends, and types `continue` into an agent whose usage limit has reset |
| `temet-work-runner-herdr` | the herdr-core socket client; puts the terminal of an agent on an issue on the herdr app's screens (an investigator's has no issue and stays off them) by starting it with `herdr.space`, `herdr.tab` and `herdr.name`, one space per group and one tab per issue; passes the runner contract against a live core (`TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-runner-herdr --features live --run-ignored only`) and opens nothing in other test runs |
| `temet-work-runtime-claude`, `temet-work-runtime-codex` | pass the runtime contract; the runtime family picks between them by the `runtime` setting. Claude runs `claude --permission-mode auto`, Codex `codex --approve-for-me` (workspace-write sandbox, automatic approval review); both pass on the one `PASSED_ON` list from `temet-work-runtime` |
| `temet-work-tracker` | the contract (issues, milestones, comment, close, create), a fake, contract tests, and reading "Depends on" lines. The family syncs each project once a minute, reports only changes, and closes verified issues |
| `temet-work-tracker-github` | issues and milestones through `gh`, and issues filed with `gh issue create`; passes the tracker contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-tracker-jira` | issues, fix versions as milestones, comments, new issues and Done transitions through the Jira Cloud REST API with `curl`, signed in from `JIRA_URL`, `JIRA_USER` and `JIRA_TOKEN`; same contract, live test reads one project (`--features live`) |
| `temet-work-codehost` | the contract, a fake and contract tests. The family reads each pull request in review every 30 seconds, reports a conflict, and reports CI finished once it has on all of an issue's pull requests, with the names of failing checks; it merges them when the loop asks, in the plan's order, each after the previous repository's main pipeline passed |
| `temet-work-codehost-github` | pull requests through `gh`, merged with `--squash`; passes the contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-codehost-gitlab` | merge requests through `glab` and its sign-in, merged with `--squash`, `--rebase` or neither; a pipeline reads as failed once a job that may not fail has failed; passes the contract on recorded output and, read only, against GitLab (`--features live`) |
| `temet-work-deploy` | runs the deploy command after a merge and the verify command after a deploy, each on its own thread, one at a time per project and `run_in` folder; a failed deploy is listed as a problem, a failed verify sends the issue back to ready; a handoff line naming a build other than the daemon's runs the deploy again, up to three times, then waits for `temet-work retry ISSUE` |
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
agents of one role have ended on one issue it stops and lists the issue as a problem. It
also asks again for a merge, deploy or verify that was lost; the parts that do them do not
repeat work already done or running; a deploy that restarts the daemon is recorded by
the new daemon, which finds the build in the handoff file (`docs/configuration.md`). An agent its
issue waits on that sits idle is reminded after ten minutes and again after twenty, and its session
is stopped after thirty so a fresh one takes over. An agent whose turn ends in an error, such as a
usage limit, is listed under Problems with the line its turn ended on: the terminal's status
message, or else the line on its screen that names the error, read once. When that line gives a
reset time the runner family can read (`resets 5pm`, `resets 5:40pm` or `resets 17:40`, in the
daemon's time zone; am or pm may follow a space in any case, as in `resets 5:40 PM`), the daemon
types `continue` one minute after it, without starting a new agent, and text waiting for the agent
goes after that. A time up to an hour past counts as passed, whether it is yesterday's, today's or
tomorrow's, so a limit seen just after its reset is continued at once. The reset time is kept in the
event log, so a restart does not lose it. A time more than six hours ahead is not read, and the
agent is treated as idle: reminded, then stopped. The same happens when an error has no reset time
the daemon can read, or comes back with a reset time no later than the one already continued after.
A reminder starts a turn, and one that ends on the same error, or on another with no
reset time to wait for, counts toward the stop, however long the turn worked: the next reminder
or the stop is still timed from the last reminder. The count starts again when such a turn ends
between turns, when the agent stops at a prompt or on a limit with a reset time, when it ends,
and when its issue stops waiting on it. A reviewer whose approval is in is not
waited on while the checks run: it is not reminded, no other is started in its place, and it stays
until the merge, so after a failed check a reviewer still running hears the next round. A refiner
stops once its plan is in and an arbiter once its ruling is in, so a later arbitration on the issue
starts a fresh arbiter that hears that dispute; an approval by ruling is not replaced while the
checks run. The author and reviewer stop once the issue's last pull request merges. A group has
no coordinator agent: in the research fleet the standing seat waited 25 of its 28 minutes, and in
mike's larger runs such seats were where the token and turn costs came from
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

When a group's work queue runs short, the daemon starts one investigator for it, which holds
no folder, reads the group's projects and files the next issues through `temet-work file`
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
makes the program with its runtime, and asks for it to run (`Effect::Run`). For a folder
on another machine the runtime also gives the program that trusts the folder there, and the
runner family runs that on the machine first (`Runner::exec`). The runner family starts the
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
