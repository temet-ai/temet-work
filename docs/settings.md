---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Settings: defaults at every level

This is how temet-work decides how a piece of work is done: which runtime and model an
agent runs, where it runs, how many review rounds there are, and the rest. It replaced
the fixed values and one-off keys that the code had.

## The chain

Every setting is looked up in one chain, most specific first:

| Level | Covers | Named as |
|---|---|---|
| Task | one issue | `radar#44` |
| Milestone | the issues in one milestone of a project or area | `radar:Slack sign-in rework` |
| Project | one repository | `radar` |
| Area | the projects in an area, and the area's own issues | `guild` |
| Global | everything | `global` |
| Built in | the value in the code, when no level sets one | |

The first level that sets a value wins; a level that does not set it passes the question
on. For an issue of an area, the project level is the repository the issue is worked in.

## Roles

A setting can also be given for one role, such as reviewers. The lookup goes level by
level, and at each level it reads the role's own value first, then the value for every
role. So when the area gives reviewers `runtime = "codex"` and the project gives
everyone `runtime = "claude"`, reviewers on that project run Claude, because the project
is asked before the area. For them to run Codex there, the project names its reviewers
too.

## The settings

| Setting | Built in | Read by |
|---|---|---|
| `runtime` | `claude` when the build has it; a build without it must set one globally | the runtime family, which picks its adapter by it (`claude`, `codex`) |
| `model` | the runtime's own default | the agent's brief, so the runtime adapter |
| `runner` | `herdr` when the build has it, likewise | the runner family, which picks its adapter by it |
| `permission` | `auto` | the agent's brief; the Claude adapter's permission mode; the Codex adapter ignores it and always runs `--approve-for-me` |
| `review_rounds` | 3 | the loop, per issue |
| `ci` | true | the code-host family, per issue |
| `deploy`, `verify`, `run_in` | none | the deploy part, per issue |
| `merge` | `squash` (or `merge`, `rebase`) | the code host's merge |
| `remind_after`, `reminders` | 10 minutes, 2 | the runner family, per agent's issue and role |
| `starts` | 3 agents of one role on one issue | the loop, per issue and role |
| `sync_every`, `ci_every` | 60 s, 30 s | the tracker family for each project or area; the code-host family for each issue |

The table is the resolver's: `temet-work settings` lists the same keys, and the code reads
every one of them through it. A setting with no value and no built-in value, such as
`model`, is left out of the listing.

How many issues a group refines at once stays on the group: it is how much the group
takes on, not how a piece of work is done.

## Setting them

In the configuration file, `[settings]` at the top is the global level, each project and
area can have its own, and a milestone is named with its project or area:

```toml
[settings]
runtime = "claude"
model = "sonnet"

[settings.reviewer]
runtime = "codex"

[[project]]
id = "radar"
tracker = "github"
name = "temet-ai/radar"

[project.settings]
model = "haiku"

[[milestone]]
of = "radar"
name = "Slack sign-in rework"

[milestone.settings]
model = "opus"
```

By command, the same way a person or an agent runs any other:

```
temet-work set radar#44 model opus
temet-work set radar --role reviewer runtime codex
temet-work unset radar#44 model
temet-work settings radar#44 --role author
```

A refiner can set task-level values in its plan, as a `settings:` map next to `checks:`,
such as a stronger model for an issue it judges hard. That is a decision like any other:
recorded as set by the refiner, and open to overturning with `temet-work set`. A refiner
can set the task level of the issue it is refining and nothing wider; no other agent can
set anything, and a wider level is a person's.

Every change, from the file, a command or a plan, is a `SettingSet` or `SettingUnset`
event in the log with who made it: `file`, `person`, or the agent's id. A later event
overrides an earlier one at the same level, role and key, so replaying the log gives the
same settings. A value that names a runtime or runner this build does not have is refused
when it is set, with the ones the build has listed, and the daemon does not start with
one in its configuration. A build with no default runtime or runner, and no global value
for it, does not start either.

Milestones are written `project:Milestone name` or `area:Milestone name`, in the
commands and in the listing; in the file, `of` and `name`. An issue is matched to its
milestone through the tracker's milestone field. Units: `remind_after` is in minutes,
`sync_every` and `ci_every` in seconds.

## Seeing them

`temet-work settings TARGET` lists every setting in effect there and the level that set it:

```
radar#44, author
  runtime     claude   global
  model       opus     task, set by refiner-3
  runner      herdr    built in
```

The listing says "set by" for everything except a value from the file, which reads as
its level alone. `temet-work issue` shows the same for the role at work on the issue.
The MCP server offers `settings` at the read level and `set` and `unset` at the act
level. `temet-work status` lists every value with the level it is set at and who set it.

## In the code

- The core keeps each level's values, built from `SettingSet` events, and one resolver
  (`World::resolve`) answers "what is this setting for this issue and role, and which
  level set it". Parts ask the world for a setting instead of reading their own fields
  or constants.
- A family reports the adapters in its build and its default one; the host puts them in
  the world, which is how a value naming an adapter the build lacks is refused. The
  runtime family picks its adapter by `runtime`, the runner family by `runner`. The Codex
  adapter (`temet-work-runtime-codex`) is the second runtime.
- Every family is tested with two adapters compiled in at once, so "the first one wins"
  cannot come back unnoticed.
- The group's `model` and the project's `ci`, `deploy`, `verify` and `run_in` are read as
  settings at their level, so configuration written today keeps working. A group's model
  becomes the model of the projects and areas it works on.
- A value in the file does not overwrite one a person or an agent has set since, and a
  value taken out of the file is unset again.

## Decided, open to change

- Groups set no defaults. A group decides which work it takes; the chain decides how that
  work is done.
- Within a level the role comes first; across levels the nearer level comes first.
