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
| `tracker`, `tracker_name` | Optional: the tracker its issues come from, when one tracker project serves all its repositories. Its projects then name no tracker of their own. |

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

Stage 1, now: areas in the configuration and the loop; area trackers synced like project
trackers; groups that take areas; dependencies across projects; an area issue worked in
one repository, the first in its plan's `repos`.

Stage 2, next: an issue that changes several repositories. Its author holds a work
folder in each and opens a pull request in each; `ready` names them all; CI must pass on
all of them; they merge in the plan's order, each after the previous repository's main
pipeline has passed.
