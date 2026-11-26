#!/usr/bin/env python3
"""timeline.py <run>...: send and receive times of every message, read from the transcripts.

sent     = time of the sender's send tool call
received = time of the record that put the message in front of the recipient's model
copies   = how many such records the recipient has for that id
Writes runs/<run>/timeline.md and prints it. Times are local.
"""
import json, re, sys
from datetime import datetime
import run

ROUTE = {1: ("reviewer", "author"), 2: ("author", "reviewer"), 3: ("reviewer", "author"),
         4: ("author", "arbiter"), 5: ("arbiter", "author"), 6: ("arbiter", "reviewer")}


def at(d):
    return datetime.fromisoformat(d["timestamp"].replace("Z", "+00:00")).astimezone()


def records(session):
    all_ = [json.loads(x) for x in run.transcript(session).splitlines()]
    return [d for d in all_ if d.get("type") in ("user", "assistant")]


def blocks(d):
    c = (d.get("message") or {}).get("content")
    return [{"type": "text", "text": c}] if isinstance(c, str) else (c or [])


def clock(t):
    return t.strftime("%H:%M:%S.%f")[:-3]


def one(name):
    state = json.loads((run.HERE / "runs" / name / "state.json").read_text())
    recs = {r: records(s["session"]) for r, s in state.items()}
    prefix = "M" if any(
        b.get("name") == "mcp__mike__send_session_message"
        for d in recs["reviewer"] if d.get("type") == "assistant" for b in blocks(d)) else "H"
    rows, problems, ruling, first, last = [], [], "", None, []
    for n, (frm, to) in ROUTE.items():
        mid = f"{prefix}{n}"
        sends, result, got = [], "", []
        for d in recs[frm]:
            for b in blocks(d):
                if b["type"] == "tool_use":
                    i = b["input"]
                    text = str(i.get("message") or i.get("command") or "")
                    if text.startswith(mid + " ") or ("hc send" in text and f"'{mid} " in text):
                        sends.append((at(d), b["id"]))
                        if n == 5:
                            ruling = text[text.index("RULING"):].rstrip("'")
        for d in recs[frm]:
            for b in blocks(d):
                if sends and b["type"] == "tool_result" and b["tool_use_id"] == sends[0][1]:
                    c = b["content"]
                    c = c if isinstance(c, str) else "".join(x.get("text", "") for x in c)
                    m = re.search(r'"delivery_state":"(\w+)"', c) or re.search(r'"status":"(\w+)"', c)
                    result = ("delivery_state " if prefix == "M" else "status ") + m.group(1) if m else c[:80]
        want = (rf'<hc from="[^"]*">{mid} ' if prefix == "H"
                else rf'(<cross-session-message [^>]*>\s*{mid} |^The tether plugin sent a message:\s+Mail from [^\n]*?\):\s+{mid} )')
        for d in recs[to]:
            if d.get("type") != "user":
                continue
            for b in blocks(d):
                if b["type"] == "text" and re.search(want, b["text"]):
                    via = ("hc" if prefix == "H" else
                           "tether plugin" if b["text"].startswith("The tether") else "peer socket")
                    got.append((at(d), via))
        if not sends:
            problems.append(f"{mid} never sent")
            continue
        if not got:
            problems.append(f"{mid} sent, never received")
        if len(sends) > 1:
            problems.append(f"{mid} sent {len(sends)} times")
        first = first or sends[0][0]
        if n >= 5 and got:
            last.append(got[0][0])
        rows.append(f"| {mid} | {frm} → {to} | {clock(sends[0][0])} | "
                    f"{clock(got[0][0]) if got else '-'} | "
                    f"{(got[0][0] - sends[0][0]).total_seconds():.2f} | {got[0][1] if got else '-'} | "
                    f"{result} | {len(got)} |" if got else f"| {mid} | {frm} → {to} | {clock(sends[0][0])} | - | - | - | {result} | 0 |")
    acks, errors, reads = {}, [], {}
    for role, rs in recs.items():
        for d in rs:
            for b in blocks(d):
                if b["type"] == "tool_use" and b["name"] == "mcp__mike__ack_session_message":
                    acks[role] = acks.get(role, 0) + 1
                if b["type"] == "tool_use" and b["name"] == "mcp__mike__get_session_messages":
                    reads[role] = reads.get(role, 0) + 1
                if b["type"] == "tool_result" and b.get("is_error"):
                    errors.append(f"{role} {clock(at(d))}: {str(b['content'])[:140]}")
    text = [f"# {name}", "",
            "| id | from → to | sent | received | seconds | arrived by | send returned | copies |",
            "|---|---|---|---|---|---|---|---|", *rows, ""]
    if last:
        text.append(f"Blocking comment sent to ruling held by both: {(max(last) - first).total_seconds():.1f} s")
    text += [f"Ruling: {ruling}", "", f"Mailbox reads: {reads or 'none'}. Ack calls: {acks or 'none'}.",
             f"Message problems: {problems or 'none'}", f"Tool errors: {errors or 'none'}"]
    out = "\n".join(text) + "\n"
    (run.HERE / "runs" / name / "timeline.md").write_text(out)
    print(out)


for name in sys.argv[1:]:
    one(name)
