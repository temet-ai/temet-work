---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
  - 30de12a8-8bb8-4310-a59b-8d5267ce0b97
  - 7d9306de-2941-41bd-8a60-efad6ed38f2a
  - 48c80237-558e-4a4a-a82d-bd96b37f1f46
  - 198d20e8-7c5c-4a63-b1e6-5f43be8c6607
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

Only a setting that is read for the role of an agent takes a role: those marked yes under
Per role in the table below. Every other is the same for every role. A role on one of
those is refused, naming the key: by `set` and `unset` with `--role`, and by the daemon
when it reads a role table in its configuration, such as `[settings.reviewer]`, and then
it does not start. A listing for a role shows such a setting at the value for every role,
which is the one in effect.

## The settings

| Setting | Per role | Built in | Read by |
|---|---|---|---|
| `runtime` | yes | `claude` when the build has it; a build without it must set one globally | the runtime family, which picks its adapter by it (`claude`, `codex`) |
| `model` | yes | the runtime's own default | the agent's brief, so the runtime adapter. A model belongs to one runtime: `model.<runtime>` (`model.codex`) names it, and a plain `model` is recorded as the global runtime's when it is written ("A model belongs to a runtime", below) |
| `runner` | yes | `herdr` when the build has it, likewise | the runner family, which picks its adapter by it |
| `machine` | yes | none: any machine | the loop, per issue and role, when it picks a work folder (below) |
| `permission` | yes | `auto` (or `default`, `acceptEdits`, `plan`, `dontAsk`, `bypassPermissions`) | the agent's brief; the Claude adapter's permission mode; the Codex adapter ignores it and always runs `--approve-for-me` |
| `review_rounds` | no | 3 | the loop, per issue |
| `ci` | no | true | the code-host family, per issue |
| `deploy`, `verify`, `run_in` | no | none | the deploy part, per issue |
| `merge` | no | `squash` (or `merge`, `rebase`) | the code host's merge |
| `remind_after`, `reminders` | yes | 10 minutes, 2 | the runner family, per agent's issue and role |
| `starts` | yes | 3 agents of one role that ended while one issue waited on them | the loop, per issue and role |
| `sync_every`, `ci_every` | no | 60 s, 30 s | the tracker family for each project or area; the code-host family for each issue |
| `investigate` | no | none | the investigator's brief, under each project's name: lines a person adds, such as the parts of the code another team owns |

The table is the resolver's: `temet-work settings` lists the same keys, and the code reads
every one of them through it. A setting with no value and no built-in value, such as
`model`, is left out of the listing.

## A model belongs to a runtime

A model's name means something to one runtime only, so a model never reaches an agent of
another one.

- `model.<runtime>`, such as `model.claude` or `model.codex`, is the model for agents of
  that runtime.
- A plain `model` is bound to the global runtime when it is written: the `runtime` set
  globally for every role, or the build's default (`claude`) when none is. It is recorded
  as `model.<that runtime>`, and keeps that runtime when the global runtime changes later.
  With `claude` global, `temet-work set radar#44 model opus` records `model.claude` and
  answers `model.claude is opus at radar#44`; a refiner's `model: opus` is recorded the
  same way, and its plan keeps `model` as the refiner wrote it.
- A plain `model` is refused, naming `model.<runtime>`, where the runtime in effect for
  that scope and role is not the global one. With reviewers on `codex`,
  `temet-work set radar --role reviewer model gpt-5-codex` is refused naming `model.codex`,
  and so is a plan's `model` on an issue whose author runs `codex`. Nothing is recorded.
  `temet-work set radar model haiku` for every role is still taken, as `model.claude`.

The lookup for an agent's model goes level by level like any other, with the role's own
value before the value for every role, and at each of those places it reads
`model.<the agent's runtime>`. So a refiner's `model: opus` on an issue beats a global
`model.claude` for a Claude author. An agent with no model of its own runtime anywhere in
the chain runs its runtime's default. After `temet-work set global runtime codex`, that
refiner's `opus` and every other plain model written before are still `model.claude`, so
a Codex author runs Codex's default until a person sets `model.codex`.

In the example under "Setting them", authors run Claude with `haiku` on `radar`'s issues
and `opus` on the milestone's. Reviewers run Codex, so none of those models reaches them;
`codex = "..."` under `[settings.reviewer.model]` would give them one.

`temet-work settings` shows the `model` an agent of that role gets, and no `model` line
when it gets none. `temet-work status` lists each value under the key it is recorded
with, so a plain model shows as `model.<runtime>`.

How many issues a group refines at once stays on the group: it is how much the group
takes on, not how a piece of work is done. So do `queue_low` and `investigate_every`, which
say when the group starts an investigator (`docs/configuration.md`).

An investigator works for its group and on no issue, so its settings (`runtime`, `model`,
`runner`, `machine`, `permission`, `remind_after`, `reminders`) are read at the project
level of the first repository the group covers: its first project, else the first project
of its first area. `investigate` is read at each project's level, for that project's lines. A value at a
task or milestone level reaches no investigator.

## Which machine an agent runs on

`machine` takes the name a `[[folder]]` entry has in its own `machine` field, and limits an
agent to those entries. The daemon's own machine is `local`. Unset, it limits nothing. It
picks the checkout, and the entry's own field still says where that checkout is
(`docs/configuration.md`).

- An author takes a free folder on its machine. With none free, the issue stays ready and
  holds no folder, and its author starts when one is released. A free folder on another
  machine does not start it. An issue across several repositories takes one there in each
  of them, or none.
- An author with no machine set leaves a free folder to a ready issue pinned to that
  folder's machine when another folder is free, and takes the first free folder in id
  order otherwise. The other folder has to be one that no such issue could use. Free
  folders are not counted against pinned issues: when those issues could use every one,
  the author takes the first, even if another machine has two free for one of them. A
  ready issue here is one whose dependencies are done and that waits only for its author.
  Pinned issues of every group count, since a folder serves every group that works its
  project. An unpinned issue never waits because of this: with one folder free, the
  higher-ranked of the two takes it.
- A refiner, reviewer or arbiter holds no checkout of its own and may share a held one on
  its machine. If the project has none there, the agent waits. It is not moved to another
  machine. An issue whose refiner cannot start stays new and takes no refining slot.
- An author that already holds a folder keeps it when the setting changes. The new value
  applies the next time a folder is picked.
- A name no folder uses is accepted, because a folder may be listed later. An empty value
  is refused.

To run authors on a Mac while reviewers stay on the daemon's machine, name both roles:

```
temet-work set radar --role author machine mac
temet-work set radar --role reviewer machine local
```

A value for every role at a nearer level, such as `temet-work set radar#44 machine mac`,
moves that issue's reviewer too, because the task is asked before the project (Roles,
above).

`temet-work status` lists each issue that waits for this under Waiting, as
`radar#44 waiting for a work folder on mac`. The line names the agent when it is not the
author, and says when the project has no work folder on that machine at all, which is how a
misspelt name shows.

## Setting them

In the configuration file, `[settings]` at the top is the global level, each project and
area can have its own, and a milestone is named with its project or area:

```toml
[settings]
runtime = "claude"

[settings.model]
claude = "sonnet"

[settings.reviewer]
runtime = "codex"

[[project]]
id = "radar"
tracker = "github"
name = "temet-ai/radar"

[project.settings.model]
claude = "haiku"

[[milestone]]
of = "radar"
name = "Slack sign-in rework"

[milestone.settings.model]
claude = "opus"
```

TOML cannot hold `model = "..."` and `model.codex` in one table, so in the file a
runtime's model is an entry of a `model` table at its level, or in a role's table
(`[settings.reviewer.model]`). Each entry is the setting `model.<runtime>`. A plain
`model = "..."` still loads. The file alone decides its runtime: the `runtime` the file
gives that table's scope and role, which has to be the file's global one (its
`[settings]` `runtime`, else the build's default). A `runtime` a person set by command does
not move it, so after `temet-work set global runtime codex` a restart on the same file
keeps its `model = "sonnet"` as `model.claude`; changing the file's own `[settings]`
`runtime` to `codex` makes it `model.codex` on the next start. The daemon does not start
on a plain `model` where the file gives that scope and role another runtime, such as
`[settings.reviewer]` with `runtime = "codex"` and `model = "gpt-5-codex"`, and it names
`model.codex`: write `codex = "gpt-5-codex"` under `[settings.reviewer.model]` instead.

By command, the same way a person or an agent runs any other:

```
temet-work set radar#44 model opus
temet-work set radar --role reviewer runtime codex
temet-work set radar --role reviewer model.codex NAME
temet-work unset radar#44 model
temet-work settings radar#44 --role author
```

A refiner can set task-level values in its plan, as a `settings:` map next to `checks:`,
such as a stronger model for an issue it judges hard. That is a decision like any other:
recorded as set by the refiner, and open to overturning with `temet-work set`. A refiner
can set and unset the task level of the issue it is refining, and only while that issue is
refining: once its plan is in, a `set` or `unset` from it is refused with the stage the
issue is at. No other agent can set anything, and a wider level is a person's.

A refiner's `model` is for the author. How hard the code is to write is what the refiner
judges; how strong the review is belongs to the person, so the models of reviewers,
arbiters, refiners and investigators are a person's to set. A value from a plan, or from a
refiner's `set` with no role or with `--role author`, is recorded for the author, and its
`unset` removes the author's value. A `set` or `unset` of `model` that names another role
is refused. The refiner's other keys, such as `merge` or `starts`, are recorded for every
role. All of this holds for `model.<runtime>` as for `model`.

A person overturns the refiner's value with a `set` on the task that names no role, such
as `temet-work set radar#44 model haiku`: it holds for every role, and the refiner's value
for the author is dropped. A person's `unset` with no role drops it too, so the author
takes the wider level's value. A value a person set with `--role author` stays.

A plain `model` is written as `model.claude` while `claude` is global, so a person's `set`
or `unset` of either name on the task, with no role or for the author, replaces or drops
the refiner's value. An `unset` of a plain `model` that finds only another runtime's key
there is refused and names that key: with a model set on `radar#44` while `claude` was
global, and `codex` global now, `temet-work unset radar#44 model` names `model.claude`, and
`temet-work unset radar#44 model.claude` takes it away.

Nine settings only a person sets, at any level and for any role they take. An agent's
plan, `set` or `unset` that names one is refused with the reason, and a refused plan is
not recorded:

| Key | Why only a person sets it |
|---|---|
| `runtime` | a runtime may not follow `permission`, so the runtime decides what an agent may do without asking |
| `runner` | it follows from the person's machines, not from the issue, and a value for one issue replaces the one a person chose for a role |
| `review_rounds` | it decides how many rounds of review an issue gets before the arbiter rules, so fewer rounds weaken its review |
| `deploy` | the daemon runs it as a shell command after a merge |
| `verify` | the daemon runs it after a deploy |
| `run_in` | it is the folder `deploy` and `verify` run in |
| `permission` | it decides what every later agent on the issue may do without asking |
| `ci` | `false` takes the CI gate off the code the issue's own author writes, in every repository the issue changes |
| `investigate` | it is text written into every later investigator's brief |

A log written before agents were refused may hold one of these set or unset for an issue by
an agent. The daemon reads such a log and leaves that change out, so the issue takes the
value of the next level, or the built-in one, and `temet-work settings` does not list the
agent's.

Likewise, a log written before a refiner's `model` was the author's may hold one set for
an issue for every role, which is read as the author's, or for another role, which is left
out. An issue refined before the upgrade is then reviewed on the person's model.

A log written before a role was refused on a setting that takes none may hold one set for a
role. It never had an effect, and the daemon reads the log and leaves it out, so neither
`temet-work settings` nor `status` lists it.

A log written before plain models were bound when written holds them as `model`. The
daemon puts its build's runtimes in the world before it replays the log, and binds each
plain `model` there, set or unset, to the runtime that was global at that point in the log,
or to the build's default when none was set. A group's old `model` is bound the same way.
`temet-work status` then lists each under `model.<runtime>`. A refiner's `model: opus`
logged while `claude` was global stays `model.claude` after a later
`temet-work set global runtime codex`, so a Codex author on that issue runs Codex's
default. As it replays, the daemon remembers which `model.<runtime>` the last plain `model`
at each scope and role was bound to, a group's old `model` included. An older log's plain
unset takes away the value its earlier plain set at that scope and role was bound to, and a
later plain set there replaces it, even after the global runtime changed: a log with
`set radar --role reviewer model opus` while `claude` was global, then
`set global runtime codex` and `unset radar --role reviewer model`, leaves radar's
reviewers with no model. A plain unset where the log set no plain model is bound to the
runtime global at that point.
Before models were keyed by runtime, a plain `model` was the only way to give Codex reviewers a model
(`[settings.reviewer] runtime = "codex"` with `model = "gpt-5-codex"`, or
`temet-work set radar --role reviewer model gpt-5-codex`). A logged one is now listed under
the runtime that was global then, usually `model.claude`, and reaches no reviewer. Set it
again as `model.codex` (`temet-work set radar --role reviewer model.codex gpt-5-codex`), and
`temet-work unset radar --role reviewer model.claude` takes the old one away. The file's
form of it stops the start until it is moved under `[settings.reviewer.model]`.

Every change, from the file, a command or a plan, is a `SettingSet` or `SettingUnset`
event in the log with who made it: `file`, `person`, or the agent's id. A later event
overrides an earlier one at the same level, role and key, so replaying the log gives the
same settings. A value that names a runtime or runner this build does not have, or a
`model.<runtime>` for such a runtime, is refused when it is set, with the ones the build
has listed, and the daemon does not start with one in its configuration. A build with no
default runtime or runner, and no global value for it, does not start either.

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
- Each setting's `Spec` says whether it is read per role (`per_role`). `World::resolve`
  drops the role for one that is not, so the listings and the parts get the same answer.
  `tests/settings.rs` fails when a reader passes a role the `Spec` does not allow for, or
  passes none for a setting that takes one.
- Each setting's `Spec` says whether an agent may set it: `May`, `ForAuthor` (a refiner
  may, for the author only: `model`) or `MayNot`. The plan check, `set`, `unset` and the
  replay of a log all read that, so a new key has to say which it is.
- A family reports the adapters in its build and its default one; the daemon puts them in
  the world before it replays the log (`temet_work::start`), which is how a value naming an
  adapter the build lacks is refused, and how an older log's plain model finds the build's
  default runtime. The
  runtime family picks its adapter by `runtime`, the runner family by `runner`. The Codex
  adapter (`temet-work-runtime-codex`) is the second runtime.
- Every family is tested with two adapters compiled in at once, so "the first one wins"
  cannot come back unnoticed.
- The group's `model` and the project's `ci`, `deploy`, `verify` and `run_in` are read as
  settings at their level, so configuration written today keeps working. A group's model
  becomes the model of the projects and areas it works on, bound like any plain `model`.
- A plain `model` is bound where its event is made: `World::model_key` for `set` and a
  plan's settings, `World::bound_key` for `unset` and for a logged event as it is applied,
  and the configuration's own reading of the file. The log then says which runtime each
  model is for.
- A value in the file does not overwrite one a person or an agent has set since, and a
  value taken out of the file is unset again.

## Decided, open to change

- Groups set no defaults. A group decides which work it takes; the chain decides how that
  work is done.
- Within a level the role comes first; across levels the nearer level comes first.
- Models are keyed by runtime, and a plain `model` is bound to the global runtime when it
  is written. Refusing a change of the global `runtime` while plain models exist
  was left out: it would block `set global runtime` on every refiner value in the log,
  which no file holds and nobody can edit, and stop the daemon from starting after a
  runtime change in the file. Applying a model only when it was set at a level as near as
  the runtime was left out too: in the example above the project's model is nearer than
  the global runtime of reviewers, so a Codex reviewer would still get a Claude model.
- A plain `model` written where that scope and role run another runtime is refused,
  naming `model.<that runtime>`. Binding it to that role's runtime was left out: refiners
  and these docs read a plain model as the global runtime's, so binding a refiner's `opus`
  to a Codex author would start Codex with a Claude model. Binding it to the global runtime there
  was left out too: the value would reach nobody, and the writer would not be told.
- The file's plain `model` is bound by the file alone. Binding it against the live
  settings was left out: it would make the file's `sonnet` Codex's on the first restart
  after a person's `set global runtime codex`.
- An older log's plain `model` replays as the model of the runtime that was global at that
  point. Binding it to the role's runtime was left out: the global one is the runtime it was
  read for when it was written, and the role's would hand a refiner's Claude model to a
  Codex author.
- An older log's plain unset takes away the key its plain set at that place was bound to,
  and a `model.claude` written by name over that plain value goes with it, because a plain
  model and the global runtime's `model.<runtime>` are one key. Keeping the plain value
  apart from the named one was left out: it would undo that one key for older logs only.
- Only a person sets `runtime`. The Codex adapter ignores `permission`, so an agent that
  chose the runtime could undo what a person set there. Refusing the change only when
  `permission` is not `auto` and the runtime named ignores it was left out: the core
  would need each runtime to report whether it follows `permission`, a refiner's value
  would still replace the runtime a person chose for authors, and every later change of
  `permission` would need checking again.
- Only a person sets `runner`. Which runner starts an issue's agents follows from the
  person's machines, not from the issue, and a value for one issue replaces a runner a
  person chose for a role.
- Only a person sets `review_rounds`. How strong the review is belongs to the person, and
  fewer rounds weaken it; the refiner judges how hard the code is to write.
- Only a person sets `ci`. `ci = false` takes the CI gate off the code the issue's own
  author writes, in every repository the issue changes, so no agent decides it.
- An agent may still set `merge`. It takes only `squash`, `merge` or `rebase`, is applied
  after review and CI have passed, runs nothing, and changes only the shape of the history
  on `main`.
