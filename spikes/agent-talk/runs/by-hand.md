<!-- slop-lint-ignore-file P-INSIDER-TERM -- raw record ids for searching mike -->
# Results that no script logged

Copied from the operating session (cc29fbec): `hc send` and mike's `send_session_message` and
`get_session_messages`. Times are UTC, as mike returns them (SAST is two hours later).

## Probe mails and G1

| id | to session | message_uuid | queued_at | channel_delivered_at | consumed_at | attempts | status when read |
|---|---|---|---|---|---|---|---|
| P4 | b5634f8a (busy) | 2c9b78f6 | 15:28:12.914 | 15:28:14.197 | 15:28:50.497 | 1 | consumed |
| K2 | b10143ed (blocked), dialog open | 83df73b4 | 15:29:47.291 | 15:29:47.313 | none | 1 | delivered |
| K2 | same, after the dialog was cancelled | 83df73b4 | | | 15:29:54.619 | 1 | consumed |
| G2 | b10143ed, killed | e195f4f0 | 15:30:02.138 | none | none | 0 | pending (read again at about 15:38, 8 minutes pending) |

G2's send returned `delivery_state: channel`, `recipient_liveness: ended`. At 15:38:31.350 mike
mailed the sender: "[mike mail] Not delivered: your message e195f4f0-… was never delivered: that
session made no request in time. mike delivers it when that session next makes a request."

G1, `hc send t1sx "G1 probe to a removed terminal" --json`, printed
`{"error":{"hint":"see `hc ls --all`","kind":"not_found","message":"no terminal \"t1sx\""}}`, exit 3.

## Dispute mails: queued and consumed (one delivery attempt each)

| run, id | queued_at | consumed_at |
|---|---|---|
| mail1 M1 | 15:21:44.749 | 15:21:45.137 |
| mail1 M2 | 15:22:05.756 | 15:22:07.054 |
| mail1 M3 | 15:22:18.962 | 15:22:19.097 |
| mail1 M4 | 15:22:26.520 | 15:22:26.639 |
| mail1 M5 | 15:22:42.159 | 15:22:43.092 |
| mail1 M6 | 15:22:42.198 | 15:22:43.102 |
| mail2 M1 | 15:30:30.053 | 15:30:30.805 |
| mail2 M2 | 15:30:48.405 | 15:30:49.548 |
| mail2 M3 | 15:30:53.367 | 15:30:54.723 |
| mail2 M4 | 15:31:01.869 | 15:31:02.307 |
| mail2 M5 | 15:31:25.324 | 15:31:26.862 |
| mail2 M6 | 15:31:25.346 | 15:31:25.542 |
