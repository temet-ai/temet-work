---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
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
its machine). `TEMET_WORK_DAEMON` picks one. The address carries the daemon's socket as its path
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
| `temet-work-codehost` | core | The code-host contract: read a pull request and its CI, merge it. A fake and the contract tests. A project's code host can differ from its tracker. |
| `temet-work-codehost-github` | codehost only | GitHub pull requests. GitLab follows as its own crate. |
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
| `temet-work-runner`, `temet-work-runtime` | the contracts, a fake each, and contract tests that fail on a runner which types into a prompt or a runtime which drops the model. The runtime family turns a decision to start an agent into a program; the runner family starts it in a labelled terminal, reports the agent's state, and holds text for an agent until its turn ends |
| `temet-work-runner-herdr` | the herdr-core socket client; passes the runner contract against a live core (`TEMET_WORK_SANDBOX= cargo nextest run -p temet-work-runner-herdr --features live --run-ignored only`) and opens nothing in other test runs |
| `temet-work-runtime-claude`, `temet-work-runtime-codex` | pass the runtime contract; the runtime family picks between them by the `runtime` setting. Claude runs `claude --permission-mode auto`, Codex `codex --approve-for-me` (workspace-write sandbox, automatic approval review); both pass on the one `PASSED_ON` list from `temet-work-runtime` |
| `temet-work-tracker` | the contract (issues, milestones, comment, close), a fake, contract tests, and reading "Depends on" lines. The family syncs each project once a minute, reports only changes, and closes verified issues |
| `temet-work-tracker-github` | issues and milestones through `gh`; passes the tracker contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-tracker-jira` | issues, fix versions as milestones, comments and Done transitions through the Jira Cloud REST API with `curl`, signed in from `JIRA_URL`, `JIRA_USER` and `JIRA_TOKEN`; same contract, live test reads one project (`--features live`) |
| `temet-work-codehost` | the contract, a fake and contract tests. The family reads the CI of each pull request in review every 30 seconds and reports it finished, with the names of failing checks; it merges when the loop asks |
| `temet-work-codehost-github` | pull requests through `gh`, merged with `--squash`; passes the contract on recorded output and, read only, against GitHub (`--features live`) |
| `temet-work-deploy` | runs the deploy command after a merge and the verify command after a deploy, each on its own thread; a failed deploy is listed as a problem, a failed verify sends the issue back to ready |
| `temet-work-store` | the event log: one JSON line per event, numbered from 1, synced before it counts; a half-written last line is dropped on reading, a damaged earlier line is an error; replaying the log gives the same state as applying the events |
| style, mike | skeletons |
| `temet-work`, the root package | the parts, the configuration (`docs/configuration.md`), `temet-workd`, which syncs GitHub issues, starts agents and answers commands on its socket, and the `temet-work` command line (`docs/commands.md`) |

Areas (`docs/areas.md`) group projects that work together; stage 1 is built, and an
issue that changes several repositories is not yet.

What does not work yet:

- A TCP transport with a token is not built: the daemon is reached over its local
  socket, or over SSH from another machine.

## Who coordinates

The daemon does, in code; agents do only what needs judgement: refining, writing code,
reviewing and arbitrating. The core's `World::apply` takes one event and returns the
effects for the daemon to carry out; after every event it takes the next issue for refining
and starts an author on a ready issue when a work folder of its project is free and the
issues it depends on are done. Review runs for at most three rounds before an arbiter
rules. Every round the daemon also starts any agent an issue needs and does not have: one
whose session ended, or one it decided on and did not start before it stopped. After three
agents of one role have ended on one issue it stops and lists the issue as a problem. It
also asks again for a merge, deploy or verify that was lost; the parts that do them do not
repeat work already done or running; a deploy that restarts the daemon is recorded by
the new daemon, which finds the build in the handoff file (`docs/configuration.md`). An agent its issue waits on that sits idle is
reminded after ten minutes and again after twenty, and its session is stopped after thirty
so a fresh one takes over. A refiner stops once its plan is in; the author, reviewer and
arbiter stop once the issue merges. A group has no coordinator agent: in the research fleet the standing seat waited 25
of its 28 minutes, and in mike's larger runs such seats were where the token and turn costs
came from (`docs/findings/fleet-run.md`, `docs/findings/mike-fleets.md`).

## How parts meet

A part never names another part. They meet through the core's events and effects, and
the host in `temet-work-core` routes them: it logs each event before applying it, then
offers each effect the event led to to every part in order until one carries it out. What
an effect leads to is handled before the next effect.

Starting an agent shows the path. The loop decides to start a refiner on an issue. The
runtime family asks the loop for the agent's id, folder and brief (`World::launch`),
makes the program with its runtime, and asks for it to run (`Effect::Run`). The runner
family starts it in a terminal labelled with the agent, group, issue and role, and reports
`AgentStarted`. From then on the runner family turns the terminal's updates into the
agent's state. Neither family names the other.

Each part is also polled every round: it returns what it has seen as events, and any
problem it has now, such as a tracker that cannot be reached. A problem stays listed
until a poll comes back without it.

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

## What the autonomy findings change

Decided on 27 November 2026. The research and the reasons are in
`docs/findings/autonomy-research.md`, and the articles behind it in
`docs/findings/autonomy-articles.md`; the finding numbers below are that document's.

temet-work has no users but its authors, and the tickets, the machines and the
repositories are theirs. A gate that stops work costs more than it saves at this stage.
So the rule is: record and count now, and add a gate only when one of the triggers below
is met. Nothing in the loop waits on a person, and that stays whole.

### Taken into the design now

Each of these makes the loop's work better or cheaper and holds nothing for a person.
None is built; they are items 7 to 11 of "What comes next".

| Found | In the design |
|---|---|
| A passing check proves little when the author controls it, and an agent's report of its own work is often wrong (finding 3). mike's floor runs a task's checks itself before any model sees the work. | A check may carry a command with an expected exit code. Before a reviewer starts, the daemon runs it in a terminal of its own in the author's work folder, on that folder's machine, so the build there is warm. It starts the run when the author's turn has ended, and first checks that the folder is clean and at the pull request's head commit. A failure, a timeout or a dirty folder goes back to the author as a fix and uses no review round; a check that fails again after that fix goes to the reviewer with its result, so the cap on rounds still ends it. A run with no result, such as a machine that cannot be reached, is a listed problem and is tried again. An author that cannot make a check pass because the check is wrong reports `blocked`. A check with no command is left to the reviewer, as now. Two contracts grow for this: a runner reports the exit code of a terminal that has ended, and a code host reports a pull request's head commit. |
| A blocked agent often finds another way to finish, and agents change code in a third to two thirds of cases where no change is needed (finding 7). | Two more steps, `blocked` and `unneeded`, each with a reason (`docs/commands.md`). `status` lists the issue with the reason, its agents are stopped, its work folder is freed and the group takes the next issue. Issues that depend on a `blocked` issue wait and are listed as waiting on it; issues that depend on an `unneeded` one start as if it were done. Each is a decision a person can overturn. |
| A reviewer that has been through a repair round grows more lenient, a judge shown its own verdicts repeats them, and a checker that reads the author's account can be steered by it (finding 4). mike's judge is not given the worker's summary. | A new reviewer session each time the pull request comes up for review; the session before it is stopped when its verdict is in. Its brief names the pull request and tells the reviewer to read the diff from the code host and the issue from the tracker, and to treat the pull request's description and the author's comments as claims to check. The brief itself carries what the daemon holds: the plan's checks, the results of the daemon's runs of them and the points that were blocking. The first round reads the whole change. A later round checks that the blocking points are met and reads the commits since for new defects; it raises nothing new about code that has not changed. A block names a `path:line` and what would satisfy it. The `starts` limit counts sessions that ended without reporting their step, so a reviewer that finished its round does not use one up. |
| A person's check adds little when it rests on the agent's account, and "nobody overturned it" is not evidence that a decision was right (finding 8). | A plain approval is recorded as a decision by the reviewer; overturning one before the merge sends the issue back to its author with the reason. `status` lists the decisions of issues that are not done and the ones overturned, and counts the rest. Every merge leaves a record the daemon writes from what it ran and saw: the commit that merged; the checks and their results; the files changed as the code host lists them, and whether they include CI, deploy or verify commands, agent instructions or settings; the review rounds it took and whether an arbiter ruled; each reviewer session with the runtime and model it was started with; and whether the daemon merged it or found it merged. The daemon counts per project: failed verifies, issues that merged more than once, follow-up issues that name the issue they fix, `blocked` and `unneeded` with how long each has waited, and settings set by a plan. |
| A learned rule is a release, and standing text grows one fix at a time unless something cuts it (finding 9). mike gives its standing text a budget that only goes down, and retires a lesson by count. | A rule has a scope (a role, a project, or all). A proposed rule starts scoped to the role that made the overturned decision; the person keeping it can change the scope and edit the text. A brief has a budget of rules, the setting `rules`; when keeping a rule would put a brief past it, the keep names a kept rule to drop, and `proposal RULE drop` works on a kept rule too. Each rule counts the issues it was given to. Trial and revert follow when there are enough issues to count. |

### Left until a trigger is met

| Left out now | Why it can wait | Trigger |
|---|---|---|
| Agents running as another user or in a sandbox, with the daemon's files and sign-in out of reach; a code-host identity of their own for agents; protected default branches (finding 5). | The agents, the machines and the repositories are the authors', and a mistake costs a revert. | Someone who is not one of the authors tells them they run temet-work, or is asked to; or it manages a repository its authors do not own; or a merge record shows a pull request the daemon found merged and did not merge itself. |
| Stronger proof of who sent a command (finding 6). | Every sender today is one of the authors or an agent they started. | The same. |
| A list of the settings a plan may set (finding 6). | A setting a refiner should not have set is listed as set by that refiner (`temet-work settings`), and a person sets it back. | A project takes tickets its authors did not write; or the count of settings set by plans shows one its issue did not ask for. |
| Ticket text marked as content in briefs, a list of whose tickets are taken, and tokens that cannot write for the roles that only read (finding 10). | The tickets are the authors'. Until a trigger is met, every group on a repository other people can open issues in is given its milestones (`docs/configuration.md`): an issue a stranger opens has no milestone and is not taken. Its title is still listed by `status`, and a stranger's comment on an issue in work can still be read by the agent working on it; both are accepted for now. | A group works such a repository with no `milestones`; or an issue the authors did not write is put into a milestone a group works. |
| A class of change held for a person: the daemon's own repository, CI and deploy commands, agent instructions, sign-in, migrations (findings 1 and 11). | Nothing waits on a person. In the open-issues fleet 14 questions went to a person and none was answered in the record (`docs/research.md`). | A managed project has users of its own or data that cannot be rebuilt; or the merge records show a merge that changed files of that class and had to be merged again. |
| A trial run of a new build on a second daemon before it replaces the live one, with the previous build kept for an automatic return (finding 11). | A daemon that breaks itself is fixed by hand by the people who wrote it. | A deploy of temet-work itself leaves the daemon down or on the wrong build and nobody sees it for a working day; or that happens twice in one month. |
| A second reviewer on another vendor's model, and a catch rate measured on seeded defects (findings 2 and 4). | A measured rate needs more merged issues than there are. | 50 issues have merged in one project and more than 6 of them merged more than once or drew a follow-up issue. The research found 11.5% of people's merged changes reverted within 90 days. |
| State files agents cannot write; a stop command for a group, a project and everything; a daily count of sessions started against sessions captured. | The machines are the authors', and nobody else uses them. Today the stop is by hand: closing an issue in the tracker stops its agents at the next sync, and stopping the daemon and the `owner=temet-work` terminals stops everything. The count needs the mike part, which is a skeleton. | Someone who is not one of the authors runs agents with temet-work; or stopping a group by hand takes more than a few minutes. |

## What comes next

In this order:

1. The settings chain (`docs/settings.md`): global, area, project, milestone and task
   levels, with roles inside each; runtimes and runners chosen by it; every family tested
   with two adapters at once.
2. The app's theme generated from the design system's tokens, then the navbar with areas
   through review.
3. The desktop window on GPUI, drawing the daemon's view data.
4. Areas, stage 2: one issue across several repositories, merged in order
   (`docs/areas.md`).
5. A GitLab code host.
6. Overturning a decision by command, and rules proposed from overturns. Done: the loop applies an
   overturn itself, and a kept rule ends every brief written afterwards (`docs/commands.md`).
7. The floor: a check can carry a command with an expected exit code, which the daemon
   runs at the pull request's head commit before review (`docs/objects.md`).
8. `blocked` and `unneeded` as steps a refiner or an author can report
   (`docs/commands.md`).
9. A new reviewer session each round, briefed with the diff, the issue and the daemon's
   check results.
10. A record of each merge written by the daemon, a plain approval recorded as a decision,
    and counts of reverts, follow-up fixes and failed verifies.
11. Rules with a scope, a budget and counts.

Items 8 and 9 are small and can be taken whenever the loop is next worked on, ahead of
their place here: 8 changes the loop and the commands, 9 the loop, the briefs and when the
runtime family starts a reviewer. Item 7 is larger: it needs the pull request's head
commit from the code host and a command's exit code from the runner, in both contracts
and their adapters.
