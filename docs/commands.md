---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Commands

`temet-work` sends one command to the running daemon and prints its reply. People and
agents use the same commands; the brief each agent starts with names the one that
reports its step done.

| Command | Who runs it | Accepted when the issue is |
|---|---|---|
| `temet-work status` | anyone | any |
| `temet-work plan ISSUE FILE` | the refiner, with the plan as YAML; `-` reads it from standard input | refining |
| `temet-work ready ISSUE PULL` | the author, once the pull request is open and the plan's checks hold; `PULL` is its number, needed the first time | working, or fixing |
| `temet-work review ISSUE approve` | the reviewer | in review |
| `temet-work review ISSUE block POINT...` | the reviewer, one argument per thing to fix | in review |
| `temet-work rule ISSUE approve REASON` | the arbiter | with the arbiter |
| `temet-work rule ISSUE fix REASON` | the arbiter | with the arbiter |

`ISSUE` is `project#key`, such as `radar#44`. A plan file looks like this:

```yaml
checks:
  - done_when: Signing in from radar stores a user token that outlives a restart
    proven_by: cargo nextest run -p radar-slack sign_in, then restart radar and open a channel
```

A command for an issue at another step is refused, with the step the issue is at. The
command line exits 0 when the daemon did it, 1 when the daemon refused it, and 2 when the
command was not understood or no daemon answered.

The daemon starts every agent with `TEMET_WORK_AGENT` set to the agent's id, so a command
an agent runs is recorded as that agent's. A ruling is recorded as a decision a person
can read and overturn.

The daemon listens at `$TEMET_WORK_SOCKET`, else `$XDG_RUNTIME_DIR/temet-work/daemon.sock`,
one JSON request per line and one JSON reply per line.
