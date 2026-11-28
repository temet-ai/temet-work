//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu. An item
//! shows only when its handler is given. The window gives Refine, in the issue's pane, where the
//! design system leaves out Open (story In Pane); Move to, Assign and Unassign wait for commands
//! the daemon has for them.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement, RenderOnce, Window};

use crate::components::icon::IconName;
use crate::components::menu::{Menu, MenuItem};
use crate::components::stage_tag::Stage;

type OnAction = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssueActions {
    stage: Stage,
    on_refine: Option<OnAction>,
}

impl IssueActions {
    /// The menu for an issue at `stage`.
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            on_refine: None,
        }
    }

    /// Opens the issue's title and checks for editing, as RefineDialog.
    pub fn on_refine(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_refine = Some(Rc::new(handler));
        self
    }

    /// The issue's stage, which Move to will leave out.
    pub fn stage(&self) -> Stage {
        self.stage
    }
}

impl RenderOnce for IssueActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let refine = self.on_refine.map(|handler| {
            MenuItem::new("issue-actions-refine", "Refine")
                .icon(IconName::Edit)
                .on_click(move |_, window, cx| handler(window, cx))
        });
        Menu::new().children(refine)
    }
}
