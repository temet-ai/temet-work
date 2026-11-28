//! Project (`pages-temet-work-radar`): one project. The left column has the projects, the
//! issues' stages and the agent groups; the middle has the page head with View and Group by, the
//! daemon's problems, Needs you, and the issues as a board of stages or a list of milestones; the
//! right has the issue open in the pane. On the board the pane lies over the board's right, so
//! its columns keep their width.
//!
//! The pane's actions menu sends the daemon a person's request: Move to sends `temet-work move`,
//! Assign and Unassign send `temet-work assign` and `unassign`, and each answer shows in a toast,
//! a refusal in a danger toast. Refine opens RefineDialog, whose Save sends `temet-work refine`
//! and shows a refusal in the dialog. An overturn waits for #238's command, so the page passes
//! DecisionRow no handler.

use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    App, Context, FocusHandle, IntoElement, ParentElement, Render, SharedString, Task, Window,
};
use temet_work_core::Command;
use temet_work_core::ids::{AgentId, IssueId};
use temet_work_core::model::Check;
use temet_work_core::view::StageWord;

use crate::components::Intent;
use crate::components::app_shell::AppShell;
use crate::components::callout::Callout;
use crate::components::icon::IconName;
use crate::components::property_list::{PropertyItem, PropertyLink};
use crate::components::segmented_control::{SegmentedControl, SegmentedControlOption};
use crate::components::stage_tag::Stage;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::toast::{OverlayToaster, Toast};
use crate::components::{Class, classed};
use crate::composites::group_list::{GroupList, GroupListGroup};
use crate::composites::issue_actions::{IssueActions, IssueActionsAgent};
use crate::composites::issue_pane::{
    IssueDecision, IssuePane, IssuePlan, IssuePullRequest, IssueTab,
};
use crate::composites::issue_table::{GroupBy, IssueSummary, IssueTable};
use crate::composites::milestone_panel::MilestonePanel;
use crate::composites::needs_you::{NeedsYou, NeedsYouDecision};
use crate::composites::page_header::{PageFact, PageHeader};
use crate::composites::problem_list::{Problem, ProblemList};
use crate::composites::refine_dialog::{RefineCheck, RefineDialog, Sending};
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
    /// The deadline every decision in Needs you shares, which the pane's decisions leave out.
    pub shared_deadline: Option<SharedString>,
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
    /// The issue as the daemon's commands name it; none when there is no daemon to send to.
    pub id: Option<IssueId>,
    /// The stages a person may move it to, as the daemon lists them; Move to offers these.
    pub moves: Vec<Stage>,
    /// The agents a person may assign to it now, each with the issue it is on now.
    pub assignable: Vec<IssueActionsAgent>,
    /// The agents on it now, which a person may unassign.
    pub assigned: Vec<IssueActionsAgent>,
    /// The review round it is in; 0 before its first review.
    pub round: usize,
    pub facts: Vec<IssueFact>,
    pub plan: Option<IssuePlan>,
    pub pull: Option<IssuePullRequest>,
    pub rounds: Option<Vec<ReviewRound>>,
    pub decisions: Option<Vec<IssueDecision>>,
    /// The checks a person holds it to, apart from the plan's; RefineDialog opens with these when
    /// there are any.
    pub person_checks: Vec<RefineCheck>,
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

/// The checks a refine compares and sends: each check's done-when and proven-by, trimmed, those
/// with no done-when left out.
fn sent_checks(checks: &[RefineCheck]) -> Vec<(String, String)> {
    checks
        .iter()
        .map(|c| {
            (
                c.done_when.trim().to_string(),
                c.proven_by.trim().to_string(),
            )
        })
        .filter(|(done_when, _)| !done_when.is_empty())
        .collect()
}

/// The `refine` command a Save sends for `issue`, from what the dialog opened with and what the
/// person has typed. The title goes when it differs from the one the dialog opened with, both
/// trimmed; the checks go when they differ as a list of done-when and proven-by. So a title edit
/// alone does not copy the plan's checks, which the dialog opens with, into the person's. With
/// nothing changed a Save sends nothing (`None`), while "Save and send to refiner" (`send`) still
/// sends `refine` with `send` and neither.
pub fn refine_command(
    issue: IssueId,
    opened: (&str, &[RefineCheck]),
    typed: (&str, &[RefineCheck]),
    send: bool,
) -> Option<Command> {
    let title = typed.0.trim();
    let title = (title != opened.0.trim()).then(|| title.to_string());
    let checks = sent_checks(typed.1);
    let checks = (checks != sent_checks(opened.1)).then(|| {
        checks
            .into_iter()
            .map(|(done_when, proven_by)| Check::new(done_when, proven_by))
            .collect()
    });
    (title.is_some() || checks.is_some() || send).then_some(Command::Refine {
        issue,
        title,
        checks,
        send,
    })
}

/// What a Save says when the window shows sample data, with no daemon to send to.
pub const NO_DAEMON: &str = "no daemon to send the refine to: the window shows sample data";

/// An issue's RefineDialog while it is open: what it opened with and what is being sent. The
/// dialog keeps the person's edit itself.
pub struct Refining {
    /// The issue's key, as the dialog's title names it.
    pub key: SharedString,
    /// The issue as the daemon's commands name it.
    pub issue: IssueId,
    /// The title and checks the dialog opened with.
    pub opened: (String, Vec<RefineCheck>),
    /// Which button's refine is being answered.
    pub sending: Option<Sending>,
    /// Why the last one was not done: the daemon's refusal word for word, or why the daemon
    /// could not be asked.
    pub not_done: Option<SharedString>,
}

/// Sends a command to the daemon as a person's request: the daemon's text when it did it, or why
/// it was not done, such as its refusal.
pub type Sender = Rc<dyn Fn(Command, &mut App) -> Task<Result<String, String>>>;

/// A toast: its message, and its intent, danger for a command the daemon did not do.
#[derive(Clone, Debug, PartialEq)]
struct Toasted {
    message: SharedString,
    intent: Intent,
}

/// What a person asks the daemon to do with an agent from an issue's actions menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Assign,
    Unassign,
}

/// The view's stage word for a stage, as `move` takes it.
fn word(stage: Stage) -> StageWord {
    match stage {
        Stage::New => StageWord::New,
        Stage::Refining => StageWord::Refining,
        Stage::Ready => StageWord::Ready,
        Stage::Working => StageWord::Working,
        Stage::InReview => StageWord::InReview,
        Stage::Fixing => StageWord::Fixing,
        Stage::WithArbiter => StageWord::Arbitration,
        Stage::Merging => StageWord::Merging,
        Stage::Deploying => StageWord::Deploying,
        Stage::Verifying => StageWord::Verifying,
        Stage::Done => StageWord::Done,
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
    /// The toast on show.
    toast: Option<Toasted>,
    /// Whether the selected issue's actions menu is open.
    actions_open: bool,
    /// How a command reaches the daemon; none on sample data.
    sender: Option<Sender>,
    /// The focus of the selected issue's actions button.
    actions_focus: Option<FocusHandle>,
    /// The RefineDialog, while it is open.
    refining: Option<Refining>,
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
            toast: None,
            actions_open: false,
            sender: None,
            actions_focus: None,
            refining: None,
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

    /// Sends the page's commands to the daemon with `sender`.
    pub fn set_sender(
        &mut self,
        sender: impl Fn(Command, &mut App) -> Task<Result<String, String>> + 'static,
    ) {
        self.sender = Some(Rc::new(sender));
    }

    /// Whether the selected issue's actions menu is open.
    pub fn actions_open(&self) -> bool {
        self.actions_open
    }

    /// Opens or closes the selected issue's actions menu.
    pub fn set_actions_open(&mut self, open: bool) {
        self.actions_open = open;
    }

    /// The command that moves the selected issue to `stage`, when the daemon gave its id.
    pub fn move_command(&self, stage: Stage) -> Option<Command> {
        let key = self.selected.as_ref()?;
        let issue = self.view.details.get(key)?.id.clone()?;
        Some(Command::Move {
            issue,
            stage: word(stage),
        })
    }

    /// Sends `command` with the page's sender and shows the answer with [`Project::sent`];
    /// `asked` names the request in a refusal, as in "Assign reviewer-4 to watch#12: ...".
    fn send(&mut self, command: Command, asked: Option<String>, cx: &mut Context<Self>) {
        let prefix = move |why: String| match &asked {
            Some(asked) => format!("{asked}: {why}"),
            None => why,
        };
        let Some(sender) = self.sender.clone() else {
            self.sent(Err(prefix(
                "no daemon to send it to: the window shows sample data".into(),
            )));
            return;
        };
        let task = sender(command, cx);
        cx.spawn(async move |page, cx| {
            let answer = task.await.map_err(prefix);
            page.update(cx, |page, cx| {
                page.sent(answer);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Moves the selected issue to `stage`: the menu closes, and the command goes to the daemon.
    /// Its answer shows in a toast: the new stage and any pull request left open, or in a danger
    /// toast why it was not done. An accepted move shows in the issue's stage from the next view.
    pub fn move_to(&mut self, stage: Stage, cx: &mut Context<Self>) {
        self.actions_open = false;
        if let Some(command) = self.move_command(stage) {
            self.send(command, None, cx);
        }
    }

    /// The command that assigns `agent` to the selected issue, or unassigns it, when the daemon
    /// gave the issue's id.
    pub fn agent_command(&self, verb: Verb, agent: &str) -> Option<Command> {
        let key = self.selected.as_ref()?;
        let issue = self.view.details.get(key)?.id.clone()?;
        let agent = AgentId::new(agent);
        Some(match verb {
            Verb::Assign => Command::Assign { issue, agent },
            Verb::Unassign => Command::Unassign { issue, agent },
        })
    }

    /// Assigns `agent` to the selected issue, or unassigns it: the menu closes, and the command
    /// goes to the daemon. Its answer shows in a toast, as a move's does; a refusal shows in a
    /// danger toast that names what was asked: "Assign reviewer-4 to watch#12: ...".
    pub fn send_agent(&mut self, verb: Verb, agent: &str, cx: &mut Context<Self>) {
        self.actions_open = false;
        let Some(command) = self.agent_command(verb, agent) else {
            return;
        };
        let asked = match &command {
            Command::Assign { issue, agent } => format!("Assign {agent} to {issue}"),
            Command::Unassign { issue, agent } => format!("Unassign {agent} from {issue}"),
            _ => String::new(),
        };
        self.send(command, Some(asked), cx);
    }

    /// Shows the daemon's answer to a command: its text in the page's toast, or why it was not
    /// done in a danger toast.
    pub fn sent(&mut self, answer: Result<String, String>) {
        self.toast = Some(match answer {
            Ok(text) => Toasted {
                message: text.into(),
                intent: Intent::None,
            },
            Err(why) => Toasted {
                message: why.into(),
                intent: Intent::Danger,
            },
        });
    }

    /// Takes the toast away.
    pub fn dismiss_toast(&mut self) {
        self.toast = None;
    }

    /// The message of the toast on show.
    pub fn toast(&self) -> Option<&SharedString> {
        self.toast.as_ref().map(|t| &t.message)
    }

    /// The intent of the toast on show: danger for a command the daemon did not do.
    pub fn toast_intent(&self) -> Option<Intent> {
        self.toast.as_ref().map(|t| t.intent)
    }

    /// The RefineDialog, while it is open.
    pub fn refining(&self) -> Option<&Refining> {
        self.refining.as_ref()
    }

    /// The focus of the selected issue's actions button.
    pub fn actions_focus(&mut self, cx: &mut App) -> FocusHandle {
        self.actions_focus
            .get_or_insert_with(|| cx.focus_handle())
            .clone()
    }

    /// Opens RefineDialog for the issue `key`, with its title, and the person's checks when it
    /// has any, else its plan's. The actions menu closes. Nothing opens for an issue the daemon
    /// gave no id.
    pub fn open_refine(&mut self, key: &SharedString) {
        self.actions_open = false;
        let Some(issue) = self
            .view
            .milestones
            .iter()
            .flat_map(|m| m.issues.iter())
            .find(|i| &i.key == key)
        else {
            return;
        };
        let Some(detail) = self.view.details.get(key) else {
            return;
        };
        let Some(id) = detail.id.clone() else {
            return;
        };
        let checks: Vec<RefineCheck> = if detail.person_checks.is_empty() {
            detail
                .plan
                .iter()
                .flat_map(|p| p.checks.iter())
                .map(|c| RefineCheck::new(c.done_when.clone(), c.proven_by.clone()))
                .collect()
        } else {
            detail.person_checks.clone()
        };
        self.refining = Some(Refining {
            key: key.clone(),
            issue: id,
            opened: (issue.title.to_string(), checks),
            sending: None,
            not_done: None,
        });
    }

    /// Closes RefineDialog without saving, unless what it sent is being answered; the focus goes
    /// back to the actions button.
    pub fn close_refine(&mut self, window: &mut Window, cx: &mut App) {
        if self.refining.as_ref().is_some_and(|r| r.sending.is_some()) {
            return;
        }
        self.refining = None;
        let focus = self.actions_focus(cx);
        window.focus(&focus, cx);
    }

    /// Shows `why` in the open dialog, as a refused refine leaves it; for drawing that state
    /// without a daemon.
    pub fn set_refine_not_done(&mut self, why: impl Into<SharedString>) {
        if let Some(r) = &mut self.refining {
            r.not_done = Some(why.into());
        }
    }

    /// Save, or "Save and send to refiner" with `send`, of `title` and `checks` as the person
    /// left them: sends what changed as a person's `refine` with the page's sender, and keeps the
    /// buttons busy until the answer. Done closes the dialog and puts the focus back on the
    /// actions button; a refusal or a failure shows its reason in the dialog, which keeps the
    /// edit. With nothing to send, the dialog closes.
    pub fn save_refine(
        &mut self,
        title: &str,
        checks: &[RefineCheck],
        send: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(r) = &self.refining else {
            return;
        };
        if r.sending.is_some() || title.trim().is_empty() {
            return;
        }
        let command = refine_command(
            r.issue.clone(),
            (&r.opened.0, &r.opened.1),
            (title, checks),
            send,
        );
        let Some(command) = command else {
            self.close_refine(window, cx);
            return;
        };
        let Some(sender) = self.sender.clone() else {
            self.set_refine_not_done(NO_DAEMON);
            return;
        };
        if let Some(r) = &mut self.refining {
            r.sending = Some(if send {
                Sending::SaveAndSend
            } else {
                Sending::Save
            });
            r.not_done = None;
        }
        let task = sender(command, cx);
        cx.spawn_in(window, async move |page, cx| {
            let answer = task.await;
            page.update_in(cx, |page, window, cx| {
                page.refine_answered(answer, window, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The daemon's answer to the refine the open dialog sent.
    fn refine_answered(
        &mut self,
        answer: Result<String, String>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(r) = &mut self.refining else {
            return;
        };
        r.sending = None;
        match answer {
            Ok(_) => self.close_refine(window, cx),
            Err(why) => r.not_done = Some(why.into()),
        }
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
        let actions_focus = self.actions_focus(cx);
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
        if board {
            // The board's cards get their actions menu with the board itself; until then the
            // pane's menu has them.
            shell = shell.wide_child(
                StageBoard::new("board", view.lanes.clone())
                    .selected(selected.clone())
                    .on_open(open.clone()),
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
                        let table = IssueTable::new(
                            SharedString::from(format!("issues-{}", m.name)),
                            m.issues.clone(),
                        )
                        .selected(selected.clone())
                        .on_select(open.clone())
                        // The agents and their issues are the left column's to show.
                        .show_agent(false);
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
                    let table = IssueTable::new("issues-all", all)
                        .group_by(by)
                        .show_milestone(true)
                        .selected(selected.clone())
                        .on_select(open.clone())
                        .show_agent(false);
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
                .shared_deadline(view.shared_deadline.clone())
                .on_tab_change(on_tab)
                .on_close(on_close);
            if let Some(tab) = self.tab {
                pane = pane.tab(tab);
            }
            // Refine, Move to, Assign and Unassign need the issue's id from the daemon; Move to,
            // Assign and Unassign offer what the daemon takes, and are left out with nothing to
            // offer.
            let actions = detail.id.is_some().then(|| {
                let refine_page = page.clone();
                let key = issue.key.clone();
                let mut actions = IssueActions::new(issue.stage).on_refine(
                    move |_: &mut Window, cx: &mut App| {
                        refine_page
                            .update(cx, |p, cx| {
                                p.open_refine(&key);
                                cx.notify();
                            })
                            .ok();
                    },
                );
                if !detail.moves.is_empty() {
                    let page = page.clone();
                    actions = actions.move_to(detail.moves.clone()).on_move(
                        move |stage, _: &mut Window, cx: &mut App| {
                            page.update(cx, |p, cx| {
                                p.move_to(stage, cx);
                                cx.notify();
                            })
                            .ok();
                        },
                    );
                }
                for (verb, agents) in [
                    (Verb::Assign, detail.assignable.clone()),
                    (Verb::Unassign, detail.assigned.clone()),
                ] {
                    let page = page.clone();
                    let handler = move |agent: &SharedString, _: &mut Window, cx: &mut App| {
                        page.update(cx, |p, cx| {
                            p.send_agent(verb, agent, cx);
                            cx.notify();
                        })
                        .ok();
                    };
                    actions = match verb {
                        Verb::Assign => actions.assignable(agents).on_assign(handler),
                        Verb::Unassign => actions.assigned(agents).on_unassign(handler),
                    };
                }
                actions
            });
            if let Some(actions) = actions {
                let page = page.clone();
                pane = pane
                    .actions(actions)
                    .actions_focus(&actions_focus)
                    .actions_open(self.actions_open)
                    .on_actions(move |open, _: &mut Window, cx: &mut App| {
                        page.update(cx, |p, cx| {
                            p.set_actions_open(open);
                            cx.notify();
                        })
                        .ok();
                    });
            }
            shell = shell.side_fit(pane);
        }

        let mut toaster = OverlayToaster::new();
        if let Some(toast) = &self.toast {
            // The page's own toast has the Notifications icon and no intent; a command the daemon
            // did not do is a danger toast with a warning.
            let icon = match toast.intent {
                Intent::Danger => IconName::WarningSign,
                _ => IconName::Notifications,
            };
            let page = page.clone();
            toaster = toaster.child(
                Toast::new(toast.message.clone())
                    .icon(icon)
                    .intent(toast.intent)
                    .on_dismiss(move |_: &mut Window, cx: &mut App| {
                        page.update(cx, |p, cx| {
                            p.dismiss_toast();
                            cx.notify();
                        })
                        .ok();
                    }),
            );
        }
        let refine = self.refining.as_ref().map(|r| {
            let close_page = page.clone();
            let save_page = page.clone();
            RefineDialog::new(
                SharedString::from(format!("refine-{}", r.key)),
                r.key.clone(),
                r.opened.0.clone(),
                r.opened.1.clone(),
            )
            .is_open(true)
            .sending(r.sending)
            .not_done(r.not_done.clone())
            .on_close(move |window: &mut Window, cx: &mut App| {
                close_page
                    .update(cx, |p, cx| {
                        p.close_refine(window, cx);
                        cx.notify();
                    })
                    .ok();
            })
            .on_save(
                move |title: &SharedString,
                      checks: &[RefineCheck],
                      send: bool,
                      window: &mut Window,
                      cx: &mut App| {
                    save_page
                        .update(cx, |p, cx| {
                            p.save_refine(title, checks, send, window, cx);
                            cx.notify();
                        })
                        .ok();
                },
            )
        });
        shell.children(refine).child(toaster).into_any_element()
    }
}
