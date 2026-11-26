#!/usr/bin/env python3
"""events.py <by> <item> <file>: log each hc event on those labels with the time it arrived."""
import json, subprocess, sys
from datetime import datetime

by, item, path = sys.argv[1:4]
p = subprocess.Popen(["hc", "events", "--json", "--label", f"by={by}", "--label", f"item={item}"],
                     stdout=subprocess.PIPE, text=True)
with open(path, "a", buffering=1) as f:
    for line in p.stdout:
        at = datetime.now().astimezone().isoformat(timespec="milliseconds")
        params = json.loads(line).get("params")
        if not params:
            continue
        data = params.get("data") or {}
        record = data.get("record") or data
        labels = record.get("labels") or data.get("labels") or {}
        f.write(json.dumps({"at": at, "type": params.get("type"), "terminal": params.get("terminal"),
                            "role": labels.get("role"), "status": record.get("status"),
                            "state": record.get("state")}) + "\n")
