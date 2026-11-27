---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# What temet-work holds

Every screen, command and message in temet-work is about one of these objects. The app,
the CLI and the MCP server show and change them through the same commands. The core crate
defines areas, projects, groups, agents, work folders, issues, plans, checks, decisions
and events today; a milestone is an id on its issues, and the other objects are not built
yet.

## Objects

| Object | Belongs to | What it is |
|---|---|---|
| Area | the app | Projects that work together, and the tracker of its own issues when one tracker project serves them all (`docs/areas.md`). |
| Project | the app or an area | A repository: the tracker its issues come from (GitHub, GitLab or Jira), the code host its pull requests are on, and its work folders. |
| Setting | a level: global, area, project, milestone or task | A value that says how work is done, such as the runtime, model or review rounds, for every role or one role. The most specific level that sets it wins (`docs/settings.md`). |
| Group | the app | Sessions working together on a goal: its agents and the issues it has taken, from one or more projects, in the order of the milestones it works on. The daemon coordinates a group; no agent does. Groups run side by side and do not wait on each other. |
| Agent | a group | One session: its role (refiner, author, reviewer, arbiter, investigator), machine, work folder, terminal and state. Its model comes from its group until the settings give runtime and model per role (`docs/settings.md`). |
| Machine | the app | Where agents run, with its build slots. |
| Limit | the app | A runtime account's usage windows (Claude 5 hours and 7 days, Codex 5 hours). |
| Style guide | the app | Tokens, components, the ban list and the checks used when reviewing UI work. A project may set its accent, display face and density. |
| Rule | the app or a project | A line in an agent's brief or in the config, with where it came from (a ruling, an overturned decision, a measured change), its state (proposed, kept or dropped; on trial and reverted are not built) and what it is measured by. |
| Milestone | a project or area | From the tracker. A group's focus is a list of milestones. |
| Issue | a project, in a milestone or none | From the tracker. One group at a time takes it through the loop. |
| Work folder | a project | A checkout on one machine, held by one issue at a time (an issue holds one in each repository it changes), with a target directory that stays warm between issues. |
| Plan | an issue | Written by the refiner as YAML: the checks and, for an area's issue, the repositories it changes in merge order (`repos`). The parts and who does each are not built yet. |
| Check | a plan | One "Done when" line from the issue and the test or command that proves it. |
| Pull request | an issue | The branch, CI, and the review rounds. An issue has one in each repository it changes. |
| Review round | a pull request | The reviewer's blocking points and the author's replies. A dispute between them goes to an arbiter. |
| Decision | an issue | A choice an agent made: the options it weighed, the one it took, why, who decided, and the point after which overturning it costs more. Work carries on at once; a person reads decisions afterwards and can overturn them. |
| Message | an agent | Agent to agent, or agent to a person. |
| Event | everything | What happened, with its time, in one append-only log. State is built from events. |

## The loop an issue goes through

A group takes an issue; the refiner writes its plan; an author works on it in a work folder
and opens a pull request; a reviewer reviews it in rounds, and an arbiter settles a dispute;
CI and the checks pass; it merges, deploys and is checked in a real session; the issue
closes and the group takes the next one. Each choice along the way is a decision a person
can overturn, and overturns change the rules agents work by.

## Where each object shows in the app

| Column | Objects |
|---|---|
| Left: spans every project | projects, groups and their agents, machines and limits, the style guide, rules |
| Middle: the selected project | milestones, issues grouped by milestone, stage or group, work folders |
| Right: what was clicked | an issue's plan, pull request, review rounds and decisions; or an agent's terminal and decisions |
