//! The loop's state as plain text, for the command line, for agents and for
//! tests: one section per kind of object, one line per object.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::IssueText;
use crate::model::{AgentState, Issue, PointState, Role, Stage};
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

/// An issue's stage, and "out of scope" when its group no longer covers it.
pub(crate) fn issue_stage(world: &World, issue: &Issue) -> String {
    let at = stage(issue.stage);
    if world.out_of_scope(issue) {
        format!("{at}, out of scope")
    } else {
        at
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
        let investigation = match world.investigations.get(&g.id) {
            Some(i) if i.running() => format!("; investigating: {}", i.agent),
            Some(i) => {
                let filed = match i.filed.len() {
                    1 => "1 issue".to_string(),
                    n => format!("{n} issues"),
                };
                let ended = i
                    .ended_ms
                    .map(crate::investigation::utc)
                    .unwrap_or_default();
                format!("; last investigated {ended}, filed {filed}")
            }
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "  {} ({}): {}; {scope}; refines {} at a time; {} in its queue, investigates below {}{investigation}",
            g.id,
            g.name,
            projects.join(", "),
            g.refining_at_once,
            world.queue(g),
            world.queue_low(g),
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
            issue_stage(world, i),
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

/// The issues waiting for a work folder on the machine a setting names, then the
/// ready ones waiting on a dependency out of scope; no section when there are
/// neither.
fn waiting(out: &mut String, world: &World) {
    let machines = world.machine_waits();
    let dependencies = world.dependency_waits();
    if machines.is_empty() && dependencies.is_empty() {
        return;
    }
    let _ = writeln!(out, "Waiting");
    for w in machines {
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
    for w in dependencies {
        let through: Vec<String> = w.through.iter().map(ToString::to_string).collect();
        let through = match through.split_last() {
            None => String::new(),
            Some((only, [])) => format!(" through {only}"),
            Some((last, rest)) => format!(" through {} and {last}", rest.join(", ")),
        };
        let why = w.left_out.map_or_else(
            || "no group covers it".to_string(),
            |m| format!("{m} is not a milestone of {}", w.group),
        );
        // A name nothing matches is shown as its "Depends on" line writes it.
        let on =
            w.on.unmatched()
                .map_or_else(|| w.on.to_string(), str::to_string);
        let _ = writeln!(
            out,
            "  {} waiting on {on}{through}, out of scope ({why})",
            w.issue
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

/// `issue`'s finished review rounds, in the order they finished: each one's
/// verdict and, for a block, each point with the author's reply to it and what
/// the next round made of it. Empty before the first round ends. An arbiter's
/// brief and [`issue`] both print it.
pub(crate) fn review_rounds(issue: &Issue) -> String {
    if issue.reviews.is_empty() {
        return String::new();
    }
    let mut out = String::from("Review rounds:\n");
    for (at, r) in issue.reviews.iter().enumerate() {
        let verdict = if r.approve { "approved" } else { "blocked" };
        let _ = writeln!(out, "  Round {}: {verdict}", r.round);
        // A verdict judges the entry before it, unless it is a round 1's
        // (`Issue::round_before`).
        let next = issue.reviews.get(at + 1).filter(|n| n.round > 1);
        for (n, point) in r.points.iter().enumerate() {
            let reply = r.replies.get(n).map_or("none on record", String::as_str);
            // A round from an older log holds no state, and one that no later
            // round judged is recorded as open: neither was kept open.
            let state = match (r.states.get(n), next) {
                (None, _) => "not on record".to_string(),
                (Some(_), None) => "not judged by a later round".into(),
                (Some(state), Some(next)) => {
                    let judged = match state {
                        PointState::Open => "kept open",
                        PointState::Resolved => "resolved",
                    };
                    // A failed check or a conflict keeps the round's number.
                    let later = if next.round == r.round {
                        "the later "
                    } else {
                        ""
                    };
                    format!("{judged} by {later}round {}", next.round)
                }
            };
            let _ = writeln!(
                out,
                "    {}. {point}\n       The author's reply: {reply}\n       State: {state}",
                n + 1
            );
        }
    }
    // The latest block is the one that sent an issue to its arbiter.
    if matches!(issue.stage, Stage::Arbitration { .. })
        && let Some(last) = issue
            .reviews
            .last()
            .filter(|r| r.blocks() && r.replies.is_empty())
    {
        let _ = writeln!(
            out,
            "  No later round judged the points of round {}, the last block, and the author has not \
             replied to it because review rounds ran out before the author's next ready.",
            last.round
        );
    }
    out
}

/// One issue in full: where it is, its own text (`read`, or why there is none to
/// show), its plan, its pull request, what it is to fix, its review rounds and
/// its agents.
pub fn issue(
    world: &World,
    id: &crate::ids::IssueId,
    read: &Result<IssueText, String>,
) -> Option<String> {
    let i = world.issues.get(id)?;
    let mut out = String::new();
    let _ = writeln!(out, "{}  {}", i.id, i.title);
    let _ = writeln!(out, "Stage: {}", issue_stage(world, i));
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
    for (repo, number) in world.pulls_of(i) {
        let state = if i.merged.contains(repo) {
            ", merged"
        } else {
            ""
        };
        let _ = writeln!(out, "Pull request: {repo} #{number}{state}");
    }
    own_text(&mut out, read);
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
    out.push_str(&review_rounds(i));
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

/// An issue's own text between a line that says where it came from and a line
/// that ends it. Text that is empty, or was not read, is one line saying so: a
/// reader must not take the rest of the output for the issue's text.
fn own_text(out: &mut String, read: &Result<IssueText, String>) {
    let _ = match read {
        Ok(t) if t.text.trim().is_empty() => {
            writeln!(out, "The issue's text is empty: {}.", t.from)
        }
        Ok(t) => writeln!(
            out,
            "Text of {}:\n{}\nEnd of the issue's text.",
            t.from,
            t.text.trim_end()
        ),
        Err(why) => {
            let why: Vec<&str> = why.split_whitespace().collect();
            writeln!(out, "The issue's text was not read: {}", why.join(" "))
        }
    };
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
                key: "permission".into(),
                value: Value::Text("plan".into()),
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
            for_reviewer.contains("permission     plan    project"),
            "{for_reviewer}"
        );
        let for_author = settings(&w, &Target::parse("radar"), Some(Role::Author)).expect("ok");
        assert!(
            for_author.contains("permission     auto    built in"),
            "{for_author}"
        );
        assert!(settings(&w, &Target::parse("nowhere"), None).is_err());
    }

    #[test]
    fn the_listing_shows_a_setting_not_read_per_role_without_the_role() {
        use crate::model::Stage;
        let mut w = world();
        let id = IssueId::new("radar", "44");
        let radar = Scope::Project(ProjectId::new("radar"));
        let every = [
            ("review_rounds", Value::Number(2), Value::Number(1)),
            ("ci", Value::Flag(false), Value::Flag(true)),
            ("merge", text_of("rebase"), text_of("merge")),
            ("deploy", text_of("make deploy"), text_of("make other")),
            ("verify", text_of("make smoke"), text_of("make other")),
            ("run_in", text_of("/srv/radar"), text_of("/srv/other")),
            ("sync_every", Value::Number(90), Value::Number(1)),
            ("ci_every", Value::Number(45), Value::Number(1)),
        ];
        // Even a role's value that is nearer than the one for every role is passed
        // over.
        for (key, for_all, for_reviewers) in &every {
            w.put(&Scope::Global, None, key, for_all, crate::settings::FILE);
            w.put(
                &radar,
                Some(Role::Reviewer),
                key,
                for_reviewers,
                crate::settings::FILE,
            );
        }
        w.put(
            &radar,
            Some(Role::Reviewer),
            "model.alpha",
            &text_of("haiku"),
            crate::settings::FILE,
        );
        w.issues.get_mut(&id).expect("synced").stage = Stage::InReview { round: 1 };
        let target = Target::parse("radar");
        let listed = settings(&w, &target, Some(Role::Reviewer)).expect("known");
        let view = issue(&w, &id, &unread()).expect("known");
        assert!(view.contains("Settings for the reviewer:\n"), "{view}");
        for (key, for_all, _) in &every {
            let want = Some(format!("{for_all} global"));
            assert_eq!(row(&listed, key), want, "{key}: {listed}");
            assert_eq!(row(&view, key), want, "{key}: {view}");
        }
        assert_eq!(
            w.setting_number(&id, Some(Role::Reviewer), "review_rounds"),
            w.setting_number(&id, None, "review_rounds")
        );
        assert_eq!(w.setting_number(&id, None, "review_rounds"), 2);
        assert_eq!(row(&listed, "model").as_deref(), Some("haiku project"));
    }

    fn text_of(value: &str) -> Value {
        Value::Text(value.into())
    }

    /// The words that follow `key` on the line of `listing` that lists it.
    fn row(listing: &str, key: &str) -> Option<String> {
        listing.lines().find_map(|line| {
            let mut words = line.split_whitespace();
            (words.next() == Some(key)).then(|| words.collect::<Vec<_>>().join(" "))
        })
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
                queue_low: None,
                investigate_every: None,
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
  watch (Night watch): radar; every milestone; refines 3 at a time; 2 in its queue, investigates below 2
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
            replies: Vec::new(),
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
    fn the_model_listed_is_the_one_the_roles_runtime_runs() {
        let mut w = world();
        let target = Target::parse("radar#44");
        let set = |w: &mut World, scope, role, key: &str, value: &str| {
            w.apply(&Event::SettingSet {
                scope,
                role,
                key: key.into(),
                value: Value::Text(value.into()),
                by: "person".into(),
            });
        };
        set(&mut w, Scope::Global, None, "model", "sonnet");
        set(
            &mut w,
            Scope::Global,
            Some(Role::Reviewer),
            "runtime",
            "beta",
        );
        let listed = |w: &World, role| settings(w, &target, Some(role)).expect("known");
        let author = listed(&w, Role::Author);
        assert!(
            author.contains("  model          opus    task, set by refiner-3\n"),
            "{author}"
        );
        let reviewer = listed(&w, Role::Reviewer);
        assert!(reviewer.contains("  runtime        beta    global, set by person\n"));
        assert!(
            !reviewer.contains("  model "),
            "no model reaches a beta reviewer: {reviewer}"
        );
        set(
            &mut w,
            Scope::Global,
            Some(Role::Reviewer),
            "model.beta",
            "gpt-x",
        );
        let reviewer = listed(&w, Role::Reviewer);
        assert!(
            reviewer.contains("  model          gpt-x   global, set by person\n"),
            "{reviewer}"
        );
        assert_eq!(listed(&w, Role::Author), author);
        let status = text(&w, &BTreeMap::new());
        assert!(
            status.contains("  global, reviewer: model.beta = gpt-x, set by person\n"),
            "{status}"
        );
        assert!(
            status.contains("  global: model.alpha = sonnet, set by person\n"),
            "{status}"
        );
    }

    /// What a build that reads no issue text gives the view.
    fn unread() -> Result<IssueText, String> {
        Err("not read here".into())
    }

    #[test]
    fn the_issue_view_shows_its_text_above_the_plan_or_one_line_saying_why_not() {
        let mut w = world();
        let id = IssueId::new("radar", "44");
        w.issues.get_mut(&id).expect("synced").plan = Some(crate::model::Plan::default());
        let view = |text: &str| {
            let read = Ok(IssueText {
                from: "issue 44 of city/radar at the last sync".into(),
                text: text.into(),
            });
            issue(&w, &id, &read).expect("known")
        };
        let shown = view("Sign in once.\n\nPlan:\nnone yet\n");
        assert!(
            shown.contains(
                "\nText of issue 44 of city/radar at the last sync:\n\
                 Sign in once.\n\nPlan:\nnone yet\nEnd of the issue's text.\nPlan:\n"
            ),
            "{shown}"
        );
        let empty = view(" \n");
        assert!(
            empty.contains(
                "\nThe issue's text is empty: issue 44 of city/radar at the last sync.\n"
            ),
            "{empty}"
        );
        assert!(!empty.contains("Text of"), "{empty}");
        let why = Err("city/radar: signed out\n  run the sign-in again".into());
        let missed = issue(&w, &id, &why).expect("known");
        assert!(
            missed.contains(
                "\nThe issue's text was not read: city/radar: signed out run the sign-in again\n\
                 Plan:\n"
            ),
            "{missed}"
        );
        assert!(!missed.contains("Text of"), "{missed}");
        assert!(missed.starts_with("radar#44  Sign in\n"), "{missed}");
    }

    #[test]
    fn the_issue_view_shows_the_settings_of_the_role_at_work_on_it() {
        let w = world();
        let text = issue(&w, &IssueId::new("radar", "44"), &unread()).expect("known");
        assert!(text.contains("Settings for the refiner:\n"), "{text}");
        assert!(
            !text.contains("  model "),
            "the refiner's model is for the author, not the refiner: {text}"
        );
    }

    #[test]
    fn the_status_shows_a_group_investigating_and_one_that_investigated() {
        use crate::ids::{AgentId, GroupId, MachineId};
        let group = |id: &str| Event::GroupCreated {
            group: GroupId::new(id),
            name: id.to_uppercase(),
            projects: vec![ProjectId::new("radar")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: Some(3),
            investigate_every: None,
        };
        let started = |group: &str, agent: &str, at_ms| {
            [
                Event::InvestigationStarted {
                    group: GroupId::new(group),
                    agent: AgentId::new(agent),
                    at_ms,
                },
                Event::AgentStarted {
                    agent: AgentId::new(agent),
                    group: GroupId::new(group),
                    role: Role::Investigator,
                    issue: None,
                    machine: MachineId::new("local"),
                    folder: None,
                    secret_digest: None,
                    runtime: None,
                    model: None,
                },
            ]
        };
        let filed = |key: &str| Event::IssueFiled {
            group: GroupId::new("guard"),
            agent: AgentId::new("investigator-1"),
            issue: IssueId::new("radar", key),
            title: format!("Issue {key}"),
            milestone: None,
        };
        let mut w = world();
        for e in [group("guard"), group("watch")]
            .into_iter()
            .chain(started("guard", "investigator-1", 0))
            .chain([filed("7"), filed("8")])
            .chain([Event::InvestigationEnded {
                group: GroupId::new("guard"),
                at_ms: 1_795_784_700_000,
                filed: 2,
            }])
            .chain(started("watch", "investigator-2", 1_795_784_700_000))
        {
            w.apply(&e);
        }
        let all = text(&w, &BTreeMap::new());
        assert!(
            all.contains(
                "Groups\n  \
                 guard (GUARD): radar; every milestone; refines 1 at a time; 1 in its queue, \
                 investigates below 3; last investigated 2026-11-27 13:05 UTC, filed 2 issues\n  \
                 watch (WATCH): radar; every milestone; refines 1 at a time; 0 in its queue, \
                 investigates below 3; investigating: investigator-2\n"
            ),
            "{all}"
        );
        assert!(all.contains("  investigator-2: idle (on local)\n"), "{all}");
    }

    #[test]
    fn the_status_counts_only_the_queue_in_scope() {
        use crate::ids::{GroupId, MilestoneId};
        let ankh = ProjectId::new("ankh");
        let mut w = World::default();
        w.apply(&Event::ProjectAdded {
            project: ankh.clone(),
            tracker: "t".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        });
        w.apply(&Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ankh],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("First")],
            refining_at_once: 1,
            model: None,
            queue_low: Some(8),
            investigate_every: None,
        });
        let sync = |key: u64, milestone: &str| Event::IssueSynced {
            issue: IssueId::new("ankh", key.to_string()),
            title: format!("Issue {key}"),
            milestone: Some(MilestoneId::new(milestone)),
            depends_on: Vec::new(),
            order: key,
            closed: false,
        };
        // Twelve refined in First; the last ten are then moved to Later.
        for key in 1..=12 {
            w.apply(&sync(key, "First"));
            w.apply(&Event::PlanWritten {
                issue: IssueId::new("ankh", key.to_string()),
                plan: Default::default(),
            });
        }
        for key in 3..=12 {
            w.apply(&sync(key, "Later"));
        }
        let all = text(&w, &BTreeMap::new());
        assert!(
            all.contains(
                "\n  watch (Night watch): ankh; First; refines 1 at a time; 2 in its queue, \
                 investigates below 8\n"
            ),
            "{all}"
        );
    }

    #[test]
    fn the_status_names_an_out_of_scope_dependency() {
        use crate::ids::{GroupId, MachineId, MilestoneId};
        let ankh = ProjectId::new("ankh");
        let mut w = World::default();
        for (id, name) in [("ankh", "city/ankh"), ("klatch", "desert/klatch")] {
            w.apply(&Event::ProjectAdded {
                project: ProjectId::new(id),
                tracker: "t".into(),
                name: name.into(),
                code_host: None,
                repo: None,
            });
        }
        w.apply(&Event::FolderAdded {
            folder: crate::ids::FolderId::new("f1"),
            project: ankh.clone(),
            machine: MachineId::new("morpork"),
            path: "/srv/ankh/f1".into(),
        });
        w.apply(&Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ankh],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("First")],
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        });
        // ankh#2 waits for a folder on a machine that has none.
        w.apply(&Event::SettingSet {
            scope: crate::settings::Scope::Task(IssueId::new("ankh", "2")),
            role: Some(Role::Author),
            key: "machine".into(),
            value: crate::settings::Value::Text("mac".into()),
            by: "person".into(),
        });
        let later = IssueId::new("ankh", "100");
        let uncovered = IssueId::new("klatch", "3");
        let sync = |issue: &IssueId, milestone: Option<&str>, depends_on: Vec<IssueId>| {
            Event::IssueSynced {
                issue: issue.clone(),
                title: format!("Issue {}", issue.key),
                milestone: milestone.map(MilestoneId::new),
                depends_on,
                order: 1,
                closed: false,
            }
        };
        w.apply(&sync(&later, Some("Later"), Vec::new()));
        w.apply(&sync(&uncovered, None, Vec::new()));
        for (key, depends_on) in [
            ("1", vec![later.clone(), uncovered.clone()]),
            ("2", Vec::new()),
            // An open dependency in scope.
            ("4", vec![IssueId::new("ankh", "2")]),
        ] {
            let issue = IssueId::new("ankh", key);
            w.apply(&sync(&issue, Some("First"), depends_on));
            w.apply(&Event::PlanWritten {
                issue,
                plan: Default::default(),
            });
        }
        let all = text(&w, &BTreeMap::new());
        assert!(
            all.contains(
                "Waiting\n  \
                 ankh#2 waiting for a work folder on mac (ankh has no work folder there)\n  \
                 ankh#1 waiting on ankh#100, out of scope (Later is not a milestone of watch)\n  \
                 ankh#1 waiting on klatch#3, out of scope (no group covers it)\n\
                 Agents\n"
            ),
            "{all}"
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
            all.contains("  radar#44, author: model.alpha = opus, set by refiner-3\n"),
            "{all}"
        );
        assert!(
            all.contains("  radar, reviewer: permission = plan, set by person\n"),
            "{all}"
        );
    }
}

#[cfg(test)]
mod review_rounds {
    use super::*;
    use crate::brief;
    use crate::event::{Event, Verdict};
    use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, ProjectId};
    use crate::model::{Check, Plan};

    fn watch12() -> IssueId {
        IssueId::new("watch", "12")
    }

    /// `watch#12` with its author at work, and no review round yet.
    fn working() -> World {
        let group = GroupId::new("night-watch");
        let started = |agent: &str, role| Event::AgentStarted {
            agent: AgentId::new(agent),
            group: group.clone(),
            role,
            issue: Some(watch12()),
            machine: MachineId::new("pseudopolis"),
            folder: None,
            secret_digest: None,
            runtime: None,
            model: None,
        };
        let events = [
            Event::ProjectAdded {
                project: ProjectId::new("watch"),
                tracker: "fake".into(),
                name: "ankh-morpork/watch".into(),
                code_host: None,
                repo: None,
            },
            Event::FolderAdded {
                folder: FolderId::new("yard-1"),
                project: ProjectId::new("watch"),
                machine: MachineId::new("pseudopolis"),
                path: "/srv/pseudopolis-yard/watch".into(),
            },
            Event::GroupCreated {
                group: group.clone(),
                name: "Night Watch".into(),
                projects: vec![ProjectId::new("watch")],
                areas: Vec::new(),
                milestones: Vec::new(),
                refining_at_once: 1,
                model: None,
                queue_low: None,
                investigate_every: None,
            },
            Event::IssueSynced {
                issue: watch12(),
                title: "Shut the gate at dusk".into(),
                milestone: None,
                depends_on: Vec::new(),
                order: 12,
                closed: false,
            },
            started("refiner-1", Role::Refiner),
            Event::PlanWritten {
                issue: watch12(),
                plan: Plan {
                    checks: vec![Check::new("the gate shuts at dusk", "the dusk test")],
                    ..Plan::default()
                },
            },
            started("author-2", Role::Author),
        ];
        World::replay(&events).0
    }

    fn ready(w: &mut World, replies: &[&str]) {
        w.apply(&Event::PullRequestReady {
            issue: watch12(),
            pull: Some("41".into()),
            pulls: BTreeMap::new(),
            replies: replies.iter().map(ToString::to_string).collect(),
        });
    }

    /// The verdict on `round`: an approval with no `points`, else a block. `open`
    /// is none in a log from before points had states.
    fn verdict(w: &mut World, round: u8, points: &[&str], open: Option<&[usize]>) {
        let verdict = if points.is_empty() {
            Verdict::Approve
        } else {
            Verdict::Block {
                points: points.iter().map(ToString::to_string).collect(),
            }
        };
        w.apply(&Event::ReviewFinished {
            issue: watch12(),
            round,
            verdict,
            failed: None,
            open: open.map(<[usize]>::to_vec),
        });
    }

    fn view(w: &World) -> String {
        let unread = Err("not read here".into());
        issue(w, &watch12(), &unread).expect("known")
    }

    #[test]
    fn the_issue_view_lists_every_review_round_with_each_reply_and_state() {
        let mut w = working();
        assert!(!view(&w).contains("Review rounds:"), "{}", view(&w));
        ready(&mut w, &[]);
        assert!(!view(&w).contains("Review rounds:"), "{}", view(&w));
        verdict(
            &mut w,
            1,
            &["the hinge squeaks", "the bar is bent"],
            Some(&[]),
        );
        // Listed while the author fixes it, after what it is to fix.
        let fixing = view(&w);
        assert!(
            fixing.contains(
                "To fix:\n  - the hinge squeaks\n  - the bar is bent\n\
                 Review rounds:\n  Round 1: blocked\n    \
                 1. the hinge squeaks\n       The author's reply: none on record\n       \
                 State: not judged by a later round\n    \
                 2. the bar is bent\n       The author's reply: none on record\n       \
                 State: not judged by a later round\n\
                 Agents:\n"
            ),
            "{fixing}"
        );
        ready(&mut w, &["oiled it", "the bar holds"]);
        // Round 2 keeps the bar open.
        verdict(
            &mut w,
            2,
            &["the lock sticks", "the bar is bent"],
            Some(&[2]),
        );
        ready(&mut w, &["filed the lock", "straightened the bar"]);
        // Round 3 keeps the lock open.
        verdict(
            &mut w,
            3,
            &["the chain is short", "the lock sticks"],
            Some(&[1]),
        );
        assert_eq!(w.issues[&watch12()].stage, Stage::Arbitration { round: 3 });
        let rounds = "Review rounds:\n  \
             Round 1: blocked\n    \
             1. the hinge squeaks\n       The author's reply: oiled it\n       \
             State: resolved by round 2\n    \
             2. the bar is bent\n       The author's reply: the bar holds\n       \
             State: kept open by round 2\n  \
             Round 2: blocked\n    \
             1. the lock sticks\n       The author's reply: filed the lock\n       \
             State: kept open by round 3\n    \
             2. the bar is bent\n       The author's reply: straightened the bar\n       \
             State: resolved by round 3\n  \
             Round 3: blocked\n    \
             1. the chain is short\n       The author's reply: none on record\n       \
             State: not judged by a later round\n    \
             2. the lock sticks\n       The author's reply: none on record\n       \
             State: not judged by a later round\n  \
             No later round judged the points of round 3, the last block, and the author has \
             not replied to it because review rounds ran out before the author's next ready.\n";
        let shown = view(&w);
        assert!(shown.contains(&format!("{rounds}Agents:\n")), "{shown}");
        // The arbiter's brief carries the same block.
        let brief = brief::write(
            Role::Arbiter,
            &w.issues[&watch12()],
            None,
            &[],
            brief::Workplace::default(),
            &[],
            &[],
        )
        .task;
        assert!(
            brief.contains(&format!("{rounds}This folder is not yours")),
            "{brief}"
        );
    }

    #[test]
    fn the_issue_view_lists_an_approved_round_and_a_repeated_round() {
        let mut w = working();
        ready(&mut w, &[]);
        verdict(&mut w, 1, &["the hinge squeaks"], Some(&[]));
        ready(&mut w, &["oiled it"]);
        verdict(&mut w, 2, &[], Some(&[]));
        // A failed check undoes the approval and keeps the round's number.
        w.apply(&Event::ChecksFinished {
            issue: watch12(),
            passed: false,
            failing: vec!["lint".into()],
        });
        ready(&mut w, &[]);
        verdict(&mut w, 2, &["the bar is bent"], Some(&[]));
        let shown = view(&w);
        assert!(
            shown.contains(
                "Review rounds:\n  \
                 Round 1: blocked\n    \
                 1. the hinge squeaks\n       The author's reply: oiled it\n       \
                 State: resolved by round 2\n  \
                 Round 2: approved\n  \
                 Round 2: blocked\n    \
                 1. the bar is bent\n       The author's reply: none on record\n       \
                 State: not judged by a later round\n\
                 Agents:\n"
            ),
            "{shown}"
        );
        // A block that a second review of its round judged is named as such.
        ready(&mut w, &["straightened it"]);
        verdict(&mut w, 3, &["the chain is short"], Some(&[]));
        w.apply(&Event::ChecksFinished {
            issue: watch12(),
            passed: false,
            failing: Vec::new(),
        });
        ready(&mut w, &["a longer chain"]);
        verdict(&mut w, 3, &[], Some(&[]));
        let shown = view(&w);
        assert!(
            shown.contains(
                "  Round 3: blocked\n    \
                 1. the chain is short\n       The author's reply: a longer chain\n       \
                 State: resolved by the later round 3\n  \
                 Round 3: approved\n"
            ),
            "{shown}"
        );
    }

    #[test]
    fn the_issue_view_says_a_state_from_an_older_log_is_not_on_record() {
        let mut w = working();
        ready(&mut w, &[]);
        verdict(&mut w, 1, &["the hinge squeaks"], None);
        ready(&mut w, &[]);
        verdict(&mut w, 2, &["the bar is bent"], None);
        let shown = view(&w);
        assert!(
            shown.contains(
                "Review rounds:\n  \
                 Round 1: blocked\n    \
                 1. the hinge squeaks\n       The author's reply: none on record\n       \
                 State: not on record\n  \
                 Round 2: blocked\n    \
                 1. the bar is bent\n       The author's reply: none on record\n       \
                 State: not on record\n\
                 Agents:\n"
            ),
            "{shown}"
        );
    }
}
