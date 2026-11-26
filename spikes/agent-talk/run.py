#!/usr/bin/env python3
"""Start arbiter, author and reviewer on haiku, let them settle the fixture dispute.

  run.py start <run-name> mail|hc    start the three, wait for the ruling to reach both
  run.py close <run-name>            kill and remove exactly the sessions that run started

Run from inside a herdr-core terminal on a machine with mike. Output: runs/<run-name>/.
"""
import json, os, subprocess, sys, time, uuid
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
FLEET = "temet-work-research"
BY = os.environ.get("MIKE_SESSION_ID", "manual")[:8]
TRANSCRIPTS = Path.home() / ".claude/projects" / str(REPO).replace("/", "-")
ALLOWED = ("mcp__mike__send_session_message,mcp__mike__ack_session_message,"
           "mcp__mike__get_session_messages,Bash(hc send:*),Read")


def now():
    return datetime.now().astimezone().isoformat(timespec="milliseconds")


def sh(*args, check=True):
    r = subprocess.run(args, capture_output=True, text=True)
    if check and r.returncode:
        raise SystemExit(f"{args}: exit {r.returncode}\n{r.stdout}\n{r.stderr}")
    return r


def log(out, event, **data):
    line = json.dumps({"at": now(), "event": event, **data})
    print(line, flush=True)
    with open(out / "run.jsonl", "a") as f:
        f.write(line + "\n")


def save(out, state):
    (out / "state.json").write_text(json.dumps(state, indent=2) + "\n")


def spawn(out, state, role, channel):
    prefix = "M" if channel == "mail" else "H"
    values = {
        "ROLE": role,
        "PEERS": " and the ".join(r for r in ("author", "reviewer", "arbiter") if r != role),
        "P": prefix,
        "ADDRKIND": "mike session id" if channel == "mail" else "terminal address",
    }
    for peer, s in state.items():
        values[peer.upper()] = s["session"] if channel == "mail" else s["terminal"]
    prompt = "".join((HERE / "prompts" / f"{n}.txt").read_text()
                     for n in ("common", channel, role))
    for k, v in values.items():
        prompt = prompt.replace("{" + k + "}", v)
    prompt_file = out / f"prompt-{role}.txt"
    prompt_file.write_text(prompt)
    open_session(out, state, role, prompt_file,
                 ["--allowedTools", ALLOWED,
                  "--disallowedTools", "Edit,Write,NotebookEdit,Agent,AskUserQuestion"])


def open_session(out, state, role, prompt_file, claude_args):
    """mike spawn open, then label the terminal it made as ours."""
    t0 = time.time()
    log(out, "spawn_called", role=role)
    r = sh("mike", "spawn", "open", str(REPO), "--session-name", f"{role}-{out.name}",
           "--role", role, "--goal", f"T4 dispute test: {role}",
           "--prompt-file", str(prompt_file), "--json",
           "--", "--model", "haiku", *claude_args, check=False)
    log(out, "spawn_returned", role=role, exit=r.returncode,
        seconds=round(time.time() - t0, 2), stdout=r.stdout.strip(), stderr=r.stderr.strip())
    res = json.loads(r.stdout)
    session = res.get("session") or res.get("session_id")
    found = []
    for _ in range(60):
        ls = json.loads(sh("hc", "ls", "--all", "--json", "--label", f"mike.session={session}").stdout)
        found = ls["terminals"]
        if found:
            break
        time.sleep(0.5)
    terminal = found[0]["address"]
    sh("hc", "label", terminal, f"fleet={FLEET}", f"by={BY}", f"role={role}", f"item={out.name}")
    log(out, "labelled", role=role, terminal=terminal, session=session,
        seconds_after_spawn_call=round(time.time() - t0, 2))
    state[role] = {"session": session, "terminal": terminal}
    save(out, state)




def status(terminal):
    d = json.loads(sh("hc", "get", terminal, "--json").stdout)
    st = d.get("terminal", d).get("status")
    return st.get("state") if isinstance(st, dict) else st


def transcript(session):
    p = TRANSCRIPTS / f"{session}.jsonl"
    return p.read_text() if p.exists() else ""


def sent(session, mid):
    """True when this session called a send whose text starts with the id."""
    for line in transcript(session).splitlines():
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("type") != "assistant":
            continue
        for c in d["message"].get("content", []):
            if c.get("type") != "tool_use":
                continue
            i = c.get("input", {})
            if str(i.get("message", "")).startswith(mid + " "):
                return True
            cmd = str(i.get("command", ""))
            if "hc send" in cmd and (f"'{mid} " in cmd or f'"{mid} ' in cmd):
                return True
    return False


def wait_status(out, state, roles, wanted, timeout):
    end = time.time() + timeout
    while time.time() < end:
        got = {r: status(state[r]["terminal"]) for r in roles}
        if all(v in wanted for v in got.values()):
            log(out, "status", **got)
            return True
        time.sleep(1)
    log(out, "status_timeout", **got)
    return False


def start(name, channel):
    out = HERE / "runs" / name
    out.mkdir(parents=True)
    state = {}
    events = subprocess.Popen(
        [sys.executable, str(HERE / "events.py"), BY, name, str(out / "events.log")],
        start_new_session=True)
    try:
        spawn(out, state, "arbiter", channel)
        spawn(out, state, "author", channel)
        wait_status(out, state, ["arbiter", "author"], {"done", "idle"}, 120)
        spawn(out, state, "reviewer", channel)
        p = "M" if channel == "mail" else "H"
        end = time.time() + 420
        seen = set()
        want = [("reviewer", 1), ("author", 2), ("reviewer", 3), ("author", 4),
                ("arbiter", 5), ("arbiter", 6)]
        while time.time() < end and len(seen) < len(want):
            for role, n in want:
                if n not in seen and sent(state[role]["session"], f"{p}{n}"):
                    seen.add(n)
                    log(out, "send_in_transcript", id=f"{p}{n}", role=role)
            time.sleep(1)
        log(out, "exchange_over", seen=sorted(seen), complete=len(seen) == len(want))
        wait_status(out, state, list(state), {"done", "idle", "blocked", "error"}, 120)
        time.sleep(90)  # a window for late or repeated deliveries
        for role, s in state.items():
            log(out, "final", role=role, status=status(s["terminal"]))
    finally:
        os.killpg(events.pid, 15)


def close(name):
    out = HERE / "runs" / name
    state = json.loads((out / "state.json").read_text())
    for role, s in state.items():
        r = sh("mike", "spawn", "kill", "--session", s["session"], "--json", check=False)
        log(out, "killed", role=role, exit=r.returncode, output=r.stdout.strip())
    own = REPO / ".claude/skills/hc-terminals/scripts/own.sh"
    r = sh(str(own), "clean", f"fleet={FLEET}", f"by={BY}", f"item={name}", check=False)
    log(out, "clean", exit=r.returncode, output=(r.stdout + r.stderr).strip())


if __name__ == "__main__":
    if sys.argv[1:2] == ["start"] and len(sys.argv) == 4 and sys.argv[3] in ("mail", "hc"):
        start(sys.argv[2], sys.argv[3])
    elif sys.argv[1:2] == ["close"] and len(sys.argv) == 3:
        close(sys.argv[2])
    else:
        raise SystemExit(__doc__)
