//! Plumbing: the daemon's [`View`] mapped into the Project page's [`ProjectView`], and the page
//! told what the daemon's last answer was. It draws nothing.
//!
//! The mapping is plain functions of the view and the clock. A field the view has no data for is
//! left empty, and so is a part's row for it, rather than filled with a stand-in: the view holds
//! no machine slots or limits, a pull request's branch, lines or files, a plan's folder or merge
//! rule, or the time a decision was made. Where the view has a word the page has no word for,
//! the choice is made once, in [`stage`] and [`agent_status`].
//!
//! An issue is named by its whole id, `project#key`, everywhere the page shows it, as the design
//! system's data and temet-work's commands name it.

use std::collections::HashMap;
use std::path::Path;

use gpui::SharedString;
use temet_work_core::View;
use temet_work_core::ids::ProjectId;
use temet_work_core::model::{CheckState, Cost, Point};
use temet_work_core::view::{
    Agent, Decision, Group, Issue, Milestone as ViewMilestone, StageWord, StateWord, VerdictWord,
};

use crate::components::Intent;
use crate::components::icon::IconName;
use crate::components::stage_tag::Stage;
use crate::components::status_tag::Status;
use crate::composites::check_list::PlanCheck;
use crate::composites::decision_row::{Decision as RowDecision, DecisionOption, UndoCost};
use crate::composites::group_list::{GroupListAgent, GroupListGroup};
use crate::composites::issue_card::IssueCardAgent;
use crate::composites::issue_pane::{IssueDecision, IssuePlan, IssuePullRequest};
use crate::composites::issue_table::IssueSummary;
use crate::composites::needs_you::NeedsYouDecision;
use crate::composites::page_header::PageFact;
use crate::composites::problem_list::Problem;
use crate::composites::review_rounds::{ReviewPoint, ReviewRound};
use crate::composites::side_nav::{SideNavArea, SideNavProject};
use crate::composites::stage_board::{BoardIssue, BoardLane};
use crate::daemon::{Daemon, Problem as DaemonProblem};
use crate::pages::project::{
    FactValue, IssueDetail, IssueFact, Milestone, Notice, Project, ProjectView,
};

fn s(text: impl Into<String>) -> SharedString {
    SharedString::from(text.into())
}

/// The page's stage for the view's: each of the daemon's eleven words is its own stage. A word
/// this build lacks, from a newer daemon, reads as new, the first stage.
pub fn stage(word: StageWord) -> Stage {
    match word {
        StageWord::New | StageWord::Unknown => Stage::New,
        StageWord::Refining => Stage::Refining,
        StageWord::Ready => Stage::Ready,
        StageWord::Working => Stage::Working,
        StageWord::InReview => Stage::InReview,
        StageWord::Fixing => Stage::Fixing,
        StageWord::Arbitration => Stage::WithArbiter,
        StageWord::Merging => Stage::Merging,
        StageWord::Deploying => Stage::Deploying,
        StageWord::Verifying => Stage::Verifying,
        StageWord::Done => Stage::Done,
    }
}

/// The status an agent's state is drawn as. An ended agent is not listed, so it has none; a
/// state this build lacks reads as waiting.
pub fn agent_status(state: StateWord) -> Option<Status> {
    match state {
        StateWord::Working => Some(Status::Running),
        StateWord::Idle => Some(Status::Idle),
        StateWord::Blocked => Some(Status::Blocked),
        StateWord::Errored => Some(Status::Failed),
        StateWord::Unknown => Some(Status::Waiting),
        StateWord::Ended => None,
    }
}

/// A check's status; none when the log has none or holds a word this build lacks.
pub fn check_status(state: Option<CheckState>) -> Option<Status> {
    match state? {
        CheckState::Passed => Some(Status::Passed),
        CheckState::Running => Some(Status::Running),
        CheckState::Waiting => Some(Status::Waiting),
        CheckState::Failed => Some(Status::Failed),
        CheckState::Unknown => None,
    }
}

/// The cost to undo as the decision row draws it; none for a ruling from an older log.
pub fn undo_cost(cost: Option<Cost>) -> Option<UndoCost> {
    match cost? {
        Cost::High => Some(UndoCost::High),
        Cost::Medium => Some(UndoCost::Medium),
        Cost::Low => Some(UndoCost::Low),
        Cost::Unknown => None,
    }
}

/// The key of an issue id (`project#key`): what follows the `#`, such as its number in its
/// tracker.
fn number(id: &str) -> &str {
    id.split_once('#').map_or(id, |(_, key)| key)
}

/// The project of an issue id.
fn project_of(id: &str) -> &str {
    id.split_once('#').map_or("", |(project, _)| project)
}

/// "2026-12-12" as "12 Dec"; a date in another form is kept as it is.
pub fn due(date: &str) -> SharedString {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if let [_, month, day] = parts[..]
        && let (Ok(month), Ok(day)) = (month.parse::<usize>(), day.parse::<u32>())
        && (1..=12).contains(&month)
    {
        return s(format!("{day} {}", MONTHS[month - 1]));
    }
    s(date)
}

/// How long ago `then_ms` was at `now_ms`, such as "4 min ago"; "just now" under a minute and for
/// a time ahead of the clock.
pub fn ago(now_ms: u64, then_ms: u64) -> SharedString {
    let minutes = now_ms.saturating_sub(then_ms) / 60_000;
    s(match minutes {
        0 => "just now".to_string(),
        1..=59 => format!("{minutes} min ago"),
        60..=1439 => format!("{} h ago", minutes / 60),
        _ => format!("{} d ago", minutes / 1440),
    })
}

/// "role · runtime" for the agent `id`, as Needs you names who decided; the agent's id when the
/// view does not list it, and the role alone when it has no runtime.
fn who(agents: &[Agent], id: &str) -> SharedString {
    match agents.iter().find(|a| a.id.0 == id) {
        Some(a) => match &a.runtime {
            Some(runtime) => s(format!("{} \u{b7} {runtime}", a.role)),
            None => s(a.role.clone()),
        },
        None => s(id),
    }
}

/// An address without its scheme, as the page header shows it.
fn address(url: &str) -> SharedString {
    s(url.split_once("://").map_or(url, |(_, rest)| rest))
}

/// The host of an address, such as github.com.
fn host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split('/').next().unwrap_or(rest)
}

/// Where a project's issues are, as the page header says it: the tracker and the tracker's name
/// for the project ("GitHub grand-trunk/clacks"), else the address.
fn tracker_fact(url: &str, name: Option<&str>) -> SharedString {
    let tracker = match host(url) {
        "github.com" => "GitHub",
        "gitlab.com" => "GitLab",
        other => other,
    };
    match name {
        Some(name) => s(format!("{tracker} {name}")),
        None => address(url),
    }
}

/// The option a decision took.
fn decision_took(d: &Decision) -> SharedString {
    d.options
        .get(d.chosen.wrapping_sub(1))
        .map_or_else(|| s(d.question.clone()), |option| s(option.clone()))
}

/// The point after which overturning a decision costs more, while its issue is not past it.
fn overturn_before(d: &Decision) -> Option<SharedString> {
    match &d.point {
        Some(point) if !point.passed && point.at == Point::Merge => Some(s("merge")),
        _ => None,
    }
}

/// A decision as Needs you draws it, as on main: the short form, the option taken as its title,
/// who decided as "role · runtime", and the issue it is about.
fn need_decision(d: &Decision, agents: &[Agent]) -> RowDecision {
    RowDecision {
        question: None,
        took: decision_took(d),
        by: who(agents, &d.by.0),
        issue: Some(s(d.issue.clone())),
        at: None,
        cost: undo_cost(d.cost),
        weighed: Vec::new(),
        why: None,
        options: Vec::new(),
        overturn_before: match &d.point {
            Some(point) if !point.passed && point.at == Point::Merge => {
                Some(s("its issue's merge"))
            }
            _ => None,
        },
        overturned: None,
        overturned_to: None,
        overturn_reason: None,
    }
}

/// A decision as the issue's pane draws it: its question as the title, the option taken, the
/// others numbered from 1 as the daemon counts them with the taken one left out, why, and once
/// overturned, what it was switched to and why. Who decided is the agent's id.
pub fn pane_decision(d: &Decision) -> RowDecision {
    let options = d
        .options
        .iter()
        .enumerate()
        .filter(|(n, _)| n + 1 != d.chosen)
        .map(|(n, label)| DecisionOption {
            id: s((n + 1).to_string()),
            label: s(label.clone()),
        })
        .collect();
    let overturned_to = d.overturned_to.map(|option| {
        d.options
            .get(option.wrapping_sub(1))
            .map_or_else(|| s(option.to_string()), |o| s(o.clone()))
    });
    RowDecision {
        question: Some(s(d.question.clone())),
        took: decision_took(d),
        by: s(d.by.0.clone()),
        issue: None,
        at: None,
        cost: undo_cost(d.cost),
        weighed: Vec::new(),
        why: (!d.reason.is_empty()).then(|| s(d.reason.clone())),
        options,
        overturn_before: overturn_before(d),
        overturned: d.overturned_to.map(|_| s("Overturned")),
        overturned_to,
        overturn_reason: (d.overturned_to.is_some() && !d.overturn_reason.is_empty())
            .then(|| s(d.overturn_reason.clone())),
    }
}

/// The rank of a cost, the costliest first, a ruling with no cost last.
fn rank(cost: Option<UndoCost>) -> u8 {
    match cost {
        Some(UndoCost::High) => 0,
        Some(UndoCost::Medium) => 1,
        Some(UndoCost::Low) => 2,
        None => 3,
    }
}

/// The decisions that wait on a person, in the order Needs you lists them: those not overturned
/// on an issue still open and not yet past the point where the loop acts on an overturn, the
/// costliest to undo first, a ruling with no cost last, and the oldest first among equals.
pub fn waiting(view: &View) -> Vec<&Decision> {
    let mut waiting: Vec<&Decision> = view
        .decisions
        .iter()
        .filter(|d| d.overturned_to.is_none())
        .filter(|d| d.point.as_ref().is_none_or(|p| !p.passed))
        .filter(|d| view.issues.iter().any(|i| i.id == d.issue))
        .collect();
    waiting.sort_by_key(|d| (rank(undo_cost(d.cost)), d.id));
    waiting
}

/// How many decisions wait on a person on each issue, by its id.
fn open_decisions(view: &View) -> HashMap<&str, usize> {
    let mut open = HashMap::new();
    for d in waiting(view) {
        *open.entry(d.issue.as_str()).or_default() += 1;
    }
    open
}

/// The left column's projects: those in an area under it, those in no area at the top level, each
/// with the count of its decisions that wait on a person, but for the project on screen, whose
/// decisions Needs you counts.
fn projects(view: &View, shown: &ProjectId) -> (Vec<SideNavArea>, Vec<SideNavProject>) {
    let waiting = waiting(view);
    let mut areas: Vec<SideNavArea> = Vec::new();
    let mut loose = Vec::new();
    for project in &view.projects {
        let item = SideNavProject {
            id: s(project.id.0.clone()),
            name: s(project.id.0.clone()),
            icon: Some(IconName::GitRepo),
            waiting: if &project.id == shown {
                0
            } else {
                waiting
                    .iter()
                    .filter(|d| project_of(&d.issue) == project.id.0)
                    .count()
            },
        };
        let Some(area) = &project.area else {
            loose.push(item);
            continue;
        };
        match areas.iter_mut().find(|a| a.id.as_ref() == area.id.0) {
            Some(known) => known.projects.push(item),
            None => areas.push(SideNavArea {
                id: s(area.id.0.clone()),
                name: s(area.name.clone()),
                projects: vec![item],
            }),
        }
    }
    (areas, loose)
}

/// The groups that work on `project`.
fn groups_of<'a>(view: &'a View, project: &ProjectId) -> Vec<&'a Group> {
    view.groups
        .iter()
        .filter(|g| g.projects.contains(project))
        .collect()
}

/// Whether a group works on a milestone: it names it, or names none, which takes every one.
fn takes(group: &Group, milestone: &str) -> bool {
    group.milestones.is_empty() || group.milestones.iter().any(|m| m.0 == milestone)
}

/// The agents that have not ended, in the view's order.
fn running(view: &View) -> impl Iterator<Item = &Agent> {
    view.agents.iter().filter(|a| a.state != StateWord::Ended)
}

/// The agents that have not ended and are on `issue`.
fn running_on<'a>(view: &'a View, issue: &'a str) -> impl Iterator<Item = &'a Agent> {
    running(view).filter(move |a| a.issue.as_deref() == Some(issue))
}

/// The groups that work on `project`, each with its agents that have not ended. An agent's row
/// reads its id, then the issue it is on (or "free") and its model; its state, machine and model
/// show on hover. A group with none running says how many have ended.
fn groups(view: &View, project: &ProjectId) -> Vec<GroupListGroup> {
    groups_of(view, project)
        .into_iter()
        .map(|g| {
            let ended = view
                .agents
                .iter()
                .filter(|a| a.group == g.id && a.state == StateWord::Ended)
                .count();
            GroupListGroup {
                id: s(g.id.0.clone()),
                name: s(g.name.clone()),
                note: (!g.milestones.is_empty()).then(|| {
                    s(g.milestones
                        .iter()
                        .map(|m| m.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", "))
                }),
                agents: running(view)
                    .filter(|a| a.group == g.id)
                    .filter_map(|a| {
                        let status = if a.issue.is_some() {
                            agent_status(a.state)?
                        } else {
                            Status::Idle
                        };
                        let place = [
                            Some(a.issue.as_deref().unwrap_or("free")),
                            a.model.as_deref(),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" \u{b7} ");
                        let runs = [Some(a.machine.0.as_str()), a.model.as_deref()]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .join(" \u{b7} ");
                        let detail = [a.state_text.as_str(), runs.as_str()]
                            .into_iter()
                            .filter(|part| !part.is_empty())
                            .collect::<Vec<_>>()
                            .join(" \u{b7} ");
                        Some(GroupListAgent {
                            id: s(a.id.0.clone()),
                            role: s(a.id.0.clone()),
                            r#where: Some(s(place)),
                            detail: Some(s(detail)),
                            status,
                        })
                    })
                    .collect(),
                empty: Some(s(match ended {
                    0 => "No agents running".to_string(),
                    n => format!("No agents running \u{b7} {n} ended"),
                })),
            }
        })
        .collect()
}

/// How many of the project's issues are at each stage, every stage in the daemon's order; done
/// is the view's count of done issues, which it does not list.
fn stage_counts(view: &View, issues: &[&Issue]) -> Vec<(Stage, usize)> {
    Stage::ALL
        .into_iter()
        .map(|st| {
            let n = if st == Stage::Done {
                view.done
            } else {
                issues.iter().filter(|i| stage(i.stage) == st).count()
            };
            (st, n)
        })
        .collect()
}

fn summary(view: &View, issue: &Issue, now_ms: u64) -> IssueSummary {
    let agents: Vec<&str> = running_on(view, &issue.id)
        .map(|a| a.id.0.as_str())
        .collect();
    IssueSummary {
        key: s(issue.id.clone()),
        title: s(issue.title.clone()),
        stage: stage(issue.stage),
        group: issue.group.as_ref().map(|group| {
            view.groups
                .iter()
                .find(|g| &g.id == group)
                .map_or_else(|| s(group.0.clone()), |g| s(g.name.clone()))
        }),
        milestone: issue.milestone.as_ref().map(|m| s(m.0.clone())),
        agent: (!agents.is_empty()).then(|| s(agents.join(", "))),
        updated: issue.updated_ms.map_or_else(|| s(""), |t| ago(now_ms, t)),
    }
}

/// The facts the pane shows for an issue, the radar page's rows: Group, Waits for, Pull request
/// with its checks, and Tracker. Its milestone, agents and last change each have one place of
/// their own on the screen (the list or board, the left column, the row), so the pane leaves
/// them out. A row the view has nothing for is left out.
pub fn facts(view: &View, issue: &Issue) -> Vec<IssueFact> {
    let fact = |label: &str, value: FactValue| IssueFact {
        label: s(label),
        value,
    };
    let group = issue.group.as_ref().map(|id| {
        view.groups
            .iter()
            .find(|g| &g.id == id)
            .map_or_else(|| s(id.0.clone()), |g| s(g.name.clone()))
    });
    let mut out = vec![fact(
        "Group",
        FactValue::Text(group.unwrap_or_else(|| s("None yet"))),
    )];
    if !issue.depends_on.is_empty() {
        out.push(fact(
            "Waits for",
            FactValue::Keys(issue.depends_on.iter().map(|k| s(k.clone())).collect()),
        ));
    }
    if let Some(number) = issue.pulls.first().and_then(|p| p.number.parse().ok()) {
        out.push(fact(
            "Pull request",
            FactValue::Pull {
                number,
                checks: issue.checks_ok,
            },
        ));
    }
    let project = view
        .projects
        .iter()
        .find(|p| p.id.0 == project_of(&issue.id));
    let key = number(&issue.id);
    let text = match project.and_then(|p| p.tracker_name.as_deref()) {
        Some(name) => s(format!("{name}#{key}")),
        None => s(issue.id.clone()),
    };
    let url = project
        .and_then(|p| p.issues_url.as_deref())
        .map(|base| s(format!("{}/{key}", base.trim_end_matches('/'))));
    out.push(fact("Tracker", FactValue::Link { text, url }));
    out
}

/// An issue's pane: its facts, and what the view holds about its plan, pull request, reviews
/// and decisions. A tab whose data is empty is `None`, so the pane does not show it. Its
/// decisions come costliest to undo first.
fn detail(view: &View, issue: &Issue) -> IssueDetail {
    let plan = (!issue.checks.is_empty()).then(|| IssuePlan {
        folder: None,
        merge: None,
        checks: issue
            .checks
            .iter()
            .map(|c| PlanCheck {
                done_when: s(c.done_when.clone()),
                proven_by: s(c.proven_by.clone()),
                status: check_status(c.state),
                status_text: None,
            })
            .collect(),
    });
    // The first pull request: the page's pane draws one. CI is the issue's, so it is shown
    // beside a pull request only.
    let many = issue.pulls.len() > 1;
    let pull = issue.pulls.first().and_then(|p| {
        Some(IssuePullRequest {
            number: p.number.parse().ok()?,
            repo: many.then(|| s(p.repo.0.clone())),
            merged: Some(p.merged),
            branch: None,
            ci: Some(match issue.checks_ok {
                Some(true) => Status::Passed,
                Some(false) => Status::Failed,
                None => Status::Running,
            }),
            ci_text: None,
            lines: None,
            files: None,
        })
    });
    let rounds = (!issue.reviews.is_empty()).then(|| {
        issue
            .reviews
            .iter()
            .map(|r| ReviewRound {
                round: usize::from(r.round),
                by: r
                    .reviewer
                    .as_ref()
                    .map_or_else(|| s("reviewer"), |id| s(id.0.clone())),
                status: match r.verdict {
                    VerdictWord::Approve => Status::Passed,
                    VerdictWord::Block => Status::Blocked,
                    VerdictWord::Unknown => Status::Open,
                },
                points: r
                    .points
                    .iter()
                    .map(|p| ReviewPoint {
                        text: s(p.text.clone()),
                        reply: p.reply.clone().map(SharedString::from),
                        resolved: p.state == Some(temet_work_core::model::PointState::Resolved),
                    })
                    .collect(),
            })
            .collect()
    });
    let mut own: Vec<&Decision> = view
        .decisions
        .iter()
        .filter(|d| d.issue == issue.id)
        .collect();
    own.sort_by_key(|d| (rank(undo_cost(d.cost)), d.id));
    let decisions: Vec<IssueDecision> = own
        .into_iter()
        .map(|d| IssueDecision {
            id: s(d.id.to_string()),
            decision: pane_decision(d),
            draft: None,
            pending: false,
            error: None,
        })
        .collect();
    IssueDetail {
        round: usize::from(issue.round),
        facts: facts(view, issue),
        plan,
        pull,
        rounds,
        decisions: (!decisions.is_empty()).then_some(decisions),
    }
}

/// For a milestone no group of the project takes: "no group takes it", and how many of its
/// issues a group still holds, out of scope.
fn scope_note(groups: &[&Group], milestone: &str, issues: &[&Issue]) -> Option<SharedString> {
    if groups.iter().any(|g| takes(g, milestone)) {
        return None;
    }
    let held: Vec<&Issue> = issues
        .iter()
        .filter(|i| i.milestone.as_ref().is_some_and(|m| m.0 == milestone))
        .filter(|i| i.group.is_some())
        .copied()
        .collect();
    let mut holders: Vec<&str> = Vec::new();
    for i in &held {
        let id = i.group.as_ref().map_or("", |g| g.0.as_str());
        let name = groups
            .iter()
            .find(|g| g.id.0 == id)
            .map_or(id, |g| g.name.as_str());
        if !holders.contains(&name) {
            holders.push(name);
        }
    }
    Some(s(match held.len() {
        0 => "no group takes it".to_string(),
        n => format!(
            "no group takes it \u{b7} {n} held by {}, out of scope",
            holders.join(", ")
        ),
    }))
}

/// The milestones of `project` that have open issues, with them, in the view's order, then one
/// for the issues that name none. A milestone with nothing open is counted in its tracker, not
/// listed.
fn milestones(view: &View, project: &ProjectId, now_ms: u64) -> Vec<Milestone> {
    let issues: Vec<&Issue> = view
        .issues
        .iter()
        .filter(|i| project_of(&i.id) == project.0)
        .collect();
    let groups = groups_of(view, project);
    let listed: Vec<&ViewMilestone> = view
        .milestones
        .iter()
        .filter(|m| &m.project == project)
        .collect();
    let mut out: Vec<Milestone> = listed
        .iter()
        .map(|m| Milestone {
            name: s(m.id.0.clone()),
            due: m.due.as_deref().map(due),
            done: m.done,
            total: m.total,
            note: scope_note(&groups, &m.id.0, &issues),
            taken: groups.iter().any(|g| takes(g, &m.id.0)),
            issues: issues
                .iter()
                .filter(|i| i.milestone.as_ref() == Some(&m.id))
                .map(|i| summary(view, i, now_ms))
                .collect(),
        })
        .filter(|m| !m.issues.is_empty())
        .collect();
    let loose: Vec<IssueSummary> = issues
        .iter()
        .filter(|i| {
            i.milestone
                .as_ref()
                .is_none_or(|name| !listed.iter().any(|m| &m.id == name))
        })
        .map(|i| summary(view, i, now_ms))
        .collect();
    if !loose.is_empty() {
        out.push(Milestone {
            name: s("No milestone"),
            due: None,
            done: 0,
            total: loose.len(),
            note: None,
            taken: true,
            issues: loose,
        });
    }
    out
}

/// The board's lanes: one per milestone the List shows, in the same order, closed at first when
/// no group of the project takes it; each card with its open decisions and running agents.
fn lanes(view: &View, milestones: &[Milestone]) -> Vec<BoardLane> {
    let open = open_decisions(view);
    milestones
        .iter()
        .map(|m| BoardLane {
            id: m.name.clone(),
            title: m.name.clone(),
            note: m.note.clone(),
            default_open: m.taken,
            issues: m
                .issues
                .iter()
                .map(|i| BoardIssue {
                    key: i.key.clone(),
                    title: i.title.clone(),
                    stage: i.stage,
                    agents: running_on(view, &i.key)
                        .filter_map(|a| {
                            Some(IssueCardAgent {
                                id: s(a.id.0.clone()),
                                label: s(a.id.0.clone()),
                                status: agent_status(a.state)?,
                            })
                        })
                        .collect(),
                    decisions: open.get(i.key.as_ref()).copied().unwrap_or(0),
                    updated: (!i.updated.is_empty()).then(|| i.updated.clone()),
                })
                .collect(),
        })
        .collect()
}

/// The Project page's view of `view`, for `project` or, when none is named or it is not in the
/// view, the first project. `now_ms` is the clock, in milliseconds since 1970, for "4 min ago".
/// `None` when the view lists no projects.
pub fn project_view(view: &View, project: Option<&ProjectId>, now_ms: u64) -> Option<ProjectView> {
    let shown = project
        .and_then(|id| view.projects.iter().find(|p| &p.id == id))
        .or_else(|| view.projects.first())?;
    let id = &shown.id;

    let mut facts = Vec::new();
    let issues_url = shown
        .issues_url
        .as_ref()
        .or_else(|| shown.area.as_ref().and_then(|a| a.issues_url.as_ref()));
    if let Some(url) = issues_url {
        facts.push(PageFact {
            label: s("Issues"),
            value: tracker_fact(url, shown.tracker_name.as_deref()),
            icon: Some(IconName::Link),
        });
    }
    if let Some(url) = &shown.code_url {
        facts.push(PageFact {
            label: s("Code"),
            value: address(url),
            icon: Some(IconName::GitBranch),
        });
    }

    // Needs you as on main: every decision that waits on a person, costliest first, in the
    // short form. What was weighed and why are in the issue's pane, under Decisions.
    let needs = waiting(view)
        .into_iter()
        .map(|d| NeedsYouDecision {
            id: s(d.id.to_string()),
            decision: need_decision(d, &view.agents),
        })
        .collect();

    let own: Vec<&Issue> = view
        .issues
        .iter()
        .filter(|i| project_of(&i.id) == id.0)
        .collect();
    let details = own
        .iter()
        .map(|i| (s(i.id.clone()), detail(view, i)))
        .collect();
    let milestones = milestones(view, id, now_ms);
    let (areas, projects) = projects(view, id);
    // The deadline every decision in Needs you shares, which the pane's rows then leave out.
    let deadlines: Vec<Option<SharedString>> =
        waiting(view).into_iter().map(overturn_before).collect();
    let shared_deadline = deadlines
        .first()
        .cloned()
        .flatten()
        .filter(|first| deadlines.iter().all(|d| d.as_ref() == Some(first)));
    Some(ProjectView {
        areas,
        projects,
        project: s(id.0.clone()),
        groups: groups(view, id),
        stages: stage_counts(view, &own),
        title: s(id.0.clone()),
        facts,
        problems: view
            .problems
            .iter()
            .map(|(name, text)| Problem {
                name: s(name.clone()),
                text: s(text.clone()),
            })
            .collect(),
        needs,
        lanes: lanes(view, &milestones),
        milestones,
        details,
        shared_deadline,
    })
}

/// What to tell the person about the daemon's last answer, or `None` when the view is current.
/// With no view the notice says why: the problem's own text and the socket's path; before the
/// first answer it says the daemon has not answered yet. With a view and a problem the view is
/// stale, and the notice says so with the problem's text.
pub fn notice(view: Option<&View>, problem: Option<&DaemonProblem>, path: &Path) -> Option<Notice> {
    let socket = s(format!("Socket: {}", path.display()));
    match (view, problem) {
        (Some(view), None) if view.projects.is_empty() => Some(Notice {
            intent: Intent::None,
            title: s("The daemon has no projects"),
            lines: vec![socket],
        }),
        (Some(_), None) => None,
        (Some(_), Some(problem)) => Some(Notice {
            intent: Intent::Warning,
            title: s("This view is out of date"),
            lines: vec![s(problem.to_string()), socket],
        }),
        (None, Some(problem)) => Some(Notice {
            intent: Intent::Danger,
            title: s("No view from the daemon"),
            lines: vec![s(problem.to_string()), socket],
        }),
        (None, None) => Some(Notice {
            intent: Intent::None,
            title: s("Waiting for the daemon"),
            lines: vec![socket],
        }),
    }
}

/// Puts the daemon's last answer on the page: the mapped view, kept through a failed ask as a
/// stale one, and the notice. Without a view the page is blank and draws its notice alone. The
/// person's choices stay (`Project::set_view`): the issue open in the pane, or the pane closed,
/// View and Group by. The first view opens its first issue.
pub fn show(
    page: &mut Project,
    view: Option<&View>,
    problem: Option<&DaemonProblem>,
    path: &Path,
    now_ms: u64,
) {
    page.set_view(view.and_then(|v| project_view(v, None, now_ms)));
    page.set_notice(notice(view, problem, path));
}

/// [`show`] for the daemon entity's current state, at the clock's time.
pub fn sync(page: &mut Project, daemon: &Daemon) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
    show(page, daemon.view(), daemon.problem(), daemon.path(), now);
}
