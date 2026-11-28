//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu. The design
//! system's menu has Open, Refine, Move to a stage, Assign and Unassign, and shows an item only
//! when its handler is given; the daemon has a command for Move to alone (`temet-work move`), so
//! that is the one the window gives. Move to opens a submenu of the stages the issue may move to,
//! each with its stage's icon and name.

use std::rc::Rc;

use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

use crate::components::icon::IconName;
use crate::components::menu::{Menu, MenuItem};
use crate::components::stage_tag::Stage;

type OnMove = Rc<dyn Fn(Stage, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssueActions {
    stage: Stage,
    move_to: Option<Vec<Stage>>,
    on_move: Option<OnMove>,
}

impl IssueActions {
    /// The menu for an issue at `stage`.
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            move_to: None,
            on_move: None,
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

impl RenderOnce for IssueActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let stages = self.stages();
        let mut menu = Menu::new();
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
        menu
    }
}
