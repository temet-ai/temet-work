//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu. The design
//! system's menu has Open, Refine, Move to a stage, Assign and Unassign, and shows an item only
//! when its handler is given; the daemon has commands for Refine (`temet-work refine`), Move to
//! (`temet-work move`), Assign (`temet-work assign`) and Unassign (`temet-work unassign`), so those
//! are the ones the window gives, Refine first and a divider after it. Refine opens RefineDialog.
//! Move to opens a submenu of the stages the issue may move to, each with its stage's icon and
//! name. Assign and Unassign show only with agents to list, each opening a submenu of agents.

use std::rc::Rc;

use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

use crate::components::icon::IconName;
use crate::components::menu::{Menu, MenuItem};
use crate::components::stage_tag::Stage;

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

type OnMove = Rc<dyn Fn(Stage, &mut Window, &mut App)>;
type OnRefine = Rc<dyn Fn(&mut Window, &mut App)>;
type OnAgent = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssueActions {
    stage: Stage,
    move_to: Option<Vec<Stage>>,
    on_move: Option<OnMove>,
    on_refine: Option<OnRefine>,
    assignable: Vec<IssueActionsAgent>,
    on_assign: Option<OnAgent>,
    assigned: Vec<IssueActionsAgent>,
    on_unassign: Option<OnAgent>,
}

impl IssueActions {
    /// The menu for an issue at `stage`.
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            move_to: None,
            on_move: None,
            on_refine: None,
            assignable: Vec::new(),
            on_assign: None,
            assigned: Vec::new(),
            on_unassign: None,
        }
    }

    /// The stages Move to offers; every stage but the issue's own and done when not given.
    pub fn move_to(mut self, stages: Vec<Stage>) -> Self {
        self.move_to = Some(stages);
        self
    }

    /// Moves the issue to another stage. Without it there is no Move to.
    pub fn on_move(mut self, handler: impl Fn(Stage, &mut Window, &mut App) + 'static) -> Self {
        self.on_move = Some(Rc::new(handler));
        self
    }

    /// Opens the issue's title and checks for editing, as RefineDialog. Without it there is no
    /// Refine.
    pub fn on_refine(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_refine = Some(Rc::new(handler));
        self
    }

    /// The agents that can be put on the issue.
    pub fn assignable(mut self, agents: Vec<IssueActionsAgent>) -> Self {
        self.assignable = agents;
        self
    }

    /// Puts an agent on the issue: called with the agent's id. Without it, or with no agent to
    /// assign, there is no Assign.
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

    /// Takes an agent off the issue: called with the agent's id. Without it, or with no agent on
    /// the issue, there is no Unassign.
    pub fn on_unassign(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_unassign = Some(Rc::new(handler));
        self
    }

    /// The stages Move to offers, in order.
    pub fn stages(&self) -> Vec<Stage> {
        self.move_to.clone().unwrap_or_else(|| {
            Stage::ALL
                .into_iter()
                .filter(|s| *s != self.stage && *s != Stage::Done)
                .collect()
        })
    }
}

/// The id of the Move to item for `stage`, such as `move-to-in-review`.
pub fn move_to_id(stage: Stage) -> SharedString {
    format!("move-to-{}", stage.name().replace(' ', "-")).into()
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
        let stages = self.stages();
        let mut menu = Menu::new();
        let refine = self.on_refine.is_some();
        let assign = self.on_assign.filter(|_| !self.assignable.is_empty());
        let unassign = self.on_unassign.filter(|_| !self.assigned.is_empty());
        if let Some(on_refine) = self.on_refine {
            menu = menu.child(
                MenuItem::new("issue-actions-refine", "Refine")
                    .icon(IconName::Edit)
                    .on_click(move |_, window, cx| on_refine(window, cx)),
            );
        }
        if refine && (self.on_move.is_some() || assign.is_some() || unassign.is_some()) {
            menu = menu.divider();
        }
        if let Some(on_move) = self.on_move {
            let items = stages.into_iter().map(|stage| {
                let handler = on_move.clone();
                MenuItem::new(move_to_id(stage), stage.name())
                    .icon(stage.icon())
                    .on_click(move |_, window, cx| handler(stage, window, cx))
            });
            menu = menu.child(
                MenuItem::new("move-to", "Move to")
                    .icon(IconName::ArrowRight)
                    .children(items),
            );
        }
        if let Some(handler) = assign {
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
        if let Some(handler) = unassign {
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
