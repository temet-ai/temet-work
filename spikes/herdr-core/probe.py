#!/usr/bin/env python3
"""Probe what herdr-core gives a daemon, over its JSON-RPC socket. Standard library only.

Run:  python3 spikes/herdr-core/probe.py        (writes run.log beside itself)

It starts only terminals labelled fleet=temet-work-research by=T1, and kills and
removes every one of them in a `finally` block.
"""
import json, os, socket, sys, threading, time, uuid, queue, datetime

HERE = os.path.dirname(os.path.abspath(__file__))
SOCK = os.environ.get("HC_TERMINAL_SOCKET") or f"/run/user/{os.getuid()}/herdr-core/default.sock"
ME = os.environ.get("HC_TERMINAL_ID")          # parent of what we start, so it dies with us
OWN = {"fleet": "temet-work-research", "by": "T1"}
DOC = "temet-work.t1.probe"          # names: 1-32 of a-z 0-9 - _ .
FLOOD = "temet-work.t1.flood"
T0 = time.monotonic()
LOG = open(os.path.join(HERE, "run.log"), "w", buffering=1)
LOCK = threading.Lock()


def short(v, limit=300):
    if isinstance(v, str):
        return v if len(v) <= limit else v[: limit // 2] + f"...[{len(v)} chars]..." + v[-limit // 2:]
    if isinstance(v, list):
        return [short(x, limit) for x in v[:20]] + ([f"...{len(v) - 20} more"] if len(v) > 20 else [])
    if isinstance(v, dict):
        return {k: ("<redacted>" if k == "env" else short(x, limit)) for k, x in v.items()
                if x not in (None, "", {}, [])}
    return v


def log(tag, text):
    wall = datetime.datetime.now().strftime("%H:%M:%S.%f")[:-3]   # local time (SAST)
    with LOCK:
        line = f"{wall} +{time.monotonic() - T0:8.3f} {tag:<9} {text}"
        LOG.write(line + "\n")
        print(line[:400])


def note(text):
    log("##", text)


class Conn:
    """One socket. A reader thread matches replies to requests and queues notifications."""

    def __init__(self, tag, read=True, quiet=False):
        self.quiet, self.eof_at = quiet, None
        self.tag, self.n, self.pending, self.events = tag, 0, {}, queue.Queue()
        self.s = socket.socket(socket.AF_UNIX)
        self.s.connect(SOCK)
        self.f = self.s.makefile("rb")
        self.reading = threading.Event()
        if read:
            self.reading.set()
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        self.reading.wait()
        try:
            for raw in self.f:
                got, mono = json.loads(raw), time.monotonic()
                if "id" in got and got["id"] in self.pending:
                    slot = self.pending.pop(got["id"])
                    slot["reply"], slot["at"] = got, mono
                    ms = (mono - slot["sent"]) * 1000
                    if not slot.get("quiet"):
                        log(self.tag + " <", f"id={got['id']} {slot['method']} {ms:.1f}ms "
                        + json.dumps(short({k: v for k, v in got.items() if k in ("result", "error")}))[:1400])
                    slot["done"].set()
                else:
                    if not self.quiet:
                        log(self.tag + " <<", f"{len(raw)}B " + json.dumps(short(got, 120))[:900])
                    self.events.put((mono, got))
        except (OSError, ValueError) as e:
            log(self.tag + " !!", f"reader ended: {e}")
        else:
            self.eof_at = time.monotonic()
            log(self.tag + " !!", "reader ended: EOF")

    def start(self, method, params=None, quiet=False):
        self.n += 1
        slot = {"method": method, "done": threading.Event(), "sent": time.monotonic()}
        self.pending[self.n] = slot
        msg = {"jsonrpc": "2.0", "id": self.n, "method": method, "params": params or {}}
        if not quiet:
            log(self.tag + " >", f"id={self.n} {method} " + json.dumps(short(params or {}))[:600])
        self.s.sendall((json.dumps(msg) + "\n").encode())
        return slot

    def finish(self, slot, timeout=200):
        if not slot["done"].wait(timeout):
            raise TimeoutError(slot["method"])
        return slot["reply"]

    def call(self, method, params=None, timeout=200):
        """The whole reply: {"result":...} or {"error":...}."""
        return self.finish(self.start(method, params), timeout)

    def ok(self, method, params=None, timeout=200):
        reply = self.call(method, params, timeout)
        if "error" in reply:
            raise RuntimeError(f"{method}: {reply['error']}")
        return reply["result"]

    def flood(self, name, version, count, pad):
        """`count` doc.set calls one after another, unlogged; returns the last version."""
        for i in range(count):
            self.n += 1
            slot = {"method": "doc.set", "done": threading.Event(), "sent": time.monotonic(), "quiet": True}
            self.pending[self.n] = slot
            self.s.sendall((json.dumps({"jsonrpc": "2.0", "id": self.n, "method": "doc.set", "params":
                            {"name": name, "value": {"n": i, "pad": pad}, "version": version}}) + "\n").encode())
            version = self.finish(slot)["result"]["version"]
        return version

    def close(self):
        try:
            self.s.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass


def hello(tag, read=True, quiet=False):
    c = Conn(tag, read, quiet)
    if read:
        c.ok("hello")
    else:                      # send hello but do not read the answer yet
        c.start("hello")
    return c


class Watch:
    """Latest facts per terminal, built from one subscription."""

    def __init__(self, conn):
        self.conn, self.history = conn, []
        self.cond = threading.Condition()
        threading.Thread(target=self._run, daemon=True).start()

    def _run(self):
        while True:
            mono, msg = self.conn.events.get()
            p = msg.get("params", {})      # event: {type, terminal: address, seq, data}
            if msg.get("method") != "event" or p.get("type") not in ("created", "status", "turn", "finished", "lost", "removed"):
                continue
            addr, kind, data = p.get("terminal"), p["type"], p.get("data") or {}
            with self.cond:
                st = data.get("status") or {}
                if kind == "status":
                    word, extra = st.get("state"), f" kind={st.get('kind')} msg={st.get('msg')!r}"
                    self.history.append((mono, addr, "status", word))
                    log("STATE", f"{addr} status={word}{extra}")
                else:
                    log("STATE", f"{addr} {kind} " + json.dumps({k: data[k] for k in ("state", "turn", "exit_code", "signal", "reason") if k in data}))
                    self.history.append((mono, addr, kind, data.get("state")))
                self.cond.notify_all()

    def until(self, addr, words, since, timeout=90):
        """First status in `words` seen for addr after monotonic time `since`."""
        end = time.monotonic() + timeout
        with self.cond:
            while True:
                for h in self.history:
                    if h[0] >= since and h[1] == addr and h[2] == "status" and h[3] in words:
                        return h
                left = end - time.monotonic()
                if left <= 0:
                    note(f"TIMEOUT waiting for {addr} to report {words}")
                    return None
                self.cond.wait(left)


def mine(conn):
    return conn.ok("terminal.list", {"all": True, "labels": OWN})["terminals"]


def cleanup(conn):
    note("cleanup: kill and remove every terminal labelled " + json.dumps(OWN))
    for t in mine(conn):
        if t["state"] == "live":
            conn.call("terminal.kill", {"terminal": t["address"], "caller": ME})
        conn.call("terminal.rm", {"terminal": t["address"], "caller": ME})
    for name in (DOC, FLOOD):
        got = conn.ok("doc.get", {"name": name})
        if got.get("version"):
            conn.call("doc.set", {"name": name, "value": None, "version": got["version"]})
    note(f"left: {len(mine(conn))}")


def run(c, watch_conn, watch):
    prompt = open(os.path.join(HERE, "prompt.txt")).read().strip()

    def start(item, role, **kw):
        params = {"labels": dict(OWN, item=item, role=role), "caller": ME, "no_notify": True, "cwd": HERE}
        params.update(kw)
        return c.ok("terminal.run", {k: v for k, v in params.items() if v is not None})

    note("1. identity")
    c.ok("server.status")
    c.ok("terminal.list", {"all": True, "labels": OWN})

    note("2. subscribe, on its own connection, to our label only")
    watch_conn.ok("events.subscribe", {"labels": OWN})

    note("3. a plain shell and a Claude Code agent on haiku")
    shell = start("2", "shell", argv=["bash", "--noprofile", "--norc"], size={"cols": 100, "rows": 30})["address"]
    env = {k: os.environ[k] for k in ("HTTPS_PROXY", "NO_PROXY", "NODE_EXTRA_CA_CERTS") if k in os.environ}
    sid = str(uuid.uuid4())
    note(f"agent mike session id {sid}; env keys passed: {sorted(env)}")
    launched = time.monotonic()
    agent = start("1", "agent", name="t1-probe-agent", env=env, size={"cols": 120, "rows": 40},
                  argv=["claude", "--model", "haiku", "--session-id", sid, prompt])["address"]

    note("4. launch: first status reports, a read while working, wait for the turn to end")
    h = watch.until(agent, ("working",), launched, 60)
    if h:
        note(f"launch -> first 'working' {h[0] - launched:.3f}s")
        c.ok("terminal.read", {"terminal": agent, "lines": 100, "caller": ME})
    c.ok("terminal.wait", {"terminal": agent, "timeout": 120, "caller": ME})
    note(f"launch -> wait returned {time.monotonic() - launched:.3f}s")
    got = c.ok("terminal.read", {"terminal": agent, "lines": 100, "caller": ME})
    note("SCREEN after first turn:\n" + got["text"].rstrip())
    c.ok("terminal.get", {"terminal": agent, "caller": ME})
    c.ok("terminal.label", {"terminal": agent, "labels": {"stage": "probe"}})
    c.ok("terminal.list", {"labels": dict(OWN, role="agent")})

    note("5. send with if=idle (the status after an unseen turn may be done, not idle), then send and wait")
    c.call("terminal.send", {"terminal": agent, "text": "Reply with exactly: PONG-1", "if": "idle", "wait": True, "timeout": 90, "caller": ME})
    sent = time.monotonic()
    c.call("terminal.send", {"terminal": agent, "text": "Reply with exactly: PONG-2", "caller": ME})
    note(f"send returned after {time.monotonic() - sent:.3f}s; calling terminal.wait at once")
    c.ok("terminal.wait", {"terminal": agent, "timeout": 90, "caller": ME})
    note(f"send request -> wait returned {time.monotonic() - sent:.3f}s")
    for word in ("working", "done"):
        h = watch.until(agent, (word,), sent, 10)
        if h:
            note(f"send request -> '{word}' event {h[0] - sent:.3f}s")
    got = c.ok("terminal.read", {"terminal": agent, "lines": 30, "caller": ME})
    note("SCREEN after PONG:\n" + got["text"].rstrip()[-1500:])
    sent = time.monotonic()
    c.call("terminal.send", {"terminal": agent, "text": "Reply with exactly: PONG-3", "wait": True, "timeout": 90, "caller": ME})
    note(f"send wait=true returned after {time.monotonic() - sent:.3f}s")

    note("6. while working: send if=idle, then interrupt")
    sent = time.monotonic()
    c.call("terminal.send", {"terminal": agent, "text": "Write the numbers from 1 to 600 in English words, one per line. No tools.", "caller": ME})
    if watch.until(agent, ("working",), sent, 30):
        c.call("terminal.send", {"terminal": agent, "text": "SHOULD-NOT-ARRIVE", "if": "idle", "caller": ME})
        c.call("terminal.send", {"terminal": agent, "text": "Afterwards also reply QUEUED-1.", "if": "working", "caller": ME})
        c.ok("terminal.read", {"terminal": agent, "lines": 100, "caller": ME})
        c.call("terminal.interrupt", {"terminal": agent, "timeout": 20})
        c.call("terminal.wait", {"terminal": agent, "timeout": 30, "caller": ME})
        c.ok("terminal.get", {"terminal": agent})

    def tail(label, n=900):
        got = c.ok("terminal.read", {"terminal": agent, "lines": 20, "caller": ME})
        note(f"SCREEN {label}:\n" + got["text"].rstrip()[-n:])

    def long_then_interrupt():
        sent = time.monotonic()
        c.call("terminal.send", {"terminal": agent, "text": "Write the numbers from 1 to 600 in English words, one per line. No tools.", "caller": ME})
        watch.until(agent, ("working",), sent, 30)
        c.call("terminal.interrupt", {"terminal": agent, "timeout": 20})
        time.sleep(1)    # the screen redraws after the status changes
        c.call("terminal.wait", {"terminal": agent, "timeout": 60, "caller": ME})

    note("6b. interrupt, then simply send the next message")
    long_then_interrupt()
    tail("after interrupt (look at the input box)")
    c.call("terminal.send", {"terminal": agent, "text": "Reply with exactly: NEXT-1", "wait": True, "timeout": 90, "caller": ME})
    tail("after the next send")

    note("6c. interrupt, clear the input box with ctrl+c, then send")
    long_then_interrupt()
    c.call("terminal.keys", {"terminal": agent, "keys": ["ctrl+c"], "caller": ME})
    tail("after ctrl+c")
    c.call("terminal.send", {"terminal": agent, "text": "Reply with exactly: NEXT-2", "wait": True, "timeout": 90, "caller": ME})
    tail("after the next send")

    note("7. waiting for a person: the agent asks a question")
    sent = time.monotonic()
    c.call("terminal.send", {"terminal": agent, "text": "Use the AskUserQuestion tool now to ask me 'Continue?' with options Yes and No.", "caller": ME})
    h = watch.until(agent, ("blocked",), sent, 60)
    if h:
        note(f"send -> 'blocked' event {h[0] - sent:.3f}s")
    c.call("terminal.wait", {"terminal": agent, "timeout": 30, "caller": ME})
    c.ok("terminal.get", {"terminal": agent})
    got = c.ok("terminal.read", {"terminal": agent, "lines": 100, "caller": ME})
    note("SCREEN while asking:\n" + got["text"].rstrip()[-1500:])
    if h:
        c.call("terminal.send", {"terminal": agent, "text": "SHOULD-BE-REFUSED", "caller": ME})
        pressed = time.monotonic()
        c.call("terminal.keys", {"terminal": agent, "keys": ["esc"], "caller": ME})
        watch.until(agent, ("idle", "done"), pressed, 30)
        c.ok("terminal.get", {"terminal": agent})

    note("8. plain shell: no status; wait on output; one connection with two requests in flight")
    c.ok("terminal.get", {"terminal": shell})
    c.call("terminal.send", {"terminal": shell, "text": "echo late-$((40+2))", "caller": ME})
    c.call("terminal.wait", {"terminal": shell, "output": "late-42", "timeout": 5, "caller": ME})
    note("same again, wait first on a second connection, then send")
    w = hello("wait")
    slot = w.start("terminal.wait", {"terminal": shell, "output": "early-42", "timeout": 20, "caller": ME})
    time.sleep(0.5)   # let the core register the wait; there is no acknowledgement for it
    c.call("terminal.send", {"terminal": shell, "text": "echo early-$((40+2))", "caller": ME})
    w.finish(slot)
    note("pipelining: terminal.wait then server.status on the same connection")
    slow = w.start("terminal.wait", {"terminal": shell, "timeout": 4, "caller": ME})
    fast = w.start("server.status")
    w.finish(fast)
    w.finish(slow)
    note(f"server.status answered {'before' if fast['at'] < slow['at'] else 'after'} the earlier terminal.wait")
    w.close()
    c.call("terminal.send", {"terminal": shell, "text": "sleep 30; echo slept", "caller": ME})
    c.call("terminal.keys", {"terminal": shell, "keys": ["ctrl+c"], "caller": ME})
    c.call("terminal.send", {"terminal": shell, "text": "echo alive-after-ctrl-c; sleep 30", "caller": ME})
    c.call("terminal.interrupt", {"terminal": shell, "timeout": 10})
    c.ok("terminal.read", {"terminal": shell, "lines": 20, "caller": ME})

    note("9. documents: versioned write, stale write, delete")
    v = c.ok("doc.get", {"name": DOC})["version"]
    c.call("doc.set", {"name": DOC, "version": v, "value": {"agent": agent, "shell": shell, "session": sid}})
    c.call("doc.set", {"name": DOC, "version": v, "value": {"stale": True}})
    now = c.ok("doc.get", {"name": DOC})["version"]
    c.call("doc.set", {"name": DOC, "version": now, "value": None})
    c.call("doc.get", {"name": DOC})

    note("10. who ends a terminal: client disconnect, parent end")
    other = hello("oneoff")
    base = {"argv": ["sleep", "300"], "cwd": HERE, "no_notify": True}
    orphan = other.ok("terminal.run", dict(base, labels=dict(OWN, item="3", role="no-caller")))["address"]
    other.close()
    time.sleep(1)     # nothing announces a client going away; give the core a moment
    c.ok("terminal.get", {"terminal": orphan})
    parent = start("4", "parent", argv=["sleep", "300"])["address"]
    child = start("5", "child", argv=["sleep", "300"], caller=parent)["address"]
    c.ok("terminal.get", {"terminal": child})
    killed = time.monotonic()
    c.call("terminal.kill", {"terminal": parent, "caller": ME})
    c.call("terminal.wait", {"terminal": child, "timeout": 20, "caller": ME})
    note(f"parent kill -> child ended {time.monotonic() - killed:.3f}s")
    c.ok("terminal.get", {"terminal": child})

    note("11. a subscriber that stops reading (documents only, so nobody else is affected)")
    began = time.monotonic()
    for count, idle in ((200, 0), (3000, 0), (3000, 40)):
        lazy = hello("lazy", read=False, quiet=True)
        lazy.start("events.subscribe", {"docs": [FLOOD]})
        time.sleep(0.5)
        t = time.monotonic()
        last = c.flood(FLOOD, c.ok("doc.get", {"name": FLOOD})["version"], count, "")
        note(f"{count} doc.set while 'lazy' was not reading took {time.monotonic() - t:.1f}s; it stays unread {idle}s more")
        time.sleep(idle)
        lazy.reading.set()
        time.sleep(3)     # drain what is buffered
        closed_by_core = lazy.eof_at is not None
        docs, kinds, other = 0, {}, []
        while not lazy.events.empty():
            m = lazy.events.get()[1]
            p = m.get("params", {})
            key = m.get("method") + (":" + p["type"] if "type" in p else "")
            kinds[key] = kinds.get(key, 0) + 1
            if m.get("method") not in ("event", "heartbeat"):
                other.append(m)
        note(f"lazy got {kinds} of {count} changes; connection closed by the core: {closed_by_core}")
        for m in other[:2]:
            note("lazy got: " + json.dumps(m)[:300])
        lazy.close()

    note("12. errors, kill, remove")
    c.call("terminal.get", {"terminal": "t999999"})
    c.call("terminal.rm", {"terminal": shell, "caller": ME})
    c.call("terminal.kill", {"terminal": shell, "caller": ME})
    c.call("terminal.read", {"terminal": shell, "lines": 5, "caller": ME})
    c.call("terminal.send", {"terminal": shell, "text": "echo no", "caller": ME})
    c.call("terminal.kill", {"terminal": agent, "caller": ME})
    c.call("terminal.rm", {"terminal": agent, "caller": ME})
    c.call("terminal.get", {"terminal": agent})

    note("13. a heartbeat, and a fresh subscription as a daemon would make after a restart")
    time.sleep(max(0, 16 - (time.monotonic() - began)))
    again = hello("resub")
    again.ok("events.subscribe", {"labels": OWN})
    again.close()


def main():
    note(f"socket {SOCK}; caller {ME}; {datetime.date.today()} (times are local, SAST)")
    c = hello("req")
    if mine(c):
        sys.exit("terminals labelled by=T1 already exist; clean them first")
    ev = hello("events")
    watch = Watch(ev)
    try:
        run(c, ev, watch)
    finally:
        cleanup(c)


if __name__ == "__main__":
    main()
