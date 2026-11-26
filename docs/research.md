---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# temet-work: research for a work app

This document collects what was found on 26 November 2026 while looking into a native app (Rust and GPUI) that takes issues from a tracker, plans them with Dawid, runs agents on them through herdr-core, reviews and merges the code, and learns from each run. The UI iterations are on the design canvas: https://claude.ai/artifact/Kj6SEmpTJZac6cXzc9u3bA

## What the sessions show

### One fleet from start to finish: open-issues, 11 to 13 November

The `open-issues` fleet (mike fleet 103) worked through 23 mike issues on one branch, `fleet/open-issues`, meant to land as one pull request.

| Measure | Value |
|---|---|
| Tasks, seats | 36 tasks, 51 seats (48 workers), 10 relaunches |
| Accepted at close | 29 of 36 tasks, 2 of 7 criteria; the fleet was not accepted |
| Seat time | 1,515 minutes working, 10,853 waiting; 959 minutes of the faults were waits for a build permit, memory or a checkout slot |
| Review | 2.4 judge rounds per task; failed verdicts mostly asked for compile and test output the seats could not produce while builds were blocked |
| Questions | 131 asked between seats, 104 answered, most within a minute; 14 sent to Dawid, none answered in the record |
| Tokens | 2.79 billion, about 73 million per accepted task; Dawid put it at "about 40% of my weekly usage" |
| Afterwards | `fleet/open-issues` is gone from the repo; which issues closed is not recorded |

Seats blocked each other because they shared one branch: T9 waited on T4, T5 and T6 to commit, T12 on T1, T5 and W1. Ten seats were launched twice, and the run had no coordinator seat.

Dawid typed 30 turns, from 18:02 on 11 November to 14:18 on 13 November (44 hours, ending with "well we got the code merged and deployed"):

| Kind | Turns |
|---|---|
| Status ("I didn't see a fleet run", "and?", "where are we?") | 9 |
| Complaints about the process | 8 |
| Merge, deploy, cleanup | 5 |
| Defining scope | 3 |
| Correcting work ("you need a branch") | 3 |
| Unsticking ("message the coordinator") | 2 |

Thirteen of the 30 the system could have answered or prevented: the nine status questions, the missing branch, the missing coordinator, the 27-minute full gate set as the fleet's check, and a "needs you" mark shown while no seat was doing anything. His debrief named the rest: "chomped about 40% of my weekly usage, it failed to start once, it was slow to start, the rules… so restrictive that it couldnt do the work… double sessions for some seats… no coordinator… ran out of memory". Two lines of his set requirements for this app: "the fleet definition and how they run should not be in code", and "how do we make it so that on the fleet screen, we can actually interact and answer the needs".

### Management turns since September

584 of Dawid's 10,666 typed turns were management. They took about 103 hours of session time (5.5% of the time he was in sessions), 54% of it about temet-proxy.

| Kind | Sep | Oct | Nov 1–26 | Total |
|---|---|---|---|---|
| Release and deploy | 36 | 80 | 115 | 231 |
| Other sessions, coordination | 30 | 42 | 78 | 150 |
| "Where are we", status | 15 | 38 | 43 | 96 |
| File an issue | 2 | 11 | 21 | 34 |
| Ask for open or new issues | 4 | 4 | 21 | 29 |
| What's left | 1 | 4 | 14 | 19 |
| Priority | 9 | 3 | 6 | 18 |
| PR status or merge | 1 | 7 | 10 | 18 |
| Close or relabel issues | 6 | 0 | 3 | 9 |
| Branch and worktree cleanup | 3 | 2 | 4 | 9 |
| Milestones | 1 | 0 | 7 | 8 |
| Issues into milestones | 0 | 0 | 4 | 4 |

Release and deploy went from 1.2 turns a day in September to 4.4 in November. Coordinating other sessions went from 1.0 to 3.0 a day. Counts come from keyword classification and are approximate.

### Babysitting across all fleets

Of 117 fleets defined since 25 September, 80 were real work and 70 of those finished. mike's judge marked 228 turns in fleet sessions as corrected or redirected, 211 of them in Dawid's own words. Of the 93 fleets that could be matched to their sessions, 3 finished with no turn from him.

## What already exists

- **mike fleets** are an orchestrator in their own right: `define_fleet`, seats with roles, tasks with acceptance criteria, verdicts, mail between seats, `fleet_commit` and `fleet_report`, and a close step with a retrospective. The code is in `temet-proxy/crates/mike-fleet/src/`. The Herdr fleet tab, bar and watch were removed on 22 November; the fleets themselves still run, and the `fleet` ask-kit kit answers questions about them.
- **The `software-factory` thread** (mike project, written in a design session on 26 November) is a design with nothing built. A dispatcher picks issues by priority and milestone. An author works under mike in one permanent work folder per project (`<repo>/.claude/worktrees/auto`, builds through `mbx` into a fixed `target/mbx-auto`). A cloud reviewer with no mike context approves by default and blocks only for an end state not met, a bug introduced or code that does not fit, with 3 rounds and then a human. Sign-in and Wright policy areas never merge on their own. Branch protection and auto-merge are not on yet: main has no branch protection and `allow_auto_merge` is false.
- **herdr-core** (`hc`, version 0.1.22) is a JSON-RPC 2.0 API on a local socket, written for app builders (`hc schema` prints it as OpenRPC). It starts programs in terminals that outlive the window (`terminal.run`), reads their screens (`terminal.read`), types into them (`terminal.send`, `terminal.keys`), waits for an agent to stop working (`terminal.wait`), labels terminals with free-form `key=value` pairs, streams state changes (`events.subscribe`), keeps small JSON documents (`doc.get`, `doc.set`) and reaches other machines over ssh (`machine.*`, `machine.exec`). The Herdr app is one client of it. A new app can be another.
- **Earlier attempts.** `temet-tasks` (Python) has specs, parallel agent dispatch, review gates and SQLite evidence, but no ticket intake and no merge. `temet-jira` is a Jira client and MCP server. `temet-run` ran Claude Code daemons with LangChain stages. `manager` is an empty folder.
- **Self-improvement loops.** mike's judge that marks corrected turns is running (3,476 corrected turns, 27 to 124 a day lately). Filter rules fire. There are 1,059 active lessons; of the 345 the judge rated, sessions used 144 and ignored 195. `get_improvement_effects` has 775 of 833 actions at "not enough data". The fleet retrospective ran 30 times: 19 finished, 10 failed, 1 blocked. The rule-auditor and capability-author agents left no trace of running since October. ask-kit's nightly kit reviews run on schedule; whether their findings became commits was not checked.

## Other tools

| Tool | Tickets from | Isolation | Review and merge | Notes |
|---|---|---|---|---|
| [OpenAI Symphony](https://openai.com/index/open-source-codex-orchestration-symphony/) | Linear only | a directory per issue | hands off at "Human Review"; at "Merging" watches CI, rebases, fixes conflicts | a spec and an Elixir reference service; OpenAI says it won't maintain it as a product |
| [Arbor](https://github.com/penso/arbor) | GitHub and GitLab issues to worktrees | worktrees, PTY terminals | diffs, PR context | built on GPUI; daemon with web UI, CLI and MCP; MIT, about 850 stars |
| [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | created in the app | a branch, terminal and dev server per workspace | human review, creates and merges PRs | Rust backend; the company [shut down](https://vibekanban.com/blog/shutdown) in April 2026 |
| [GitHub Agent HQ](https://github.blog/news-insights/company-news/welcome-home-agents/) | issues, Slack, Linear, Jira | Actions runners | Copilot reviews its own work first | closed |
| [Claude Code agent teams](https://code.claude.com/docs/en/agent-teams) | the prompt | none; the docs warn two teammates editing one file overwrite each other | teammate reviewers, `TaskCompleted` hooks as gates | experimental; no resume, one team per session |
| [Claude Code on the web](https://code.claude.com/docs/en/claude-code-on-the-web) | the prompt | a cloud VM | auto-fix answers CI failures and review comments; can't resolve merge conflicts alone | closed |
| [Multica](https://github.com/multica-ai/multica) | its own board, webhooks, GitHub and GitLab | not described | agent hands back for review | agents are assignees like people; saves workflows as skills |

Conductor, Claude Squad, Sculptor, Superset, Agent Deck, Devin, Factory and Linear's agents were also read. None of the tools runs agent reviewers, agents fixing review comments, merge and deploy as one loop, and none was found that changes its own orchestration from its run history. Write-ups about these tools name review as the bottleneck ([Addy Osmani](https://addyosmani.com/blog/code-agent-orchestra/): "The bottleneck is no longer generation. It's verification").

## GPUI

- The `gpui` crate on crates.io is 0.2.2, published 22 October 2025, pre-1.0 with breaking changes between versions. Projects outside Zed seem to pin a Zed git snapshot; that was not confirmed.
- [gpui-component](https://github.com/longbridge/gpui-component) 0.11.0 (23 November 2026, weekly releases) provides virtual tables, virtual lists, a dock layout, tabs, a code editor, charts and AccessKit roles. It is the UI of Longbridge Pro.
- Linux and Wayland are supported. Zed still has open Wayland issues in the GPUI area (IME preedit, a launch icon flash).
- No local crate uses GPUI yet. Herdr's mods use gpui-kit, a JavaScript layer over GPUI, inside the Herdr app.

## What the design takes from this

| Found | In the design |
|---|---|
| 88% of seat time spent waiting for builds, memory or a checkout | fixed work folders per machine, one cargo build at a time per machine, items placed by a scheduler, waits drawn on the timeline |
| one branch for 36 tasks; tasks blocked on each other's uncommitted work | one branch and one pull request per item; the issue's "Depends on #491" orders the work |
| 14 housekeeping questions sent to Dawid, none answered | rules settle rebinding, second launches and build slots, logged under "Answered for you"; only decisions, disputes and sign-in merges reach him |
| 9 status turns in one run, 96 since September | the overview answers "where are we" without a question |
| verdicts failed on proof the seats couldn't produce | each Done when line maps to a test, run by CI, shown beside the line |
| fleet not accepted, branch gone, issues unrecorded | an item is done when it is merged, deployed, checked in a real session and its issue is closed |
| release and deploy: 231 turns and rising | deploy, the real-session check and cleanup are steps in the repo's config |
| "the fleet definition and how they run should not be in code" | `.temet-work/config.yaml`, `briefs/*.md` and `items/<n>.yaml` live in the repo |
| "on the fleet screen, we can actually interact and answer the needs" | every "needs you" item carries its own buttons |
| about 40% of weekly usage in one run | usage windows in the top bar and on the timeline; heavy work placed against them |
| the retrospective failed 10 of 30 times; lesson effects mostly unmeasured | Learn: each proposed change cites the ruling or wait it came from, goes into a brief file, is measured on the next items and is kept or reverted |

## A first architecture

| Part | What it does |
|---|---|
| `temet-workd`, a Rust daemon | holds the state and runs the loop; connects to the tracker (GitHub first), to herdr-core over its socket (agents start in terminals labelled with item and role, `events.subscribe` streams their state, `terminal.read` and `terminal.send` read and message them), to mike (capture, the corrected-turn judge, lessons), to ask-kit (investigations) and to GitHub Actions (the cloud reviewer) |
| The GPUI app | one client of the daemon, built on gpui-component; "Open in Herdr" hands a terminal to the Herdr app instead of drawing terminals itself |
| CLI and MCP server | on the daemon, so agents and other sessions read and act on the same state |
| Files in the repo | `.temet-work/config.yaml`, `.temet-work/briefs/{author,reviewer,refiner,investigator}.md`, `.temet-work/items/<n>.yaml` |

## Open decisions

1. Which overview leads: A (board by stage), B (timeline by place) or C (queue of what needs you). The canvas draws all three.
2. How temet-work relates to `software-factory`: the daemon could be that loop, with mike's work-folder lease (its first build step) underneath.
3. Whether merge also deploys. `software-factory` leaves this open too.
4. Whether to start from Arbor (GPUI, worktrees, issue-to-worktree, MIT) or start clean.
5. GPUI from crates.io 0.2.2 or a pinned Zed git revision.

## Sources

<!-- slop-lint-ignore P-INSIDER-TERM -- session ids here are for searching mike -->

- open-issues fleet: mike fleet 103, run `fleet-open-issues-0c16da5b`; definition session "Mike's open issues, prioritised" (request 773353); debrief at session 0efed272, turn 55085, request 782810; "should not be in code" at turn 55090, request 783075; "interact and answer the needs" at session 1f567cdd, turn 55076, request 782755.
- `software-factory`: session 5fdbb32f-62b3-4a5a-8578-fcbd48808965, thread `software-factory` in the mike project.
- Management and babysitting counts: mike capture, 2026-09-01 to 2026-11-26, keyword classification over operator-typed turns.
- Self-improvement state: mike `list_turn_verdicts`, `get_improvement_effects`, `list_filter_rules`, fleet definitions with `retrospective_job`, on 26 November.
- herdr-core: `hc --help` and `hc schema`, version 0.1.22.
- Other tools and GPUI: the links in the tables above, fetched on 26 November 2026.
