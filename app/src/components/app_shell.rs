//! AppShell (`temet-appshell`): temet-work's frame, a left column and the main column. It fills
//! the window and each column scrolls on its own. The left column takes the theme's width for
//! the window's width, so the middle keeps its room.

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::theme;

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
        use theme::{colour, shape, size, space, text};
        let left = size::col_left(cx, window.viewport_size().width);
        div()
            .size_full()
            .flex()
            .bg(colour::page(cx))
            .text_color(colour::text(cx))
            .font_family(text::family(cx))
            .text_size(text::body(cx))
            .line_height(text::line_height(cx))
            .child(
                div()
                    .id("nav")
                    .flex_none()
                    .w(left)
                    .h_full()
                    .overflow_y_scroll()
                    .p(space::s4(cx))
                    .bg(colour::panel(cx))
                    .border_r(shape::border(cx))
                    .border_color(colour::edge(cx))
                    .child(div().flex().flex_col().gap(space::s6(cx)).child(self.nav)),
            )
            .child(
                div()
                    .id("main")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .p(space::s5(cx))
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
