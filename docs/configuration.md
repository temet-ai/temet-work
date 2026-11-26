---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Configuring temet-workd

`temet-workd` reads one TOML file at start. It lists the projects whose issues it works
on, the work folders agents use, and the groups of agents.

| What | Where |
|---|---|
| Configuration | `$XDG_CONFIG_HOME/temet-work/config.toml` (`~/.config/temet-work/config.toml`), or `--config FILE` |
| Event log | `$XDG_STATE_HOME/temet-work/events.jsonl` (`~/.local/state/temet-work/events.jsonl`), or `--log FILE` |

The daemon works in rounds, one every two seconds, and prints each new problem to
stderr. `--once` stops after the first round and prints the state as text.

## Example

```toml
# A project: a repository and the tracker its issues come from.
[[project]]
id = "radar"              # short id, used in issue ids such as radar#44
tracker = "github"        # the tracker adapter
name = "temet-ai/radar"   # the tracker's name for it
# code_host = "github"    # where its pull requests are; the tracker when left out
# repo = "temet-ai/radar" # the code host's name for it; name when left out
# deploy = "make deploy"  # run after a merge; merged counts as deployed when left out
# verify = "make smoke"   # run after a deploy; deployed counts as verified when left out
# run_in = "/home/me/src/radar-deploy"   # where both run; the home folder when left out

# A work folder: a checkout an author works in, one issue at a time.
[[folder]]
id = "radar-1"
project = "radar"
path = "/home/me/src/radar-1"
# machine = "local"       # the default

[[folder]]
id = "radar-2"
project = "radar"
path = "/home/me/src/radar-2"

# A group: agents working through some projects' issues.
[[group]]
id = "group-1"
name = "Group 1"
projects = ["radar"]
milestones = ["Slack sign-in rework"]   # first to last; leave out for every issue
refining_at_once = 1                    # the default
```

A group takes its projects' open issues in milestone order, then in the tracker's order.
It refines `refining_at_once` issues at a time. It starts an author on a refined issue
when one of the project's folders is free and every issue in the issue's "Depends on"
lines is done. A project with two folders has at most two authors working at once.

Each project, folder and group in the file becomes an event in the log the first time the
daemon reads it, and again whenever it changes. Removing one from the file does not
remove it from the daemon yet.
