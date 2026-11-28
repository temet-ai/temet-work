//! Plumbing: the daemon's [`View`] mapped into the Project page's [`ProjectView`], and the page
//! told what the daemon's last answer was. It draws nothing.
//!
//! The mapping is plain functions of the view and the clock. A field the view has no data for is
//! left empty, and so is a part's row for it, rather than filled with a stand-in: the view holds
//! no machine slots or limits, a pull request's branch, lines or files, a plan's folder or
//! merge rule, or the time a decision was made. Where the view has a word the page has no word
//! for, the choice is made once, in [`stage`] and [`agent_status`].

use std::collections::HashMap;
use std::path::Path;

use gpui::SharedString;
use temet_work_core::View;
use temet_work_core::ids::ProjectId;
use temet_work_core::model::{CheckState, Cost};
use temet_work_core::view::{
    Agent, Decision, Issue, Milestone as ViewMilestone, StageWord, StateWord, VerdictWord,
};

use crate::components::Intent;
use crate::components::icon::IconName;
use crate::components::stage_tag::Stage;
use crate::components::status_tag::Status;
use crate::composites::check_list::PlanCheck;
use crate::composites::decision_row::{Decision as RowDecision, UndoCost};
use crate::composites::group_list::{GroupListAgent, GroupListGroup};
use crate::composites::issue_pane::{IssueDecision, IssuePlan, IssuePullRequest};
use crate::composites::issue_table::IssueSummary;
use crate::composites::needs_you::NeedsYouDecision;
use crate::composites::page_header::PageFact;
use crate::composites::review_rounds::{ReviewPoint, ReviewRound};
use crate::composites::side_nav::{SideNavArea, SideNavProject};
use crate::daemon::{Daemon, Problem};
use crate::pages::project::{IssueDetail, Milestone, Notice, Project, ProjectView};

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

/// The key of an issue id (`project#key`).
fn key(id: &str) -> &str {
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

/// "role · runtime" for the agent `id`, as the page names who decided or reviewed; the agent's
/// id when the view does not list it, and the role alone when it has no runtime.
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

fn decision_took(d: &Decision) -> SharedString {
    d.options
        .get(d.chosen.wrapping_sub(1))
        .map_or_else(|| s(d.question.clone()), |option| s(option.clone()))
}

/// The decision as a row draws it. `by` is who made it and `issue` the issue's name on the row,
/// left out inside the issue's own pane.
fn row_decision(d: &Decision, agents: &[Agent], issue: Option<SharedString>) -> RowDecision {
    let overturned = d.overturned_to.map(|option| {
        let to = d
            .options
            .get(option.wrapping_sub(1))
            .cloned()
            .unwrap_or_else(|| option.to_string());
        if d.overturn_reason.is_empty() {
            s(format!("Overturned to {to}"))
        } else {
            s(format!("Overturned to {to}: {}", d.overturn_reason))
        }
    });
    RowDecision {
        took: decision_took(d),
        by: who(agents, &d.by.0),
        issue,
        at: None,
        cost: undo_cost(d.cost),
        weighed: d.options.iter().map(|o| s(o.clone())).collect(),
        why: (!d.reason.is_empty()).then(|| s(d.reason.clone())),
        overturn_before: match &d.point {
            Some(point) if !point.passed && point.at == temet_work_core::model::Point::Merge => {
                Some(s("its issue's merge"))
            }
            _ => None,
        },
        overturned,
    }
}

/// The areas and their projects. A project with no area goes under "No area".
fn areas(view: &View) -> Vec<SideNavArea> {
    let mut areas: Vec<SideNavArea> = Vec::new();
    for project in &view.projects {
        let (id, name) = match &project.area {
            Some(area) => (area.id.0.clone(), area.name.clone()),
            None => ("no-area".to_string(), "No area".to_string()),
        };
        let item = SideNavProject {
            id: s(project.id.0.clone()),
            name: s(project.id.0.clone()),
            icon: None,
            waiting: 0,
        };
        match areas.iter_mut().find(|a| a.id.as_ref() == id) {
            Some(area) => area.projects.push(item),
            None => areas.push(SideNavArea {
                id: s(id),
                name: s(name),
                projects: vec![item],
            }),
        }
    }
    areas
}

/// The groups that work on `project`, each with its agents that have not ended.
fn groups(view: &View, project: &ProjectId) -> Vec<GroupListGroup> {
    view.groups
        .iter()
        .filter(|g| g.projects.contains(project))
        .map(|g| GroupListGroup {
            id: s(g.id.0.clone()),
            name: s(g.name.clone()),
            note: (!g.milestones.is_empty()).then(|| {
                s(g.milestones
                    .iter()
                    .map(|m| m.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", "))
            }),
            agents: view
                .agents
                .iter()
                .filter(|a| a.group == g.id)
                .filter_map(|a| {
                    let status = agent_status(a.state)?;
                    let place = [Some(a.machine.0.as_str()), a.model.as_deref()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" \u{b7} ");
                    Some(GroupListAgent {
                        id: s(a.id.0.clone()),
                        role: s(a.role.clone()),
                        r#where: Some(s(place)),
                        status,
                    })
                })
                .collect(),
        })
        .collect()
}

/// The agent working on `issue` now: the latest started of its agents that has not ended.
fn current_agent<'a>(view: &'a View, issue: &Issue) -> Option<&'a Agent> {
    view.agents
        .iter()
        .filter(|a| issue.agents.contains(&a.id) && a.state != StateWord::Ended)
        .max_by_key(|a| a.started)
}

fn summary(view: &View, issue: &Issue, now_ms: u64) -> IssueSummary {
    IssueSummary {
        key: s(key(&issue.id)),
        title: s(issue.title.clone()),
        stage: stage(issue.stage),
        group: issue.group.as_ref().map(|group| {
            view.groups
                .iter()
                .find(|g| &g.id == group)
                .map_or_else(|| s(group.0.clone()), |g| s(g.name.clone()))
        }),
        agent: current_agent(view, issue).map(|a| who(&view.agents, &a.id.0)),
        updated: issue.updated_ms.map_or_else(|| s(""), |t| ago(now_ms, t)),
    }
}

/// An issue's pane: what the view holds about its plan, pull request, reviews and decisions. A
/// tab whose data is empty is `None`, so the pane does not show it.
fn detail(view: &View, issue: &Issue) -> IssueDetail {
    let folder = current_agent(view, issue).and_then(|a| {
        a.folder
            .as_ref()
            .map(|folder| s(format!("{}:{}", a.machine, folder)))
    });
    let plan = (!issue.checks.is_empty()).then(|| IssuePlan {
        folder,
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
                    .map_or_else(|| s("reviewer"), |id| who(&view.agents, &id.0)),
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
    let decisions: Vec<IssueDecision> = view
        .decisions
        .iter()
        .filter(|d| d.issue == issue.id)
        .map(|d| IssueDecision {
            id: s(d.id.to_string()),
            decision: row_decision(d, &view.agents, None),
        })
        .collect();
    IssueDetail {
        round: usize::from(issue.round),
        plan,
        pull,
        rounds,
        decisions: (!decisions.is_empty()).then_some(decisions),
    }
}

/// The milestones of `project` with their open issues, then one for the issues that name none.
fn milestones(view: &View, project: &ProjectId, now_ms: u64) -> Vec<Milestone> {
    let issues: Vec<&Issue> = view
        .issues
        .iter()
        .filter(|i| project_of(&i.id) == project.0)
        .collect();
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
            issues: issues
                .iter()
                .filter(|i| i.milestone.as_ref() == Some(&m.id))
                .map(|i| summary(view, i, now_ms))
                .collect(),
        })
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
            issues: loose,
        });
    }
    out
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
    let rank = |d: &Decision| match undo_cost(d.cost) {
        Some(UndoCost::High) => 0,
        Some(UndoCost::Medium) => 1,
        Some(UndoCost::Low) => 2,
        None => 3,
    };
    waiting.sort_by_key(|d| (rank(d), d.id));
    waiting
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
            value: address(url),
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

    let mut needs = Vec::new();
    let mut need_project = HashMap::new();
    let mut same_decision = HashMap::new();
    for d in waiting(view) {
        let own = project_of(&d.issue) == id.0;
        let name = s(if own { key(&d.issue) } else { d.issue.as_str() });
        let need = s(d.id.to_string());
        need_project.insert(need.clone(), s(project_of(&d.issue)));
        if own {
            same_decision.insert(need.clone(), (s(key(&d.issue)), need.clone()));
        }
        // Needs you draws DecisionRow's short form, the decision and its button. What was
        // weighed and why are in the issue's pane, under Decisions.
        let mut decision = row_decision(d, &view.agents, Some(name));
        decision.weighed.clear();
        decision.why = None;
        needs.push(NeedsYouDecision { id: need, decision });
    }

    let details = view
        .issues
        .iter()
        .filter(|i| project_of(&i.id) == id.0)
        .map(|i| (s(key(&i.id)), detail(view, i)))
        .collect();

    Some(ProjectView {
        areas: areas(view),
        project: s(id.0.clone()),
        groups: groups(view, id),
        // The view holds no build slots and no usage limits.
        machines: Vec::new(),
        limits: Vec::new(),
        title: s(id.0.clone()),
        facts,
        needs,
        milestones: milestones(view, id, now_ms),
        details,
        need_project,
        same_decision,
    })
}

/// What to tell the person about the daemon's last answer, or `None` when the view is current.
/// With no view the notice says why: the problem's own text and the socket's path; before the
/// first answer it says the daemon has not answered yet. With a view and a problem the view is
/// stale, and the notice says so with the problem's text.
pub fn notice(view: Option<&View>, problem: Option<&Problem>, path: &Path) -> Option<Notice> {
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
/// person's choices stay (`Project::set_view`).
pub fn show(
    page: &mut Project,
    view: Option<&View>,
    problem: Option<&Problem>,
    path: &Path,
    now_ms: u64,
) {
    page.set_view(view.and_then(|v| project_view(v, None, now_ms)));
    page.set_notice(notice(view, problem, path));
    // With nothing selected the pane would be empty: select the first issue listed.
    if page.selected().is_none() {
        let first = page
            .view()
            .milestones
            .iter()
            .flat_map(|m| m.issues.iter())
            .map(|i| i.key.clone())
            .next();
        if let Some(key) = first {
            page.select(&key);
        }
    }
}

/// [`show`] for the daemon entity's current state, at the clock's time.
pub fn sync(page: &mut Project, daemon: &Daemon) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
    show(page, daemon.view(), daemon.problem(), daemon.path(), now);
}
