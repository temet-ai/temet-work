---
session_id:
  - d8b56904-12b7-4f79-b22a-277694261d14
  - e4a814a9-1735-455e-9162-778decf89d5b
  - d1ccda48-cbb5-44cc-9ea0-c03999c970e9
---

# herdr-core: what a daemon can rely on

herdr-core 0.1.22 on omit, 26 November 2026. Everything below comes from one run of
[`spikes/herdr-core/probe.py`](https://github.com/temet-ai/temet-work/blob/88a79e6e7e11f4c9805d821a4d7ebf91c1a3ffc3/spikes/herdr-core/probe.py) unless it says otherwise; its
output is [`spikes/herdr-core/run.log`](https://github.com/temet-ai/temet-work/blob/88a79e6e7e11f4c9805d821a4d7ebf91c1a3ffc3/spikes/herdr-core/run.log). Each log line is the local
time (SAST), seconds since the probe started, the connection, and the message. Quoted lines are cut
at the right where marked `…`. Both files are kept at commit 88a79e6; to run the probe again, check that commit out and
run `python3 spikes/herdr-core/probe.py` (it overwrites `run.log`, takes about two minutes and removes what it started).

The probe started a plain `bash`, Claude Code 2.1.332 on haiku with `prompt.txt` ("List the files in
this folder, then stop."), and three `sleep` terminals, all labelled
`fleet=temet-work-research by=T1 item=<n> role=<r>`. It ended with:

```text
17:31:01.847 + 126.706 ##        left: 0
```

## Answer

A daemon can drive agents through this socket. Starting, labelling, status events, sending, waiting,
killing and the documents all worked. It has to handle these itself:

- A finished turn is usually `done`, and `send` with `if: "idle"` refuses it.
- `terminal.interrupt` leaves the interrupted prompt in Claude Code's input box when it had produced
  nothing yet, and the next send is glued to it.
- Requests on one connection are answered one after another, so a `terminal.wait` blocks every call behind it.
- `terminal.wait` with `output` only sees text printed after the wait began.
- A subscriber that stops reading gets `missed`, and after a longer stall the core closes the connection.
- Reading an idle agent with history took 2.5 to 3 seconds.

## Verdict per method

| Method | Verdict | Time seen | Caveat |
|---|---|---|---|
| `hello` | works | 0.2 ms | |
| `server.status` | works | 0.1 ms | |
| `events.subscribe` | works with caveats | snapshot 1 ms | `missed`, then disconnect, for a slow reader |
| `terminal.run` | works | 7 ms | per `hc --skill`, terminals get the core's environment; pass proxy variables in `env` |
| `terminal.list` | works | 4 ms | the reply also lists every added machine |
| `terminal.get` | works | 0.2 ms | |
| `terminal.label` | works | 4 ms | something else on this machine also adds a `where` label |
| `terminal.read` | works with caveats | 0.2 ms to 3 s | screen only while busy; slow when idle with history |
| `terminal.send` | works with caveats | about 200 ms | reply carries the earlier status; `if` has no `done` |
| `terminal.wait` | works with caveats | returns with the status change | blocks its connection; `output` misses earlier text |
| `terminal.keys` | works | 31 ms | reply carries the status from before the key |
| `terminal.interrupt` | works with caveats | 23 ms to 10 s | restored draft; `unknown` for a plain shell once |
| `terminal.kill` | works | 5 ms to 1 s | Claude Code ended on SIGKILL |
| `terminal.rm` | works | 2 to 4 ms | refuses a live terminal |
| `doc.get` | works | 0.1 ms | names are 1 to 32 of `a-z 0-9 - _ .` |
| `doc.set` | works | 2 ms | |

Not exercised: `terminal.attach`, `terminal.keep`, `terminal.put*`, `machine.*`, `file.*`,
`templates.list`, `server.restart`, `server.stop`, launching a template, the `children_of`
filter, and a `send` where `if` matched `idle`.

## Evidence

### Connecting: `hello`, `server.status`

```text
17:28:55.141 +   0.000 req >     id=1 hello {}
17:28:55.142 +   0.001 req <     id=1 hello 0.2ms {"result": {"core": {"id": "caf39453aac447cd4014b43e107f73de", "name": "omit"}, "features": ["framed", "attach", "events", "status", "shell", "resize_notifications", "automatic_inp …
17:28:55.146 +   0.005 req >     id=3 server.status {}
17:28:55.147 +   0.006 req <     id=3 server.status 0.1ms {"result": {"core": {"id": "caf39453aac447cd4014b43e107f73de", "name": "omit"}, "finished": 29, "holder": {"current": true, "instance": "9554eaa846bf66411e62acd0dd81f07d",  …
```

`hello` without `framed` leaves the connection as one JSON object per line.

### Starting: `terminal.run`

```text
17:28:55.160 +   0.019 req >     id=6 terminal.run {"labels": {"fleet": "temet-work-research", "by": "T1", "item": "1", "role": "agent"}, "caller": "t1s0", "no_notify": true, "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "name": "t1-probe-agent", "env": "<redacted>", "size": {"cols": 120, "rows": 40}, "argv": ["claude", "--model", "haiku", "--session-id", "766cd245-9f11-4385-a676-f4ec21e52e03", "Lis …
17:28:55.168 +   0.027 req <     id=6 terminal.run 7.4ms {"result": {"address": "t1sv", "argv": ["claude", "--model", "haiku", "--session-id", "766cd245-9f11-4385-a676-f4ec21e52e03", "List the files in this folder, then stop."], "cols": 120, "created_ms": 1795706935163, "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "die_with_parent": true, "identity": "ghostty", "keep": false, "labels": {"by": "T1", …
```

The probe passed `caller` (its own `HC_TERMINAL_ID`), `no_notify: true` so nothing is typed into the
parent, `cwd`, `size`, `labels`, and `env` with `HTTPS_PROXY`, `NO_PROXY` and
`NODE_EXTRA_CA_CERTS` (redacted in the log). The task went in `argv`. Mike captured the session: a
separate `find_text` on session `766cd245-9f11-4385-a676-f4ec21e52e03` after the run found the
probe's `PONG-2` message in requests 900823 and 900825 (not in `run.log`).

### Following: `events.subscribe`

One connection subscribed with `labels: {fleet, by}`. Events are
`{"method": "event", "params": {"type", "terminal", "seq", "data"}}`. Types seen in this run were
`created`, `foreground`, `labels`, `title`, `status`, `turn`, `finished`, `removed` and `doc`.
A heartbeat came every 15 s:

```text
17:28:55.166 +   0.025 events << 757B {"jsonrpc": "2.0", "method": "event", "params": {"data": {"address": "t1sv", "argv": ["claude", "--model", "haiku", "--session-id", "766cd245-9f11-4385-a676-f4ec21e52e03", "List the files in this folder, then stop."], "cols": 120, "created_ms": 1795706935163, "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "die_with_parent": true, "identity": "ghostty", "keep": false, "labels": {"by": "T1", "fleet": "temet-work-research", "item": "1", "role": "agent"}, "name": "t1-probe-agent", "notify": false, "parent": "t1s0", "pid": 1054313, "reported": false, "rm": false, "rows": 40, "screen": "exact", "state": "live", "turn": 0}, "seq": 45260, "terminal": "t1sv", "type": "created"}}
17:29:10.153 +  15.012 events << 62B {"jsonrpc": "2.0", "method": "heartbeat", "params": {"seq": 45280}}
17:29:25.154 +  30.013 events << 62B {"jsonrpc": "2.0", "method": "heartbeat", "params": {"seq": 45314}}
```

`seq` counts across the whole core, so a filtered subscriber sees gaps and cannot use them to
detect loss.

A fresh subscription returns a snapshot that includes finished terminals not yet removed, which is
what a restarted daemon would resync from:

```text
17:31:01.816 + 126.675 resub <   id=2 events.subscribe 1.0ms {"result": {"seq": 51632, "terminals": [{"address": "t1st", "argv": ["bash", "--noprofile", "--norc"], "cols": 100, "created_ms": 179570693 …
```

**A reader that falls behind.** The probe subscribed to one document name, did not read, and wrote
to the document. 200 changes were all delivered later. With 3000, 275 arrived, then a `missed`
notification replaced the rest. With 3000 and 40 more seconds without reading, 274 arrived and the
core had closed the connection, with no `missed`:

```text
17:29:58.682 +  63.541 ##        200 doc.set while 'lazy' was not reading took 0.5s; it stays unread 0s more
17:30:01.683 +  66.542 ##        lazy got {'event:doc': 200} of 200 changes; connection closed by the core: False
17:30:07.099 +  71.958 ##        3000 doc.set while 'lazy' was not reading took 4.9s; it stays unread 0s more
17:30:10.102 +  74.961 ##        lazy got {'event:doc': 275, 'missed': 1} of 3000 changes; connection closed by the core: False
17:30:10.102 +  74.962 ##        lazy got: {"jsonrpc": "2.0", "method": "missed", "params": {"seq": 48575, "terminals": []}}
17:30:17.758 +  82.617 ##        3000 doc.set while 'lazy' was not reading took 7.2s; it stays unread 40s more
17:31:00.759 + 125.618 ##        lazy got {'event:doc': 274} of 3000 changes; connection closed by the core: True
```

A `missed` carries `terminals` (a fresh snapshot, empty here because only documents were subscribed)
and no document values, so a client must `doc.get` again. The exact stall that triggers the close
was not measured. This was tested with `doc` events only.

### How agent state shows up

Status comes in `status` events (`idle`, `working`, `done`, `blocked`), and `turn` events record the
state each turn ended in. The launch turn:

```text
17:28:55.160 +   0.019 req >     id=6 terminal.run {"labels": {"fleet": "temet-work-research", "by": "T1", "item": "1", "role": "agent"}, "caller": "t1s0", "no_notify": true, "cwd": "/mnt/data/project …
17:28:56.194 +   1.053 STATE     t1sv status=idle kind=None msg=None
17:28:58.201 +   3.060 STATE     t1sv status=working kind=None msg=None
17:29:01.106 +   5.965 STATE     t1sv status=working kind=None msg='Listing files in the herdr-core spike folder'
17:29:03.940 +   8.799 STATE     t1sv status=done kind=None msg=None
17:29:03.940 +   8.799 STATE     t1sv turn {"state": "done", "turn": 1}
17:29:05.581 +  10.441 STATE     t1sv status=done kind=None msg='listed spikes/herdr-core: probe.py, prompt.txt, run.log'
```

- `idle` 1.0 s after the launch, `working` after 3.0 s, `done` after 8.8 s.
- The agent adds a `msg` describing its work; the summary of a finished turn arrived 1.6 s after `done`.
- After a send, `working` arrived in 0.22 s:

```text
17:29:07.180 +  12.040 ##        send returned after 0.227s; calling terminal.wait at once
17:29:08.568 +  13.427 ##        send request -> wait returned 1.615s
17:29:08.568 +  13.427 ##        send request -> 'working' event 0.224s
17:29:08.568 +  13.427 ##        send request -> 'done' event 1.609s
```

**Waiting for a person** is `blocked`, with `kind` and `msg`. The probe asked the agent to use
`AskUserQuestion`; `blocked` arrived 1.9 s after the send:

```text
17:29:35.672 +  40.531 STATE     t1sv status=blocked kind=question msg='answer: Continue? (Yes · No)'
17:29:35.672 +  40.531 ##        send -> 'blocked' event 1.854s
```

```text
17:29:35.678 +  40.537 req >     id=42 terminal.send {"terminal": "t1sv", "text": "SHOULD-BE-REFUSED", "caller": "t1s0"}
17:29:35.678 +  40.537 req <     id=42 terminal.send 0.2ms {"error": {"code": -32000, "data": {"hint": "typing would answer its dialog; read its screen first (hc read t1sv), or --force", "kind": "blocked"}, "message": "t1sv is blo …
17:29:35.678 +  40.537 req >     id=43 terminal.keys {"terminal": "t1sv", "keys": ["esc"], "caller": "t1s0"}
17:29:35.709 +  40.568 req <     id=43 terminal.keys 30.7ms {"result": {"status": "blocked", "turn": 9}}
```

After `esc` the status went `working` then `idle`. A permission prompt was not tested (the session
ran in auto mode), and `error` was never seen.

A plain shell reports no status at all, only `foreground` and `turn: 0`.

### Reading: `terminal.read`

While working or blocked, a read with `lines: 100` returned at once with `complete: false`:

```text
17:28:58.204 +   3.063 req <     id=7 terminal.read 2.6ms {"result": {"complete": false, "how": "screen", "text": "\n \u2590\u259b\u2588\u2588\u2588\u259b\u2588   Claude  …
17:29:06.946 +  11.805 req <     id=9 terminal.read 3001.1ms {"result": {"complete": true, "how": "stitched", "text": "\n \u2590\u259b\u2588\u2588\u2588\u259b\u2588   Cla …
17:29:35.678 +  40.537 req <     id=41 terminal.read 0.3ms {"result": {"complete": false, "how": "screen", "text": "\n\u273b Cooked for 1s \u00b7 done 5:29 PM\n\n\u276f < …
```

`how` was `screen` while busy and `stitched` when idle, which took 3.0 s here because the core
scrolls the full-screen app to collect history. A shell's history (`how: "history"`) took 0.2 ms.
The screen after the first turn, as returned:

```text
▝▜██████▀  Haiku 5.5 with medium effort · Claude Ultra
 ▝▝   ▝▝   /mnt/data/projects/temet/temet-work/spikes/herdr-core · /rc
❯ List the files in this folder, then stop.
  Listed 1 directory
● The spikes/herdr-core folder contains three files:
  - probe.py (20 KB, modified 26 Nov 17:28)
  - prompt.txt (42 bytes, modified 26 Nov 17:19)
  - run.log (13 KB, modified 26 Nov 17:29)
✻ Crunched for 5s · done 5:29 PM
                                                                                                    ◐ medium · /effort
❯
  ⏵⏵ auto mode on (shift+tab to cycle) · ← 1 agent · 󱘶 200+ changes by others
```

A message sent by a terminal arrives in the agent wrapped as `<hc from="t1s0">…</hc>`.

### Sending: `terminal.send`

```text
17:29:06.953 +  11.812 req >     id=13 terminal.send {"terminal": "t1sv", "text": "Reply with exactly: PONG-1", "if": "idle", "wait": true, "timeout": 90, "caller": "t1s0"}
17:29:06.954 +  11.813 req <     id=13 terminal.send 0.2ms {"error": {"code": -32000, "data": {"hint": "nothing was typed", "kind": "status_mismatch"}, "message": "t1sv is done, not idle"}}
17:29:06.954 +  11.813 req >     id=14 terminal.send {"terminal": "t1sv", "text": "Reply with exactly: PONG-2", "caller": "t1s0"}
17:29:07.180 +  12.039 req <     id=14 terminal.send 226.6ms {"result": {"started": true, "status": "done", "turn": 1}}
17:29:11.233 +  16.092 req >     id=17 terminal.send {"terminal": "t1sv", "text": "Reply with exactly: PONG-3", "wait": true, "timeout": 90, "caller": "t1s0"}
17:29:12.712 +  17.571 req <     id=17 terminal.send 1478.9ms {"result": {"started": true, "status": "done", "turn": 2, "wait": {"ended": true, "reason": "status", "status": "done", "terminal": "t1sv", "turn": 3}}}
```

- After an unseen turn the status is `done`, and `if` accepts only `idle` or `working`.
- The reply's `status` and `turn` describe the terminal before this message; `started: true` says
  the turn began.
- `terminal.wait` called straight after `send` returned waited for the new turn, although the old
  status was still `done` when `send` was called.
- `wait: true` returns the new turn inside `wait`.

While the agent was working:

```text
17:29:12.938 +  17.797 req >     id=19 terminal.send {"terminal": "t1sv", "text": "SHOULD-NOT-ARRIVE", "if": "idle", "caller": "t1s0"}
17:29:12.938 +  17.798 req <     id=19 terminal.send 0.2ms {"error": {"code": -32000, "data": {"hint": "nothing was typed", "kind": "status_mismatch"}, "message": "t1sv is working, not idle"}}
17:29:12.939 +  17.798 req >     id=20 terminal.send {"terminal": "t1sv", "text": "Afterwards also reply QUEUED-1.", "if": "working", "caller": "t1s0"}
17:29:13.140 +  17.999 req <     id=20 terminal.send 201.0ms {"result": {"started": true, "status": "working", "turn": 3}}
```

The second message was accepted and Claude Code ran it as a later turn.

### Waiting: `terminal.wait`

On an agent it returns when the status is no longer `working`; `blocked` ends it too:

```text
17:29:03.944 +   8.803 req <     id=8 terminal.wait 5740.0ms {"result": {"ended": true, "reason": "status", "status": "done", "terminal": "t1sv", "turn": 1}}
17:29:35.677 +  40.536 req <     id=39 terminal.wait 4.4ms {"result": {"ended": true, "reason": "status", "status": "blocked", "terminal": "t1sv", "turn": 9}}
```

On a plain shell, `output` matched only text printed after the wait started. Sending first and then
waiting timed out although the text was on screen:

```text
17:29:35.769 +  40.628 req >     id=46 terminal.send {"terminal": "t1st", "text": "echo late-$((40+2))", "caller": "t1s0"}
17:29:35.970 +  40.829 req <     id=46 terminal.send 200.7ms {"result": {"turn": 0}}
17:29:35.970 +  40.829 req >     id=47 terminal.wait {"terminal": "t1st", "output": "late-42", "timeout": 5, "caller": "t1s0"}
17:29:40.971 +  45.830 req <     id=47 terminal.wait 5001.2ms {"result": {"ended": false, "reason": "timeout", "terminal": "t1st", "turn": 0}}
```

With the wait started first, on a second connection, it matched:

```text
17:29:40.972 +  45.831 wait >    id=2 terminal.wait {"terminal": "t1st", "output": "early-42", "timeout": 20, "caller": "t1s0"}
17:29:41.472 +  46.331 req >     id=48 terminal.send {"terminal": "t1st", "text": "echo early-$((40+2))", "caller": "t1s0"}
17:29:41.677 +  46.536 wait <    id=2 terminal.wait 705.0ms {"result": {"ended": true, "reason": "output", "terminal": "t1st", "turn": 0}}
```

There is no acknowledgement that a wait is registered; the probe slept 0.5 s before sending.

Requests on one connection are handled in order. A `server.status` sent right after a 4 s wait was
answered after 4 s:

```text
17:29:41.677 +  46.536 wait >    id=3 terminal.wait {"terminal": "t1st", "timeout": 4, "caller": "t1s0"}
17:29:41.677 +  46.536 wait >    id=4 server.status {}
17:29:45.679 +  50.538 wait <    id=3 terminal.wait 4001.8ms {"result": {"ended": false, "reason": "timeout", "terminal": "t1st", "turn": 0}}
17:29:45.679 +  50.538 wait <    id=4 server.status 4001.9ms {"result": {"core": {"id": "caf39453aac447cd4014b43e107f73de", "name": "omit"}, "finished": 29, "holder": {"c …
```

### Interrupting: `terminal.interrupt`

```text
17:29:15.346 +  20.205 req <     id=22 terminal.interrupt 2205.5ms {"result": {"key": "esc", "outcome": "stopped"}}
17:29:15.593 +  20.452 req <     id=26 terminal.interrupt 22.8ms {"result": {"key": "esc", "outcome": "stopped"}}
17:29:56.122 +  60.981 req <     id=52 terminal.interrupt 10008.4ms {"result": {"key": "ctrl+c", "outcome": "unknown"}}
```

The first two are the agent (`esc`), the third is the shell (`ctrl+c`) running `sleep 30`, where
the core could not tell what happened after 10 s.

The interrupted prompt went back into the input box. The next send was appended to it and both were
submitted as one message:

```text
✻ Churned for 2s · done 5:29 PM
❯ <hc from="t1s0">Write the numbers from 1 to 600 in English words, one per line. No tools.</hc><hc from="t1s0">Reply
  with exactly: NEXT-1</hc>
● NEXT-1
✻ Cogitated for 2s · done 5:29 PM
❯
  ⏵⏵ auto mode on (shift+tab to cycle) · ← 1 agent
```

The probe then interrupted the same way and sent `terminal.keys ["ctrl+c"]` once. The input box was
empty afterwards and the following send arrived alone. The box was not read between the interrupt and
the key, so that a draft was there to clear rests on the case above. Claude Code then shows "Press
Ctrl-C again to exit", so a second key would have ended the session:

```text
  with exactly: NEXT-1</hc>
● NEXT-1
✻ Cogitated for 2s · done 5:29 PM
❯
  Press Ctrl-C again to exit
```

An earlier run of the probe, whose log was overwritten, reported `stopped` for an interrupt after
which the agent had written all 600 lines. Do not treat `stopped` as proof that the turn was cut.

### Labels and listing: `terminal.label`, `terminal.list`, `terminal.get`

```text
17:29:06.946 +  11.805 req >     id=11 terminal.label {"terminal": "t1sv", "labels": {"stage": "probe"}}
17:29:06.949 +  11.808 req <     id=11 terminal.label 2.9ms {"result": {"address": "t1sv", "argv": ["claude", "--model", "haiku", "--session-id", "766cd245-9f11-4385-a676-f4ec21e52e03", "List the files in this folder, then stop."] …
17:29:06.949 +  11.808 req >     id=12 terminal.list {"labels": {"fleet": "temet-work-research", "by": "T1", "role": "agent"}}
17:29:06.953 +  11.812 req <     id=12 terminal.list 4.0ms {"result": {"hidden_finished": 0, "machines": [{"host": "infra", "id": "771475aca891210f7103b208adb6c2c0", "name": "infra", "state": "connected", "target": "infra", "tempo …
17:29:06.946 +  11.805 req >     id=10 terminal.get {"terminal": "t1sv", "caller": "t1s0"}
17:29:06.946 +  11.805 req <     id=10 terminal.get 0.2ms {"result": {"address": "t1sv", "argv": ["claude", "--model", "haiku", "--session-id", "766cd245-9f11-4385-a676-f4ec21e52e03", "List the files in this folder, then stop."],  …
```

Labels were also filters for `events.subscribe`. Within a second of each start, `labels` events
added `where: "temet-work / 3"`, which the probe did not set:

```text
17:28:55.308 +   0.167 events << 203B {"jsonrpc": "2.0", "method": "event", "params": {"data": {"labels": {"by": "T1", "fleet": "temet-work-research", "item": "2", "role": "shell", "where": "temet-work / 3"}}, "seq": 45261, "termi …
```

**Naming a space** (checked 27 November 2026, hc 0.1.22, herdr 0.9.5). herdr-core has no method,
command or label that names a space. `hc schema` has no method or field for spaces
(`terminal.run`'s `name` names the terminal). `hc --help` has no command for them. `hc --skill`
says `hc` does not arrange windows and points to `herdr-app --skill`. `herdr workspace --help`
offers `rename`, but `herdr workspace list` showed the workspaces of `herdr`, the older
multiplexer, not the app's spaces. The app has its own command, `space.rename`, sent over the
app's socket, and it keeps spaces in the core's `herdr-app` document, where a space no one had
named read `"name": ""`. mike sets no space name either (the labels it sets are in
[mike-fleets.md](mike-fleets.md)). The runner sets none: the daemon talks to herdr-core only, no
app need be running, and writing the app's document with `doc.set` would change another
program's state. The app calls an unnamed space after where its first terminal is; in the run
below that was the folder `textkit-watch`.

**Who writes what** (a real run, 27 November 2026, 21:33 SAST). A second `temet-workd` built from
the branch of temet-work#89 started a refiner and then an author on a sandbox issue, in a group `watch`. The
core was on omit, where no app window was open; herdr app 0.1.15 was open on a Mac that has omit
as a machine. `hc events --label group=watch` showed the same steps for both terminals. For the
refiner, cut to the type and the fields that matter:

```text
created   labels agent=refiner-1 group=watch issue=sandbox#32 role=refiner log=1a21e7fd7475c954
          owner=temet-work herdr.space=sptemet-work-watch
          herdr.tab=tbtemet-work-watch-sandbox-32 herdr.name="refiner-1 sandbox#32"; keep false
keep      keep true
labels    the same, and where="textkit-watch / 1"
finished  reason killed, signal 9
removed
```

- The runner's labels are on `terminal.run`, so the first event already has them.
- The app then turned `keep` on and added `where`, which reads `<space name> / <tab name>`. The
  Mac app's `herdr-app ls` listed the space `omit/sptemet-work-watch`, named `textkit-watch`,
  with the tab `omit/tbtemet-work-watch-sandbox-32`, named `1`, holding the terminal. On seven
  other temet-work terminals `where` matched the names the app listed, and one's changed from
  `work / 2` to `work / 4` as tabs moved.
- The app does not show a terminal without the labels. At 21:34 it listed all 15 of omit's local
  terminals that had `herdr.space`, and not the one that had neither that label nor a parent.
  It showed them although omit is another machine to it.
- The app showed no `herdr.name`: the string is in neither the 0.1.15 nor the 0.1.18 program, and
  `herdr-app ls` gave temet-work's terminals `"name": null`. The label is there for `hc get`.
- The daemon's stop ended a kept terminal and removed it. **Read, not run:** `hc --skill` says a
  kept terminal lost to a signal with no exit code, a crash or a reboot gets a shell in its place,
  so the daemon would still see a live terminal with the agent's labels. `herdr-app --skill` says
  closing a pane, tab or space ends the terminals in it.
- An agent on another machine gets the same labels there; that was not run.
- An investigator works on no issue, so the runner gives its terminal no `herdr.space` or
  `herdr.tab`, and the app does not show it.

### Documents: `doc.get`, `doc.set`

```text
17:29:56.122 +  60.981 req >     id=54 doc.get {"name": "temet-work.t1.probe"}
17:29:56.122 +  60.981 req <     id=54 doc.get 0.1ms {"result": {"version": 6}}
17:29:56.122 +  60.982 req >     id=55 doc.set {"name": "temet-work.t1.probe", "version": 6, "value": {"agent": "t1sv", "shell": "t1st", "session": "766cd245-9f11-4385-a676-f4ec21e52e03"}}
17:29:56.125 +  60.984 req <     id=55 doc.set 2.3ms {"result": {"version": 7}}
17:29:56.125 +  60.984 req >     id=56 doc.set {"name": "temet-work.t1.probe", "version": 6, "value": {"stale": true}}
17:29:56.125 +  60.984 req <     id=56 doc.set 0.2ms {"error": {"code": -32000, "data": {"hint": "read doc.get and retry with its version", "kind": "conflict"}, "message": "document \"temet-work.t1.probe\" is at version 7, not 6"} …
17:29:56.125 +  60.984 req >     id=58 doc.set {"name": "temet-work.t1.probe", "version": 7}
17:29:56.128 +  60.987 req <     id=58 doc.set 2.2ms {"result": {"version": 8}}
17:29:56.128 +  60.987 req >     id=59 doc.get {"name": "temet-work.t1.probe"}
17:29:56.128 +  60.987 req <     id=59 doc.get 0.1ms {"result": {"version": 8}}
```

A `value: null` write is shown without `value` because the log drops empty fields. A deleted
document keeps its version number. The name limit showed up in a first attempt (not in this log):
`bad name "temet-work-research.T1.probe": use 1-32 of a-z 0-9 - _ . starting with a letter or digit, not shaped like an id (t7)`.

### Ending: parents, `terminal.kill`, `terminal.rm`

A terminal started over the socket without `caller` had no parent and `die_with_parent: false`, and
was still live a second after the client disconnected:

```text
17:29:56.129 +  60.988 oneoff >  id=2 terminal.run {"argv": ["sleep", "300"], "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "no_notify": true, "labels": {"fleet": "temet-work-research", "by": "T1", "item": "3", "role": "no-caller"}}
17:29:56.137 +  60.996 oneoff <  id=2 terminal.run 8.5ms {"result": {"address": "t1sy", "argv": ["sleep", "300"], "cols": 200, "created_ms": 1795706996132, "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "die_with_parent": false, "identity": "ghostty", "keep": false, "labels": {"by": "T1", "fleet": "temet-work-re …
17:29:56.137 +  60.996 oneoff !! reader ended: EOF
17:29:57.138 +  61.997 req <     id=60 terminal.get 0.3ms {"result": {"address": "t1sy", "argv": ["sleep", "300"], "cols": 200, "created_ms": 1795706996132, "cwd": "/mnt/data/projects/temet/temet-work/spikes/herdr-core", "die_with_parent": false, "foreground": {"argv": ["sleep", "300"], "cwd": "/mnt/data/projects/temet/temet-wor …
```

A child ended when its parent was killed:

```text
17:29:57.444 +  62.303 STATE     t1sz finished {"exit_code": null, "signal": 1, "reason": "killed"}
17:29:57.449 +  62.308 STATE     t1t0 finished {"exit_code": null, "signal": 1, "reason": "killed"}
17:29:57.632 +  62.491 ##        parent kill -> child ended 0.197s
```

```text
17:31:00.760 + 125.619 req <     id=6271 terminal.rm 0.1ms {"error": {"code": -32000, "data": {"hint": "kill it first: hc kill t1st", "kind": "live"}, "message": "t1st is still running"}}
17:31:00.771 + 125.630 req <     id=6274 terminal.send 0.2ms {"error": {"code": -32000, "data": {"hint": "hc read t1st", "kind": "finished"}, "message": "t1st has finished"}}
17:31:00.759 + 125.618 req <     id=6270 terminal.get 0.1ms {"error": {"code": -32000, "data": {"hint": "see `hc ls --all`", "kind": "not_found"}, "message": "no terminal \"t999999\""}}
17:31:01.809 + 126.668 STATE     t1sv finished {"exit_code": null, "signal": 9, "reason": "killed"}
17:31:01.813 + 126.672 STATE     t1sv removed {}
```

Errors are `{code: -32000, message, data: {kind, hint}}`; kinds seen were `status_mismatch`,
`blocked`, `conflict`, `live`, `finished` and `not_found` (and `invalid_params`, code -32602, in the
first attempt). A finished terminal could still be read.

## When the Herdr window closes

**Read, not run.** Dawid's window was not closed and the core was not stopped.

- `~/.config/herdr-app/AGENTS.md`, "Two programs": herdr-core "runs every terminal and agent. They
  keep running when the window closes or updates."
- `hc --help`: "run programs and coding agents in terminals that stay alive"; `server stop` "parks
  kept terminals; next core start gives them shells".
- `hc --skill`, `hc keep`: it "replaces a lost program with a shell, never reruns a task". AGENTS.md
  says the same for a core crash, reboot or logout, with the same address.

So closing the window should not touch a daemon's agents. A core crash or reboot ends the agent's
program, and the daemon would have to start the session again itself.

## For temet-work

### Daemon

Build on:
- One subscription filtered by the daemon's own labels, as the source of agent state (How agent state shows up).
- Labels for item and role, set in `terminal.run` so the first event already has them (Following).
- `blocked` with `kind` and `msg` as the "needs you" signal, with `send` refusing to type into it (Waiting for a person).
- `send` then `wait` from the same `caller` for one turn (Sending).
- `doc.set` with versions for small shared state (Documents).
- No `caller` for agents that must outlive a daemon restart, or a parent terminal for agents that must end together (Ending). With no parent, the daemon has to find and remove its own by label.

Avoid:
- One connection for everything: give each wait its own (Waiting).
- `if: "idle"` as "ready for the next task"; treat `idle` and `done` alike (Sending).
- Sending after an interrupt without reading the input box first (Interrupting).
- Reading history on every state change: 3 s each on an idle agent (Reading).
- Doing slow work in the subscription's read loop; handle `missed` and a closed connection by subscribing again and re-reading documents (A reader that falls behind).
- `terminal.wait output` after the send that causes the output (Waiting).

Still unknown:
- Permission prompts and the `error` status (How agent state shows up).
- Whether a `send` right after a launch is dropped; the probe put the first task in `argv`, as `hc --skill` advises (Starting).
- Behaviour across a core restart or update, and on other machines (Verdict table, not exercised).
- Delivery when a person is typing in the same terminal: `hc --skill` says sends wait up to 60 s (read, not run).

### App

Build on:
- `herdr.space` and `herdr.tab`, set in `terminal.run`, to put a terminal on the app's screens; the app does not show a terminal without them (Labels and listing).
- `status.msg` and `title` events for a one-line "what it is doing" per item, without reading screens (How agent state shows up).
- "Open in Herdr" for the terminal itself, since terminals outlive windows (When the Herdr window closes, read only).

Avoid:
- Treating `where` as fixed: the app writes it, and changes it as tabs move (Labels and listing).
- Showing agent text from `terminal.read` while it works: only the visible screen comes back, marked incomplete (Reading).

Still unknown:
- `terminal.attach` for drawing a terminal in the app was not tried.
