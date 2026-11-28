//! IssuePane (`temet-issuepane`): the right column: the selected issue's title, key and stage
//! with its round, its actions menu behind a button, then its facts, plan, pull request, review
//! rounds and decisions as tabs, each with its count; with only one of those it shows it with no
//! tab row. A tab shows only when its data is given.
//!
//! The head stays put with a quiet rule under it, and the facts, tabs and content scroll in
//! their own area under it (`tm-issue-scroll`), so the close button is always in view and nothing
//! passes under the head. With `on_close` the head has a close button, and q closes the pane
//! unless the focus is in a text field, where q is typed. It goes in AppShell's `side_fit`.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
};

use super::check_list::{CheckList, PlanCheck};
use super::decision_row::{Decision, DecisionRow, OverturnChoice, OverturnDraft};
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

/// A decision taken on the issue, with an id to tell it apart, and where its overturn stands
/// when one is under way.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueDecision {
    /// Unique among the issue's decisions.
    pub id: SharedString,
    pub decision: Decision,
    /// Opens the row's overturn confirm at first, with these filled in.
    pub draft: Option<OverturnDraft>,
    /// Set while the app sends the overturn.
    pub pending: bool,
    /// Why the last send failed, such as the daemon's reply.
    pub error: Option<SharedString>,
}

type OnTabChange = Rc<dyn Fn(IssueTab, &mut Window, &mut App)>;
type OnOverturn = Rc<dyn Fn(&SharedString, &OverturnChoice, &mut Window, &mut App)>;
type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;
type OnActions = Rc<dyn Fn(bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssuePane {
    issue_key: SharedString,
    title: SharedString,
    stage: Stage,
    round: Option<usize>,
    facts: Option<Vec<PropertyItem>>,
    plan: Option<IssuePlan>,
    pull: Option<IssuePullRequest>,
    rounds: Option<Vec<ReviewRound>>,
    decisions: Option<Vec<IssueDecision>>,
    tab: Option<IssueTab>,
    on_tab_change: Option<OnTabChange>,
    on_overturn: Option<OnOverturn>,
    on_close: Option<OnClose>,
    actions: Option<IssueActions>,
    actions_open: bool,
    on_actions: Option<OnActions>,
    actions_focus: Option<FocusHandle>,
    here: bool,
    shared_deadline: Option<SharedString>,
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
            on_close: None,
            actions: None,
            actions_open: false,
            on_actions: None,
            actions_focus: None,
            here: false,
            shared_deadline: None,
        }
    }

    /// The issue's actions menu, behind a button at the right of the head.
    pub fn actions(mut self, actions: IssueActions) -> Self {
        self.actions = Some(actions);
        self
    }

    /// Whether the actions menu is open: its Popover's `isOpen`, which the page holds.
    pub fn actions_open(mut self, open: bool) -> Self {
        self.actions_open = open;
        self
    }

    /// Called with the state the person asks the actions menu for: its Popover's
    /// `onInteraction`.
    pub fn on_actions(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_actions = Some(Rc::new(handler));
        self
    }

    /// The focus the actions button holds, so the focus can go back to it once what its menu
    /// opened, such as RefineDialog, has closed.
    pub fn actions_focus(mut self, focus: &FocusHandle) -> Self {
        self.actions_focus = Some(focus.clone());
        self
    }

    /// The review round, shown with the stage as the daemon writes it ("fixing, round 1").
    pub fn round(mut self, round: Option<usize>) -> Self {
        self.round = round;
        self
    }

    /// Facts about the issue, such as its milestone, group and agents, in a panel above the
    /// tabs.
    pub fn facts(mut self, facts: Vec<PropertyItem>) -> Self {
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

    /// Called with a decision's id and the option and reason given, once its overturn is
    /// confirmed. Without it the decisions have no Overturn button.
    pub fn on_overturn(
        mut self,
        handler: impl Fn(&SharedString, &OverturnChoice, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overturn = Some(Rc::new(handler));
        self
    }

    /// Closes the pane. With it the head has a close button, and q closes the pane unless the
    /// focus is in a text field.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    /// A deadline the screen already states for every open decision, such as Needs you's
    /// "overturn before merge": decisions with it leave it out, and only one whose deadline
    /// differs shows its own.
    pub fn shared_deadline(mut self, deadline: Option<SharedString>) -> Self {
        self.shared_deadline = deadline;
        self
    }

    /// Marks the facts panel with CutPanel's `here` edge, as the item the person is in.
    pub fn here(mut self, here: bool) -> Self {
        self.here = here;
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

/// The deadline every open decision shares, which the rows then leave out; none when there is
/// one open decision or fewer, or they differ.
pub fn one_deadline(decisions: &[Decision]) -> Option<SharedString> {
    let open: Vec<&Decision> = decisions
        .iter()
        .filter(|d| d.overturned.is_none())
        .collect();
    let first = open.first()?.overturn_before.clone();
    (open.len() > 1 && open.iter().all(|d| d.overturn_before == first))
        .then_some(first)
        .flatten()
}

/// What closes the pane on q: the latest `on_close`, read by an observer of every keystroke the
/// app takes, which lives as long as the pane is drawn.
struct CloseOnQ {
    on_close: Rc<RefCell<Option<OnClose>>>,
    _keys: Subscription,
}

/// Whether a keystroke closes the pane: a q with no modifier but shift, typed outside a text
/// field (an `Input` key context on the focused path).
pub fn closes(key: &str, command: bool, in_field: bool) -> bool {
    key == "q" && !command && !in_field
}

impl RenderOnce for IssuePane {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let key = self.issue_key.clone();
        if let Some(on_close) = self.on_close.clone() {
            let id = ElementId::Name(SharedString::from(format!("pane-q-{key}")));
            let state = window.use_keyed_state(id, cx, |_, cx| CloseOnQ {
                on_close: Rc::default(),
                _keys: cx.observe_keystrokes(|this: &mut CloseOnQ, event, window, cx| {
                    let m = &event.keystroke.modifiers;
                    let in_field = event.context_stack.iter().any(|c| c.contains("Input"));
                    let command = m.control || m.alt || m.platform;
                    if !closes(&event.keystroke.key, command, in_field) {
                        return;
                    }
                    let handler = this.on_close.borrow().clone();
                    if let Some(handler) = handler {
                        handler(window, cx);
                    }
                }),
            });
            *state.read(cx).on_close.borrow_mut() = Some(on_close);
        }

        let shown = tabs_shown(
            self.plan.is_some(),
            self.pull.is_some(),
            self.rounds.is_some(),
            self.decisions.is_some(),
        );
        let close = self.on_close.clone().map(|on_close| {
            Button::new(SharedString::from(format!("pane-close-{key}")))
                .variant(Variant::Minimal)
                .size(Size::Small)
                .icon(IconName::Cross)
                .on_click(move |_, window, cx| on_close(window, cx))
        });
        // The design system's head buttons: a small minimal button with More, opening the menu
        // below it, its right edge on the button's; then the close button.
        let actions_focus = self.actions_focus;
        let on_actions = self.on_actions;
        let actions_open = self.actions_open;
        let actions = self.actions.map(|menu| {
            let mut button = Button::new("issue-actions")
                .icon(IconName::More)
                .variant(Variant::Minimal)
                .size(Size::Small);
            if let Some(focus) = &actions_focus {
                button = button.track_focus(focus);
            }
            let mut popover = Popover::new("issue-actions-popover")
                .placement(Placement::BottomEnd)
                .is_open(actions_open)
                .content(menu)
                .child(button);
            if let Some(handler) = on_actions {
                popover = popover.on_interaction(move |open, window, cx| handler(open, window, cx));
            }
            popover
        });
        let buttons = (actions.is_some() || close.is_some()).then(|| {
            div()
                .flex()
                .flex_none()
                .gap(space::s1(cx))
                .children(actions)
                .children(close)
        });
        let head = classed(Class::IssueHead, cx)
            .flex()
            .flex_none()
            .justify_between()
            .items_start()
            .gap(space::s4(cx))
            .debug_selector(|| "pane-head".to_string())
            .child(
                div()
                    .min_w_0()
                    .child(Heading::new(self.title).size(HeadingSize::Pane))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(space::s4(cx))
                            .child(classed(Class::Mono, cx).child(key.clone()))
                            .child(StageTag::new(self.stage).round(self.round)),
                    ),
            )
            .children(buttons);
        let facts = self.facts.map(|facts| {
            CutPanel::new()
                .padded(true)
                .here(self.here)
                .child(PropertyList::new(facts))
        });

        // Each tab's panel and its count.
        let mut panels: Vec<(IssueTab, &'static str, Option<usize>, AnyElement)> = Vec::new();
        if let Some(plan) = self.plan {
            let props = [
                plan.folder
                    .map(|folder| PropertyItem::new("Folder", folder).mono(true)),
                plan.merge.map(|merge| PropertyItem::new("Merge", merge)),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            let count = plan.checks.len();
            let panel = div()
                .flex()
                .flex_col()
                .gap(space::s5(cx))
                .children((!props.is_empty()).then(|| PropertyList::new(props)))
                .child(CheckList::new(
                    SharedString::from(format!("checks-{key}")),
                    plan.checks,
                ));
            panels.push((
                IssueTab::Plan,
                "Plan",
                Some(count),
                panel.into_any_element(),
            ));
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
            panels.push((
                IssueTab::Pull,
                "Pull request",
                None,
                props.into_any_element(),
            ));
        }
        if let Some(rounds) = self.rounds {
            let count = rounds.len();
            panels.push((
                IssueTab::Review,
                "Review",
                Some(count),
                ReviewRounds::new(rounds).into_any_element(),
            ));
        }
        if let Some(decisions) = self.decisions {
            let plain: Vec<Decision> = decisions.iter().map(|d| d.decision.clone()).collect();
            let open = plain.iter().filter(|d| d.overturned.is_none()).count();
            let shared = one_deadline(&plain).is_some();
            let screen = self.shared_deadline;
            let empty = decisions
                .is_empty()
                .then(|| ListRow::new("No decisions yet"));
            let on_overturn = self.on_overturn;
            let rows = decisions.into_iter().map(|d| {
                let mut decision = d.decision;
                // A deadline the screen states, or every decision here shares, is not said again.
                if shared || (screen.is_some() && decision.overturn_before == screen) {
                    decision.overturn_before = None;
                }
                let row = DecisionRow::new(d.id.clone(), decision)
                    .default_draft(d.draft)
                    .pending(d.pending)
                    .error(d.error);
                match on_overturn.clone() {
                    Some(handler) => {
                        let id = d.id;
                        row.on_overturn(move |choice, window, cx| handler(&id, choice, window, cx))
                    }
                    None => row,
                }
            });
            let panel = div().children(empty).children(rows);
            panels.push((
                IssueTab::Decisions,
                "Decisions",
                Some(open),
                panel.into_any_element(),
            ));
        }

        let content = if panels.len() == 1 {
            panels.pop().map(|(_, _, _, panel)| panel)
        } else if panels.is_empty() {
            None
        } else {
            let mut tabs = Tabs::new(SharedString::from(format!("tabs-{key}")));
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
            for (tab, title, count, panel) in panels {
                let count = count.filter(|n| *n > 0).map(|n| n.to_string());
                tabs = tabs.child(Tab::new(tab.id(), title).tag_content(count).panel(panel));
            }
            Some(tabs.into_any_element())
        };

        let scroll = classed(Class::IssueScroll, cx)
            .id(SharedString::from(format!("pane-scroll-{key}")))
            .debug_selector(|| "pane-scroll".to_string())
            .flex()
            .flex_col()
            .min_h_0()
            .gap(space::s5(cx))
            .overflow_y_scroll()
            .children(facts)
            .children(content);
        div()
            .flex()
            .flex_col()
            .min_h_0()
            .debug_selector(|| "pane".to_string())
            .child(head)
            .child(scroll)
    }
}
