//! Project (`pages-temet-work-radar`): one project: the areas, projects, agent groups, machines
//! and limits on the left; what needs the person and the project's issues in the middle; the
//! selected issue on the right, with its actions menu. Assign and Unassign in that menu send the
//! command to the daemon as a person's request, and a refusal shows in a danger toast.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui::{App, Context, IntoElement, ParentElement, Render, SharedString, Window};
use temet_work_core::Command;
use temet_work_core::ids::{AgentId, IssueId};

use crate::components::Intent;
use crate::components::alert::Alert;
use crate::components::app_shell::AppShell;
use crate::components::callout::Callout;
use crate::components::icon::IconName;
use crate::components::segmented_control::{SegmentedControl, SegmentedControlOption};
use crate::components::toast::{OverlayToaster, Toast};
use crate::composites::group_list::{GroupList, GroupListGroup};
use crate::composites::issue_actions::{IssueActions, IssueActionsAgent};
use crate::composites::issue_pane::{
    IssueDecision, IssuePane, IssuePlan, IssuePullRequest, IssueTab,
};
use crate::composites::issue_table::{GroupBy, IssueSummary, IssueTable};
use crate::composites::meter_list::{MeterList, MeterListItem};
use crate::composites::milestone_panel::MilestonePanel;
use crate::composites::needs_you::{NeedsYou, NeedsYouDecision};
use crate::composites::page_header::{PageFact, PageHeader};
use crate::composites::review_rounds::ReviewRound;
use crate::composites::side_nav::{SideNav, SideNavArea};
use crate::daemon::{self, Problem};

/// What the Project page shows, in the shape of its parts' props.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectView {
    /// The areas and projects. Each project's count is worked out from `needs`.
    pub areas: Vec<SideNavArea>,
    /// The id of the project on screen.
    pub project: SharedString,
    pub groups: Vec<GroupListGroup>,
    pub machines: Vec<MeterListItem>,
    pub limits: Vec<MeterListItem>,
    /// The page title: the project's name.
    pub title: SharedString,
    /// Where the project's issues and code live.
    pub facts: Vec<PageFact>,
    /// The agents' decisions for the person to read, costliest to undo first.
    pub needs: Vec<NeedsYouDecision>,
    pub milestones: Vec<Milestone>,
    /// What each issue has so far beyond its row, by key.
    pub details: HashMap<SharedString, IssueDetail>,
    /// The project each decision in `needs` belongs to, by the decision's id.
    pub need_project: HashMap<SharedString, SharedString>,
    /// The decision in an issue's pane that is the same as one in `needs`: the need's id, and
    /// the issue's key and the decision's id. Overturning either overturns both.
    pub same_decision: HashMap<SharedString, (SharedString, SharedString)>,
}

/// Something to tell the person about where the page's data comes from, such as the daemon not
/// answering: a Callout above the page's content, or alone when the page has no view.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub intent: Intent,
    pub title: SharedString,
    /// The lines under the title.
    pub lines: Vec<SharedString>,
}

/// A milestone and its open issues.
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub name: SharedString,
    pub due: Option<SharedString>,
    /// How many of its issues are done; they are not listed.
    pub done: usize,
    /// How many issues it has, done or not.
    pub total: usize,
    pub issues: Vec<IssueSummary>,
}

/// What an issue has beyond its row: its id, its review round, the agents a person may assign
/// to it or take off it, and the parts of its pane it has so far.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IssueDetail {
    /// The issue's id as the daemon names it, `project#key`, which a command names it by.
    pub id: SharedString,
    /// The agents a person may assign to it now, each with the issue it is on now.
    pub assignable: Vec<IssueActionsAgent>,
    /// The agents on it now, which a person may unassign.
    pub assigned: Vec<IssueActionsAgent>,
    /// The review round it is in; 0 before its first review.
    pub round: usize,
    pub plan: Option<IssuePlan>,
    pub pull: Option<IssuePullRequest>,
    pub rounds: Option<Vec<ReviewRound>>,
    pub decisions: Option<Vec<IssueDecision>>,
}

/// How the middle column groups the issues: a panel per milestone, or one table grouped by
/// stage or agent group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    #[default]
    Milestone,
    Stage,
    Group,
}

impl Grouping {
    pub const ALL: [Grouping; 3] = [Self::Milestone, Self::Stage, Self::Group];

    /// The option's value and label in the Group by control.
    pub fn value(self) -> &'static str {
        match self {
            Self::Milestone => "milestone",
            Self::Stage => "stage",
            Self::Group => "group",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Milestone => "Milestone",
            Self::Stage => "Stage",
            Self::Group => "Group",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.value() == value)
    }

    /// The IssueTable grouping, for a single table.
    pub fn table(self) -> Option<GroupBy> {
        match self {
            Self::Milestone => None,
            Self::Stage => Some(GroupBy::Stage),
            Self::Group => Some(GroupBy::Group),
        }
    }
}

/// The issue selected in the page's tables: one at most, across every milestone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection(Option<SharedString>);

impl Selection {
    pub fn new(key: Option<SharedString>) -> Self {
        Self(key)
    }

    /// Selects the issue `key`. Returns whether the selection changed.
    pub fn select(&mut self, key: &SharedString) -> bool {
        if self.0.as_ref() == Some(key) {
            return false;
        }
        self.0 = Some(key.clone());
        true
    }

    /// The selected issue's key.
    pub fn key(&self) -> Option<&SharedString> {
        self.0.as_ref()
    }
}

/// The facts the pane shows for an issue: its milestone, group, agent, round and last change.
pub fn issue_facts(
    milestone: &Milestone,
    issue: &IssueSummary,
    detail: Option<&IssueDetail>,
) -> Vec<(SharedString, SharedString)> {
    let none = || SharedString::from("None");
    let round = match detail.map(|d| d.round).unwrap_or_default() {
        0 => "Not reviewed yet".to_string(),
        n => n.to_string(),
    };
    vec![
        ("Milestone".into(), milestone.name.clone()),
        ("Group".into(), issue.group.clone().unwrap_or_else(none)),
        ("Agent".into(), issue.agent.clone().unwrap_or_else(none)),
        ("Round".into(), round.into()),
        ("Updated".into(), issue.updated.clone()),
    ]
}

/// "7 issues across 3 milestones": the summary of the one panel that groups by stage or group.
pub fn all_issues_summary(milestones: &[Milestone]) -> String {
    let issues: usize = milestones.iter().map(|m| m.issues.len()).sum();
    let across = match milestones.len() {
        1 => "1 milestone".to_string(),
        n => format!("{n} milestones"),
    };
    let issues = match issues {
        1 => "1 issue".to_string(),
        n => format!("{n} issues"),
    };
    format!("{issues} across {across}")
}

/// A decision the person has asked to overturn, waiting on the confirm question: the decision
/// in Needs you, the one in an issue's pane, or both when they are the same.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub issue: SharedString,
    pub by: SharedString,
    pub took: SharedString,
    pub need: Option<SharedString>,
    pub decision: Option<(SharedString, SharedString)>,
}

impl Ask {
    /// The confirm question, such as "Overturn the arbiter's decision on WATCH-51? Claude chose
    /// to page the tower list 200 at a time instead of loading all."
    pub fn question(&self) -> String {
        let (role, runtime) = self
            .by
            .split_once(" · ")
            .unwrap_or((self.by.as_ref(), self.by.as_ref()));
        let mut took = self.took.chars();
        let took = match took.next() {
            Some(first) => first.to_lowercase().chain(took).collect::<String>(),
            None => String::new(),
        };
        format!(
            "Overturn the {role}'s decision on {}? {runtime} chose to {took}.",
            self.issue
        )
    }

    /// The toast once it is overturned, such as "Overturned on WATCH-51: Page the ...".
    pub fn done(&self) -> String {
        format!("Overturned on {}: {}", self.issue, self.took)
    }
}

/// The note that replaces an overturned decision's action.
pub const OVERTURNED: &str = "Overturned just now";

/// What a person asks the daemon to do to an issue from its actions menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Assign,
    Unassign,
}

/// A person's request from the actions menu: the verb, the issue's id and the agent's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonAction {
    pub verb: Verb,
    /// The issue's id, `project#key`.
    pub issue: SharedString,
    pub agent: SharedString,
}

impl PersonAction {
    /// The command it sends; none when the issue's id is not of the form `project#key`.
    pub fn command(&self) -> Option<Command> {
        let issue = IssueId::parse(&self.issue)?;
        let agent = AgentId::new(self.agent.to_string());
        Some(match self.verb {
            Verb::Assign => Command::Assign { issue, agent },
            Verb::Unassign => Command::Unassign { issue, agent },
        })
    }

    /// What it asked, naming the action and the agent: "Assign reviewer-4 to watch#12".
    pub fn asked(&self) -> String {
        match self.verb {
            Verb::Assign => format!("Assign {} to {}", self.agent, self.issue),
            Verb::Unassign => format!("Unassign {} from {}", self.agent, self.issue),
        }
    }

    /// The toast when it fails, with the daemon's reply word for word, or what went wrong on the
    /// way: "Assign reviewer-4 to watch#12: reviewer-4 is not a reviewer".
    pub fn failed(&self, problem: &Problem) -> String {
        let why = match problem {
            Problem::Refused { why } => why.clone(),
            other => other.to_string(),
        };
        format!("{}: {why}", self.asked())
    }
}

/// The toast on show: its message and its intent.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub message: SharedString,
    pub intent: Intent,
}

/// The Project page: a window's view, holding what it shows and what the person has chosen.
pub struct Project {
    view: ProjectView,
    /// Whether there is no view to draw: the page then draws its notice and nothing else.
    blank: bool,
    notice: Option<Notice>,
    /// The notice that a newer build of the app is installed.
    newer_build: Option<Notice>,
    selection: Selection,
    grouping: Grouping,
    /// The open tab of the selected issue's pane; its first tab when none is chosen.
    tab: Option<IssueTab>,
    /// The decision waiting on the confirm question.
    asking: Option<Ask>,
    /// The decisions in Needs you that are overturned, by id.
    overturned_needs: HashSet<SharedString>,
    /// The decisions in issues' panes that are overturned, by issue key and id.
    overturned_decisions: HashSet<(SharedString, SharedString)>,
    /// The toast on show.
    toast: Option<Shown>,
    /// The daemon's socket, which a person's request goes to; none for the sample data.
    socket: Option<PathBuf>,
}

impl Project {
    pub fn new(view: ProjectView, selected: Option<SharedString>) -> Self {
        Self {
            view,
            blank: false,
            notice: None,
            newer_build: None,
            selection: Selection::new(selected),
            grouping: Grouping::Milestone,
            tab: None,
            asking: None,
            overturned_needs: HashSet::new(),
            overturned_decisions: HashSet::new(),
            toast: None,
            socket: None,
        }
    }

    /// Sets the daemon's socket, which the view is read from and a person's request goes to.
    pub fn set_socket(&mut self, socket: Option<PathBuf>) {
        self.socket = socket;
    }

    /// Sends `action` to the daemon off the window's thread. A done reply needs no word: the next
    /// view shows the change. A refusal, or a request that never got a reply, shows in a danger
    /// toast with the daemon's reply.
    pub fn send(&mut self, action: PersonAction, cx: &mut Context<Self>) {
        let (Some(path), Some(command)) = (self.socket.clone(), action.command()) else {
            let why = if self.socket.is_none() {
                "the window shows sample data and has no daemon to send to".to_string()
            } else {
                format!(
                    "{} is not an issue id of the form project#key",
                    action.issue
                )
            };
            self.fail(format!("{}: {why}", action.asked()));
            cx.notify();
            return;
        };
        cx.spawn(async move |page, cx| {
            let executor = cx.background_executor().clone();
            let reply = daemon::send(path, command, &executor).await;
            page.update(cx, |page, cx| {
                page.took(&action, reply);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Takes the daemon's reply to `action`.
    pub fn took(&mut self, action: &PersonAction, reply: Result<String, Problem>) {
        if let Err(problem) = reply {
            self.fail(action.failed(&problem));
        }
    }

    fn fail(&mut self, message: String) {
        self.toast = Some(Shown {
            message: message.into(),
            intent: Intent::Danger,
        });
    }

    /// A page with no view yet: it draws `notice` alone until [`Project::set_view`] gives it one.
    pub fn blank(notice: Option<Notice>) -> Self {
        let mut page = Self::new(ProjectView::default(), None);
        page.blank = true;
        page.notice = notice;
        page
    }

    /// Replaces what the page shows with `view`, or with nothing for `None`. What the person
    /// chose stays: the selected issue, Group by, the open tab, a question being asked and what
    /// is overturned.
    pub fn set_view(&mut self, view: Option<ProjectView>) {
        self.blank = view.is_none();
        self.view = view.unwrap_or_default();
    }

    /// Sets the notice above the page, or clears it.
    pub fn set_notice(&mut self, notice: Option<Notice>) {
        self.notice = notice;
    }

    /// Sets the notice that a newer build of the app is installed, or clears it.
    pub fn set_newer_build(&mut self, notice: Option<Notice>) {
        self.newer_build = notice;
    }

    /// The notice that a newer build of the app is installed, when there is one.
    pub fn newer_build(&self) -> Option<&Notice> {
        self.newer_build.as_ref()
    }

    /// The notice on show.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// Whether the page has no view to draw.
    pub fn is_blank(&self) -> bool {
        self.blank
    }

    /// The view the page draws.
    pub fn view(&self) -> &ProjectView {
        &self.view
    }

    /// Asks to overturn the decision `id` in Needs you: the confirm question opens.
    pub fn ask_need(&mut self, id: &SharedString) {
        let Some(n) = self.view.needs.iter().find(|n| &n.id == id) else {
            return;
        };
        self.asking = Some(Ask {
            issue: n.decision.issue.clone().unwrap_or_default(),
            by: n.decision.by.clone(),
            took: n.decision.took.clone(),
            need: Some(n.id.clone()),
            decision: self.view.same_decision.get(id).cloned(),
        });
    }

    /// Asks to overturn the decision `id` in the pane of the issue `key`.
    pub fn ask_decision(&mut self, key: &SharedString, id: &SharedString) {
        let found = self
            .view
            .details
            .get(key)
            .and_then(|d| d.decisions.as_ref())
            .and_then(|ds| ds.iter().find(|d| &d.id == id));
        let Some(d) = found else {
            return;
        };
        let need = self
            .view
            .same_decision
            .iter()
            .find(|(_, (k, i))| k == key && i == id)
            .map(|(need, _)| need.clone());
        self.asking = Some(Ask {
            issue: key.clone(),
            by: d.decision.by.clone(),
            took: d.decision.took.clone(),
            need,
            decision: Some((key.clone(), id.clone())),
        });
    }

    /// The decision waiting on the confirm question, if any.
    pub fn asking(&self) -> Option<&Ask> {
        self.asking.as_ref()
    }

    /// Confirms the question: the decision is overturned in Needs you and in its issue's pane,
    /// and the toast says so. Nothing is sent to the daemon yet: its command, `temet-work
    /// overturn DECISION OPTION REASON`, needs the option chosen in place of the decision's and a
    /// reason, which this confirm does not ask for.
    pub fn confirm(&mut self) {
        let Some(ask) = self.asking.take() else {
            return;
        };
        if let Some(need) = &ask.need {
            self.overturned_needs.insert(need.clone());
        }
        if let Some(decision) = &ask.decision {
            self.overturned_decisions.insert(decision.clone());
        }
        self.toast = Some(Shown {
            message: ask.done().into(),
            intent: Intent::None,
        });
    }

    pub fn cancel(&mut self) {
        self.asking = None;
    }

    pub fn dismiss_toast(&mut self) {
        self.toast = None;
    }

    /// The message of the toast on show.
    pub fn toast(&self) -> Option<&SharedString> {
        self.toast.as_ref().map(|t| &t.message)
    }

    /// The intent of the toast on show.
    pub fn toast_intent(&self) -> Option<Intent> {
        self.toast.as_ref().map(|t| t.intent)
    }

    /// The decisions in Needs you that are not overturned, in order.
    pub fn needs(&self) -> Vec<NeedsYouDecision> {
        self.view
            .needs
            .iter()
            .filter(|n| !self.overturned_needs.contains(&n.id))
            .cloned()
            .collect()
    }

    /// The areas, each project's count the decisions in Needs you that belong to it.
    pub fn areas(&self) -> Vec<SideNavArea> {
        let needs = self.needs();
        let mut areas = self.view.areas.clone();
        for project in areas.iter_mut().flat_map(|a| a.projects.iter_mut()) {
            project.waiting = needs
                .iter()
                .filter(|n| self.view.need_project.get(&n.id) == Some(&project.id))
                .count();
        }
        areas
    }

    /// The selected issue's decisions, each overturned one with its note.
    fn decisions(&self, key: &SharedString, decisions: Vec<IssueDecision>) -> Vec<IssueDecision> {
        decisions
            .into_iter()
            .map(|mut d| {
                if self
                    .overturned_decisions
                    .contains(&(key.clone(), d.id.clone()))
                {
                    d.decision.overturned = Some(OVERTURNED.into());
                }
                d
            })
            .collect()
    }

    /// Selects the issue `key` and opens it in the pane, on its first tab.
    pub fn select(&mut self, key: &SharedString) -> bool {
        let changed = self.selection.select(key);
        if changed {
            self.tab = None;
        }
        changed
    }

    /// The selected issue's key.
    pub fn selected(&self) -> Option<&SharedString> {
        self.selection.key()
    }

    /// The open tab of the selected issue's pane, when the person chose one.
    pub fn tab(&self) -> Option<IssueTab> {
        self.tab
    }

    pub fn grouping(&self) -> Grouping {
        self.grouping
    }

    pub fn set_grouping(&mut self, grouping: Grouping) {
        self.grouping = grouping;
    }

    pub fn set_tab(&mut self, tab: IssueTab) {
        self.tab = Some(tab);
    }
}

impl Render for Project {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = cx.entity().downgrade();
        let on_select = {
            let page = page.clone();
            move |key: &SharedString, _: &mut Window, cx: &mut App| {
                page.update(cx, |page, cx| {
                    if page.select(key) {
                        cx.notify();
                    }
                })
                .ok();
            }
        };
        let on_group = {
            let page = page.clone();
            move |value: &SharedString, _: &mut Window, cx: &mut App| {
                if let Some(grouping) = Grouping::from_value(value) {
                    page.update(cx, |page, cx| {
                        page.set_grouping(grouping);
                        cx.notify();
                    })
                    .ok();
                }
            }
        };
        let on_tab = {
            let page = page.clone();
            move |tab: IssueTab, _: &mut Window, cx: &mut App| {
                page.update(cx, |page, cx| {
                    page.set_tab(tab);
                    cx.notify();
                })
                .ok();
            }
        };
        let update = |f: fn(&mut Project)| {
            let page = page.clone();
            move |_: &mut Window, cx: &mut App| {
                page.update(cx, |p, cx| {
                    f(p);
                    cx.notify();
                })
                .ok();
            }
        };
        let ask_need = {
            let page = page.clone();
            move |id: &SharedString, _: &mut Window, cx: &mut App| {
                page.update(cx, |p, cx| {
                    p.ask_need(id);
                    cx.notify();
                })
                .ok();
            }
        };

        let view = &self.view;
        let selected = self.selection.key().cloned();
        let notice: Vec<Callout> = [&self.newer_build, &self.notice]
            .into_iter()
            .flatten()
            .map(|n| {
                let mut callout = Callout::new().intent(n.intent).title(n.title.clone());
                for line in &n.lines {
                    callout = callout.child(line.clone());
                }
                callout
            })
            .collect();
        if self.blank {
            return AppShell::new(SideNav::new(Vec::new()))
                .children(notice)
                .into_any_element();
        }
        // A list with nothing in it is left out, so no heading stands over an empty space.
        let mut nav = SideNav::new(self.areas()).selected(view.project.clone());
        if !view.groups.is_empty() {
            nav = nav.child(GroupList::new(view.groups.clone()));
        }
        if !view.machines.is_empty() {
            nav = nav.child(MeterList::new("Machines", view.machines.clone()));
        }
        if !view.limits.is_empty() {
            nav = nav.child(MeterList::new("Limits", view.limits.clone()));
        }

        let group_by = SegmentedControl::new(
            "group-by",
            Grouping::ALL
                .into_iter()
                .map(|g| SegmentedControlOption {
                    label: g.label().into(),
                    value: g.value().into(),
                })
                .collect(),
        )
        .intent(Intent::Primary)
        .value(self.grouping.value())
        .on_value_change(on_group);
        let header = PageHeader::new(view.title.clone())
            .facts(view.facts.clone())
            .actions_label("Group by")
            .actions(group_by);
        let open = self.needs();
        let needs = NeedsYou::new(open.clone())
            .note(format!("{}, costliest to undo first", open.len()))
            .on_overturn(ask_need);

        let mut shell = AppShell::new(nav)
            .children(notice)
            .child(header)
            .child(needs);
        match self.grouping.table() {
            None => {
                for m in &view.milestones {
                    let mut panel = MilestonePanel::new(m.name.clone())
                        .done(m.done)
                        .total(m.total);
                    if let Some(due) = &m.due {
                        panel = panel.due(due.clone());
                    }
                    shell = shell.child(
                        panel.child(
                            IssueTable::new(
                                SharedString::from(format!("issues-{}", m.name)),
                                m.issues.clone(),
                            )
                            .selected(selected.clone())
                            .on_select(on_select.clone()),
                        ),
                    );
                }
            }
            Some(by) => {
                let all: Vec<IssueSummary> = view
                    .milestones
                    .iter()
                    .flat_map(|m| m.issues.clone())
                    .collect();
                let mut panel =
                    MilestonePanel::new("All issues").summary(all_issues_summary(&view.milestones));
                // With no issues at all the panel says so, as an empty milestone does.
                if all.is_empty() {
                    panel = panel.total(0);
                }
                shell = shell.child(
                    panel.child(
                        IssueTable::new("issues-all", all)
                            .group_by(by)
                            .selected(selected.clone())
                            .on_select(on_select),
                    ),
                );
            }
        }

        let found = selected.as_ref().and_then(|key| {
            view.milestones
                .iter()
                .find_map(|m| m.issues.iter().find(|i| &i.key == key).map(|i| (m, i)))
        });
        if let Some((milestone, issue)) = found {
            let detail = view.details.get(&issue.key).cloned().unwrap_or_default();
            let person = |verb: Verb| {
                let page = page.clone();
                let issue = detail.id.clone();
                move |agent: &SharedString, _: &mut Window, cx: &mut App| {
                    let action = PersonAction {
                        verb,
                        issue: issue.clone(),
                        agent: agent.clone(),
                    };
                    page.update(cx, |p, cx| p.send(action, cx)).ok();
                }
            };
            let actions = IssueActions::new()
                .assignable(detail.assignable.clone())
                .on_assign(person(Verb::Assign))
                .assigned(detail.assigned.clone())
                .on_unassign(person(Verb::Unassign));
            let mut pane = IssuePane::new(issue.key.clone(), issue.title.clone(), issue.stage)
                .round(Some(detail.round))
                .facts(issue_facts(milestone, issue, Some(&detail)))
                .plan(detail.plan)
                .pull(detail.pull)
                .rounds(detail.rounds)
                .decisions(detail.decisions.map(|d| self.decisions(&issue.key, d)))
                .tab(self.tab.unwrap_or(IssueTab::Plan))
                .on_tab_change(on_tab)
                .on_overturn({
                    let page = page.clone();
                    let key = issue.key.clone();
                    move |id: &SharedString, _: &mut Window, cx: &mut App| {
                        page.update(cx, |p, cx| {
                            p.ask_decision(&key, id);
                            cx.notify();
                        })
                        .ok();
                    }
                });
            // The actions button shows only when its menu lists something.
            if actions.has_items() {
                pane = pane.actions(actions);
            }
            shell = shell.side(pane);
        }

        // Overturning asks first, then says it is done, as the radar page does.
        let alert = Alert::new()
            .is_open(self.asking.is_some())
            .intent(Intent::Primary)
            .cancel_button_text("Cancel")
            .confirm_button_text("Overturn")
            .on_cancel(update(Project::cancel))
            .on_confirm(update(Project::confirm))
            .children(self.asking.as_ref().map(|ask| ask.question()));
        let mut toaster = OverlayToaster::new();
        if let Some(shown) = &self.toast {
            // The overturn toast as the radar page draws it; a failed request in danger.
            let icon = match shown.intent {
                Intent::Danger => IconName::WarningSign,
                _ => IconName::Notifications,
            };
            toaster = toaster.child(
                Toast::new(shown.message.clone())
                    .icon(icon)
                    .intent(shown.intent)
                    .on_dismiss(update(Project::dismiss_toast)),
            );
        }
        shell.child(alert).child(toaster).into_any_element()
    }
}
