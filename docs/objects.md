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
| Agent | a group | One session: its role (refiner, author, reviewer, arbiter, investigator), machine, work folder, terminal and state. Its model comes from its group until the settings give runtime and model per role (`docs/settings.md`). It will also hold its session id, which mike and ask-kit know the session by: for Claude Code one the daemon chooses, for Codex the one Codex gives it (not built). |
| Machine | the app | Where agents run, with its build slots. |
| Limit | the app | A runtime account's usage windows (Claude 5 hours and 7 days, Codex 5 hours). |
| Style guide | the app | Tokens, components, the ban list and the checks used when reviewing UI work. A project may set its accent, display face and density. |
| Rule | the app or a project | A line in an agent's brief or in the config, with where it came from (a ruling, an overturned decision, a measured change), its state (proposed, kept or dropped; on trial and reverted are not built) and what it is measured by. It will carry a scope (a role, a project, or all), carry its id and revision on its line in a brief, count against a brief's budget of rules (the setting `rules`), and count the issues it was given to (not built). |
| Milestone | a project or area | From the tracker. A group's focus is a list of milestones. |
| Issue | a project, in a milestone or none | From the tracker. One group at a time takes it through the loop. |
| Work folder | a project | A checkout on one machine, used by one agent at a time, with a target directory that stays warm between issues. A reviewer will read its author's folder while the author waits, and change nothing there (not built). |
| Plan | an issue | Written by the refiner as YAML: the checks and, for an area's issue, the repositories it changes in merge order (`repos`). The parts and who does each, the folder and the merge rule are not built yet. |
| Check | a plan | One "Done when" line from the issue and the test or command that proves it. It may also carry a command with an expected exit code, which the daemon runs itself in the author's work folder at the pull request's head commit before review (not built). |
| Pull request | an issue | The branch, CI, and the review rounds. |
| Review round | a pull request | The reviewer's blocking points and the author's replies. A dispute between them goes to an arbiter. Each time the pull request comes up for review it will get a new reviewer session, whose brief carries the plan's checks, the daemon's results for them, the earlier blocking points by number and the author's answer to each; today one session serves every round and the author's answers are not recorded. |
| Decision | an issue | A choice an agent made: the options it weighed, the one it took, why, who decided, and the point after which overturning it costs more. Work carries on at once; a person reads decisions afterwards and can overturn them. Today the only decisions are rulings at arbitration; a plain approval, `blocked` and `unneeded` will be decisions too (not built). |
| Merge record | an issue | What the daemon ran and saw when the issue merged: the commit, the checks and their results, the files changed, the review rounds and whether an arbiter ruled, each agent session's id, runtime, model and brief version, what the daemon said on the issue and whether it was heard, and whether the daemon merged it or found it merged. The daemon writes it; no agent does (not built). |
| Message | an agent | Something said, on the route its kind has (`docs/architecture.md`, "How things are said"). The daemon to an agent: a prompt typed through the runner. An agent to another: a claim the daemon carries into the next brief, or a mike question. A person to an agent: words typed in its terminal. Today the first exists, a person can type in an agent's terminal, and the daemon carries a reviewer's points and an arbiter's reason to the author. Marking the daemon's text as its own, storing it with a version, the author's answer to each point, a mike question and `temet-work tell` are not built. |
| Event | everything | What happened, with its time, in one append-only log. State is built from events. |

## The loop an issue goes through

A group takes an issue; the refiner writes its plan; an author works on it in a work folder
and opens a pull request; a reviewer reviews it in rounds, and an arbiter settles a dispute;
CI and the checks pass; it merges, deploys and is checked in a real session; the issue
closes and the group takes the next one. Each choice along the way is a decision a person
can overturn, and overturns change the rules agents work by.

Two additions to this loop are decided and not built (`docs/architecture.md`, "What the
autonomy findings change"). Before the review, the daemon runs the command of each check
that has one and sends a failure straight back to the author. An agent that cannot do its
step, or finds the issue needs no change, says so with `blocked` or `unneeded`; `status`
lists the issue with the reason and the group takes the next one.

## Where each object shows in the app

| Column | Objects |
|---|---|
| Left: spans every project | projects, groups and their agents, machines and limits, the style guide, rules |
| Middle: the selected project | milestones, issues grouped by milestone, stage or group, work folders |
| Right: what was clicked | an issue's plan, pull request, review rounds and decisions; or an agent's terminal and decisions |
