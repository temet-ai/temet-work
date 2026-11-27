//! The loop's state as plain text, for the command line, for agents and for
//! tests: one section per kind of object, one line per object.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::model::{AgentState, Role, Stage};
use crate::world::World;

/// A stage in a word or two.
pub fn stage(stage: Stage) -> String {
    match stage {
        Stage::New => "new".into(),
        Stage::Refining => "refining".into(),
        Stage::Ready => "ready".into(),
        Stage::Working => "working".into(),
        Stage::InReview { round } => format!("in review, round {round}"),
        Stage::Fixing { round } => format!("fixing, round {round}"),
        Stage::Arbitration { round } => format!("with the arbiter after round {round}"),
        Stage::Merging => "merging".into(),
        Stage::Deploying => "deploying".into(),
        Stage::Verifying => "verifying".into(),
        Stage::Done => "done".into(),
    }
}

/// An agent's state in a few words.
pub fn agent_state(state: &AgentState) -> String {
    match state {
        AgentState::Working { doing } if doing.is_empty() => "working".into(),
        AgentState::Working { doing } => format!("working: {doing}"),
        AgentState::Idle => "idle".into(),
        AgentState::Blocked { reason } => format!("blocked: {reason}"),
        AgentState::Errored { line, .. } if line.is_empty() => "error".into(),
        AgentState::Errored { line, .. } => format!("error: {line}"),
        AgentState::Ended => "ended".into(),
    }
}

/// The whole state, with what is wrong now. Done issues are counted, not listed.
pub fn text(world: &World, problems: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Projects");
    let project = |out: &mut String, p: &crate::model::Project, indent: &str| {
        let from = if p.tracker.is_empty() {
            "issues from its area".to_string()
        } else {
            format!("{} in {}", p.name, p.tracker)
        };
        let _ = writeln!(out, "{indent}{}: {from}", p.id);
    };
    for a in world.areas.values() {
        let from = if a.tracker.is_empty() {
            String::new()
        } else {
            format!(", issues from {} in {}", a.tracker_name, a.tracker)
        };
        let _ = writeln!(out, "  {} ({}){from}", a.id, a.name);
        for p in a.projects.iter().filter_map(|id| world.projects.get(id)) {
            project(&mut out, p, "    ");
        }
    }
    for p in world.projects.values() {
        if !world.areas.values().any(|a| a.projects.contains(&p.id)) {
            project(&mut out, p, "  ");
        }
    }
    let _ = writeln!(out, "Groups");
    for g in world.groups.values() {
        let projects: Vec<String> = g.projects.iter().map(ToString::to_string).collect();
        let milestones: Vec<String> = g.milestones.iter().map(ToString::to_string).collect();
        let scope = if milestones.is_empty() {
            "every milestone".to_string()
        } else {
            milestones.join(", ")
        };
        let _ = writeln!(
            out,
            "  {} ({}): {}; {scope}; refines {} at a time",
            g.id,
            g.name,
            projects.join(", "),
            g.refining_at_once
        );
    }
    let _ = writeln!(out, "Issues");
    let mut done = 0;
    for i in world.issues.values() {
        if i.stage == Stage::Done {
            done += 1;
            continue;
        }
        let milestone = i
            .milestone
            .as_ref()
            .map_or_else(String::new, |m| format!(" [{m}]"));
        // An issue that changes several repositories shows where each pull request is.
        let pulls = if i.pulls.len() > 1 {
            let each: Vec<String> = world
                .repos_of(i)
                .iter()
                .filter_map(|r| i.pulls.get(r).map(|n| (r, n)))
                .map(|(r, n)| {
                    let merged = if i.merged.contains(r) { " merged" } else { "" };
                    format!("{r} #{n}{merged}")
                })
                .collect();
            format!(" ({})", each.join(", "))
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  {}  {}  {}{milestone}{pulls}",
            i.id,
            stage(i.stage),
            i.title
        );
    }
    if done > 0 {
        let _ = writeln!(out, "  and {done} done");
    }
    waiting(&mut out, world);
    let _ = writeln!(out, "Agents");
    for a in world.agents.values() {
        let issue = a
            .issue
            .as_ref()
            .map_or_else(String::new, |i| format!(" on {i}"));
        let _ = writeln!(
            out,
            "  {}{issue}: {} (on {})",
            a.id,
            agent_state(&a.state),
            a.machine
        );
    }
    let _ = writeln!(out, "Decisions");
    for d in world.decisions.values() {
        let chosen = d.options.get(d.chosen).map_or("?", String::as_str);
        let overturned = d
            .overturned_to
            .and_then(|o| d.options.get(o).map(|text| (o, text)))
            .map_or_else(String::new, |(o, text)| {
                format!(
                    "; overturned to option {} ({text}): {}",
                    o + 1,
                    d.overturn_reason
                )
            });
        let options: Vec<String> = d
            .options
            .iter()
            .enumerate()
            .map(|(n, o)| format!("{}={o}", n + 1))
            .collect();
        let _ = writeln!(
            out,
            "  {} {}: {} [{}] chose {chosen}, by {}: {}{overturned}",
            d.id.0,
            d.issue,
            d.question,
            options.join(", "),
            d.by,
            d.reason
        );
    }
    let _ = writeln!(out, "Rules");
    for r in world
        .rules
        .values()
        .filter(|r| r.state != crate::model::RuleState::Dropped)
    {
        let _ = writeln!(
            out,
            "  {} {}: {} (from decision {} on {}, decided by {}, overturned by {})",
            r.id.0,
            r.state.name(),
            r.text,
            r.decision.0,
            r.issue,
            r.decided_by,
            r.overturned_by
        );
    }
    let _ = writeln!(out, "Settings");
    for ((scope, role, key), e) in &world.settings {
        let role = role.map_or_else(String::new, |r| format!(", {}", r.name()));
        let _ = writeln!(out, "  {scope}{role}: {key} = {}, set by {}", e.value, e.by);
    }
    let _ = writeln!(out, "Problems");
    for (from, what) in problems {
        let _ = writeln!(out, "  {from}: {what}");
    }
    out
}

/// The issues waiting for a work folder on the machine a setting names; no
/// section when there are none.
fn waiting(out: &mut String, world: &World) {
    let waits = world.machine_waits();
    if waits.is_empty() {
        return;
    }
    let _ = writeln!(out, "Waiting");
    for w in waits {
        let agent = if w.role == Role::Author {
            String::new()
        } else {
            format!(" for its {}", w.role.name())
        };
        let none = w.none_in.map_or_else(String::new, |repo| {
            format!(" ({repo} has no work folder there)")
        });
        let _ = writeln!(
            out,
            "  {} waiting for a work folder on {}{agent}{none}",
            w.issue, w.machine
        );
    }
}

/// The settings in effect over `target` for `role`, one per line with the level
/// that set each, under a line naming the target and the role.
pub fn settings(
    world: &World,
    target: &crate::settings::Target,
    role: Option<crate::model::Role>,
) -> Result<String, String> {
    let scope = world.scope_of(target)?;
    let head = role.map_or_else(|| target.to_string(), |r| format!("{target}, {}", r.name()));
    Ok(format!("{head}\n{}", lines(world, &scope, role)))
}

/// The settings in effect, as lines indented two spaces.
fn lines(
    world: &World,
    scope: &crate::settings::Scope,
    role: Option<crate::model::Role>,
) -> String {
    let rows = world.in_effect(scope, role);
    let key = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let value = rows
        .iter()
        .map(|(_, r)| r.value.to_string().len())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (k, r) in rows {
        let _ = writeln!(
            out,
            "  {k:key$}  {:value$}  {}",
            r.value.to_string(),
            r.source()
        );
    }
    out
}

/// One issue in full: where it is, its plan, its pull request, what it is to fix,
/// and its agents.
pub fn issue(world: &World, id: &crate::ids::IssueId) -> Option<String> {
    let i = world.issues.get(id)?;
    let mut out = String::new();
    let _ = writeln!(out, "{}  {}", i.id, i.title);
    let _ = writeln!(out, "Stage: {}", stage(i.stage));
    if let Some(m) = &i.milestone {
        let _ = writeln!(out, "Milestone: {m}");
    }
    if let Some(g) = &i.group {
        let _ = writeln!(out, "Group: {g}");
    }
    if !i.depends_on.is_empty() {
        let deps: Vec<String> = i
            .depends_on
            .iter()
            .map(|d| {
                let at = world
                    .issues
                    .get(d)
                    .map_or("unknown".to_string(), |x| stage(x.stage));
                format!("{d} ({at})")
            })
            .collect();
        let _ = writeln!(out, "Depends on: {}", deps.join(", "));
    }
    let folders: Vec<String> = i
        .folder
        .iter()
        .chain(&i.other_folders)
        .map(ToString::to_string)
        .collect();
    if !folders.is_empty() {
        let label = if folders.len() > 1 {
            "Folders"
        } else {
            "Folder"
        };
        let _ = writeln!(out, "{label}: {}", folders.join(", "));
    }
    // One line per repository, in the order they merge.
    for repo in world
        .repos_of(i)
        .iter()
        .chain(i.pulls.keys().filter(|r| !world.repos_of(i).contains(r)))
    {
        if let Some(number) = i.pulls.get(repo) {
            let state = if i.merged.contains(repo) {
                ", merged"
            } else {
                ""
            };
            let _ = writeln!(out, "Pull request: {repo} #{number}{state}");
        }
    }
    if let Some(plan) = &i.plan {
        let _ = writeln!(out, "Plan:");
        for c in &plan.checks {
            let _ = writeln!(out, "  - {} (shown by: {})", c.done_when, c.proven_by);
        }
    }
    if !i.fix.is_empty() {
        let _ = writeln!(out, "To fix:");
        for p in &i.fix {
            let _ = writeln!(out, "  - {p}");
        }
    }
    let _ = writeln!(out, "Agents:");
    for a in world
        .agents
        .values()
        .filter(|a| a.issue.as_ref() == Some(id))
    {
        let _ = writeln!(
            out,
            "  {} ({}): {}",
            a.id,
            a.role.name(),
            agent_state(&a.state)
        );
    }
    let role = crate::settings::role_at(i.stage);
    let for_role = role.map_or_else(String::new, |r| format!(" for the {}", r.name()));
    let _ = writeln!(out, "Settings{for_role}:");
    out.push_str(&lines(
        world,
        &crate::settings::Scope::Task(id.clone()),
        role,
    ));
    Some(out)
}

#[cfg(test)]
mod settings {
    use super::*;
    use crate::event::Event;
    use crate::ids::{IssueId, ProjectId};
    use crate::settings::{Choice, Scope, Target, Value};

    fn world() -> World {
        let mut w = World::default();
        w.catalog.insert(
            "runtime".into(),
            Choice {
                key: "runtime".into(),
                allowed: vec!["alpha".into(), "beta".into()],
                default: Some("alpha".into()),
            },
        );
        let id = IssueId::new("radar", "44");
        let set = |scope, key: &str, value: &str, by: &str| Event::SettingSet {
            scope,
            role: None,
            key: key.into(),
            value: Value::Text(value.into()),
            by: by.into(),
        };
        for e in [
            Event::ProjectAdded {
                project: ProjectId::new("radar"),
                tracker: "t".into(),
                name: "city/radar".into(),
                code_host: None,
                repo: None,
            },
            Event::IssueSynced {
                issue: id.clone(),
                title: "Sign in".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            },
            set(Scope::Global, "runtime", "alpha", "file"),
            set(Scope::Task(id.clone()), "model", "opus", "refiner-3"),
            Event::SettingSet {
                scope: Scope::Project(ProjectId::new("radar")),
                role: Some(Role::Reviewer),
                key: "merge".into(),
                value: Value::Text("rebase".into()),
                by: "person".into(),
            },
        ] {
            w.apply(&e);
        }
        w
    }

    #[test]
    fn lists_each_setting_in_effect_with_the_level_that_set_it() {
        let w = world();
        let target = Target::parse("radar#44");
        let want = "\
radar#44, author
  runtime        alpha   global
  model          opus    task, set by refiner-3
  permission     auto    built in
  review_rounds  3       built in
  ci             true    built in
  merge          squash  built in
  remind_after   10      built in
  reminders      2       built in
  starts         3       built in
  sync_every     60      built in
  ci_every       30      built in
";
        assert_eq!(
            settings(&w, &target, Some(Role::Author)).expect("a known issue"),
            want
        );
    }

    #[test]
    fn the_role_changes_what_is_listed() {
        let w = world();
        let for_reviewer = settings(&w, &Target::parse("radar"), Some(Role::Reviewer)).expect("ok");
        assert!(
            for_reviewer.starts_with("radar, reviewer\n"),
            "{for_reviewer}"
        );
        assert!(
            for_reviewer.contains("merge          rebase  project"),
            "{for_reviewer}"
        );
        let for_author = settings(&w, &Target::parse("radar"), Some(Role::Author)).expect("ok");
        assert!(
            for_author.contains("merge          squash  built in"),
            "{for_author}"
        );
        assert!(settings(&w, &Target::parse("nowhere"), None).is_err());
    }

    #[test]
    fn machine_is_listed_with_its_level_only_when_a_level_sets_it() {
        use crate::command::Command;
        let mut w = world();
        let target = Target::parse("radar#44");
        let listed = |w: &World| settings(w, &target, Some(Role::Author)).expect("known");
        assert!(!listed(&w).contains("machine"), "{}", listed(&w));
        for (at, role, value) in [
            ("radar", None, "local"),
            ("radar#44", None, "klatch"),
            ("radar#44", Some(Role::Author), "mac"),
        ] {
            let set = Command::Set {
                target: Target::parse(at),
                role,
                key: "machine".into(),
                value: value.into(),
            };
            for e in w.events(None, &set).expect("accepted") {
                w.apply(&e);
            }
        }
        assert!(
            listed(&w).contains("  machine        mac     task, set by person\n"),
            "{}",
            listed(&w)
        );
        let reviewer = settings(&w, &target, Some(Role::Reviewer)).expect("known");
        assert!(
            reviewer.contains("  machine        klatch  task, set by person\n"),
            "{reviewer}"
        );
        let project = settings(&w, &Target::parse("radar"), None).expect("known");
        assert!(
            project.contains("  machine        local   project, set by person\n"),
            "{project}"
        );
    }

    #[test]
    fn the_status_names_the_machine_an_issue_waits_for() {
        use crate::ids::{FolderId, GroupId, MachineId};
        use crate::model::Plan;
        let issue = |key: &str| IssueId::new("radar", key);
        let synced = |key: &str, depends_on: Vec<IssueId>| Event::IssueSynced {
            issue: issue(key),
            title: format!("Issue {key}"),
            milestone: None,
            depends_on,
            order: key.parse().unwrap_or_default(),
            closed: false,
        };
        let mac = |key: &str| Event::SettingSet {
            scope: Scope::Task(issue(key)),
            role: None,
            key: "machine".into(),
            value: Value::Text("mac".into()),
            by: "person".into(),
        };
        let planned = |key: &str| Event::PlanWritten {
            issue: issue(key),
            plan: Plan::default(),
        };
        let (mut w, _) = World::replay(&[
            Event::ProjectAdded {
                project: ProjectId::new("radar"),
                tracker: "t".into(),
                name: "city/radar".into(),
                code_host: None,
                repo: None,
            },
            Event::FolderAdded {
                folder: FolderId::new("radar-mac"),
                project: ProjectId::new("radar"),
                machine: MachineId::new("mac"),
                path: "/srv/radar".into(),
            },
            Event::GroupCreated {
                group: GroupId::new("watch"),
                name: "Night watch".into(),
                projects: vec![ProjectId::new("radar")],
                areas: Vec::new(),
                milestones: Vec::new(),
                refining_at_once: 3,
                model: None,
            },
            // radar#1 takes the Mac folder, radar#2 waits for it, radar#3 for radar#2.
            synced("1", vec![]),
            synced("2", vec![]),
            synced("3", vec![issue("2")]),
            mac("1"),
            mac("2"),
            mac("3"),
            planned("1"),
            planned("2"),
            planned("3"),
        ]);
        let want = "\
Projects
  radar: city/radar in t
Groups
  watch (Night watch): radar; every milestone; refines 3 at a time
Issues
  radar#1  working  Issue 1
  radar#2  ready  Issue 2
  radar#3  ready  Issue 3
Waiting
  radar#2 waiting for a work folder on mac
Agents
Decisions
Rules
Settings
  radar#1: machine = mac, set by person
  radar#2: machine = mac, set by person
  radar#3: machine = mac, set by person
Problems
";
        assert_eq!(text(&w, &BTreeMap::new()), want);
        // `set` accepts a name no folder uses, and the status says there is none.
        for (key, role) in [("2", None), ("1", Some(Role::Reviewer))] {
            let set = crate::command::Command::Set {
                target: Target::Issue(issue(key)),
                role,
                key: "machine".into(),
                value: "mac2".into(),
            };
            for e in w.events(None, &set).expect("accepted") {
                w.apply(&e);
            }
        }
        w.apply(&Event::PullRequestReady {
            issue: issue("1"),
            pull: Some("7".into()),
            pulls: Default::default(),
        });
        let status = text(&w, &BTreeMap::new());
        assert!(
            status.contains(
                "Waiting\n  \
                 radar#1 waiting for a work folder on mac2 for its reviewer (radar has no work folder there)\n  \
                 radar#2 waiting for a work folder on mac2 (radar has no work folder there)\nAgents\n"
            ),
            "{status}"
        );
    }

    #[test]
    fn the_issue_view_shows_the_settings_of_the_role_at_work_on_it() {
        let w = world();
        let text = issue(&w, &IssueId::new("radar", "44")).expect("known");
        assert!(text.contains("Settings for the refiner:\n"), "{text}");
        assert!(
            !text.contains("  model "),
            "the refiner's model is for the author, not the refiner: {text}"
        );
    }

    #[test]
    fn the_status_lists_each_event_with_who_made_it() {
        let w = world();
        let all = text(&w, &BTreeMap::new());
        assert!(
            all.contains("  global: runtime = alpha, set by file\n"),
            "{all}"
        );
        assert!(
            all.contains("  radar#44, author: model = opus, set by refiner-3\n"),
            "{all}"
        );
        assert!(
            all.contains("  radar, reviewer: merge = rebase, set by person\n"),
            "{all}"
        );
    }
}
