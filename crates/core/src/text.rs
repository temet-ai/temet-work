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
    for p in world.projects.values() {
        let _ = writeln!(out, "  {}: {} in {}", p.id, p.name, p.tracker);
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
    let _ = writeln!(out, "Problems");
    for (from, what) in problems {
        let _ = writeln!(out, "  {from}: {what}");
    }
    out
}
