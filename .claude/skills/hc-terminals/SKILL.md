---
name: hc-terminals
description: Start, label, talk to, wait on and clean up herdr-core terminals on this machine, including Claude Code sessions on haiku that mike captures. Use whenever you run `hc run`, `hc send`, `hc wait`, `hc read`, `hc kill`/`hc rm`, or call the hc socket from code, and before you start any agent in a terminal, even for a quick test. Dawid's own terminals are live in the same core.
---

# herdr-core terminals

The core is shared with Dawid's live work (`hc ls` shows his terminals too). Everything
below keeps you on your own terminals.

## Label every terminal you start

Give each one `fleet=<fleet slug>` (when in a fleet), `by=<your id>` and `role=<r>` (or
`item=<n>`). `by` is what makes it yours: other seats share the fleet label. Use your mike
session id's first 8 characters, or `$HC_TERMINAL_ID`. Put labels in the `run` call, not
in a later `hc label`, so no event or listing ever shows the terminal without them.

## Start a Claude Code session on haiku

```sh
SID=$(uuidgen)
hc run --bg --json --name <role>-<by> --cwd <dir> \
  --label fleet=<slug> --label by=<by> --label role=<role> \
  -- env HTTPS_PROXY="$HTTPS_PROXY" NO_PROXY="$NO_PROXY" NODE_EXTRA_CA_CERTS="$NODE_EXTRA_CA_CERTS" \
     claude --model haiku --session-id "$SID" "<first task>"
```

- Terminals get the core's environment, not yours, so pass the proxy variables
  explicitly or mike does not capture the session; check all three are set in your shell
  first. mike's own launcher passes the same three
  (`temet-proxy/crates/mike-launch/src/launch/env.rs`). `--session-id` gives you the
  mike session id up front.
- Put the first task in `run`: a send to a booting agent can be dropped.
- A prompt in argv reaches Claude without hc typing it. If the status stays `blocked`,
  `hc read <t>`: a folder-trust dialog needs `hc keys <t> down enter`. Before sending
  anything again, read: it may have arrived.
- Children end when your program ends. Do not add `--outlive-parent`.

When the session must receive mike mail, start it with mike's `spawn_session` instead
(`args: ["--model","haiku"]`, `name`, `role`, `prompt`): it turns on Remote Control and
the mike channel, without which an idle session never wakes to read its mail. It labels
the terminal `mike.session=<id>`; find it with `hc ls --json --label mike.session=<id>` and add your labels at once with
`hc label <t> fleet=<slug> by=<by> role=<role>`. Close it with `kill_session`, then `hc rm`.

## Talk and wait

- `hc send <t> "<text>"` pastes and presses Enter; `--if idle` sends only when idle,
  `--wait` waits for the turn it started. Reports come back as `<hc from="t7 name">…</hc>`.
- `hc wait <t>` until it stops working; `reason: timeout` (180 s) is a check-in, wait again.
  Never `sleep` to wait on an hc terminal.
- Status (`idle`, `working`, `done`, `blocked`, `error`) comes from the program's own
  reports; never infer it from screen text. `blocked` means a person is needed: read the
  screen and report; never `send --force` past a permission dialog.
- `hc read <t> --lines 100` reads screen and history; while `working` it returns only the
  screen, marked incomplete.

## From code: the socket

`$HC_TERMINAL_SOCKET` (else `/run/user/$UID/herdr-core/default.sock`) speaks JSON-RPC 2.0,
one JSON object per line, unframed until `hello` with `framed: true`. Send `hello` first.
`hc schema` is the OpenRPC document: read each method's params there, not from memory.
`events.subscribe` takes `labels`: subscribe to your own `by=` label so you never see
or act on other terminals. Errors on the CLI are `{error: {kind, message, hint}}` with exit
codes 3 not_found, 9 blocked, 10 status_mismatch, 11 finished.

## Clean up, and only yours

```sh
${CLAUDE_SKILL_DIR}/scripts/own.sh ls    fleet=<slug> by=<by>
${CLAUDE_SKILL_DIR}/scripts/own.sh clean fleet=<slug> by=<by>   # kill + rm, exits 1 if any remain
```

It refuses a label set without a label other than `fleet=`. Quote its `left: 0` line as
proof of cleanup. Never `hc server stop` or `restart`, never close the Herdr window: both
reach every terminal Dawid has open.
