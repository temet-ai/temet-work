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

## How things are said

Decided on 27 November 2026. It rests on work already done and measured: mike's mail,
questions, notes and delivery routes (`docs/findings/mike-fleets.md`, and
`docs/INJECTION_TRUST.md` in mike), the dispute run over mike mail and `hc send`
(`docs/findings/agent-talk.md`), herdr-core (`docs/findings/herdr-core.md`) and ask-kit.
The route decides how a session treats a piece of text, so each kind of thing has one
route. Of the Route column, these exist today: the brief as the first prompt, prompts
typed through the runner, the step commands, `status`, a person's commands and a person
typing in a terminal. The rest of that column and all of the last column are not built;
they are items 7 and 13 of "What comes next".

| Who says it | Route | Why this route | Not built yet |
|---|---|---|---|
| The daemon to an agent: the brief, points to fix, a check's result, a reminder | The brief as the first prompt, and after that a prompt typed through the runner between turns. | Typed text needs nothing loaded in the session and no mike, stays in the transcript and is read as the user's turn: under 1 s to an idle session, held until a busy one's turn ends, refused at a permission prompt. The daemon has no session of its own, so mike mail from it would arrive as from an unknown sender, and an idle session started without Remote Control and the mike channel does not read its mail. Text the proxy injects is seen for one request, and a message on mike's channel was acknowledged and then declined. A Remote Control push and a mod's prompt were kept and acted on in mike's samples, and each needs mike or tether in the session. | Each message opens with a fixed line that names temet-work and the step, so mike's capture can tell the daemon's text from a person's; mike needs a marker for that line. A message is stored with a version and `temet-work issue` prints it; text longer than a short prompt is stored and pointed at. A message counts as heard when the terminal turns `working` after it or the step arrives. If neither happens it is sent once more, and after that the reminder takes over. A thing that was heard is not said again. |
| An agent to the daemon: its step | The `temet-work` command its brief names. | mike draws the same line: a worker's report and a judge's verdict are typed calls, never mail, and a short one is refused with the missing field named. | `ready` carries the commit, what the author ran for each check (the command and its exit code), the tests it changed and what it did not establish. `review` and `rule` carry the commit and what was read or run. Each step is answered with one line that says what was recorded and what happens next. |
| One agent to another | Through the daemon for what binds: review points, the author's answer to each, a ruling. They are steps, held in the log and carried into the next brief as that agent's claims. A question from a live agent to a live agent of another issue goes as a mike question (`ask`, `answer`, `decline`) when the mike part is on. | Text one agent types to another with `hc send` reads as the user's. A mike question names the sender, keeps a record with times, and arrives as information the reader may check, not as an order. | Sessions started so that mike knows them. For Claude Code: a session id the daemon chooses and records, a name, and Remote Control and the mike channel on, as mike's own launcher sets them; the mike part registers the role and goal on mike's roster when it sees the agent start. Claude Code stops at a "Loading development channels" confirmation at every start with the channel on, and no setting skips it. mike's launcher reads the screen for that text and presses Enter, and the runner has to do the same for that one dialog. The text to wait for is a setting, set beside the start arguments that turn the channel on, and the brief carries it to the runner, so the runner names no runtime. When the text is not seen within 30 s, or the session is still at a prompt after the key, the daemon stops the session, lists a problem and starts it again without those arguments, so no start waits for a person. A Codex session has no Remote Control and no channel, and Codex gives it its id: a Codex agent asks and is asked no mike question, and how the daemon learns its id is still to be designed. When the daemon stops a session, the mike part ends that registration and declines the session's open questions as that session. mike still accepts a question to a session whose registration has ended, so the mike part declines any that arrives for one. A question marked blocked that gets no answer in a set time, shorter than the first reminder, is told to the asker as unanswered in one typed message. The asker then takes the default it would defend and says so in its step, or reports `blocked` itself. The daemon never reports a step for an agent. An agent waiting on such a question is not reminded until that message is sent; from then on it is reminded and stopped as any idle agent is, whether or not the question is still open. |
| An agent to a person | `temet-work status`, and the app drawing the same view data. An agent never asks in its terminal and waits. | In the open-issues fleet 14 questions went to a person and none was answered in the record (`docs/research.md`). mike's fleets leave a note and carry on. | Each line a person may act on is said once, worded as what the person is asked to decide, with the time it was raised and the command that settles it. Such lines are kept apart from what the loop is still settling by itself and from a line saying nothing is moving. Each decision is also written as a note on the deciding agent's session in mike, where mike's tools and ask-kit's mike kit find it. |
| A person to an agent or the daemon | The commands (`overturn`, `rule`, `proposal`, `retry`, `set`). To steer a running agent, the person types in its terminal. | Text typed in the terminal is the answer a session treats as its user's, and mike records it as the person's turn, which is what its judge of corrected turns reads. | `temet-work tell AGENT TEXT`, a person's only, for a front end: it types the words through the runner with no leading line, and records who said them. |
| An agent finding something out | ask-kit's `ask`, before it asks anyone: the code domain for the folder it is in, the sessions domain for past sessions on that machine, the mike kit where mike runs. | About 12 s and $0.003 for a question about code, with a few hundred tokens back to the caller and a `path:line` for each fact. A standing seat kept to answer questions waited 25 of its 28 minutes (`docs/findings/fleet-run.md`). | Every brief names `temet-work issue` for the plan, the checks and their results. Where ask-kit is installed, the brief also says to ask before digging. That line is a rule in the config with the scope `all`, so it counts against the brief's budget and a person can drop it. Having ask-kit installed does not mean agents use it: on the machine measured, asks covered 19% of lookups, so asks are counted per issue. |
| The loop learning | mike's capture of each session, its judge of corrected turns and its lessons, read by the mike part. | The daemon already passes the proxy variables on, so a daemon started under mike has its sessions captured. | The mike part is a skeleton. It has to find the sessions the daemon started in mike's capture, by the session ids above, and read the turns a person typed into them and which of those mike's judge marked as corrections. Once the daemon's text is marked, those are the only turns mike judges on these sessions, so this says how often a person stepped in, not how the loop did. It also counts each day the sessions started against the sessions captured. |

Three things the measurements say not to assume. A message delivered is not a message
read, and one read is not one acted on: mike marked each mail of the dispute run consumed
within 1.6 s, before any acknowledgement. A terminal that is `done` has ended a turn, not
finished a step. One arbiter's ruling is not settled fact: the same dispute run four
times gave two different rulings.

Not used, and why:

- mike's conditions. They have no kind for a check or a pull request, read mike's own
  database only, and add a 10 s pass to a result the daemon has the moment its own run
  ends.
- Plain mail between agents. A statement that binds is a step, and anything else one
  agent wants of another is a question, which stays open on record until it is settled.
- `hc send` from one agent to another. It reads as the user's.
- A Remote Control push, a steering note and a prompt submitted by tether, for the
  daemon's own text. Each needs mike or tether in the session; typed text needs neither.
- mike's work threads and notes as the store. The daemon's log is the store.
- The routes of mike's fleet for a report, a verdict and the floor. They need a running
  fleet, so their shapes are copied and the routes are not called.

tether stays optional. Mail and questions reach a session in two forms: in the dispute
run 10 of the 12 mails came over Claude Code's peer socket and 2 as a tether prompt, both
forms to one session, and what picks the form is not known
(`docs/findings/agent-talk.md`). So with the mike part the merge record holds what each
session had loaded and the form each question arrived in. `ask` does the work the
investigator role was meant for; the role stays in the core until asks have been counted.

mike and ask-kit stay optional, and the core names neither. The core writes every brief
and reminder, so a line that tells an agent about a tool comes from outside it. The
`Part` contract grows one call, the lines a part adds to a brief, and the mike part uses
it for its lines about questions. A line about `ask` is a rule's text, and such a rule
starts in the config. What a start needs beyond the task is carried by the brief: the
session id the daemon chose and, from settings, extra arguments, variables and the text
of a start dialog to confirm. The runtime adapter passes the first three on and the
runner reads the last, so no part names another. The steps, events and effects for
questions, registration and notes carry no tool's name (`tests/parts.rs`,
`core_names_no_tool_or_service`).

Without the mike part there are no questions between agents, no notes and nothing read
back from mike's capture; sessions are still captured whenever the daemon was started
under mike. Without ask-kit a brief carries no line about asking: the agent digs by hand
and cites what it found. The rest works as described.

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
None is built; they are items 8 to 12 of "What comes next". Each says what it does and
how it says it, on the routes of "How things are said".

#### The floor

Found: a passing check proves little when the author controls it, and an agent's report
of its own work is often wrong (finding 3). mike's floor runs a task's checks itself
before any model sees the work.

- A check may carry a command with an expected exit code. Before a reviewer starts, the
  daemon runs it in a terminal of its own in the author's work folder, on that folder's
  machine, so the build there is warm. It starts the run when the author's turn has
  ended, and first checks that the folder is clean and at the pull request's head commit.
- A pass says nothing to the author. A failure is one typed message in the shape of
  mike's floor answer: the check's line, the command, the exit code got and expected, the
  commit, one line of reason and where the full output is. Its code keeps three cases
  apart: a failed check, a timeout and a folder that is not clean. The message is keyed
  by issue, check and commit, so a repeated run does not repeat it.
- A run the daemon could not make is not said to the author. It is a listed problem that
  says the work has not failed, and it is tried again.
- The full output is stored where `temet-work issue` prints it, and the next reviewer's
  brief points at it.
- A failure, a timeout or a dirty folder goes back to the author as a fix and uses no
  review round; a check that fails again after that fix goes to the reviewer with its
  result, so the cap on rounds still ends it. An author that cannot make a check pass
  because the check is wrong reports `blocked`. A check with no command is left to the
  reviewer, as now.
- Two contracts grow for this: a runner reports the exit code of a terminal that has
  ended and gives its whole output, not only the visible screen, and a code host reports
  a pull request's head commit.

#### `blocked` and `unneeded`

Found: a blocked agent often finds another way to finish, and agents change code in a
third to two thirds of cases where no change is needed (finding 7). mike's fleets leave a
note for the person and carry on, and a mike question can be marked blocked.

- Two more steps, each with a reason (`docs/commands.md`). The reason carries the
  evidence: a `path:line`, a command with its exit code, or a session and event. Where
  ask-kit is installed, the agent settles its claim with `ask` first.
- The daemon refuses a reason that carries none of these, which it can tell from the
  text alone. It checks one kind with no model, a `path:line`: before the agent is
  stopped it runs a command in the agent's folder whose exit code says whether the file
  has that line, through the runner as the floor does. A citation that does not resolve
  is shown beside the reason in `status`, as ask-kit adds a line to an answer; it does
  not undo the step. A command with its exit code and a session and event are recorded
  as the agent's claims and are not checked.
- `ready` says what the author did not establish, so there is a place for doubt short of
  `blocked`.
- With the mike part, an agent held up by an agent on another issue asks a mike question
  marked blocked first. When the question is declined or goes unanswered it takes the
  default it would defend, and reports `blocked` only when there is none.
- `status` lists the issue in the shape of mike's note to its operator: once, worded as
  what the person is asked to decide, with how long it has waited and the commands that
  answer it. The issue's agents are stopped only after the step's reply is recorded;
  their open questions are declined and their registration with mike is ended. The work
  folder is freed and the group takes the next issue.
- Issues that depend on a `blocked` issue wait and are listed as waiting on it; issues
  that depend on an `unneeded` one start as if it were done. Each is a decision a person
  can overturn.

#### A new reviewer session each round

Found: a reviewer that has been through a repair round grows more lenient, a judge shown
its own verdicts repeats them, and a checker that reads the author's account can be
steered by it (finding 4). mike's judge is not given the worker's summary, and the
dispute run settled four disputes of four in 25 to 60 s.

- A new reviewer session each time the pull request comes up for review; the session
  before it is stopped when its verdict is in. No message goes to a running reviewer about
  a next round.
- The brief is the whole hand-over. The daemon builds it from its log in the shape of the
  input mike gives its judge: the plan's checks, the daemon's results for them with exit
  code and commit, where to read the diff, and each earlier round as its blocking points
  with the author's answer to each. It does not carry the author's summary, and it tells
  the reviewer to read the pull request's description and comments as claims to check. It
  is stored with a version, so a session that replaces another reads the same brief.
- Nothing passes from one reviewer session to the next, or from the author to the
  reviewer, except through that brief. The core's text in the reviewer's and the
  arbiter's briefs says so and names no tool: the agent reads no account of this issue's
  sessions, and neither asks nor answers a question from an agent on this issue.
- The tools' own words come from outside the core. With the mike part, its lines in
  those two briefs name mike's history and mike questions. Where ask-kit is installed, a
  rule in the config for each of the two roles says that `ask` is for the code at the
  pull request's head commit and not for the sessions domain.
- A mike question is for an agent on another issue, such as the author of another issue
  in work on the same repository. The author of an issue this one depends on is not such
  an agent: that issue is done before this one's author starts, and its author was
  stopped when it merged.
- The daemon numbers each blocking point. The author answers each by its number in its
  `ready` step as fixed or disputed, a dispute with its evidence, and a later
  `review block` says by number which earlier points it holds. A disputed point that is
  held goes to the arbiter then, and that visit uses no round.
- The arbiter's brief is written from the same record. The arbiter works in the author's
  work folder as the reviewer does, checked the same way, so it has the repository at the
  pull request's head commit.
- An early ruling settles the disputed points only and rules on each by its number
  (`docs/commands.md`): a point ruled for the author is dropped, one ruled for the
  reviewer is a fix, and the block's other points stand. The issue then goes back to its
  author with what is left; with nothing left the block is void and the review counts as
  approved. `rule approve` and `rule fix` rule on the whole pull request and are accepted
  only after the last round, as now. An early ruling is a decision about its one point,
  and overturning it changes that point only.
- A ruling has a fixed shape: the ruling, the blocking point or check it rests on, and
  the command the arbiter ran when the ruling turns on a claim about the code. An
  arbiter's ruling with no evidence is sent back; a person's `rule` needs only its
  reason.
- The first round reads the whole change. A later round checks that the blocking points
  are met and reads the commits since for new defects; it raises nothing new about code
  that has not changed. A block names a `path:line` and what would satisfy it. The daemon
  checks that line as it checks one in a `blocked` reason, and the next brief says when
  it did not resolve.
- The reviewer works in the author's work folder and changes nothing there. Before each
  round the daemon checks through the runner that the folder is clean and at the pull
  request's head commit, as it does for the floor, and a fix sent to the author while a
  reviewer is running stops that reviewer first. Today a reviewer gets any free folder of
  the project, or one an author is changing.
- The `starts` limit counts sessions that ended without reporting their step, so a
  reviewer that finished its round does not use one up.

#### The merge record and the counts

Found: a person's check adds little when it rests on the agent's account, and "nobody
overturned it" is not evidence that a decision was right (finding 8). In mike's run
record every number is measured and none is rewritten by a model.

- A plain approval is recorded as a decision by the reviewer, in the shape of mike's
  verdict: the verdict, the commit, and what was read or run, which may not be empty.
  Overturning one before the merge sends the issue back to its author with the reason.
  `status` lists the decisions of issues that are not done and the ones overturned, and
  counts the rest.
- Every merge leaves a record the daemon writes from what it ran and saw: the commit that
  merged; the checks and their results; the files changed as the code host lists them,
  and whether they include CI, deploy or verify commands, agent instructions or settings;
  the review rounds it took and whether an arbiter ruled; each agent session with its
  session id, its runtime and model, and the version of the brief it worked from; and
  whether the daemon merged it or found it merged.
- The record also holds what was said on the issue: each message the daemon sent and
  whether it was heard and, with the mike part, the questions its agents asked and were
  asked, with how each ended, and what each session had loaded. No agent's message or
  summary is the source of any field.
- The daemon counts per project: failed verifies, issues that merged more than once,
  follow-up issues that name the issue they fix, `blocked` and `unneeded` with how long
  each has waited, settings set by a plan, a person's turns typed into agents' terminals,
  daemon messages that were never heard, reminders sent, sessions started against
  sessions that reported their step and, where a project's config names the command that
  prints them (`ask-kit gaps` where ask-kit is installed on the agent's machine), asks
  against lookups done by hand, per session and summed per issue. The daemon runs that
  command through the runner on the agent's machine when the session ends, as it runs a
  check's command, so no crate names ask-kit. The count waits for the floor's runner
  contract, and with no command set it is left out.
- `temet-work issue ISSUE` prints the issue's merge record, and `temet-work status`
  prints each project's counts under their own heading; the app draws the same view
  data. A count that meets a trigger of "Left until a trigger is met" is listed once as a
  line a person may act on, with the trigger named.

#### Rules with a scope, a budget and counts

Found: a learned rule is a release, and standing text grows one fix at a time unless
something cuts it (finding 9). mike gives its standing text a budget that only goes down,
tags each lesson with an id and a revision, and retires a lesson by count.

- A rule can also start in the config, kept from the start; the line that says to ask
  before digging is one.
- A rule has a scope (a role, a project, or all), the keys mike's roster addresses a
  session by. A proposed rule starts scoped to the role that made the overturned
  decision; the person keeping it can change the scope and edit the text.
- The daemon gives an agent a rule in its brief and nowhere else, from the next start.
  Text the proxy injects is seen for one request, and mail is read once. A session mike
  captures also gets mike's own lessons by mike's routes. Those are mike's: the budget
  does not count them, and the mike part reports how many each session held.
- Rules sit before the task in the brief. mike recorded a rule ignored on first read
  about 9,000 characters into a 33,859-character block.
- Each rule line carries the rule's id and revision, and the daemon stores which rules at
  which revision each session's brief carried. That is the count of issues a rule was
  given to.
- A brief has a budget, the setting `rules`: the most rules a brief may carry, each one
  line. When keeping a rule would put a brief past it, the keep names a kept rule to
  drop, and `proposal RULE drop` works on a kept rule too.
- A rule's effect is counted from what the daemon records on the issues whose briefs
  carried it: a failed run of a check, a blocking point the next reviewer held, an
  overturn, a failed verify. It is read against the issues before the rule, and only
  above a stated sample; mike reads a lesson at 20 judged turns on each side.
- mike's corrected turns cannot be that measure here: its judge reads turns a person
  opened, and in this loop a person rarely types. With the mike part they are added as a
  count of how often a person stepped in. The daemon's own messages are left out of it,
  by mike's marker or, until mike has one, by the mike part matching the fixed line.
- Candidates may come from mike's corrected turns as well as from overturns; none reaches
  a brief until a person keeps it. A rule kept from a mike lesson carries mike's tag
  `[lesson:<id> rev:<n>]` on its line, so mike does not give that session the same text
  again.
- Trial and revert follow when there are enough issues to count.

### Left until a trigger is met

| Left out now | Why it can wait | Trigger |
|---|---|---|
| Agents running as another user or in a sandbox, with the daemon's files and sign-in out of reach; a code-host identity of their own for agents; protected default branches (finding 5). | The agents, the machines and the repositories are the authors', and a mistake costs a revert. | Someone who is not one of the authors tells them they run temet-work, or is asked to; or it manages a repository its authors do not own; or a merge record shows a pull request the daemon found merged and did not merge itself. |
| Stronger proof of who sent a command (finding 6). | Every sender today is one of the authors or an agent they started. When it is wanted, mike's way is there to copy: its bridge confirms a caller by finding the captured call that carries the exact message. | The same. |
| A list of the settings a plan may set (finding 6). | A setting a refiner should not have set is listed as set by that refiner (`temet-work settings`), and a person sets it back. | A project takes tickets its authors did not write; or the count of settings set by plans shows one its issue did not ask for. |
| Ticket text marked as content in briefs, a list of whose tickets are taken, and tokens that cannot write for the roles that only read (finding 10). | The tickets are the authors'. Until a trigger is met, every group on a repository other people can open issues in is given its milestones (`docs/configuration.md`): an issue a stranger opens has no milestone and is not taken. Its title is still listed by `status`, and a stranger's comment on an issue in work can still be read by the agent working on it; both are accepted for now. | A group works such a repository with no `milestones`; or an issue the authors did not write is put into a milestone a group works. |
| A class of change held for a person: the daemon's own repository, CI and deploy commands, agent instructions, sign-in, migrations (findings 1 and 11). | Nothing waits on a person. In the open-issues fleet 14 questions went to a person and none was answered in the record (`docs/research.md`). | A managed project has users of its own or data that cannot be rebuilt; or the merge records show a merge that changed files of that class and had to be merged again. |
| A trial run of a new build on a second daemon before it replaces the live one, with the previous build kept for an automatic return (finding 11). | A daemon that breaks itself is fixed by hand by the people who wrote it. | A deploy of temet-work itself leaves the daemon down or on the wrong build and nobody sees it for a working day; or that happens twice in one month. |
| A second reviewer on another vendor's model, and a catch rate measured on seeded defects (findings 2 and 4). | A measured rate needs more merged issues than there are. | 50 issues have merged in one project and more than 6 of them merged more than once or drew a follow-up issue. The research found 11.5% of people's merged changes reverted within 90 days. |
| State files agents cannot write; a stop command for a group, a project and everything. | The machines are the authors', and nobody else uses them. Today the stop is by hand: closing an issue in the tracker stops its agents at the next sync, and stopping the daemon and the `owner=temet-work` terminals stops everything. | Someone who is not one of the authors runs agents with temet-work; or stopping a group by hand takes more than a few minutes. |

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
7. How things are said, the part that needs only the daemon: each daemon message opens
   with its fixed line and is stored with a version, a message counts as heard, step
   reports are typed and answered with one line, and each session's id is recorded (for
   Claude Code, one the daemon chooses).
8. The floor: a check can carry a command with an expected exit code, which the daemon
   runs at the pull request's head commit before review (`docs/objects.md`).
9. `blocked` and `unneeded` as steps a refiner or an author can report
   (`docs/commands.md`).
10. A new reviewer session each round, with the brief as the whole hand-over, the
    author's answer to each point, and a ruling in a fixed shape.
11. A record of each merge written by the daemon, a plain approval recorded as a decision,
    and the counts.
12. Rules with a scope, a budget and counts, tagged with an id and a revision; a rule can
    also start in the config.
13. How things are said, with the mike part: sessions on mike's roster, the start dialog
    confirmed by the runner, the lines a part adds to a brief, questions between agents,
    decisions as notes, and the mike part reading the turns a person typed and counting
    sessions started against sessions captured. The `mike` MCP server has to be
    registered at user scope on each machine agents run on.

Nothing waits on item 13. Items 8 to 12 use the fixed line and the stored briefs of item
7 where they exist and are not held for them. Items 9 and 10 change the loop, the
commands and the briefs, and can be taken whenever the loop is next worked on. Two checks
in them use item 8's contracts and are left out until item 8 is in: the check of a
`path:line` in a `blocked` reason or a block, and the check that the reviewer's folder is
clean and at the head commit. Until then a citation is recorded unchecked and a reviewer
gets its folder as now. Item 8 is larger: it needs the pull request's head commit from
the code host and, from the runner, a command's exit code and whole output, in both
contracts and their adapters.
