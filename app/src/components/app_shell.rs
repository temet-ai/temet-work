//! AppShell (`temet-appshell`): temet-work's frame, a left column and the main column. It fills
//! the window and each column scrolls on its own. The left column takes the theme's width for
//! the window's width, so the middle keeps its room.

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{size, space, text};

#[derive(IntoElement)]
pub struct AppShell {
    nav: AnyElement,
    children: Vec<AnyElement>,
}

impl AppShell {
    /// The frame, with `nav` in the left column: projects, groups and their agents.
    pub fn new(nav: impl IntoElement) -> Self {
        Self {
            nav: nav.into_any_element(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for AppShell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AppShell {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let shell = look::get(cx, "AppShell", "tm-shell tm-shell-side", rest);
        let nav = look::get(
            cx,
            "AppShell",
            "tm-shell tm-shell-col tm-shell-nav tm-shell-side",
            rest,
        );
        let main = look::get(
            cx,
            "AppShell",
            "tm-shell tm-shell-col tm-shell-main tm-shell-side",
            rest,
        );
        let left = size::col_left(cx, window.viewport_size().width);
        type_of(div(), &shell)
            .size_full()
            .flex()
            .bg(shell.background)
            .font_family(text::family(cx))
            .line_height(text::line_height(cx))
            .child(
                box_of(div().id("nav"), &nav)
                    .flex_none()
                    .w(left)
                    .h_full()
                    .overflow_y_scroll()
                    .child(div().flex().flex_col().gap(space::s6(cx)).child(self.nav)),
            )
            .child(
                box_of(div().id("main"), &main)
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(space::s5(cx))
                            .children(self.children),
                    ),
            )
    }
}
