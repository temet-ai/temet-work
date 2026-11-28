//! StageBoard (`temet-stageboard`): temet-work's issues as a board: a column per stage, a lane
//! per milestone, a card per issue. Built in temet-work#253; this file holds its props until the
//! part lands.

use gpui::SharedString;

use super::issue_card::IssueCardAgent;
use crate::components::stage_tag::Stage;

/// One issue on the board.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardIssue {
    /// The issue's key.
    pub key: SharedString,
    pub title: SharedString,
    /// Its stage: the column it sits in.
    pub stage: Stage,
    /// The agents on it now.
    pub agents: Vec<IssueCardAgent>,
    /// How many of its decisions are open.
    pub decisions: usize,
    /// How long ago it changed.
    pub updated: Option<SharedString>,
}

/// A row of the board, such as a milestone, with its issues.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardLane {
    /// The lane's id, such as the milestone's name.
    pub id: SharedString,
    /// The lane's heading.
    pub title: SharedString,
    /// Muted text after the count, such as "no group takes it".
    pub note: Option<SharedString>,
    /// Whether the lane starts open.
    pub default_open: bool,
    /// Its issues, in order.
    pub issues: Vec<BoardIssue>,
}

// A stand-in with the agreed API until the part lands from the board branch.
type OnOpen = std::rc::Rc<dyn Fn(&SharedString, &mut gpui::Window, &mut gpui::App)>;
type Menu =
    std::rc::Rc<dyn Fn(&BoardIssue, &mut gpui::Window, &mut gpui::App) -> Option<gpui::AnyElement>>;

#[derive(gpui::IntoElement)]
pub struct StageBoard {
    id: gpui::ElementId,
    lanes: Vec<BoardLane>,
    selected: Option<SharedString>,
    on_open: Option<OnOpen>,
    menu: Option<Menu>,
}

impl StageBoard {
    pub fn new(id: impl Into<gpui::ElementId>, lanes: Vec<BoardLane>) -> Self {
        Self {
            id: id.into(),
            lanes,
            selected: None,
            on_open: None,
            menu: None,
        }
    }
    pub fn selected(mut self, key: Option<SharedString>) -> Self {
        self.selected = key;
        self
    }
    pub fn on_open(
        mut self,
        handler: impl Fn(&SharedString, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_open = Some(std::rc::Rc::new(handler));
        self
    }
    pub fn menu(
        mut self,
        menu: impl Fn(&BoardIssue, &mut gpui::Window, &mut gpui::App) -> Option<gpui::AnyElement>
        + 'static,
    ) -> Self {
        self.menu = Some(std::rc::Rc::new(menu));
        self
    }
}

impl gpui::RenderOnce for StageBoard {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl gpui::IntoElement {
        use gpui::{InteractiveElement, ParentElement};
        let _ = (&self.selected, &self.on_open, &self.menu);
        gpui::div().id(self.id).children(
            self.lanes
                .into_iter()
                .flat_map(|l| l.issues)
                .map(|i| gpui::div().child(i.key)),
        )
    }
}
