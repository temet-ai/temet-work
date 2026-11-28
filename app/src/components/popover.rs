//! Popover (`blueprint-popover`): content, such as a menu, floating beside its target, here
//! controlled as Blueprint's `isOpen` and `onInteraction` control it: the page holds whether it is
//! open, a click on the target asks to open or close it, and a press anywhere outside it asks to
//! close it.
//!
//! It is drawn with no arrow, one menu padding from its target, above everything but the window
//! itself; the content draws its own box.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement, Point,
    RenderOnce, StatefulInteractiveElement, Styled, Window, anchored, deferred, div,
};

use crate::theme::size;

type OnInteraction = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Where the content goes from its target. Only the placement temet-work uses is here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// Below the target, with their right edges in line.
    #[default]
    BottomEnd,
}

#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    target: Option<AnyElement>,
    content: Option<AnyElement>,
    placement: Placement,
    is_open: bool,
    on_interaction: Option<OnInteraction>,
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            target: None,
            content: None,
            placement: Placement::BottomEnd,
            is_open: false,
            on_interaction: None,
        }
    }

    /// The target, such as a button.
    pub fn child(mut self, target: impl IntoElement) -> Self {
        self.target = Some(target.into_any_element());
        self
    }

    /// What floats beside the target while it is open.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Called with the state the person asks for: true from a click on the closed target, false
    /// from a click on the open target or a press outside the content.
    pub fn on_interaction(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_interaction = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.is_open;
        let mut target = div().id(self.id).relative().flex().flex_none();
        if let Some(handler) = self.on_interaction.clone() {
            target = target.on_click(move |_, window, cx| handler(!open, window, cx));
        }
        target = target.children(self.target);
        let Some(content) = self.content.filter(|_| open) else {
            return target;
        };
        let gap = size::menu_pad(cx);
        let screen = window.viewport_size();
        let mut outside = div().occlude().w(screen.width).h(screen.height);
        if let Some(handler) = self.on_interaction {
            outside = outside.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                handler(false, window, cx)
            });
        }
        let placed = match self.placement {
            Placement::BottomEnd => div().absolute().right_0().bottom_0().child(
                anchored()
                    .anchor(gpui::Anchor::TopRight)
                    .offset(gpui::point(gpui::Pixels::ZERO, gap))
                    .snap_to_window()
                    .child(div().occlude().child(content)),
            ),
        };
        target
            .child(deferred(anchored().position(Point::default()).child(outside)).with_priority(3))
            .child(deferred(placed).with_priority(4))
    }
}
