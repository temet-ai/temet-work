//! Project (`pages-temet-work-radar`): one project. The left column has the projects, the
//! issues' stages and the agent groups; the middle has the page head with View and Group by, the
//! daemon's problems, Needs you, and the issues as a board of stages or a list of milestones; the
//! right has the issue open in the pane. On the board the pane lies over the board's right, so
//! its columns keep their width.
//!
//! Open is the one action the page offers: Refine, Move to, Assign and Unassign wait for the
//! commands of temet-work#250, #251 and #252, and an overturn for #238's, so the page passes
//! their parts no handler and nothing it draws sends a command.

use std::collections::HashMap;

use gpui::{App, Context, IntoElement, ParentElement, Render, SharedString, Window};

use crate::components::Intent;
use crate::components::app_shell::AppShell;
use crate::components::callout::Callout;
use crate::components::property_list::{PropertyItem, PropertyLink};
use crate::components::segmented_control::{SegmentedControl, SegmentedControlOption};
use crate::components::stage_tag::Stage;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::{Class, classed};
use crate::composites::group_list::{GroupList, GroupListGroup};
use crate::composites::issue_actions::IssueActions;
use crate::composites::issue_pane::{
    IssueDecision, IssuePane, IssuePlan, IssuePullRequest, IssueTab,
};
use crate::composites::issue_table::{GroupBy, IssueSummary, IssueTable};
use crate::composites::milestone_panel::MilestonePanel;
use crate::composites::needs_you::{NeedsYou, NeedsYouDecision};
use crate::composites::page_header::{PageFact, PageHeader};
use crate::composites::problem_list::{Problem, ProblemList};
use crate::composites::review_rounds::ReviewRound;
use crate::composites::side_nav::{SideNav, SideNavArea, SideNavProject};
use crate::composites::stage_board::{BoardLane, StageBoard};
use crate::composites::stage_counts::StageCounts;

/// What the Project page shows, in the shape of its parts' props.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectView {
    /// The areas and their projects.
    pub areas: Vec<SideNavArea>,
    /// The projects in no area, at the top level.
    pub projects: Vec<SideNavProject>,
    /// The id of the project on screen.
    pub project: SharedString,
    pub groups: Vec<GroupListGroup>,
    /// How many of the project's issues are at each stage, every stage in the daemon's order.
    pub stages: Vec<(Stage, usize)>,
    /// The page title: the project's name.
    pub title: SharedString,
    /// Where the project's issues and code live.
    pub facts: Vec<PageFact>,
    /// What the daemon reports wrong now.
    pub problems: Vec<Problem>,
    /// The agents' decisions for the person to read, costliest to undo first.
    pub needs: Vec<NeedsYouDecision>,
    /// The milestones with open issues, for the list.
    pub milestones: Vec<Milestone>,
    /// The same issues as the board's lanes.
    pub lanes: Vec<BoardLane>,
    /// What each issue's pane shows beyond its row, by key.
    pub details: HashMap<SharedString, IssueDetail>,
    /// Whether an agent runs on any of the project's issues: the tables leave out their Agent
    /// column when none does, so a column of None takes no room.
    pub agents_running: bool,
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
    /// Muted text after its figures, such as "no group takes it".
    pub note: Option<SharedString>,
    /// Whether a group of the project takes it; its board lane starts closed when none does.
    pub taken: bool,
    pub issues: Vec<IssueSummary>,
}

/// A fact's value in the issue's pane.
#[derive(Clone, Debug, PartialEq)]
pub enum FactValue {
    Text(SharedString),
    /// Issue keys, each a link that opens that issue in the pane.
    Keys(Vec<SharedString>),
    /// A pull request's number and whether its checks passed; none until they finish.
    Pull {
        number: u32,
        checks: Option<bool>,
    },
    /// Text that links to a web address, such as the issue in its tracker.
    Link {
        text: SharedString,
        url: Option<SharedString>,
    },
}

/// One row of the facts panel in the issue's pane.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueFact {
    pub label: SharedString,
    pub value: FactValue,
}

/// What an issue's pane shows beyond its row: its review round, its facts and the tabs it has
/// data for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IssueDetail {
    /// The review round it is in; 0 before its first review.
    pub round: usize,
    pub facts: Vec<IssueFact>,
    pub plan: Option<IssuePlan>,
    pub pull: Option<IssuePullRequest>,
    pub rounds: Option<Vec<ReviewRound>>,
    pub decisions: Option<Vec<IssueDecision>>,
}

impl IssueDetail {
    /// How many of its decisions are open: not overturned.
    pub fn open_decisions(&self) -> usize {
        self.decisions.as_ref().map_or(0, |ds| {
            ds.iter()
                .filter(|d| d.decision.overturned.is_none())
                .count()
        })
    }
}

/// How the middle column shows the issues: a board of stages or a list of milestones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    Board,
    #[default]
    List,
}

impl ViewMode {
    pub const ALL: [ViewMode; 2] = [Self::Board, Self::List];

    /// The option's value in the View control.
    pub fn value(self) -> &'static str {
        match self {
            Self::Board => "board",
            Self::List => "list",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::List => "List",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.value() == value)
    }
}

/// How the list groups the issues: a panel per milestone, or one table grouped by stage or agent
/// group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    #[default]
    Milestone,
    Stage,
    Group,
}

impl Grouping {
    pub const ALL: [Grouping; 3] = [Self::Milestone, Self::Stage, Self::Group];

    /// The option's value in the Group by control.
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

/// "53 open in 0.4, 0.5, Backlog": the summary of the one panel that groups by stage or group.
pub fn all_issues_summary(milestones: &[Milestone]) -> String {
    let issues: usize = milestones.iter().map(|m| m.issues.len()).sum();
    let names: Vec<&str> = milestones.iter().map(|m| m.name.as_ref()).collect();
    format!("{issues} open in {}", names.join(", "))
}

/// The tab an issue's pane opens on: Decisions when it has open decisions, else Plan, as the
/// radar page opens it.
pub fn first_tab(detail: Option<&IssueDetail>) -> IssueTab {
    if detail.is_some_and(|d| d.open_decisions() > 0) {
        IssueTab::Decisions
    } else {
        IssueTab::Plan
    }
}

/// The Project page: a window's view, holding what it shows and what the person has chosen.
pub struct Project {
    view: ProjectView,
    /// Whether there is no view to draw: the page then draws its notice and nothing else.
    blank: bool,
    /// Whether the page has had a view: the first one opens its first issue.
    opened: bool,
    notice: Option<Notice>,
    /// The notice that a newer build of the app is installed.
    newer_build: Option<Notice>,
    /// The issue open in the pane; none with the pane closed.
    selected: Option<SharedString>,
    mode: ViewMode,
    grouping: Grouping,
    /// The open tab of the pane.
    tab: Option<IssueTab>,
}

impl Project {
    /// A page showing `view`, with the issue `selected` open in the pane.
    pub fn new(view: ProjectView, selected: Option<SharedString>) -> Self {
        let mut page = Self {
            view,
            blank: false,
            opened: true,
            notice: None,
            newer_build: None,
            selected: None,
            mode: ViewMode::List,
            grouping: Grouping::Milestone,
            tab: None,
        };
        if let Some(key) = selected {
            page.select(&key);
        }
        page
    }

    /// A page with no view yet: it draws `notice` alone until [`Project::set_view`] gives it one.
    pub fn blank(notice: Option<Notice>) -> Self {
        let mut page = Self::new(ProjectView::default(), None);
        page.blank = true;
        page.opened = false;
        page.notice = notice;
        page
    }

    /// Replaces what the page shows with `view`, or with nothing for `None`. What the person
    /// chose stays: the issue open in the pane or the pane closed, View, Group by and the tab.
    /// The first view the page gets opens its first issue.
    pub fn set_view(&mut self, view: Option<ProjectView>) {
        self.blank = view.is_none();
        self.view = view.unwrap_or_default();
        if !self.blank && !self.opened {
            self.opened = true;
            let first = self
                .view
                .milestones
                .iter()
                .flat_map(|m| m.issues.iter())
                .map(|i| i.key.clone())
                .next();
            if let Some(key) = first {
                self.select(&key);
            }
        }
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

    /// Opens the issue `key` in the pane, on Decisions when it has open decisions and on Plan
    /// otherwise. Returns whether anything changed.
    pub fn select(&mut self, key: &SharedString) -> bool {
        let tab = Some(first_tab(self.view.details.get(key)));
        if self.selected.as_ref() == Some(key) && self.tab == tab {
            return false;
        }
        self.selected = Some(key.clone());
        self.tab = tab;
        true
    }

    /// Closes the pane.
    pub fn close(&mut self) {
        self.selected = None;
    }

    /// The key of the issue open in the pane.
    pub fn selected(&self) -> Option<&SharedString> {
        self.selected.as_ref()
    }

    /// The open tab of the pane.
    pub fn tab(&self) -> Option<IssueTab> {
        self.tab
    }

    pub fn set_tab(&mut self, tab: IssueTab) {
        self.tab = Some(tab);
    }

    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
    }

    pub fn grouping(&self) -> Grouping {
        self.grouping
    }

    pub fn set_grouping(&mut self, grouping: Grouping) {
        self.grouping = grouping;
    }

    /// The issue open in the pane, with its milestone's row, when the view lists it.
    fn shown(&self) -> Option<&IssueSummary> {
        let key = self.selected.as_ref()?;
        self.view
            .milestones
            .iter()
            .flat_map(|m| m.issues.iter())
            .find(|i| &i.key == key)
    }
}

/// Opens a web address in the person's browser, unless the app runs sandboxed, as it does under
/// cargo's tests (`TEMET_WORK_SANDBOX`).
fn open_url(url: &str, cx: &mut App) {
    if std::env::var_os("TEMET_WORK_SANDBOX").is_some_and(|v| !v.is_empty()) {
        return;
    }
    cx.open_url(url);
}

impl Render for Project {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = cx.entity().downgrade();
        let open = {
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
        let on_mode = {
            let page = page.clone();
            move |value: &SharedString, _: &mut Window, cx: &mut App| {
                if let Some(mode) = ViewMode::from_value(value) {
                    page.update(cx, |page, cx| {
                        page.set_mode(mode);
                        cx.notify();
                    })
                    .ok();
                }
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
        let on_close = {
            let page = page.clone();
            move |_: &mut Window, cx: &mut App| {
                page.update(cx, |page, cx| {
                    page.close();
                    cx.notify();
                })
                .ok();
            }
        };

        let view = &self.view;
        let selected = self.selected.clone();
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

        let nav = SideNav::new(view.areas.clone())
            .projects(view.projects.clone())
            .selected(view.project.clone())
            .child(StageCounts::new(view.stages.clone()))
            .children((!view.groups.is_empty()).then(|| GroupList::new(view.groups.clone())));

        let control = |id: &'static str, options: Vec<SegmentedControlOption>, value| {
            SegmentedControl::new(id, options)
                .intent(Intent::Primary)
                .value(value)
        };
        let mut actions = vec![
            control(
                "view",
                ViewMode::ALL
                    .into_iter()
                    .map(|v| SegmentedControlOption {
                        label: v.label().into(),
                        value: v.value().into(),
                    })
                    .collect(),
                self.mode.value(),
            )
            .on_value_change(on_mode)
            .into_any_element(),
        ];
        if self.mode == ViewMode::List {
            actions.push(
                classed(Class::TextMuted, cx)
                    .child("Group by")
                    .into_any_element(),
            );
            actions.push(
                control(
                    "group-by",
                    Grouping::ALL
                        .into_iter()
                        .map(|g| SegmentedControlOption {
                            label: g.label().into(),
                            value: g.value().into(),
                        })
                        .collect(),
                    self.grouping.value(),
                )
                .on_value_change(on_group)
                .into_any_element(),
            );
        }
        let header = PageHeader::new(view.title.clone())
            .facts(view.facts.clone())
            .actions_label("View")
            .actions(actions);
        let needs = NeedsYou::new(view.needs.clone())
            .note(format!("{}, costliest to undo first", view.needs.len()));

        let board = self.mode == ViewMode::Board;
        let mut shell = AppShell::new(nav)
            .side_over(board)
            .children(notice)
            .child(header)
            .child(ProblemList::new(view.problems.clone()))
            .child(needs);
        let show_agent = view.agents_running;
        if board {
            // A card's menu offers Open alone: the other actions wait for their commands.
            let menu_open = open.clone();
            shell = shell.wide_child(
                StageBoard::new("board", view.lanes.clone())
                    .selected(selected.clone())
                    .on_open(open.clone())
                    .menu(move |issue, _, _| {
                        let open = menu_open.clone();
                        let key = issue.key.clone();
                        Some(
                            IssueActions::new(
                                SharedString::from(format!("actions-{}", issue.key)),
                                issue.stage,
                            )
                            .on_open(move |window, cx| open(&key, window, cx))
                            .into_any_element(),
                        )
                    }),
            );
        } else {
            match self.grouping.table() {
                None => {
                    for m in &view.milestones {
                        let mut panel = MilestonePanel::new(m.name.clone())
                            .done(m.done)
                            .total(m.total)
                            .note(m.note.clone());
                        if let Some(due) = &m.due {
                            panel = panel.due(due.clone());
                        }
                        let mut table = IssueTable::new(
                            SharedString::from(format!("issues-{}", m.name)),
                            m.issues.clone(),
                        )
                        .selected(selected.clone())
                        .on_select(open.clone());
                        if !show_agent {
                            table = table.show_agent(false);
                        }
                        shell = shell.child(panel.child(table));
                    }
                }
                Some(by) => {
                    let all: Vec<IssueSummary> = view
                        .milestones
                        .iter()
                        .flat_map(|m| m.issues.clone())
                        .collect();
                    let mut panel = MilestonePanel::new("All issues")
                        .summary(all_issues_summary(&view.milestones));
                    // With no issues at all the panel says so, as an empty milestone does.
                    if all.is_empty() {
                        panel = panel.total(0);
                    }
                    let mut table = IssueTable::new("issues-all", all)
                        .group_by(by)
                        .show_milestone(true)
                        .selected(selected.clone())
                        .on_select(open.clone());
                    if !show_agent {
                        table = table.show_agent(false);
                    }
                    shell = shell.child(panel.child(table));
                }
            }
        }

        if let Some(issue) = self.shown() {
            let detail = view.details.get(&issue.key).cloned().unwrap_or_default();
            let facts = detail
                .facts
                .iter()
                .map(|fact| match &fact.value {
                    FactValue::Text(text) => PropertyItem::new(fact.label.clone(), text.clone()),
                    FactValue::Keys(keys) => PropertyItem::links(
                        fact.label.clone(),
                        keys.iter()
                            .map(|key| {
                                let open = open.clone();
                                let to = key.clone();
                                PropertyLink::new(format!("waits-for-{key}"), key.clone())
                                    .mono(true)
                                    .on_click(move |window, cx| open(&to, window, cx))
                            })
                            .collect(),
                    ),
                    FactValue::Pull { number, checks } => {
                        let mut set = vec![format!("#{number}").into_any_element()];
                        if let Some(passed) = checks {
                            let (status, words) = if *passed {
                                (Status::Passed, "Checks passed")
                            } else {
                                (Status::Failed, "Checks failed")
                            };
                            set.push(StatusTag::new(status).children(words).into_any_element());
                        }
                        PropertyItem::set(fact.label.clone(), set)
                    }
                    FactValue::Link { text, url: None } => {
                        PropertyItem::new(fact.label.clone(), text.clone())
                    }
                    FactValue::Link {
                        text,
                        url: Some(url),
                    } => {
                        let url = url.clone();
                        PropertyItem::links(
                            fact.label.clone(),
                            vec![
                                PropertyLink::new("tracker", text.clone())
                                    .on_click(move |_, cx| open_url(&url, cx)),
                            ],
                        )
                    }
                })
                .collect();
            let mut pane = IssuePane::new(issue.key.clone(), issue.title.clone(), issue.stage)
                .round(Some(detail.round))
                .here(true)
                .facts(facts)
                // The pull request is a fact here, as the radar page has it, not a tab.
                .plan(detail.plan)
                .rounds(detail.rounds)
                .decisions(detail.decisions)
                .on_tab_change(on_tab)
                .on_close(on_close);
            if let Some(tab) = self.tab {
                pane = pane.tab(tab);
            }
            shell = shell.side_fit(pane);
        }
        shell.into_any_element()
    }
}
