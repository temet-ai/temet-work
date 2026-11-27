---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Settings: defaults at every level

This is the plan for how temet-work decides how a piece of work is done: which runtime
and model an agent runs, where it runs, how many review rounds there are, and the rest.
None of it is built yet. It replaces the fixed values and one-off keys listed under "The
settings".

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

Any level can also set a value for one role, such as reviewers. At each level the
agent's role is checked before the value for every role, and only then is the next level
asked: the nearest level that says anything wins. So when the area sets
`reviewer.runtime = "codex"` and the project sets `runtime = "claude"`, reviewers on that
project run Claude; for them to run Codex there, the project names its reviewers too.

## The settings

| Setting | Built in | Where it is today |
|---|---|---|
| `runtime` | `claude` when the build has it; a build without it must set one globally | every agent gets the first runtime in the build |
| `model` | the runtime's own default | a group's `model` |
| `runner` | `herdr` when the build has it, likewise | every agent gets the first runner in the build |
| `permission` | `auto` | fixed in the Claude adapter |
| `review_rounds` | 3 | `REVIEW_ROUNDS` in the core |
| `ci` | true | a project's `ci` |
| `deploy`, `verify`, `run_in` | none | a project's keys |
| `merge` | squash | fixed in the GitHub adapter |
| `remind_after`, `reminders` | 10 minutes, 2 | fixed in the runner family |
| `starts` | 3 agents of one role on one issue | `MAX_STARTS` in the core |
| `sync_every`, `ci_every` | 60 s, 30 s | fixed in the tracker and code-host families |

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

A refiner can set task-level values in its plan, such as a stronger model for an issue it
judges hard. That is a decision like any other: recorded, and open to overturning.

Every change, from the file, a command or a plan, is an event in the log with who made
it. A value that names a runtime or runner this build does not have is refused when it is
set, and the daemon does not start with one in its configuration.

## Seeing them

`temet-work settings TARGET` lists every setting in effect there and the level that set it:

```
radar#44, author
  runtime     claude   global
  model       opus     task, set by refiner-3
  runner      herdr    built in
```

`temet-work issue` shows the same for the role at work on the issue. The MCP server offers
`settings` at the read level and `set` at the act level.

## In the code

- The core keeps each level's values, built from `SettingSet` events, and one resolver
  answers "what is this setting for this issue and role, and which level set it". Parts
  ask the world for a setting instead of reading their own fields or constants.
- The runtime family picks its adapter by `runtime`, the runner family by `runner`. The
  Codex adapter becomes the second runtime.
- Every family is tested with two adapters compiled in at once, so "the first one wins"
  cannot come back unnoticed.
- The group's `model` and the project's `ci`, `deploy`, `verify` and `run_in` are read as
  settings at their level, so configuration written today keeps working.

## Decided, open to change

- Groups set no defaults. A group decides which work it takes; the chain decides how that
  work is done.
- Within a level the role comes first; across levels the nearer level comes first.
