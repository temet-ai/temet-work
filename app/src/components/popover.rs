//! Popover (`blueprint-popover`): content, such as a Menu, that a click on its target opens
//! beside the target, and a press outside it or Escape closes. Blueprint draws an arrow toward
//! the target; the design system measured the content's box alone, and the arrow is not drawn.

use gpui::{
    Anchor, AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Window, div,
};
use gpui_base::Popover as BasePopover;

use super::box_of;
use crate::theme::look::{self, State};
use crate::theme::space;

/// Where the content opens, as Blueprint names it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// Under the target, its left edges in line.
    #[default]
    BottomStart,
    /// Under the target, its right edges in line, as an IssuePane's actions menu.
    BottomEnd,
}

#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    placement: Placement,
    is_open: Option<bool>,
    content: Option<AnyElement>,
    target: Option<AnyElement>,
}

impl Popover {
    /// A popover. `id` is its identity in the window, which keeps it open between frames.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            placement: Placement::BottomStart,
            is_open: None,
            content: None,
            target: None,
        }
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Holds it open, or shut, whatever the person does, as the Opened story does.
    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = Some(open);
        self
    }

    /// What opens.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// The target, which a click opens it from.
    pub fn child(mut self, target: impl IntoElement) -> Self {
        self.target = Some(target.into_any_element());
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = look::get(cx, "Popover", "default", State::Rest);
        // The anchor names the corner of the content that meets the target's opposite corner.
        let anchor = match self.placement {
            Placement::BottomStart => Anchor::TopLeft,
            Placement::BottomEnd => Anchor::TopRight,
        };
        let target = self.target.unwrap_or_else(|| div().into_any_element());
        let content = self.content;
        let mut popover = BasePopover::new(self.id)
            .anchor(anchor)
            .offset(space::s2(cx))
            .trigger_with(move |_, _, _| target)
            .content(move |_, _, _| box_of(div(), &look).children(content));
        if let Some(open) = self.is_open {
            popover = popover.open(open);
        }
        popover
    }
}
