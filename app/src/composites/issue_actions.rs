//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu: Open,
//! Refine, Move to a stage, Assign an agent and Unassign one. Built in temet-work#253; this file
//! holds its props until the part lands.

use gpui::SharedString;

/// An agent in the Assign or Unassign list.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueActionsAgent {
    /// The agent's id, handed back to on_assign or on_unassign.
    pub id: SharedString,
    /// The agent as the list names it, such as author-71.
    pub label: SharedString,
    /// Muted text at the right, such as the issue it is on now or "free".
    pub detail: Option<SharedString>,
}

// A stand-in with the agreed API until the part lands from the board branch.
#[derive(gpui::IntoElement)]
pub struct IssueActions {
    id: gpui::ElementId,
    on_open: Option<std::rc::Rc<dyn Fn(&mut gpui::Window, &mut gpui::App)>>,
}

impl IssueActions {
    pub fn new(
        id: impl Into<gpui::ElementId>,
        _stage: crate::components::stage_tag::Stage,
    ) -> Self {
        Self {
            id: id.into(),
            on_open: None,
        }
    }
    pub fn on_open(
        mut self,
        handler: impl Fn(&mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open = Some(std::rc::Rc::new(handler));
        self
    }
}

impl gpui::RenderOnce for IssueActions {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl gpui::IntoElement {
        use gpui::InteractiveElement;
        let _ = &self.on_open;
        gpui::div().id(self.id)
    }
}
