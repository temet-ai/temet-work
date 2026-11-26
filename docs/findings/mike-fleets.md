---
session_id:
  - ed6a5524-c57e-4a16-b4a9-a0ccf9ae9c1c
---

# What temet-work can take from mike fleets

Read on 26 November 2026 in `/mnt/data/projects/temet/temet-proxy` at commit
`42cdc4b133b1cf878999513a317b8a247ae6fc9f`. Every `path:line` below is at that commit.

Short paths used below:

| Short | Full |
|---|---|
| `fleet/` | `crates/mike-fleet/src/launch/fleet/` |
| `mf/` | `crates/mike-fleet/src/` |

## Summary

- temet-work can call mike's mail, questions, session notes and conditions over HTTP or MCP without running a fleet.
- Verdicts, the judge, the floor, the gate, build admission and the retrospective are tied to a fleet definition and a fleet run. temet-work can copy their design; it can't call them for its own items.
- Of the eight fleet-friction issues, four are fixed in the code (#531 for `#N` references, #532, #536 for mods, #537) and four are partly fixed (#530, #533, #534, #535).

The code was read, not built or tested. What was run is listed under [What was run](#what-was-run).

## How a separate daemon can reach mike

**As a Rust crate: not worth it.** `mike-fleet` is `publish = false` and depends on `mike-base`, `mike-runtime`, `mike-launch`, `mike-code` and `mike-codex` (`crates/mike-fleet/Cargo.toml`). Its handlers take `BaseState`, which holds the proxy's request store, event channel and config (`crates/mike-base/src/base_state.rs:61`), and its records are rows in mike's Postgres through `PersistHandle`; that database has 485 migration files (`crates/mike-base/src/store/persistence/migrations/`). `mf/lib.rs` also re-exports all of `mike_launch`. The licence is MIT, so small pure functions can be copied.

**Over HTTP.** mike's inspect API is on `127.0.0.1:9989` and answered `GET` requests without credentials on this machine. Routes are listed with their MCP tool names in `mf/api/manifest/` and `crates/mike/src/api/manifest/`.

**Over MCP.** The same routes, for agent sessions.

## Reusable parts

| Part | Reach it by | Depends on | Reuse |
|---|---|---|---|
| Mail | HTTP, MCP | mike running; session known to mike | call it |
| Questions | HTTP, MCP | mail; fleet only for `reference` routing | call it |
| Session notes | HTTP, MCP | mike's database | call it |
| Conditions | HTTP, MCP | mike's database and workers; mike's launcher to start a session | call it |
| Labelled launch | `POST /api/launches`, `spawn_session` | herdr-core (`hc`) | call it, or copy the labels |
| Commit, report, verdict | HTTP, MCP | a defined, running fleet | copy the design |
| Judge | no call of its own | fleet tick; OpenRouter model and key | copy prompt and input shape |
| Floor and gate | fleet HTTP, CLI, MCP | fleet definition and checkout | copy the design |
| Build admission | in-process only | file locks | copy the code |
| Retrospective, close note | runs when a fleet closes | fleet record, session notes | copy the design |

### Mail

- Send: `POST /api/sessions/{id}/message` (`crates/mike/src/api/manifest/sessions.rs:75-79`), MCP `send_session_message`. Body fields start at `crates/mike/src/api/handlers/steering_views_02.rs:202`.
- The handler is `crates/mike/src/api/handlers/mailbox_post/mod.rs:28`. It refuses a closed fleet seat (`:57`, `:195-216`) and an ended recipient (409 `recipient_ended`, `:67-72`), so a sender learns at once that nobody will read it.
- Read, acknowledge, verify: `crates/mike/src/api/manifest/sessions.rs:79-101` (`get_session_messages`, `ack_session_message`, `verify_session_message`, `verify_message_read`, `confirm_message_receipt`).
- Delivery has more than one path. A client-side mod can claim a session's mail (`POST /api/sessions/{id}/mail/claim`, `crates/mike/src/api/manifest/session_mail.rs:11`; the handler marks the mailbox as served, `crates/mike-launch/src/api/session_mail.rs:55`). Otherwise, if no mike channel is live, mike pushes over Remote Control (`crates/mike-features/src/proxy/remote_control/inject.rs:114-128`). A push that fails or is not allowed leaves a reminder queued for the session's next request through the proxy (`inject.rs:39-53`; reload from the database, `crates/mike-features/src/steerer/stage/unqueued_mail.rs:6`).
- How a message is framed depends on the path: a Remote Control push reads to Claude Code as a teammate's request, a channel message as untrusted external data (`crates/mike-features/src/proxy/remote_control/inject.rs:69-73`).
- Seen in this run: this seat's brief arrived as "Mail from mike", submitted by the tether plugin as the turn.

### Questions

- `POST /api/questions`, `.../answer`, `.../decline` (`crates/mike/src/api/manifest/questions.rs:9-15`), MCP `ask`, `answer`, `decline`.
- Handler `mf/api/questions/ask.rs:110`; `blocked` and `about` (a note id) are body fields (`ask.rs:19-27`). Storing and mailing: `fleet/questions.rs:48`. Table `workspace.questions`.
- A question to a closed fleet seat is refused (`fleet/questions.rs:53-57`).

### Verdicts and the judge

- A worker commits through mike (`fleet/commit.rs:69`, `POST /api/fleets/{fleet}/commit`) and reports typed facts (`fleet_report`: criteria, the command exercised with exit and output, tests changed, what it did not establish). A verdict is recorded at `mf/api/fleet/verdict.rs:79` (`POST .../verdict`). Routes: `mf/api/manifest/fleets.rs:72-120`.
- The judge is a single model call, not a session. It runs from the tick, two reports at a time (`fleet/leaves/judge_pass.rs:23`, `:203`), through a one-method trait `LeafModel` (`fleet/leaves/mod.rs:80-82`). The model is `steerer.model` over OpenRouter (`leaves/mod.rs:44-60`). With no model or key the report stays unjudged (`judge_pass.rs:278`).
- Input: `JudgeInput` (`fleet/leaves/judge.rs:63-74`): criteria with their `check` text, report facts, floor result, the commit's diff, the task brief, open obligations, tasks sharing a criterion and earlier verdicts. The prompt is `.claude/fleet/leaves/judge.md`.
- Rules in that prompt that apply to any reviewer:
  - the judge is not given the worker's summary;
  - it may ask for one round of bounded checks, then must pass or fail;
  - a later round asks only whether the last round's `needed` is done and nothing broke;
  - a criterion shared between tasks is judged only on this task's share.
- Repeated fails on one criterion open a problem (`fleet/leaves/report_judge.rs:127`).

The `software-factory` thread (mike project, one checkpoint, 26 November, nothing built) has already settled a different reviewer: a cloud reviewer with no mike context; approve by default; block only for end state not met, a bug introduced by the diff, or code that doesn't fit, each with `path:line` and what would satisfy it; later rounds check only earlier blocking items; the author may push back; 3 rounds, then a human. Its author works in one permanent folder per project (`<repo>/.claude/worktrees/auto`, builds through `mbx` into a fixed target directory), and its first build step is a `mike work-folder` lease. The judge's input shape and its later-round rule fit that design.

### Conditions that wake a session

- `POST`, `GET`, `DELETE /api/conditions` (`mf/api/manifest/conditions.rs:10-12`), MCP `wait_for`, `list_conditions`, `cancel_condition`. Table `launch.conditions`.
- Six kinds: `time`, `session_turn_ended`, `session_ended`, `fleet_event`, `sql`, `words` (`mf/store/persistence/conditions/spec.rs:13`). There is no GitHub, pull request or CI kind.
- Evaluation: `mf/store/persistence/conditions/holds.rs:22-39`; worker `mf/conditions/worker.rs:39`; default check interval 300 s (`mf/api/conditions.rs:20`).
- `sql` runs the caller's query against mike's own database, read-only with a statement timeout (`mf/store/persistence/conditions/holds.rs:113-123`). It can't see another program's data. `words` is judged by a model.
- A condition's action is `message`, `return` or `start` (`mf/conditions/fire.rs:225-237`). `return` mails the session if it is live and either recent or has no work thread, resumes it if it ended within 60 minutes, and otherwise starts a new one (`mf/conditions/decide.rs:20-51`, `fire.rs:56-78`, `:144-163`).
- For temet-work: `choose_return` is a short pure function worth copying. Waking on a review or a CI result would need a new kind, or the daemon watching GitHub itself.

### Floor and gate

- The floor is a task's machine checks before the judge sees anything: declared checks, files changed, a test added, the exercise ran (`fleet/floor.rs:102`, `:467`). A failed floor never reaches the judge (`floor.rs:51-54`), and its code says which part failed (`floor.rs:17-37`).
- The gate is the project's check run by mike: a quick command per pushed commit and an optional full one at the end (`docs/fleets/README.md`, "What to put in `gate`"). Routes: `check_fleet` (`mf/api/fleet/check.rs:15`) and `gate_fleet` (`:64`); the gate route has no MCP tool (`mf/api/manifest/fleets.rs:87`).
- Checks of a commit run in mike's own clean checkouts (`docs/fleets/isolated-checks.md`).
- mike also runs a check on the shared, uncommitted tree and mails the seat whose uncommitted lines broke it (`fleet/shared_tree.rs:196`, `:393`).
- Executable rulings: acceptance registers a command and expected exit code, and each affected task needs a receipt for a clean commit (`docs/FLEET_RULINGS.md`).

### Build slots and capacity

- Named resources with a capacity, held as file locks: `fleet/operation_admission.rs:50` over `fleet/build_admission.rs`. A caller takes all its resources or none (`operation_admission.rs:80-89`). This code needs no database and could be copied.
- mike's own check queue gives the gate one slot and the check two (`fleet/managed_queue.rs:55-62`).
- It covers operations mike runs. A command a seat runs in its own shell is not held.
- Staffing stops below a free-memory floor (`fleet/staff_limits.rs:65-76`, `fleet/host_memory.rs:34-49`); the capacity report is `fleet/capacity.rs:123`.
- Commit `07823b56e` removed the earlier cargo wrapper and the `build_slots` number; `execution.cargo` and `build_slots` still parse and do nothing.

### Close note and retrospective

- Closing writes a note listing tasks done and open and who stopped the run (`fleet/close_note.rs:88`).
- The retrospective is a detached Claude session, `sonnet` by default, that asks four fixed questions of the run: effort split, noisy or contradictory instructions, what was missing, the system prompt the seat wants (`fleet/retrospective.rs:23-44`). Without write authorization it gets read-only tools and `dontAsk` permissions (`fleet/retrospective_launch.rs:47-73`). It writes one `finding` entry on a work thread and is recorded failed if it writes none (`retrospective_launch.rs:102`).
- The research document counts 19 of 30 retrospectives finished.
- A separate rule audit is started from the tick (`fleet/rule_audit.rs:217`, `fleet/tick.rs:300`); the research document found no trace of it running since October.

### The run record

A run is one append-only table, `launch.fleet_events`, and every shown state is folded from its rows (`docs/fleets/events.md`; `fleet/fold.rs`). The doc says 4 of the 26 event kinds have no writer. temet-work's daemon can use the same shape for its items.

### Launching with labels

mike starts agents with `hc run --bg --keep --json --no-parent --cwd <folder> --label k=v ... -- <command>` (`crates/mike-launch/src/launch/herdr_core/open.rs:111-132`). The label keys are `herdr.space`, `herdr.tab`, `mike.session`, `mike.name`, `mike.run`, `mike.seat` and `mike.launched` (`.../herdr_core/terminal.rs:8-14`; applied at `open.rs:91-109`). A temet-work daemon calling `hc` directly can add item and role labels beside these. mike's own route for this is `POST /api/launches` (`mf/api/launches/mod.rs:168`), MCP `spawn_session`.

For mike to capture a session, mike's launcher puts `HTTPS_PROXY`, `NO_PROXY` and `NODE_EXTRA_CA_CERTS` in its environment (`crates/mike-launch/src/launch/env.rs:11-20`, `:24-29`). A daemon that starts `claude` through `hc` itself would have to provide the same, or the session is not captured.

## Issues #530 to #537

State and title as `gh issue view <n> -R temet-ai/mike-private --json state,title,closedAt` printed them. All eight were closed on 8 October.

The fix commits predate the public history, which starts again at `00ac2666b` (6 November), so they are not ancestors of the pinned commit. The verdicts below come from the code at that commit.

| Issue | State | Title | Does the code do it now |
|---|---|---|---|
| #530 | `CLOSED` | One checkout means one cargo build-directory lock: seven of eight cargo jobs queued with no rustc, shown as finished | Partly |
| #531 | `CLOSED` | Workers' slice commits close the issue on push, before acceptance has judged the task | Yes, for `#N` references |
| #532 | `CLOSED` | The gate's lint step rewrites scripts/rc-protocol-state/binary-last-verified.json | Yes |
| #533 | `CLOSED` | One seat's unprepared sqlx queries in the shared tree stopped crate mike compiling for five seats for 14 minutes | Partly |
| #534 | `CLOSED` | The gate failed with a test database that did not exist while a seat ran DB tests on the same instance | Partly; cause never established in the issue |
| #535 | `CLOSED` | Acceptance read 155 M tokens and passed 690 k context in one run; control seats read more than the workers | Partly |
| #536 | `CLOSED` | Nothing says which mods or hooks a session loaded, so checking one means driving its pane by hand | Yes for mods; hooks outside mods are not listed |
| #537 | `CLOSED` | A digest turn starts in every control seat on each small change and carries the whole table | Yes |

### #530: partly

- Done: a backgrounded command stays open until its end is captured (`mf/store/persistence/fleet/tree.rs:160-170`, `fleet/background_result.rs:31-46`), so it no longer shows as finished at launch.
- Partly: "waiting" is inferred only for commands the project declares under `timed_commands` with resources (`fleet/reuse.rs:88-111`, `fleet/operation_settings.rs:45-56`). temet-proxy's `.claude/fleet/operations.json` declares none, so a seat's cargo command queued on the lock is not marked.
- Not done: seats get no target directory of their own. `operations.json` strips `CARGO_TARGET_DIR` from their environment, and one checkout means one `target/` and one cargo lock. The place for a per-seat directory would be the seat's launch settings (`fleet/seat_settings.rs`).

### #531: yes, with limits

- `fleet_commit` rewrites closing words to `Refs` (`fleet/commit.rs:234-240`, `:306-322`) and records the numbers (`commit.rs:119-126`). The issue is closed with `gh issue close` once acceptance passes the task (`fleet/issue_close.rs:79-110`, `:185-204`; called from `fleet/staff.rs:258`).
- Limit, checked by running the same regular expression in Python: `Closes #531` becomes `Refs #531`; `Closes temet-ai/mike#37`, `closes https://github.com/temet-ai/mike/issues/37` and `Closes GH-12` pass through unchanged.
- Read, not run: the numbers are stored on the committing session, and `tick` reads them from the task's current session (`issue_close.rs:188-196`). A task handed to a successor seat may leave its earlier issues open.

### #532: yes

`make lint` no longer runs the binary check (`Makefile:156-157`). Recording is the separate target `rc-protocol-record` (`Makefile:232-237`), and `rc-protocol-check` runs with `--no-record` (`Makefile:239-243`).

### #533: partly

- Removed: the fix made `scripts/cx` run `scripts/sqlx-prepare` by itself (`859467477`). Commit `e8bad318e` (20 November) took that out; its message says "cx no longer runs sqlx-prepare on its own". `scripts/cx` at the pinned commit has no `sqlx` handling.
- Present: the issue's other half, telling the owner. mike now checks the shared tree and mails the seat with the uncommitted lines that fail (`fleet/shared_tree.rs:196-218`), and mails the reference seat after 10 minutes (`shared_tree.rs:17`, `:220-239`).

### #534: partly

- Done: test setup refuses the capture database's instance (`crates/mike-base/src/store/persistence/test_support/provisioning.rs:62-75`); the sweep takes only the `claude_proxy_test_` prefix, spares connected databases and the run database, and needs 60 minutes of age (`provisioning.rs:35-37`, `:212-231`), with tests for each (`provisioning_tests.rs:106-181`).
- Open: the issue asked for age by creation time or a separate gate prefix or instance. Age is still the `PG_VERSION` file time (`provisioning.rs:224-225`), and the prefix is shared. The issue left the cause unestablished, and nothing read here establishes it.

### #535: partly

- A control seat at 600,000 tokens of context gets mailed to the coordinator once, with how to start a successor (`fleet/tell_context.rs:6-45`, wired at `fleet/tell.rs:245`). Nothing starts the successor.
- Since 14 November mike opens no coordinator or acceptance seat itself: the coordinator is a named session or the fleet's author, and the observer is always none (`fleet/start.rs:161-171`, `fleet/plan.rs:41-57`). The judge model call does acceptance's work.
- The issue's third ask, a one-call check of loaded plugins, is covered by #536's listing.

### #536: yes for mods

The mike mod posts what a session loaded as a session note (`mods/mike/hooks/loadedMods.ts`, `mods/mike/hooks/register.tsx:220`), and `get_session` returns it as `loaded_mods` (`crates/mike/src/api/handlers/session_detail/loaded_mods.rs:14-42`, `get_session.rs:128`). Run today: `GET /api/sessions/<this seat>` returned a `loaded_mods.entries` list with each mod's folder, scope, commands, bands and acts. It depends on the mike mod being loaded in that session.

### #537: yes

A digest now carries only changed lines (`fleet/tell_digest.rs:253-264`). A row's change key leaves out idle time (`:191-209`), and waits are listed only past their thresholds (`:235-250`). It still goes to every control seat that exists (`:286-294`), which since 14 November is at most coordinator and acceptance.

## What was run

All on 26 November, read-only:

| Command | Result |
|---|---|
| `gh issue view 530..537 -R temet-ai/mike-private` | all `CLOSED`; titles as in the table |
| `GET /api/conditions?limit=2` | 200 in 5 ms; rows include a `sql` condition waiting on a fleet's state |
| `GET /api/fleets/6cfef2b7-…/status` | 200 in under 1 ms; criteria, capacity, 4 seats live, 19.8 GB free |
| `GET /api/fleets/6cfef2b7-…/results` | 200 in 16 ms; each row has `command`, `seat`, `running`, `still_valid` |
| `GET /api/sessions/ed6a5524-…` | has `loaded_mods` |
| the `#531` regular expression in Python | results under #531 |

The results route lists every Bash command each seat runs, with the command text, including read-only ones. Its own note says tool success is not a test pass.

No `POST` route was exercised by this task, and no temet-proxy test was run (the repository is read-only for this run).

## For temet-work

Each line names the evidence above it rests on.

### Daemon: build on

- mike mail, questions and session notes over HTTP, for agent-to-agent talk. They need no fleet (Mail, Questions).
- `hc run` with `--label`, adding item and role labels beside mike's `mike.*` keys, with mike's proxy environment so the session is captured (Launching with labels).
- An append-only event table folded into state (The run record).
- A floor before any model review: machine checks, then a reviewer that never sees the author's summary, with later rounds limited to the previous `needed` (Floor and gate, Verdicts and the judge). The `software-factory` thread already limits rounds to 3.
- Copied code: `choose_return` for waking sessions, the resource lock for one build at a time per machine, `rewrite_closing` extended to `owner/repo#N` and URLs (Conditions, Build slots, #531).
- Closing the issue from the daemon after the verdict, never from a commit message (#531).

### Daemon: avoid

- Depending on `mike-fleet` as a crate: it brings mike's state and database (How a separate daemon can reach mike).
- Calling the fleet commit, report and verdict routes for its own items: they need a fleet definition and a running run (Reusable parts).
- Several agents in one checkout. #530 and #533 are partly open because of it, and the `shared_tree.rs` machinery exists only to manage it. `docs/research.md` already plans a work folder per item.
- Control seats that live for the whole run: #535 and #537 were their costs, and mike has since replaced acceptance with a model call.
- Treating "delivered" as one thing: mail reaches a session by a mod claim, a channel, a Remote Control push or the next request, and the framing differs (Mail).

### App: build on

- `GET /api/fleets/{fleet}/status` as an example of what an overview needs: criteria passed and ready, blocked items, builds queued, free memory, live seats (What was run).
- `loaded_mods` from `get_session`, to show what an agent session actually loaded (#536).
- A digest that shows only what changed and what needs an action (#537).

### App: avoid

- Showing a command as passed because the tool call succeeded (What was run).
- Showing "waiting for build" from inference over declared commands only; temet-work's daemon should queue the builds itself and so know (#530).

### Still unknown

- Whether mike's conditions should gain a pull-request or CI kind, or the daemon should watch GitHub (Conditions; the same question is open on the `software-factory` thread).
- Delivery time and loss for mail sent by a program that is not a Claude session. This task sent none; `docs/findings/agent-talk.md` measures session-to-session mail.
- Whether the judge prompt works outside a fleet's `JudgeInput`, and what a judge call costs.
- The cause of #534.
- Whether a successor seat leaves issues open under #531.
- How temet-work relates to `software-factory`: whether the daemon is that loop, with mike's work-folder lease underneath (`docs/research.md`, Open decisions).
