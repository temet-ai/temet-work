---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# How the research fleet ran

The four findings beside this file came from one mike fleet, `temet-work-research`, run on
26 November 2026. This document is about the run itself: what it needed from outside, where
its seats waited, and what temet-work takes from that. Figures are from mike's run record;
times are SAST.

## The run

| Measure | Value |
|---|---|
| Start to close | 29.9 minutes (17:17 to 17:47) |
| Tasks | 4, all accepted |
| Seats | 4 workers on Sonnet, 1 reference seat, 4 template seats |
| Seat minutes | 57.8 working, 45 waiting |
| Waiting, by seat | reference 24.7, GPUI 15.8, herdr-core 4.5, the other two 0 |
| Questions between seats | 3, all from the GPUI seat, all about the same fault |
| Questions to a person | 1, answered 10 seconds after it was asked |
| Relaunches, manual stops, nudges | 0 |
| Failed verdicts | 1 (GPUI, then passed one commit later) |
| Tokens | 49.3 million, 12.3 million per accepted task |

For comparison, the `open-issues` fleet of 11 to 13 November (`docs/research.md`) had 36
tasks on one shared branch, 2.79 billion tokens (73 million per accepted task), 10,853
waiting seat minutes against 1,515 working, 14 questions sent to a person and none
answered in the record. The two runs are not alike in difficulty: these were four
separate research tasks, each writing its own files.

## What it needed from outside

| Time | What happened | Settled by | Could the system have prevented it? |
|---|---|---|---|
| Before the start | The task about mike's fleet faults named its eight issues on the wrong repository. | The capability author, which checks each brief before seats start, found it; the author corrected the brief. | Yes: a check at definition time that every issue a brief names exists. |
| Before the start | `mike` refused the branch: it did not exist and the new repository had no remote. | The author made the branch and a local remote. | Yes: setting up the branch is the system's job. |
| 17:19 to 17:21 | `fleet_commit` refused the GPUI seat's commit: the shared checkout was on `main`, the run branch was `research`. The seat asked the reference seat, which answered once and declined once, then asked the author. | The author switched the branch and answered, 10 seconds after the question. | Yes: the start checks confirmed the branch could be pushed, not that the checkout was on it. |
| 17:45 | The judge failed the GPUI task on its learnings criterion ("not shown met by the report and the diff"). It read the last commit only, not the task's earlier commit with most of the findings. | The seat added the evidence behind each item and passed at 17:46. | Partly: a reviewer that reads the whole branch against its base would have seen the earlier commit. |

## For temet-work

Build on:

- Tasks whose files do not overlap, each with its own output. No seat waited on another, and
  none of the waiting came from a build slot or a shared tree (Waiting, by seat).
- A specialist agent per task, written before the start, which also checks the briefs: it
  caught the wrong repository before any seat ran (What it needed from outside).
- Criteria with a derived check each, judged against the seat's own report (The run).

Avoid:

- Starting work before checking the place it runs: the branch exists, the checkout is on
  it, it can be pushed, and every file and issue a brief names exists. Three of the four
  events above were this (What it needed from outside).
- Judging the last commit alone. A reviewer reads the change from the base of the branch.
- Seats that stand by for the whole run. The reference seat waited 24.7 of its 28.4 minutes
  and the four template seats did almost nothing. In larger runs such seats read 155 million
  tokens in one run and took a turn on every small change (`mike-fleets.md`, the faults on
  control seats and digests).

Still unknown:

- What the retrospective concluded: it ran and finished, and its output was not found.
- How the same setup runs at ten times the tasks, with tasks that do share files.
