#!/usr/bin/env python3
"""One haiku session per delivery case that the dispute runs do not reach.

  probe.py busy <name>      working in a 45 s command; two hc sends arrive meanwhile
  probe.py booting <name>   an hc send the moment `mike spawn open` returns
  probe.py blocked <name>   held at a permission dialog; hc send is tried against it
Mail for the same cases is sent by the operating session with send_session_message.
Close with: run.py close <name>
"""
import json, sys, time
import run

CASES = {
    "busy": ("Run this exact Bash command in the foreground: python3 -c 'import time; time.sleep(45)'. "
             "After it ends, list the ids (like P1) of every message you have received, in the order "
             "they reached you, then say DONE. Never edit files or send anything.",
             ["--allowedTools", "Bash(python3 -c:*)"]),
    "booting": ("Reply READY. If a message with an id like B1 reaches you, reply with its id. "
                "Never edit files or send anything.", []),
    "blocked": ("Run this exact Bash command: touch /var/tmp/agent-talk-probe-never-created. "
                "Do nothing else. If a message with an id reaches you, reply with its id.",
                ["--permission-mode", "default"]),
}

if __name__ == "__main__":
    case, name = sys.argv[1], sys.argv[2]
    prompt, args = CASES[case]
    out = run.HERE / "runs" / name
    out.mkdir(parents=True)
    (out / "prompt.txt").write_text(prompt + "\n")
    state = {}
    run.open_session(out, state, case, out / "prompt.txt", args)
    t = state[case]["terminal"]

    def send(mid, *extra):
        r = run.sh("hc", "send", t, f"{mid} probe message", "--json", *extra, check=False)
        run.log(out, "hc_send", id=mid, status_before=before, exit=r.returncode,
                output=(r.stdout + r.stderr).strip())

    if case == "booting":
        before = run.status(t)
        send("B1")
    else:
        wanted = "working" if case == "busy" else "blocked"
        end = time.time() + 60
        while time.time() < end and run.status(t) != wanted:
            time.sleep(0.5)
        time.sleep(5)
        before = run.status(t)
        run.log(out, "state", status=before)
        send("P1" if case == "busy" else "K1")
        if case == "busy":
            time.sleep(2)
            send("P2", "--if", "idle")
            send("P3")
