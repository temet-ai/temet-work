//! IssuePane (`temet-issuepane`): the right column: the selected issue's title, key and stage,
//! with its actions menu behind a button at the right; its facts; then its plan, pull request,
//! review rounds and decisions as tabs. A tab shows only when its data is given.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::check_list::{CheckList, PlanCheck};
use super::decision_row::{Decision, DecisionRow};
use super::issue_actions::IssueActions;
use super::review_rounds::{ReviewRound, ReviewRounds};
use crate::components::button::{Button, Size, Variant};
use crate::components::cut_panel::CutPanel;
use crate::components::heading::{Heading, HeadingSize};
use crate::components::icon::IconName;
use crate::components::list_row::ListRow;
use crate::components::popover::{Placement, Popover};
use crate::components::property_list::{PropertyItem, PropertyList};
use crate::components::stage_tag::{Stage, StageTag};
use crate::components::status_tag::{Status, StatusTag};
use crate::components::tabs::{Tab, Tabs};
use crate::components::{Class, classed};
use crate::theme::space;

/// The tabs of an issue's pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueTab {
    Plan,
    Pull,
    Review,
    Decisions,
}

impl IssueTab {
    pub const ALL: [IssueTab; 4] = [Self::Plan, Self::Pull, Self::Review, Self::Decisions];

    /// The tab's id, as the design system names it.
    pub fn id(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Pull => "pull",
            Self::Review => "review",
            Self::Decisions => "decisions",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.id() == id)
    }
}

/// How an issue is to be done and shown to be done.
#[derive(Clone, Debug, PartialEq)]
pub struct IssuePlan {
    /// The folder the work happens in, with its machine, such as omit:~/work/watch-2.
    pub folder: Option<SharedString>,
    /// How the work is merged.
    pub merge: Option<SharedString>,
    pub checks: Vec<PlanCheck>,
}

/// An issue's pull request.
#[derive(Clone, Debug, PartialEq)]
pub struct IssuePullRequest {
    pub number: u32,
    /// The repository it is in, when the issue has pull requests in more than one.
    pub repo: Option<SharedString>,
    /// Whether it has merged.
    pub merged: Option<bool>,
    pub branch: Option<SharedString>,
    /// Where its CI stands.
    pub ci: Option<Status>,
    /// Text in place of the CI status word, such as "CI passed in 3 m 12 s".
    pub ci_text: Option<SharedString>,
    /// Lines added and removed.
    pub lines: Option<(u32, u32)>,
    pub files: Option<u32>,
}

/// A decision taken on the issue, with an id to tell it apart.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueDecision {
    pub id: SharedString,
    pub decision: Decision,
}

type OnTabChange = Rc<dyn Fn(IssueTab, &mut Window, &mut App)>;
type OnOverturn = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssuePane {
    issue_key: SharedString,
    title: SharedString,
    stage: Stage,
    round: Option<usize>,
    facts: Option<Vec<(SharedString, SharedString)>>,
    plan: Option<IssuePlan>,
    pull: Option<IssuePullRequest>,
    rounds: Option<Vec<ReviewRound>>,
    decisions: Option<Vec<IssueDecision>>,
    tab: Option<IssueTab>,
    on_tab_change: Option<OnTabChange>,
    on_overturn: Option<OnOverturn>,
    actions: Option<IssueActions>,
}

impl IssuePane {
    pub fn new(
        issue_key: impl Into<SharedString>,
        title: impl Into<SharedString>,
        stage: Stage,
    ) -> Self {
        Self {
            issue_key: issue_key.into(),
            title: title.into(),
            stage,
            round: None,
            facts: None,
            plan: None,
            pull: None,
            rounds: None,
            decisions: None,
            tab: None,
            on_tab_change: None,
            on_overturn: None,
            actions: None,
        }
    }

    /// The issue's actions menu, behind a button at the top right.
    pub fn actions(mut self, actions: IssueActions) -> Self {
        self.actions = Some(actions);
        self
    }

    /// The review round, shown with the stage as the daemon writes it ("fixing, round 1").
    pub fn round(mut self, round: Option<usize>) -> Self {
        self.round = round;
        self
    }

    /// Facts about the issue, such as milestone, agent and round, in a panel above the tabs.
    pub fn facts(mut self, facts: Vec<(SharedString, SharedString)>) -> Self {
        self.facts = Some(facts);
        self
    }

    pub fn plan(mut self, plan: Option<IssuePlan>) -> Self {
        self.plan = plan;
        self
    }

    pub fn pull(mut self, pull: Option<IssuePullRequest>) -> Self {
        self.pull = pull;
        self
    }

    /// The review rounds; an empty list says the issue is not reviewed yet.
    pub fn rounds(mut self, rounds: Option<Vec<ReviewRound>>) -> Self {
        self.rounds = rounds;
        self
    }

    pub fn decisions(mut self, decisions: Option<Vec<IssueDecision>>) -> Self {
        self.decisions = decisions;
        self
    }

    /// The open tab; the first one shown when it is not shown.
    pub fn tab(mut self, tab: IssueTab) -> Self {
        self.tab = Some(tab);
        self
    }

    pub fn on_tab_change(
        mut self,
        handler: impl Fn(IssueTab, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_tab_change = Some(Rc::new(handler));
        self
    }

    /// Called with a decision's id by its Overturn button.
    pub fn on_overturn(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overturn = Some(Rc::new(handler));
        self
    }
}

/// The tabs an issue shows, in order: one for each part of its data that is given.
pub fn tabs_shown(plan: bool, pull: bool, rounds: bool, decisions: bool) -> Vec<IssueTab> {
    [
        (plan, IssueTab::Plan),
        (pull, IssueTab::Pull),
        (rounds, IssueTab::Review),
        (decisions, IssueTab::Decisions),
    ]
    .into_iter()
    .filter_map(|(shown, tab)| shown.then_some(tab))
    .collect()
}

impl RenderOnce for IssuePane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let shown = tabs_shown(
            self.plan.is_some(),
            self.pull.is_some(),
            self.rounds.is_some(),
            self.decisions.is_some(),
        );
        // `tm-issue-head`: the title and its key line at the left, the buttons at the right.
        let buttons = self.actions.map(|actions| {
            let popover = Popover::new(SharedString::from(format!("actions-{}", self.issue_key)))
                .placement(Placement::BottomEnd)
                .content(actions)
                .child(
                    Button::new("issue-actions")
                        .size(Size::Small)
                        .variant(Variant::Minimal)
                        .icon(IconName::More),
                );
            div().flex().flex_none().gap(space::s1(cx)).child(popover)
        });
        let head = div()
            .flex()
            .justify_between()
            .items_start()
            .gap(space::s4(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Heading::new(self.title).size(HeadingSize::Pane))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(space::s4(cx))
                            .child(classed(Class::Mono, cx).child(self.issue_key.clone()))
                            .child(StageTag::new(self.stage).round(self.round)),
                    ),
            )
            .children(buttons);
        let facts = self.facts.map(|facts| {
            let items = facts
                .into_iter()
                .map(|(label, value)| PropertyItem::new(label, value))
                .collect();
            CutPanel::new().padded(true).child(PropertyList::new(items))
        });

        let mut tabs = Tabs::new(SharedString::from(format!("tabs-{}", self.issue_key)));
        if let Some(open) = self.tab.filter(|t| shown.contains(t)) {
            tabs = tabs.selected_tab_id(open.id());
        }
        if let Some(handler) = self.on_tab_change {
            tabs = tabs.on_change(move |id, window, cx| {
                if let Some(tab) = IssueTab::from_id(id) {
                    handler(tab, window, cx);
                }
            });
        }
        if let Some(plan) = self.plan {
            let props = PropertyList::new(
                [
                    plan.folder
                        .map(|folder| PropertyItem::new("Folder", folder).mono(true)),
                    plan.merge.map(|merge| PropertyItem::new("Merge", merge)),
                ]
                .into_iter()
                .flatten()
                .collect(),
            );
            let panel = div()
                .flex()
                .flex_col()
                .gap(space::s5(cx))
                .child(props)
                .child(CheckList::new(
                    SharedString::from(format!("checks-{}", self.issue_key)),
                    plan.checks,
                ));
            tabs = tabs.child(Tab::new("plan", "Plan").panel(panel));
        }
        if let Some(pull) = self.pull {
            let ci = pull.ci.map(|status| match pull.ci_text {
                Some(text) => StatusTag::new(status).children(text),
                None => StatusTag::new(status),
            });
            let props = PropertyList::new(
                [
                    Some(PropertyItem::new("Number", format!("#{}", pull.number))),
                    pull.repo
                        .map(|repo| PropertyItem::new("Repository", repo).mono(true)),
                    pull.merged.map(|merged| {
                        PropertyItem::new("State", if merged { "Merged" } else { "Open" })
                    }),
                    pull.branch
                        .map(|branch| PropertyItem::new("Branch", branch).mono(true)),
                    ci.map(|ci| PropertyItem::element("CI", ci)),
                    pull.lines.map(|(added, removed)| {
                        PropertyItem::new("Lines", format!("+{added} \u{2212}{removed}"))
                    }),
                    pull.files
                        .map(|files| PropertyItem::new("Files", files.to_string())),
                ]
                .into_iter()
                .flatten()
                .collect(),
            );
            tabs = tabs.child(Tab::new("pull", "Pull request").panel(props));
        }
        if let Some(rounds) = self.rounds {
            tabs = tabs.child(Tab::new("review", "Review").panel(ReviewRounds::new(rounds)));
        }
        if let Some(decisions) = self.decisions {
            let empty = decisions
                .is_empty()
                .then(|| ListRow::new("No decisions yet"));
            let on_overturn = self.on_overturn;
            let rows = decisions.into_iter().map(|d| {
                let row = DecisionRow::new(d.id.clone(), d.decision);
                match on_overturn.clone() {
                    Some(handler) => {
                        let id = d.id;
                        row.on_overturn(move |window, cx| handler(&id, window, cx))
                    }
                    None => row,
                }
            });
            let panel = div().children(empty).children(rows);
            tabs = tabs.child(Tab::new("decisions", "Decisions").panel(panel));
        }
        div()
            .flex()
            .flex_col()
            .gap(space::s5(cx))
            .child(head)
            .children(facts)
            .children((!shown.is_empty()).then_some(tabs))
    }
}
