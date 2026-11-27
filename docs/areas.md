---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Areas: repositories that work together

This document says what an area is in temet-work, why it exists, and what is built.

## Why

Some repositories only make sense together. In the case that prompted this, about fifteen
repositories in two code-host groups share one tracker project. A ticket often changes
two or three of them on one branch name: a user interface, the service behind it, and a
base image. The merge requests are tied together only by the ticket's key in their
branch names, and are merged in order: the interface first, then, once its main
pipeline and its post-deploy tests pass, the service. Shared CI libraries and an API
contract between two of the repositories are the dependencies between them.

A project in temet-work is one repository. An area groups projects.

## What an area is

| Field | What it holds |
|---|---|
| `id`, `name` | Its id, shared with projects (an id names a project or an area, never both), and the name shown for it. |
| `projects` | The repositories in it. A project is in one area at most. |
| `tracker`, `tracker_project` | Optional, both or neither: the tracker its issues come from, when one tracker project serves all its repositories. Its projects then name no tracker of their own. |

An issue from an area's tracker has the id `area#KEY`, and its plan says which of the
area's repositories it changes, in the order they merge (`repos`). For an issue from a
project's own tracker, `repos` is that project when left empty.

An area is one level of the settings chain (`docs/settings.md`): a value it sets holds for
its projects and its own issues unless a project, milestone or task sets its own.

A group can take whole areas (`areas = ["guild"]`) as well as single projects: it then takes
the area's own issues and the issues of every project in it.

A dependency on another project or area is written `guild-ui#12`, by id, or
`owner/repo#12`, by the tracker's name for it; a bare `#12` or `ENG-41` stays where the
issue that names it is.

## On screen

The design system's rule (two levels in the nav bar, no more) decides the layout. The
area is a collapsible group label, opened and closed by clicking the label; its
repositories are the nav items, each with an icon, its name and a count of what waits
on it. Milestones go in the main area as a list, and groups of agents in the side pane.
The area's tracker and code-host groups go in its page header, not in the nav bar. With
more than about five areas, the area becomes a selector at the top of the nav bar.

## What is built

Stage 1: areas in the configuration and the loop; area trackers synced like project
trackers; groups that take areas; dependencies across projects; an area issue worked in
one repository, the first in its plan's `repos`.

Stage 2: an issue that changes several repositories.

- The issue holds one free work folder in each repository its plan names, all on one
  machine (the issue's `machine`, when that setting names one) so that its author reaches
  every one, and all or none, so two issues cannot each
  hold half of what the other needs. Its author starts in the first, and makes the branch
  and one pull request per repository itself, on one branch name; the daemon only holds the
  folders.
- `temet-work ready ISSUE REPO=NUMBER...` names the pull requests; a bare number works
  when the plan names one repository. `temet-work issue` lists each with its repository
  and whether it merged. Event logs from stage 1 replay: their single pull request is the
  first repository's.
- CI must pass on every pull request, and a failure or a conflict names its repository.
  Each repository's own `ci` setting says whether it runs any.
- They merge in the plan's order, each after the previous repository's main pipeline has
  passed (with `ci = false` there, without waiting). A main branch that still shows no run
  for the merge at a later read is listed as a problem until it has one, and so is a GitLab
  pipeline held at a manual job.
- Deploy and verify run once, after the last merge. The folders are released together,
  then or when the issue is closed.
- Merged pull requests are recorded per repository, so a restart or a refused merge
  part-way never merges one twice. Nothing is reverted: the author hears what merged and
  what cannot, and fixes forward. To change a repository that already merged, the author
  opens a new pull request there and names it with `ready`; it merges in its turn.
- Once one has merged, closing the issue in the tracker no longer drops the rest, since a
  pull request that closes its issue on merge would otherwise do so.

Not built: deploy and verify run with the settings of the first repository only, and a
reviewer reads the other repositories' changes from the code host.
