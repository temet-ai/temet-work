#!/usr/bin/env bash
# List, or kill and remove, only the herdr-core terminals that carry ALL the given labels.
# Usage: own.sh ls|clean K=V [K=V...]
# Refuses a label set that is only fleet=...: other seats share that label.
set -euo pipefail
cmd=${1:-}; shift || true
[[ $cmd == ls || $cmd == clean ]] || { echo "usage: own.sh ls|clean K=V [K=V...]" >&2; exit 2; }
own=0; args=()
for l in "$@"; do
  [[ $l == *=* ]] || { echo "not a label: $l" >&2; exit 2; }
  [[ $l == fleet=* ]] || own=1
  args+=(--label "$l")
done
(( own )) || { echo "refused: add a label only you set (by=<your id>); fleet= alone matches other seats" >&2; exit 2; }
addrs=$(hc ls --all --json "${args[@]}" | python3 -I -c 'import json,sys; [print(t["address"], t["state"], t.get("status") or "-") for t in json.load(sys.stdin)["terminals"]]')
[[ -n $addrs ]] || { echo "none"; exit 0; }
echo "$addrs"
[[ $cmd == clean ]] || exit 0
while read -r a state _; do
  [[ $state == live ]] && hc kill "$a" >/dev/null && echo "killed $a"
  hc rm "$a" >/dev/null && echo "removed $a"
done <<< "$addrs"
hc ls --all --json "${args[@]}" | python3 -I -c 'import json,sys; t=json.load(sys.stdin)["terminals"]; print("left:", len(t)); sys.exit(1 if t else 0)'
