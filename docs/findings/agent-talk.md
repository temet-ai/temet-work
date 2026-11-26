---
session_id:
  - cc29fbec-3b84-4b78-9ebc-6fe866b89227
---

# Agents settling a review dispute over mike mail and hc send

What happened on 26 November 2026 when an author, a reviewer and an arbiter, all Claude Code on
haiku, disputed one review comment on omit. Times are SAST. The script, fixture and raw records
are in [spikes/agent-talk/](../../spikes/agent-talk/README.md).

## Result

| Run | Channel | Blocking comment sent to ruling held by both | Ruling | Messages delivered |
|---|---|---|---|---|
| mail1 | mike mail | 59.5 s | reviewer | 6 of 6, once each |
| mail2 | mike mail | 58.2 s | reviewer | 6 of 6, once each |
| hc1 | hc send | 28.5 s | **author** | 6 of 6, once each |
| hc2 | hc send | 25.5 s | reviewer | 6 of 6, once each |

All four disputes ended in a ruling with no turn from a person. The same fixture and rules, with the dispute
worded afresh by each run's agents, gave two different rulings.

## Setup

- Fixture: temet-ai/mike issue 516 (numeric tool schemas) as `gh issue view` printed it, a diff written for this test that
  walks tool input schemas only, the reviewer's comment (elicitation fields must be tested too),
  the author's answer (none are sent today) and four rules for the arbiter.
- Messages, in order: block (1), dispute (2), hold (3), file with the arbiter (4), ruling to the
  author (5), ruling to the reviewer (6). `M` ids went by mail and `H` ids by `hc send`.
- The reviewer was told to keep its block, so every run reached the arbiter. Whether haiku agents
  concede to each other was not tested.
- Each session was started with `mike spawn open` (the fleet seat's mike profile has no
  `spawn_session` tool). It turns on Remote Control and the mike channel.
- Two runs per channel. Claude Code 2.1.332, hc 0.1.22, mike 0.1.0-alpha.5.

## Every dispute message

`sent` is the sender's send call; `received` is the record that put the message in front of the
recipient's model. Both come from the transcripts. Every message has one receiving record.

### mail1

| id | from → to | sent | received | seconds | arrived by |
|---|---|---|---|---|---|
| M1 | reviewer → author | 17:21:43.516 | 17:21:44.921 | 1.41 | peer socket |
| M2 | author → reviewer | 17:22:04.532 | 17:22:06.937 | 2.40 | peer socket |
| M3 | reviewer → author | 17:22:17.328 | 17:22:20.311 | 2.98 | tether plugin |
| M4 | author → arbiter | 17:22:25.696 | 17:22:26.683 | 0.99 | tether plugin |
| M5 | arbiter → author | 17:22:40.932 | 17:22:42.953 | 2.02 | peer socket |
| M6 | arbiter → reviewer | 17:22:41.925 | 17:22:42.973 | 1.05 | peer socket |

### mail2

| id | from → to | sent | received | seconds | arrived by |
|---|---|---|---|---|---|
| M1 | reviewer → author | 17:30:28.417 | 17:30:30.533 | 2.12 | peer socket |
| M2 | author → reviewer | 17:30:47.171 | 17:30:49.372 | 2.20 | peer socket |
| M3 | reviewer → author | 17:30:52.939 | 17:30:54.558 | 1.62 | peer socket |
| M4 | author → arbiter | 17:31:00.236 | 17:31:02.023 | 1.79 | peer socket |
| M5 | arbiter → author | 17:31:24.501 | 17:31:26.582 | 2.08 | peer socket |
| M6 | arbiter → reviewer | 17:31:25.150 | 17:31:25.389 | 0.24 | peer socket |

### hc1

| id | from → to | sent | received | seconds |
|---|---|---|---|---|
| H1 | reviewer → author | 17:25:08.386 | 17:25:08.724 | 0.34 |
| H2 | author → reviewer | 17:25:14.298 | 17:25:14.865 | 0.57 |
| H3 | reviewer → author | 17:25:17.453 | 17:25:17.819 | 0.37 |
| H4 | author → arbiter | 17:25:20.601 | 17:25:20.970 | 0.37 |
| H5 | arbiter → author | 17:25:34.466 | 17:25:34.937 | 0.47 |
| H6 | arbiter → reviewer | 17:25:36.611 | 17:25:36.917 | 0.31 |

### hc2

| id | from → to | sent | received | seconds |
|---|---|---|---|---|
| H1 | reviewer → author | 17:33:34.172 | 17:33:34.674 | 0.50 |
| H2 | author → reviewer | 17:33:39.164 | 17:33:39.894 | 0.73 |
| H3 | reviewer → author | 17:33:42.148 | 17:33:42.865 | 0.72 |
| H4 | author → arbiter | 17:33:45.387 | 17:33:45.723 | 0.34 |
| H5 | arbiter → author | 17:33:57.203 | 17:33:57.696 | 0.49 |
| H6 | arbiter → reviewer | 17:33:59.369 | 17:33:59.672 | 0.30 |

### Why mail took twice as long

Delivery is a small part of it: the six mail hops of a run add up to 10.9 and 10.1 s, the six hc
hops to 2.4 and 3.1 s. In the mail runs the agents also loaded the mail tools, read their mailbox and
acked. From receiving a mail to calling its ack took 4.1 to 17.4 s (nine acks; the longer ones
include loading tools or reading a fixture file).

## The rulings

| Run | Ruling as sent to the author |
|---|---|
| mail1 | REVIEWER. "Scope names every elicitation field, and the author's claim that there are none is not shown in the fixture (rules.md, point 3)". Add a second loop over every elicitation schema; "if the author shows that no elicitation schema exists, the loop may stay empty". |
| mail2 | REVIEWER. The claim that none are sent "has not been shown". Walk every elicitation field schema alongside the tool schemas. |
| hc1 | AUTHOR. "The Done when line is met and Scope has no elicitation field to test today […] the no-elicitation claim is not shown in the fixture and a grep for elicitation requests in mike-mcp would settle it. Merge as it is." |
| hc2 | REVIEWER. The claim "has not been shown". Walk every elicitation field schema the server defines. |

- Rule 3 of the fixture says an unshown claim about the code is not evidence. The hc1 arbiter
  said the claim was unshown and still ruled for the author.
  Rule 3 also asks the arbiter to say what would settle such a claim; hc1 and mail1 did, mail2 and
  hc2 did not.
- The author's claim holds in the real repository. `git grep -il elicit` in temet-proxy at
  `42cdc4b13` lists three Codex schema fixtures under `crates/mike-runtime` and nothing in
  `crates/mike-mcp`. The arbiter was given only the fixture files and ran no search, so three of
  the four rulings asked for a loop over schemas that do not exist.

## Delivery cases outside the clean runs

One session per case (`probe.py`, one try each). `probe.py` logged the `hc` sends. The mails and
G1 were sent by hand, and their results are in
[runs/by-hand.md](../../spikes/agent-talk/runs/by-hand.md).

| Case | id | Channel | Sent | What happened |
|---|---|---|---|---|
| Recipient 4 s into a 45 s command | P1 | hc | 17:28:08.573 | accepted (`started: true`), queued, read at 17:28:50.4 when the command ended |
| same | P2 | hc `--if idle` | 17:28:10.575 | refused, exit 10 `status_mismatch`, nothing typed |
| same | P3 | hc | 17:28:10.778 | as P1 |
| same | P4 | mail | 17:28:12.914 | on the channel at 17:28:14.197, read at 17:28:50.4 |
| Just after `mike spawn open` returned | B1 | hc | about 17:29:24.5 | the session was working on its first prompt; the call returned at 17:29:27.5, after that turn; read at 17:29:27.520 and answered |
| Open permission dialog | K1 | hc | 17:29:44.982 | refused, exit 9 `blocked`, nothing typed |
| same | K2 | mail | 17:29:47.291 | held: `delivered`, not consumed. The dialog was cancelled at 17:29:54.3; read at 17:29:54.475 |
| Session killed and terminal removed | G1 | hc | 17:30:02 | refused, exit 3 `not_found` |
| same | G2 | mail | 17:30:02.138 | send succeeded with `delivery_state: channel` and `recipient_liveness: ended`; still `pending`, 0 attempts, not dead-lettered. At 17:38:31, 8.5 minutes after the send, mike mailed the sender "Not delivered … that session made no request in time" |

## Failures and faults

All 24 dispute messages arrived once and were acted on. The faults seen:

| Fault | Evidence |
|---|---|
| Different rulings from the same fixture and rules | hc1 against mail1, mail2 and hc2 |
| A delivered mail does not show its `message_uuid`, which `ack_session_message` needs | mail1 arbiter: "I did not ack M4 because the mail envelope did not give its message_uuid"; mail2 reviewer said the same of M2. Agents that did ack read their mailbox first |
| 3 of 12 mails never acked | M4 in mail1, M2 and M6 in mail2. All three show `consumed`, one delivery attempt, and none was redelivered during the 90 s after the exchange |
| Mail arrives in two different forms | 10 of 12 came over Claude Code's peer socket (`<cross-session-message>`), 2 as a tether plugin prompt (`Mail from …`), both forms in mail1 |
| A call to a tool that does not exist | mail1 author, 17:21:46.884: `mcp__plugin_tether_tether__ToolSearch` |
| An hc message looks like the user typing | it arrives as a user turn `<hc from="t1sm">…</hc>`, with no session id. In the busy probe haiku listed P1 and P3 as "from the user" |
| A model's account of message order was wrong | busy probe: it listed the mail (sent last) first; the transcript has P1, P3, P4 |
| Every `mike spawn open` from the fleet seat warned that it could not reach the database | "the database is unreachable from this process (persistence is enabled but MIKE_DATABASE_URL is not set …); launches are recorded through the proxy" (`stderr` in `run.jsonl`). The sessions started normally |
| A terminal has no fleet label when it starts | `mike spawn open` takes no labels; the script added them 4.1 to 5.0 s after the call (12 dispute sessions) |

The first attempt at mail1 was stopped by a bug in the test script before the reviewer started;
its two sessions were killed, and the repeated run is the mail1 reported here.

mike's own tool descriptions say an unacked mail is redelivered. On this path (Remote Control and
mike channel on) the mailbox marked each dispute mail consumed within 1.6 s of being queued,
before any ack.

## For temet-work

### Daemon

Build on:

- Agents can settle a dispute without a person: four of four reached a ruling in 25 to 60 s.
- Starting every agent with Remote Control and the mike channel on (`mike spawn open` does). The
  repository's agent notes say an idle session without them never reads its mail; not tried here.
- mike mail for anything one agent tells another, for sessions started that way. It names the sender's session, is held while the
  recipient is at a permission dialog (K2) and keeps a record with times that can be queried.
- `hc send` for the daemon's own nudges and prompts: under 1 s to an idle session (a busy one
  read it 42 s later, when its command ended), `--if idle` when a nudge must not queue, a clear refusal for blocked (exit
  9), busy with `--if idle` (exit 10) and gone (exit 3).
- Labels and `hc events` to follow each session. `working` and `done`, with the agent's own status
  line, arrived as events (`runs/*/events.log`). Label each terminal right after the spawn, since
  `mike spawn open` takes no labels.

Avoid:

- Treating a successful mail send as delivery. G2 returned success for a dead recipient and
  stayed pending; the notice to the sender came 8.5 minutes later. Check `recipient_liveness`, and read the mailbox row for `consumed`.
- Reading `done` as finished. It means the turn ended, including while the agent waits for a
  reply (`events.log`: `done` with "sent M1 to author; waiting on their reply (M2)").
- `hc send` for what one agent tells another. The recipient gets it as a user turn, and haiku
  called it a message "from the user".
- Relying on acks. Agents could not ack from what they were shown, and an unacked mail still
  counted as consumed.
- Letting an arbiter rule on files handed to it alone. Give it the repository and require the
  command it ran when a ruling turns on a claim about the code.
- Merging or reworking on a single arbiter's verdict without checking its form. hc1 called the claim unshown
  and still ruled on it, against rule 3. The daemon could require a fixed shape (ruling, rule, evidence) and
  send back one that has no evidence.

Still unknown:

- How often an arbiter on sonnet or opus gives the same ruling twice. Three of the four haiku rulings
  went to the reviewer and one to the author.
- Whether agents concede without being told to hold.
- Mail to an idle session without Remote Control or the mike channel (not tried here).
- Whether quotes and apostrophes survive an agent's `hc send` command line; the prompts forbade
  apostrophes, so it was not tried.
- What picks the peer socket or the tether prompt for a given mail.
- Whether a pending mail to an ended session is ever dead-lettered.
- Delivery between two machines; both channels were measured on omit only.

### App

Build on:

- A dispute is six short messages with ids and times, so it can be shown as one thread with the
  ruling and the seconds it took. The mail record already holds sender, recipient, queued and
  consumed times.
- `blocked` is a state the app can show as "needs you": hc refused to type into the dialog and
  mail waited behind it.

Avoid:

- Reading only one mail form from transcripts: mail arrived both as a peer-socket message and as
  a tether plugin prompt.
- Showing a mail as received because the send returned `channel`.
- Showing a ruling as settled fact. Show the rule it cites and what was checked, with a way to
  overturn it.

Still unknown:

- Whether the app should show hc messages at all: they leave no mailbox record, and here they were
  recovered from transcripts.
