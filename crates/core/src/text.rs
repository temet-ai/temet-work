//! The loop's state as plain text, for the command line, for agents and for
//! tests: one section per kind of object, one line per object.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::IssueText;
use crate::ids::GroupId;
use crate::model::{AgentState, Issue, PointState, Review, Role, Stage};
use crate::placement::{Hold, StartWait, Unlisted, Why};
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

/// The reply to a person's accepted move of `issue`: the stage it is at now,
/// which is the next one when the loop took it at once (refining, or working),
/// and each pull request the move dropped, left open on the code host for the
/// person to close or reuse. `dropped` is what [`World::open_pulls`] gave for
/// the issue before the move.
pub fn moved(
    world: &World,
    issue: &crate::ids::IssueId,
    dropped: &[(crate::ids::ProjectId, String)],
) -> String {
    let at = world
        .issues
        .get(issue)
        .map_or_else(|| "gone".to_string(), |i| issue_stage(world, i));
    let head = format!("{issue} is {at}");
    let named = |(repo, number): &(crate::ids::ProjectId, String)| {
        if dropped.len() > 1 || *repo != issue.project {
            format!("{number} in {repo}")
        } else {
            number.clone()
        }
    };
    match dropped {
        [] => head,
        [one] => format!(
            "{head}; the move dropped its pull request {}, which is left open on the code host \
             for you to close or reuse",
            named(one)
        ),
        [first @ .., last] => format!(
            "{head}; the move dropped its pull requests {} and {}, which are left open on the \
             code host for you to close or reuse",
            first.iter().map(named).collect::<Vec<_>>().join(", "),
            named(last)
        ),
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

/// Where an agent works, after its id: ` on ISSUE`, with `, assigned by a person`
/// when a person put it there, or ` unassigned from ISSUE by a person`, with
/// `, stopping` until it has ended. Empty for an agent on no issue.
pub fn placed(a: &crate::model::Agent) -> String {
    let Some(issue) = &a.issue else {
        return String::new();
    };
    if a.unassigned {
        let stopping = if a.state == AgentState::Ended {
            ""
        } else {
            ", stopping"
        };
        return format!(" unassigned from {issue} by a person{stopping}");
    }
    if a.assigned() {
        return format!(" on {issue}, assigned by a person");
    }
    format!(" on {issue}")
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

/// The line that says the daemon is held, and since when; none while it is not.
pub fn held(world: &World) -> Option<String> {
    world.held.then(|| {
        let since = world
            .held_since
            .map(|at| format!(" since {}", crate::investigation::utc(at)))
            .unwrap_or_default();
        format!(
            "Held{since}: no issue is taken and no investigation starts; an issue with nothing \
             merged gets no new agent, fix, review or merge, and agents at work finish their turn. \
             temet-work resume lifts it."
        )
    })
}

/// What an agent's line says when a person stopped it, whatever state it is
/// in: `stopped by a person at TIME`; none when no person did.
pub fn agent_stopped(world: &World, agent: &crate::ids::AgentId) -> Option<String> {
    let at = world.person_stopped(agent)?;
    let at = at
        .map(|at| format!(" at {}", crate::investigation::utc(at)))
        .unwrap_or_default();
    Some(format!("stopped by a person{at}"))
}

/// What an issue's line says while a person has stopped an agent of it: the
/// agent, since when, and how the issue goes on; none while it is not stopped.
pub fn issue_stopped(world: &World, issue: &Issue) -> Option<String> {
    let agent = world.stopped_by(&issue.id)?;
    let since = world
        .person_stopped(agent)
        .flatten()
        .map(|at| format!(" since {}", crate::investigation::utc(at)))
        .unwrap_or_default();
    Some(format!(
        "stopped by a person ({agent}){since}: no agent starts on it until temet-work start {}",
        issue.id
    ))
}

/// The whole state, with what is wrong now. Done issues are counted, not listed.
pub fn text(world: &World, problems: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    if let Some(line) = held(world) {
        let _ = writeln!(out, "{line}");
    }
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
        let stopped = issue_stopped(world, i).map_or_else(String::new, |s| format!("; {s}"));
        let _ = writeln!(
            out,
            "  {}  {}  {}{milestone}{pulls}{stopped}",
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
        let stopped = agent_stopped(world, &a.id).map_or_else(String::new, |s| format!("; {s}"));
        let _ = writeln!(
            out,
            "  {}{}: {} (on {}){stopped}",
            a.id,
            placed(a),
            agent_state(&a.state),
            a.machine
        );
    }
    let _ = writeln!(out, "Decisions");
    // A later line of a decision's or a rule's text sits further in than a row,
    // so none reads as one.
    let inside = |text: &str| indented(text, 4);
    for d in world.decisions.values() {
        let options: Vec<String> = d.options.iter().map(|o| inside(o)).collect();
        let chosen = options.get(d.chosen).map_or("?", String::as_str);
        let overturned = d
            .overturned_to
            .and_then(|o| options.get(o).map(|text| (o, text)))
            .map_or_else(String::new, |(o, text)| {
                format!(
                    "; overturned to option {} ({text}): {}",
                    o + 1,
                    inside(&d.overturn_reason)
                )
            });
        let numbered: Vec<String> = options
            .iter()
            .enumerate()
            .map(|(n, o)| format!("{}={o}", n + 1))
            .collect();
        // What overturning it costs and the point after which it costs more; a
        // ruling from an older log has neither.
        let point = d.point.map(|p| {
            let passed = world.issues.get(&d.issue).is_some_and(|i| p.passed(i));
            format!(
                "point {}{}",
                p.name(),
                if passed { " (passed)" } else { "" }
            )
        });
        let costs: Vec<String> = d
            .cost
            .map(|c| format!("cost {}", c.name()))
            .into_iter()
            .chain(point)
            .collect();
        let costs = if costs.is_empty() {
            String::new()
        } else {
            format!("; {}", costs.join(", "))
        };
        let _ = writeln!(
            out,
            "  {} {}: {} [{}] chose {chosen}, by {}: {}{costs}{overturned}",
            d.id.0,
            d.issue,
            inside(&d.question),
            numbered.join(", "),
            d.by,
            inside(&d.reason)
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
            inside(&r.text),
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

/// Every issue a person closed in the projects and areas `group` takes issues
/// from, one to a line in the tracker's order, as its investigator's brief lists
/// them, each cleaned and any later line of it indented; one line saying so when
/// there is none. `None` when there is no such group.
pub fn closed(world: &World, group: &GroupId) -> Option<String> {
    let issues = world.closed_by_persons(world.groups.get(group)?);
    if issues.is_empty() {
        return Some(format!(
            "A person closed no issue in the projects and areas {group} takes issues from.\n"
        ));
    }
    Some(
        issues
            .into_iter()
            .map(|i| indented(&crate::brief::closed_issue(i), 4) + "\n")
            .collect(),
    )
}

/// The ready issues whose authors wait for a work folder on the machine a setting
/// names, then the ready ones that would not start once a folder is free, with
/// each reason; no section when there are neither.
fn waiting(out: &mut String, world: &World) {
    let machines = world.machine_waits();
    let starts: Vec<StartWait> = world
        .issues
        .values()
        .filter(|i| i.stage == Stage::Ready)
        .filter_map(|i| Some((world.groups.get(i.group.as_ref()?)?, i)))
        .flat_map(|(group, i)| world.start_waits(group, i))
        .collect();
    if machines.is_empty() && starts.is_empty() {
        return;
    }
    let _ = writeln!(out, "Waiting");
    for w in machines {
        let none = w.none_in.map_or_else(String::new, |repo| {
            format!(" ({repo} has no work folder there)")
        });
        let _ = writeln!(
            out,
            "  {} waiting for a work folder on {}{none}",
            w.issue, w.machine
        );
    }
    for w in starts {
        let _ = writeln!(out, "  {} {}", w.issue, start_wait(&w));
    }
}

/// Why a ready issue waits, after its key: `waiting on klatch#5, never synced`.
fn start_wait(w: &StartWait) -> String {
    let group = &w.group;
    let (on, through, hold) = match &w.why {
        Why::OutOfScope(unlisted) => {
            let what = match unlisted {
                Unlisted::Project(p) => format!("{p} is not a project of {group}"),
                Unlisted::Area(a) => format!("{a} is not an area of {group}"),
                Unlisted::Milestone(m) => format!("{m} is not a milestone of {group}"),
                Unlisted::NoMilestone => "it has no milestone".to_string(),
            };
            return format!("waiting, out of scope ({what})");
        }
        Why::On { on, through, hold } => (on, through, hold),
    };
    let through: Vec<String> = through.iter().map(ToString::to_string).collect();
    let through = match through.split_last() {
        None => String::new(),
        Some((only, [])) => format!(" through {only}"),
        Some((last, rest)) => format!(" through {} and {last}", rest.join(", ")),
    };
    let why = match hold {
        Hold::NeverSynced => "never synced".to_string(),
        Hold::OutOfScope(None) => "out of scope (no group covers it)".to_string(),
        Hold::OutOfScope(Some(m)) => {
            format!("out of scope ({m} is not a milestone of {group})")
        }
        Hold::NotDone(at) => format!("not done ({})", stage(*at)),
    };
    // A name nothing matches is shown as its "Depends on" line writes it.
    let on = on
        .unmatched()
        .map_or_else(|| on.to_string(), str::to_string);
    format!("waiting on {on}{through}, {why}")
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

/// The longest value listed in the value column.
const VALUE_COLUMN: usize = 40;

/// The settings in effect, as lines indented two spaces. A value of several lines,
/// or of more than [`VALUE_COLUMN`] characters, leaves the value column empty and
/// is printed below its key, indented further than the keys so that none of it
/// reads as one.
fn lines(
    world: &World,
    scope: &crate::settings::Scope,
    role: Option<crate::model::Role>,
) -> String {
    let rows: Vec<_> = world
        .in_effect(scope, role)
        .into_iter()
        .map(|(k, r)| (k, r.value.to_string(), r.source()))
        .collect();
    let in_column = |v: &str| !v.contains('\n') && v.chars().count() <= VALUE_COLUMN;
    let key = rows.iter().map(|(k, ..)| k.len()).max().unwrap_or(0);
    let value = rows
        .iter()
        .filter(|(_, v, _)| in_column(v))
        .map(|(_, v, _)| v.chars().count())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (k, v, source) in rows {
        let (here, below) = if in_column(&v) {
            (v.as_str(), None)
        } else {
            ("", Some(v.as_str()))
        };
        let _ = writeln!(out, "  {k:key$}  {here:value$}  {source}");
        for line in below.into_iter().flat_map(str::lines) {
            let line = format!("      {line}");
            let _ = writeln!(out, "{}", line.trim_end());
        }
    }
    out
}

/// `issue`'s finished review rounds, all of them and each one whole, in the
/// order they finished. Empty before the first round ends. [`issue`] prints it.
pub(crate) fn review_rounds(issue: &Issue) -> String {
    let blocks = round_blocks(issue, None);
    if blocks.is_empty() {
        return String::new();
    }
    format!("Review rounds:\n{}{}", blocks.concat(), after_rounds(issue))
}

/// One block of lines for each of `issue`'s finished review rounds: its verdict
/// and, for a block, each point with the author's reply to it and what the next
/// round made of it. Each point and reply is whole, or with `pointer` cut as
/// [`crate::fit::cut`] cuts it, as a brief prints it.
pub(crate) fn round_blocks(issue: &Issue, pointer: Option<&str>) -> Vec<String> {
    let said = |text: &str| match pointer {
        None => indented(text, 9),
        Some(pointer) => crate::fit::Item::new(text).printed(9, usize::MAX, pointer),
    };
    let block = |(at, r): (usize, &Review)| {
        let mut out = String::new();
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
                        PointState::Unknown => "marked with a word this build lacks",
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
                "    {}. {}\n       The author's reply: {}\n       State: {state}",
                n + 1,
                said(point),
                said(reply),
            );
        }
        out
    };
    issue.reviews.iter().enumerate().map(block).collect()
}

/// What is said after an issue's review rounds: for one with its arbiter, that
/// no round judged the last block. Empty otherwise.
pub(crate) fn after_rounds(issue: &Issue) -> String {
    // The latest block is the one that sent an issue to its arbiter.
    if matches!(issue.stage, Stage::Arbitration { .. })
        && let Some(last) = issue
            .reviews
            .last()
            .filter(|r| r.blocks() && r.replies.is_empty())
    {
        return format!(
            "  No later round judged the points of round {}, the last block, and the author has not \
             replied to it because review rounds ran out before the author's next ready.\n",
            last.round
        );
    }
    String::new()
}

/// `text` with each line after its first indented `by` spaces, so that none of
/// them reads as a line of the list it is printed in, and with the controls
/// [`clean`] drops dropped.
pub(crate) fn indented(text: &str, by: usize) -> String {
    lined(&clean(text), by)
}

/// `text`, which [`clean`] has cleaned, with each line after its first indented
/// `by` spaces.
pub(crate) fn lined(text: &str, by: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines.join(&format!("\n{:by$}", ""))
}

/// `text` with every control character dropped but a line feed and a tab, and
/// each other line break made a line feed: a carriage return, alone or before a
/// line feed, NEL (U+0085), and the line and paragraph separators (U+2028 and
/// U+2029). The controls are those Unicode names: U+0000 to U+001F, DEL and
/// U+0080 to U+009F, among them ESC, NUL, VT, FF and BS. Text an agent or a
/// person sends is stored this way, and text from an older log, or from a code
/// host, is printed this way.
pub(crate) fn clean(text: &str) -> String {
    cleaned(text).collect()
}

/// The characters of `text` as [`clean`] leaves them, one at a time.
pub(crate) fn cleaned(text: &str) -> impl Iterator<Item = char> + '_ {
    let mut chars = text.chars().peekable();
    std::iter::from_fn(move || {
        loop {
            match chars.next()? {
                '\r' if chars.peek() == Some(&'\n') => {}
                '\r' | '\u{85}' | '\u{2028}' | '\u{2029}' => return Some('\n'),
                c @ ('\n' | '\t') => return Some(c),
                c if c.is_control() => {}
                c => return Some(c),
            }
        }
    })
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
    if let Some(stopped) = issue_stopped(world, i) {
        let _ = writeln!(out, "Stopped: {stopped}");
    }
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
            let check = format!("{} (shown by: {})", c.done_when, c.proven_by);
            let _ = writeln!(out, "  - {}", indented(&check, 4));
        }
    }
    if !i.person_checks.is_empty() {
        let _ = writeln!(out, "Person's checks:");
        for c in &i.person_checks {
            let check = if c.proven_by.trim().is_empty() {
                c.done_when.clone()
            } else {
                format!("{} (shown by: {})", c.done_when, c.proven_by)
            };
            let _ = writeln!(out, "  - {}", indented(&check, 4));
        }
    }
    if !i.fix.is_empty() {
        let _ = writeln!(out, "To fix:");
        for p in &i.fix {
            let _ = writeln!(out, "  - {}", indented(p, 4));
        }
    }
    out.push_str(&review_rounds(i));
    let _ = writeln!(out, "Agents:");
    for a in world
        .agents
        .values()
        .filter(|a| a.issue.as_ref() == Some(id))
    {
        let stopped = agent_stopped(world, &a.id).map_or_else(String::new, |s| format!("; {s}"));
        // Only a person's assign or unassign is said: every agent listed is on
        // the issue.
        let by_a_person = if a.assigned() || a.unassigned {
            placed(a)
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  {} ({}){by_a_person}: {}{stopped}",
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

/// What a person's refine of `id` did, from the world `before` it and the
/// world after: the new title, the person's checks, and for a send the stage,
/// the agents it stopped and each pull request it dropped from the record,
/// which stays open on the code host.
pub fn refined(before: &World, world: &World, id: &crate::ids::IssueId) -> String {
    let (Some(was), Some(now)) = (before.issues.get(id), world.issues.get(id)) else {
        return format!("there is no issue {id}");
    };
    let mut said = Vec::new();
    if now.title != was.title {
        said.push(format!("its title is now \"{}\" in its tracker", now.title));
    }
    if now.person_checks != was.person_checks {
        said.push(match now.person_checks.len() {
            0 => "it has no person's checks now".to_string(),
            1 => "it has 1 person's check".to_string(),
            n => format!("it has {n} person's checks"),
        });
    }
    if now.moved != was.moved {
        said.push(format!(
            "it is {}, and a refiner starts with a new brief",
            stage(now.stage)
        ));
        let stopped: Vec<String> = before
            .agents
            .values()
            .filter(|a| a.issue.as_ref() == Some(id) && a.state != AgentState::Ended)
            .map(|a| a.id.to_string())
            .collect();
        if !stopped.is_empty() {
            said.push(format!(
                "{} {} stopped",
                and_list(&stopped),
                if stopped.len() == 1 { "is" } else { "are" }
            ));
        }
        let left: Vec<String> = before
            .pulls_of(was)
            .into_iter()
            .filter(|(repo, number)| now.pulls.get(*repo) != Some(*number))
            .map(|(repo, number)| format!("{repo} #{number}"))
            .collect();
        if !left.is_empty() {
            let (what, are, them) = if left.len() == 1 {
                ("pull request", "is", "it")
            } else {
                ("pull requests", "are", "them")
            };
            said.push(format!(
                "{what} {} {are} left open on the code host and dropped from the record; close \
                 {them} there if no longer wanted",
                and_list(&left),
            ));
        }
    }
    if said.is_empty() {
        return format!("nothing changed on {id}");
    }
    format!("{id}: {}", said.join("; "))
}

/// `items` joined as a list in words: `a`, `a and b`, `a, b and c`.
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
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
  watch (Night watch): radar; every milestone; refines 3 at a time; 1 in its queue, investigates below 2
Issues
  radar#1  working  Issue 1
  radar#2  ready  Issue 2
  radar#3  ready  Issue 3
Waiting
  radar#2 waiting for a work folder on mac
  radar#3 waiting on radar#2, not done (ready)
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
        // Only authors wait: the reviewer runs in the readers' folder on mac2.
        assert!(
            status.contains(
                "Waiting\n  \
                 radar#2 waiting for a work folder on mac2 (radar has no work folder there)\n  \
                 radar#3 waiting on radar#2, not done (ready)\nAgents\n"
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

    /// Both listings of `w`'s issue: `temet-work settings`, then `temet-work issue`.
    fn listings(w: &World) -> [String; 2] {
        let id = IssueId::new("radar", "44");
        [
            settings(w, &Target::Issue(id.clone()), Some(Role::Author)).expect("a known issue"),
            issue(w, &id, &unread()).expect("a known issue"),
        ]
    }

    /// The test world with `key` set to `value` for its project, by a person.
    fn with_setting(key: &str, value: &str) -> World {
        let mut w = world();
        let radar = Scope::Project(ProjectId::new("radar"));
        w.put(&radar, None, key, &text_of(value), "person");
        w
    }

    /// `listing` taken apart at the row of `key`: that row, the lines printed below
    /// it with their indent removed, and every other line as it was.
    fn take_row<'a>(listing: &'a str, key: &str) -> (&'a str, Vec<&'a str>, String) {
        let start = format!("  {key} ");
        let (mut row, mut below, mut others) = ("", Vec::new(), String::new());
        for line in listing.split_inclusive('\n') {
            let bare = line.trim_end_matches('\n');
            if line.starts_with(&start) {
                row = bare;
            } else if !row.is_empty() && (bare.is_empty() || bare.starts_with("      ")) {
                below.push(bare.strip_prefix("      ").unwrap_or(bare));
            } else {
                others.push_str(line);
            }
        }
        (row, below, others)
    }

    #[test]
    fn a_multi_line_deploy_is_printed_below_its_key_and_widens_no_row() {
        let command = "git pull --ff-only &&\n  make install &&\n  make smoke &&\n  make restart";
        let without = listings(&world());
        // The key column is `review_rounds`, the value column `squash`.
        let widest_gap = " ".repeat("review_rounds".len() + "squash".len() + 4);
        for (before, after) in without
            .iter()
            .zip(listings(&with_setting("deploy", command)))
        {
            let (row, below, others) = take_row(&after, "deploy");
            assert_eq!(
                row, "  deploy                 project, set by person",
                "{after}"
            );
            assert_eq!(below.join("\n"), command, "{after}");
            assert_eq!(&others, before);
            for line in after.lines() {
                assert!(!line.ends_with(' '), "{line:?} in {after}");
                assert!(
                    !line.contains(&format!("{widest_gap} ")),
                    "{line:?} in {after}"
                );
            }
        }
        // A line break at the end adds no line, and a blank line gets no indent.
        for (value, want) in [
            ("make deploy\n", vec!["make deploy"]),
            (
                "make build  \r\n\r\nmake deploy",
                vec!["make build", "", "make deploy"],
            ),
        ] {
            for after in listings(&with_setting("deploy", value)) {
                let (row, below, _) = take_row(&after, "deploy");
                assert!(row.ends_with("  project, set by person"), "{after}");
                assert_eq!(below, want, "{after}");
                assert!(!after.lines().any(|l| l.ends_with(' ')), "{after}");
            }
        }
    }

    #[test]
    fn a_value_over_the_limit_is_printed_below_its_key() {
        let long = format!("/srv/{}", "r".repeat(36));
        assert_eq!(long.chars().count(), 41);
        let without = listings(&world());
        for (before, after) in without.iter().zip(listings(&with_setting("run_in", &long))) {
            let (row, below, others) = take_row(&after, "run_in");
            assert_eq!(
                row, "  run_in                 project, set by person",
                "{after}"
            );
            assert_eq!(below, [long.as_str()], "{after}");
            assert_eq!(&others, before);
        }
        // The limit counts characters, so the two bytes of the u-umlaut are one.
        let fits = format!("/srv/\u{fc}berwald/{}", "r".repeat(26));
        assert_eq!(fits.chars().count(), 40);
        for after in listings(&with_setting("run_in", &fits)) {
            let (row, below, _) = take_row(&after, "run_in");
            assert_eq!(
                row,
                format!("  run_in         {fits}  project, set by person"),
                "{after}"
            );
            assert!(below.is_empty(), "{after}");
            assert!(
                after.contains(&format!("  merge          {:40}  built in\n", "squash")),
                "{after}"
            );
        }
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
            // An open dependency in scope, which counts in its place.
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
                 ankh#1 waiting on klatch#3, out of scope (no group covers it)\n  \
                 ankh#4 waiting on ankh#2, not done (ready)\n\
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
mod closed_issues {
    use super::*;
    use crate::event::Event;
    use crate::ids::{AgentId, IssueId, ProjectId};
    use crate::{Command, World};

    fn group(id: &str) -> GroupId {
        GroupId::new(id)
    }

    /// Project ankh with the group watch, and klatch with the group guild.
    fn world() -> World {
        let project = |id: &str| Event::ProjectAdded {
            project: ProjectId::new(id),
            tracker: "fake".into(),
            name: format!("ankh-morpork/{id}"),
            code_host: None,
            repo: None,
        };
        let takes = |id: &str, project: &str| Event::GroupCreated {
            group: group(id),
            name: "Night watch".into(),
            projects: vec![ProjectId::new(project)],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        };
        let events = [
            project("ankh"),
            project("klatch"),
            takes("watch", "ankh"),
            takes("guild", "klatch"),
        ];
        World::replay(&events).0
    }

    /// Syncs `ankh#key` as the tracker lists it, `closed` or open.
    fn sync(w: &mut World, key: u64, title: &str, order: u64, closed: bool) {
        w.apply(&Event::IssueSynced {
            issue: IssueId::new("ankh", key.to_string()),
            title: title.into(),
            milestone: None,
            depends_on: Vec::new(),
            order,
            closed,
        });
    }

    #[test]
    fn closed_lists_every_issue_a_person_closed_in_the_briefs_order() {
        let mut w = world();
        // The tracker's order runs against the keys'. Each is a copy of one a
        // person closed, since 2,200 syncs would take seconds.
        for is_closed in [false, true] {
            sync(&mut w, 3, "Wind the clock", 3, is_closed);
        }
        let one = w.issues.remove(&IssueId::new("ankh", "3")).expect("closed");
        let mut expected = Vec::new();
        for n in 0..1100 {
            let mut issue = one.clone();
            issue.id = IssueId::new("ankh", (2100 - n).to_string());
            issue.title = format!("{:-<100}", format!("Wind {} ", issue.id));
            issue.order = n;
            expected.push(format!("{} {}", issue.id, issue.title));
            w.issues.insert(issue.id.clone(), issue);
        }
        // Closed by the loop after verifying.
        sync(&mut w, 4, "Sweep the yard", 2000, false);
        let swept = IssueId::new("ankh", "4");
        w.issues.get_mut(&swept).expect("held").stage = Stage::Verifying;
        w.apply(&Event::Verified {
            issue: swept,
            passed: true,
            output: String::new(),
        });
        // Closed and then reopened.
        for is_closed in [false, true, false] {
            sync(&mut w, 8, "Paint the door", 2001, is_closed);
        }

        let text = closed(&w, &group("watch")).expect("the group");
        assert_eq!(text.lines().collect::<Vec<_>>(), expected);
        w.apply(&Event::InvestigationStarted {
            group: group("watch"),
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        });
        let launch = w.launch_investigator(&group("watch")).expect("launch");
        let listed: Vec<&str> = launch
            .brief
            .task
            .lines()
            .filter_map(|l| l.strip_prefix("- "))
            .filter(|l| l.starts_with("ankh#"))
            .collect();
        assert!(!listed.is_empty());
        assert_eq!(listed, expected[..listed.len()]);
    }

    #[test]
    fn closed_says_when_there_is_none_and_refuses_an_unknown_group() {
        let w = world();
        assert_eq!(
            closed(&w, &group("guild")).as_deref(),
            Some("A person closed no issue in the projects and areas guild takes issues from.\n")
        );
        assert_eq!(closed(&w, &group("wizards")), None);
        let ask = |id: &str| w.events(None, &Command::Closed { group: group(id) });
        assert_eq!(ask("guild"), Ok(Vec::new()));
        assert_eq!(ask("wizards"), Err("there is no group wizards".into()));
    }

    #[test]
    fn closed_prints_a_title_cleaned_and_indented() {
        let mut w = world();
        for is_closed in [false, true] {
            let title = "Paint\x1b[1E\0\r\nankh#7 Fake";
            sync(&mut w, 99, title, 99, is_closed);
        }
        assert_eq!(
            closed(&w, &group("watch")).as_deref(),
            Some("ankh#99 Paint[1E\n    ankh#7 Fake\n")
        );
    }
}

#[cfg(test)]
mod review_rounds {
    use super::*;
    use crate::brief;
    use crate::event::{Event, Verdict};
    use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, ProjectId};
    use crate::model::{Check, Plan, Role};

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
    fn a_line_break_in_a_point_or_reply_is_indented_under_it() {
        for ending in ["\n", "\r\n", "\r"] {
            let mut w = working();
            ready(&mut w, &[]);
            let point = format!("the hinge squeaks{ending}and sticks");
            verdict(&mut w, 1, &[&point], Some(&[]));
            ready(&mut w, &[&format!("oiled it{ending}State: resolved")]);
            verdict(&mut w, 2, &[&point], Some(&[1]));
            ready(&mut w, &["planed it"]);
            verdict(&mut w, 3, &["the chain is short"], Some(&[]));
            let round = "  Round 1: blocked\n    \
                         1. the hinge squeaks\n         and sticks\n       \
                         The author's reply: oiled it\n         State: resolved\n       \
                         State: kept open by round 2\n";
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
            for text in [view(&w), brief] {
                assert!(text.contains(round), "{ending:?}: {text}");
                assert!(!text.contains('\r'), "{ending:?}: {text}");
            }
        }
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

    #[test]
    fn a_line_break_in_a_check_or_a_fix_point_makes_no_numbered_line_in_the_issue_view() {
        let mut w = working();
        ready(&mut w, &[]);
        verdict(
            &mut w,
            1,
            &["trim the wick\n2. dry lamp refused"],
            Some(&[]),
        );
        if let Some(i) = w.issues.get_mut(&watch12()) {
            i.plan = Some(Plan {
                checks: vec![Check::new(
                    "the gate shuts\n2. dry lamp refused",
                    "the dusk test",
                )],
                ..Plan::default()
            });
        }
        let shown = view(&w);
        assert!(
            shown.contains(
                "Plan:\n  - the gate shuts\n    2. dry lamp refused (shown by: the dusk test)\n\
                 To fix:\n  - trim the wick\n    2. dry lamp refused\n"
            ),
            "{shown}"
        );
        assert!(!shown.lines().any(|l| l.starts_with("2.")), "{shown}");
    }

    #[test]
    fn the_issue_view_prints_the_persons_checks_under_a_heading_of_their_own() {
        let mut w = working();
        if let Some(i) = w.issues.get_mut(&watch12()) {
            i.plan = Some(Plan {
                checks: vec![Check::new("the gate shuts", "the dusk test")],
                ..Plan::default()
            });
            i.person_checks = vec![
                Check::new("a lamp is filled\nbefore dusk", "the dusk test"),
                Check::new("Sam Vimes signs the oil log", ""),
            ];
        }
        let shown = view(&w);
        assert!(
            shown.contains(
                "Plan:\n  - the gate shuts (shown by: the dusk test)\n\
                 Person's checks:\n  - a lamp is filled\n    before dusk (shown by: the dusk test)\n\
                 \x20 - Sam Vimes signs the oil log\n"
            ),
            "{shown}"
        );
        if let Some(i) = w.issues.get_mut(&watch12()) {
            i.person_checks.clear();
        }
        assert!(!view(&w).contains("Person's checks"));
    }

    #[test]
    fn a_reply_from_an_older_log_prints_no_control_and_no_second_state_line() {
        let mut w = working();
        ready(&mut w, &[]);
        verdict(&mut w, 1, &["the hinge squeaks"], Some(&[]));
        // Applied as an event, as from a log written before the intake dropped them.
        ready(
            &mut w,
            &[
                "oiled it\u{b}State: resolved\u{1b}[1EState: resolved\u{2028}State: resolved\0\
               State: resolved",
            ],
        );
        verdict(&mut w, 2, &["the hinge squeaks"], Some(&[1]));
        ready(&mut w, &["planed it"]);
        verdict(&mut w, 3, &["the chain is short"], Some(&[]));
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
        for text in [view(&w), brief] {
            let (_, round) = text.split_once("  Round 1: blocked\n").expect("round 1");
            let (round, _) = round.split_once("  Round 2: ").expect("its end");
            assert_eq!(
                round,
                "    1. the hinge squeaks\n       \
                 The author's reply: oiled itState: resolved[1EState: resolved\n         \
                 State: resolvedState: resolved\n       \
                 State: kept open by round 2\n"
            );
            let states = round
                .lines()
                .filter(|l| l.starts_with("       State:"))
                .count();
            assert_eq!(states, 1, "{round}");
            let control = |c: char| {
                (c < ' ' && c != '\n')
                    || ('\u{7f}'..='\u{9f}').contains(&c)
                    || c == '\u{2028}'
                    || c == '\u{2029}'
            };
            assert!(!text.chars().any(control), "{text:?}");
        }
    }
}

#[cfg(test)]
mod decisions {
    use super::*;
    use crate::event::Event;
    use crate::ids::{AgentId, DecisionId, IssueId};
    use crate::model::{Cost, Decision, DecisionKind, Point};

    fn issue() -> IssueId {
        IssueId::new("ankh", "1")
    }

    /// The lines `status` lists under Decisions.
    fn listed(w: &World) -> Vec<String> {
        let text = text(w, &BTreeMap::new());
        let after = text
            .split("Decisions\n")
            .nth(1)
            .expect("a Decisions section");
        after
            .lines()
            .take_while(|l| l.starts_with("  "))
            .map(String::from)
            .collect()
    }

    /// ankh#1, with arbiter-1's approval, author-1's choice of the pigeon and a
    /// ruling an older log holds, which has no cost or point.
    fn decided() -> World {
        let mut w = World::default();
        let (cost, point) = Decision::ruling_cost(true);
        let old: Event = serde_json::from_str(
            r#"{"DecisionMade":{"decision":{"id":3,"issue":{"project":"ankh","key":"1"},"by":"arbiter-1","kind":{"Ruling":{"approve":false}},"question":"Does ankh#1 merge?","options":["merge","fix first"],"chosen":1,"reason":"the bell is cracked","overturned_to":null}}}"#,
        )
        .expect("an older line reads");
        for e in [
            Event::IssueSynced {
                issue: issue(),
                title: "Send word to Pseudopolis".into(),
                milestone: None,
                depends_on: vec![],
                order: 1,
                closed: false,
            },
            Event::DecisionMade {
                decision: Decision {
                    id: DecisionId(1),
                    issue: issue(),
                    by: AgentId::new("arbiter-1"),
                    kind: DecisionKind::Ruling { approve: true },
                    question: "Does ankh#1 merge after 3 review rounds?".into(),
                    options: vec!["merge".into(), "fix first".into()],
                    chosen: 0,
                    reason: "it rings true".into(),
                    overturned_to: None,
                    overturn_reason: String::new(),
                    cost: Some(cost),
                    point: Some(point),
                },
            },
            Event::DecisionMade {
                decision: Decision {
                    id: DecisionId(2),
                    issue: issue(),
                    by: AgentId::new("author-1"),
                    kind: DecisionKind::Choice,
                    question: "Which way does the clacks message go?".into(),
                    options: vec!["clacks".into(), "pigeon".into(), "Post Office".into()],
                    chosen: 1,
                    reason: "the towers are down".into(),
                    overturned_to: None,
                    overturn_reason: String::new(),
                    cost: Some(Cost::Medium),
                    point: Some(Point::Merge),
                },
            },
            Event::DecisionOverturned {
                decision: DecisionId(2),
                to: 2,
                reason: "the Post Office delivers".into(),
                by: "person".into(),
            },
            old,
        ] {
            w.apply(&e);
        }
        w
    }

    #[test]
    fn each_decision_lists_its_cost_and_point_and_an_older_ruling_neither() {
        let mut w = decided();
        assert_eq!(
            listed(&w),
            [
                "  1 ankh#1: Does ankh#1 merge after 3 review rounds? [1=merge, 2=fix first] chose \
                 merge, by arbiter-1: it rings true; cost high, point merge",
                "  2 ankh#1: Which way does the clacks message go? [1=clacks, 2=pigeon, 3=Post \
                 Office] chose pigeon, by author-1: the towers are down; cost medium, point merge; \
                 overturned to option 3 (Post Office): the Post Office delivers",
                "  3 ankh#1: Does ankh#1 merge? [1=merge, 2=fix first] chose fix first, by \
                 arbiter-1: the bell is cracked",
            ]
        );
        // Merging is not past the merge; deploying, verifying and done are.
        let stage = |w: &mut World, s| w.issues.get_mut(&issue()).expect("synced").stage = s;
        stage(&mut w, Stage::Merging);
        assert!(listed(&w)[0].ends_with("; cost high, point merge"));
        for s in [Stage::Deploying, Stage::Verifying, Stage::Done] {
            stage(&mut w, s);
            let lines = listed(&w);
            assert!(
                lines[0].ends_with("; cost high, point merge (passed)"),
                "{lines:?}"
            );
            assert!(
                lines[1].contains("; cost medium, point merge (passed); overturned"),
                "{lines:?}"
            );
            assert!(lines[2].ends_with(": the bell is cracked"), "{lines:?}");
        }
    }
}

#[cfg(test)]
mod person_stops {
    use super::*;
    use crate::event::Event;
    use crate::ids::{AgentId, GroupId, IssueId, MachineId};

    fn started(agent: &str, role: Role) -> Event {
        Event::AgentStarted {
            agent: AgentId::new(agent),
            group: GroupId::new("watch"),
            role,
            issue: Some(IssueId::new("ankh", "1")),
            machine: MachineId::new("morpork"),
            folder: None,
            secret_digest: None,
            runtime: None,
            model: None,
        }
    }

    /// ankh#1 refining, its refiner working and an author idle, with the
    /// refiner stopped by a person at `at`.
    fn stopped(at: u64) -> World {
        let mut w = World::default();
        w.apply(&Event::IssueSynced {
            issue: IssueId::new("ankh", "1"),
            title: "Mend the clock".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        w.issues
            .get_mut(&IssueId::new("ankh", "1"))
            .expect("synced")
            .stage = Stage::Refining;
        w.apply(&started("refiner-1", Role::Refiner));
        w.apply(&started("author-1", Role::Author));
        w.apply(&Event::AgentChanged {
            agent: AgentId::new("refiner-1"),
            state: AgentState::Working {
                doing: "reading the clock".into(),
            },
            lost: false,
        });
        w.apply_at(
            at,
            &Event::AgentStoppedByPerson {
                agent: AgentId::new("refiner-1"),
            },
        );
        w
    }

    #[test]
    fn text_status_shows_the_agent_stopped_by_a_person_whatever_its_state_and_its_issue_stopped() {
        let at = 1_795_784_700_000;
        let w = stopped(at);
        let status = text(&w, &BTreeMap::new());
        assert!(
            status.contains(
                "refiner-1 on ankh#1: working: reading the clock (on morpork); stopped by a \
                 person at 2026-11-27 13:05 UTC"
            ),
            "{status}"
        );
        assert!(
            status.contains("author-1 on ankh#1: idle (on morpork)\n"),
            "{status}"
        );
        assert!(
            status.contains(
                "ankh#1  refining  Mend the clock; stopped by a person (refiner-1) since \
                 2026-11-27 13:05 UTC: no agent starts on it until temet-work start ankh#1"
            ),
            "{status}"
        );
        let shown = issue(
            &w,
            &IssueId::new("ankh", "1"),
            &Err("not read in this test".into()),
        )
        .expect("an issue");
        assert!(
            shown.contains("Stopped: stopped by a person (refiner-1) since 2026-11-27 13:05 UTC"),
            "{shown}"
        );
        assert!(
            shown.contains("refiner-1 (refiner): working: reading the clock; stopped by a person"),
            "{shown}"
        );

        // Started again: the issue's line says nothing, the agent's still does.
        let mut w = w;
        w.apply(&Event::IssueStartedByPerson {
            issue: IssueId::new("ankh", "1"),
        });
        let status = text(&w, &BTreeMap::new());
        assert!(
            status.contains("ankh#1  refining  Mend the clock\n"),
            "{status}"
        );
        assert!(status.contains("; stopped by a person at "), "{status}");
    }
}
