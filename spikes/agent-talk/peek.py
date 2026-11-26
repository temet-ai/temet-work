#!/usr/bin/env python3
"""peek.py <run-name>: print every transcript record of a run's sessions with its UTC time."""
import json, sys
from pathlib import Path
import run

for role, v in json.loads((run.HERE / "runs" / sys.argv[1] / "state.json").read_text()).items():
    print("=====", role, v["session"], v["terminal"])
    for line in run.transcript(v["session"]).splitlines():
        d = json.loads(line)
        t, ts = d.get("type"), d.get("timestamp", "")[11:23]
        c = (d.get("message") or {}).get("content")
        if t == "assistant":
            for x in c:
                if x["type"] == "tool_use":
                    print(ts, "tool", x["name"], json.dumps(x["input"])[:200])
                elif x["type"] == "text":
                    print(ts, "says", x["text"][:300].replace("\n", " "))
        elif t == "user":
            for x in [{"type": "text", "text": c}] if isinstance(c, str) else c:
                body = x.get("text") or str(x.get("content"))
                print(ts, "got " if x["type"] == "text" else "result", body[:260].replace("\n", " "))
        elif t in ("attachment", "queue-operation", "system"):
            s = json.dumps(d)
            if any(k in s for k in ("probe message", "mike-mail", "<hc ")) and "api-request" not in t:
                print(ts, t, (d.get("attachment") or {}).get("type") or d.get("operation") or d.get("subtype"), s[:300])
