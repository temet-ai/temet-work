//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu. The
//! design system's menu also has Open, Refine and Move to; an item shows only when its handler
//! is given, and the window gives Assign and Unassign only. Assign and Unassign show only with
//! agents to list, each opening a submenu of agents.

use std::rc::Rc;

use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

use crate::components::icon::IconName;
use crate::components::menu::{Menu, MenuItem};

/// An agent in the Assign or Unassign list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueActionsAgent {
    /// The agent's id, passed back to `on_assign` or `on_unassign`.
    pub id: SharedString,
    /// The agent as the list names it, such as reviewer-4.
    pub label: SharedString,
    /// Muted text at the right, such as the issue it is on now or "free".
    pub detail: Option<SharedString>,
}

type OnAgent = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement, Default)]
pub struct IssueActions {
    assignable: Vec<IssueActionsAgent>,
    on_assign: Option<OnAgent>,
    assigned: Vec<IssueActionsAgent>,
    on_unassign: Option<OnAgent>,
}

impl IssueActions {
    pub fn new() -> Self {
        Self::default()
    }

    /// The agents that can be put on the issue.
    pub fn assignable(mut self, agents: Vec<IssueActionsAgent>) -> Self {
        self.assignable = agents;
        self
    }

    /// Puts an agent on the issue: called with the agent's id.
    pub fn on_assign(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_assign = Some(Rc::new(handler));
        self
    }

    /// The agents on the issue now.
    pub fn assigned(mut self, agents: Vec<IssueActionsAgent>) -> Self {
        self.assigned = agents;
        self
    }

    /// Takes an agent off the issue: called with the agent's id.
    pub fn on_unassign(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_unassign = Some(Rc::new(handler));
        self
    }

    /// Whether the menu lists anything: Assign or Unassign, each with agents and its handler.
    pub fn has_items(&self) -> bool {
        (self.on_assign.is_some() && !self.assignable.is_empty())
            || (self.on_unassign.is_some() && !self.assigned.is_empty())
    }
}

/// The submenu item for `agent`, which calls `handler` with its id.
fn agent_item(prefix: &str, agent: IssueActionsAgent, handler: OnAgent) -> MenuItem {
    let id = agent.id;
    let mut item = MenuItem::new(SharedString::from(format!("{prefix}-{id}")), agent.label);
    if let Some(detail) = agent.detail {
        item = item.label(detail);
    }
    item.on_click(move |_, window, cx| handler(&id, window, cx))
}

impl RenderOnce for IssueActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut menu = Menu::new();
        if let Some(handler) = self.on_assign.filter(|_| !self.assignable.is_empty()) {
            menu = menu.child(
                MenuItem::new("issue-actions-assign", "Assign")
                    .icon(IconName::Person)
                    .children(
                        self.assignable
                            .into_iter()
                            .map(|a| agent_item("assign", a, handler.clone())),
                    ),
            );
        }
        if let Some(handler) = self.on_unassign.filter(|_| !self.assigned.is_empty()) {
            menu = menu.child(
                MenuItem::new("issue-actions-unassign", "Unassign")
                    .icon(IconName::Cross)
                    .children(
                        self.assigned
                            .into_iter()
                            .map(|a| agent_item("unassign", a, handler.clone())),
                    ),
            );
        }
        menu
    }
}
