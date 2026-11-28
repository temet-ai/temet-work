//! Popover (`blueprint-popover`): content, such as a Menu, that opens over the page from its
//! target, such as an actions button. A click on the target opens it below the target, its end
//! edge on the target's (`placement="bottom-end"`); a click on its content or anywhere outside
//! it closes it. It keeps whether it is open itself, as Blueprint's uncontrolled popover does.

use gpui::{
    Anchor, AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, anchored, deferred, div,
};

use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{shape, size};

#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    target: AnyElement,
    content: AnyElement,
}

impl Popover {
    /// A popover opened by a click on `target`. `id` keeps its open state across frames.
    pub fn new(
        id: impl Into<ElementId>,
        target: impl IntoElement,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            target: target.into_any_element(),
            content: content.into_any_element(),
        }
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let is_open = *open.read(cx);
        let set = |to: bool| {
            let open = open.clone();
            move |cx: &mut App| {
                open.update(cx, |o, cx| {
                    if *o != to {
                        *o = to;
                        cx.notify();
                    }
                })
            }
        };
        let opens = set(true);
        let target = div()
            .id(self.id.clone())
            .on_click(move |_, _, cx| opens(cx))
            .child(self.target);
        let mut root = div().relative().flex_none().child(target);
        if is_open {
            let card = look::get(cx, "Popover", "default", State::Rest);
            let close_out = set(false);
            let close_in = set(false);
            let content = box_of(type_of(div(), &card), &card)
                .shadow(shape::overlay_shadow(cx))
                .occlude()
                .mt(size::popover_gap(cx))
                .on_mouse_down_out(move |_, _, cx| close_out(cx))
                .on_mouse_up(MouseButton::Left, move |_, _, cx| close_in(cx))
                .child(self.content);
            // A point at the target's bottom end, which the content's top end is anchored to.
            root = root.child(
                div().absolute().top_full().right_0().child(
                    deferred(
                        anchored()
                            .anchor(Anchor::TopRight)
                            .snap_to_window()
                            .child(content),
                    )
                    .with_priority(1),
                ),
            );
        }
        root
    }
}
