//! The loop's state as plain text, for the command line, for agents and for
//! tests: one section per kind of object, one line per object.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::model::{AgentState, Stage};
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
        let _ = writeln!(
            out,
            "  {}  {}  {}{milestone}",
            i.id,
            stage(i.stage),
            i.title
        );
    }
    if done > 0 {
        let _ = writeln!(out, "  and {done} done");
    }
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
            .and_then(|o| d.options.get(o))
            .map_or_else(String::new, |o| format!("; overturned to {o}"));
        let _ = writeln!(
            out,
            "  {} {}: {} {chosen}, by {}: {}{overturned}",
            d.id.0, d.issue, d.question, d.by, d.reason
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
    if let Some(f) = &i.folder {
        let _ = writeln!(out, "Folder: {f}");
    }
    if let Some(p) = &i.pull {
        let _ = writeln!(out, "Pull request: {p}");
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
    use crate::model::Role;
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
    fn the_issue_view_shows_the_settings_of_the_role_at_work_on_it() {
        let w = world();
        let text = issue(&w, &IssueId::new("radar", "44")).expect("known");
        assert!(text.contains("Settings for the refiner:\n"), "{text}");
        assert!(
            text.contains("  model          opus    task, set by refiner-3\n"),
            "{text}"
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
            all.contains("  radar#44: model = opus, set by refiner-3\n"),
            "{all}"
        );
        assert!(
            all.contains("  radar, reviewer: merge = rebase, set by person\n"),
            "{all}"
        );
    }
}
