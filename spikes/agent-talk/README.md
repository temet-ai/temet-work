---
session_id:
  - cc29fbec-3b84-4b78-9ebc-6fe866b89227
---

# agent-talk: three haiku sessions settle a review dispute

An author, a reviewer and an arbiter dispute one review comment, over mike mail in one run and
over `hc send` in another. Results are in [docs/findings/agent-talk.md](../../docs/findings/agent-talk.md).

## Run it

Needs a herdr-core terminal on a machine with mike, and a repository Claude Code already trusts.

```sh
cd spikes/agent-talk
python3 run.py start mail3 mail    # or: start hc3 hc
python3 timeline.py mail3          # send and receive time of every message, local time
python3 peek.py mail3              # every transcript record of the three, UTC
python3 run.py close mail3         # kill and remove only this run's three sessions
```

A run name can be used once: `runs/<name>/` must not exist.

## What `start` does

1. `mike spawn open <repo> … -- --model haiku` for the arbiter, then the author. `mike spawn open`
   turns on Remote Control and the mike channel, which this repository's agent notes say an idle
   session needs to be woken by mail (not tried here). It takes no labels, so the script labels
   the terminal as soon as it appears
   (`fleet=temet-work-research`, `by=`, `role=`, `item=<run>`).
2. Waits until both are idle, then starts the reviewer, whose first step sends message 1. Each
   later session is given the earlier ones' session ids (mail) or terminal addresses (hc).
3. Watches the transcripts until all six sends appear, waits for the three to stop working, then
   waits 90 s more so a late or repeated delivery would show.

The sessions get `Read`, the mike mail tools and `Bash(hc send:*)` allowed, and `Edit`, `Write`,
`NotebookEdit`, `Agent` and `AskUserQuestion` disallowed.

## Files

| Path | What it is |
|---|---|
| `fixture/issue-516.json` | temet-ai/mike issue 516 as `gh issue view` printed it |
| `fixture/change.diff` | the author's change (written for this test, not from mike) |
| `fixture/review-comment.md` | the reviewer's blocking comment |
| `fixture/author-position.md` | the author's answer |
| `fixture/rules.md` | what the arbiter rules by |
| `prompts/` | the role prompts, one set per channel |
| `run.py`, `events.py`, `peek.py`, `timeline.py` | start and close a run, log `hc events`, print the transcripts, tabulate the message times |
| `probe.py` | one session per delivery case: busy, booting, blocked at a dialog |
| `runs/<name>/` | `run.jsonl` (the script's log), `events.log`, prompts, `timeline.md`, `transcript.txt` |

The six messages of a run are `M1` to `M6` over mail and `H1` to `H6` over hc: block, dispute, hold,
file with the arbiter, ruling to the author, ruling to the reviewer.

## Probes

```sh
python3 probe.py busy busy2       # two hc sends while a 45 s command runs
python3 probe.py booting boot2    # an hc send as soon as `mike spawn open` returns
python3 probe.py blocked blk2     # an hc send against an open permission dialog
python3 run.py close busy2
```

The mail half of each probe (`P4`, `K2`, `G2`) was sent by hand with mike's
`send_session_message` from the operating session, to the session id in
`runs/<name>/state.json`, and the results are in `runs/by-hand.md`.
