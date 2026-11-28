//! AppShell (`temet-appshell`): temet-work's frame, a left column, the main column and an
//! optional right pane. It fills the window and each column scrolls on its own. The left column
//! takes the theme's width for the window's width, so the middle keeps its room.
//!
//! With `side_over` the pane lies over the right of the main column instead of taking a column
//! of its own, for a main column that needs its full width, such as a StageBoard; only a child
//! added with `wide_child` runs under the pane, and the others keep the width left of it.

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{size, space, text};

/// What the right column holds and how it holds it.
enum Side {
    /// Content the column pads and scrolls.
    Plain(AnyElement),
    /// A part that scrolls on its own, such as IssuePane: the column neither pads nor scrolls,
    /// ends where the part ends, no taller than the window, with an edge under it
    /// (`.tm-shell-pane:has(.tm-issue-pane)`).
    Fit(AnyElement),
}

/// A child of the main column: one that keeps the width left of a pane lying over it, or one
/// that runs under the pane (`.tm-shell-over > .tm-shell-main > .tm-board`).
enum Child {
    Narrow(AnyElement),
    Wide(AnyElement),
}

#[derive(IntoElement)]
pub struct AppShell {
    nav: AnyElement,
    side: Option<Side>,
    side_over: bool,
    children: Vec<Child>,
}

impl AppShell {
    /// The frame, with `nav` in the left column: projects, groups and their agents.
    pub fn new(nav: impl IntoElement) -> Self {
        Self {
            nav: nav.into_any_element(),
            side: None,
            side_over: false,
            children: Vec::new(),
        }
    }

    /// The right column: the selected item. The column pads it and scrolls it.
    pub fn side(mut self, side: impl IntoElement) -> Self {
        self.side = Some(Side::Plain(side.into_any_element()));
        self
    }

    /// The right column holding a part that scrolls on its own, such as IssuePane: the column
    /// ends where the part ends, capped at the window's height, with an edge under it.
    pub fn side_fit(mut self, side: impl IntoElement) -> Self {
        self.side = Some(Side::Fit(side.into_any_element()));
        self
    }

    /// Lays the pane over the right of the main column instead of giving it a column.
    pub fn side_over(mut self, over: bool) -> Self {
        self.side_over = over;
        self
    }

    /// A child of the main column that runs under a pane lying over it, such as a StageBoard.
    pub fn wide_child(mut self, child: impl IntoElement) -> Self {
        self.children.push(Child::Wide(child.into_any_element()));
        self
    }
}

impl ParentElement for AppShell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(Child::Narrow));
    }
}

/// The width from which a pane lying over the main column leaves the other children the width
/// left of it: `@media (min-width: 1100px)` in the design system's rules.
const ROOM_BESIDE_PANE: f32 = 1100.;

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
        let pane = look::get(
            cx,
            "AppShell",
            "tm-shell tm-shell-col tm-shell-pane tm-shell-side",
            rest,
        );
        let width = window.viewport_size().width;
        let left = size::col_left(cx, width);
        let right = size::col_right(cx, width);
        let over = self.side_over && self.side.is_some();
        // The content box of the main column, and what a narrow child may take of it beside a
        // pane lying over it.
        let content = width - left - main.padding.left - main.padding.right;
        let beside: Option<Pixels> =
            (over && width >= gpui::px(ROOM_BESIDE_PANE)).then(|| content - right);
        let children = self
            .children
            .into_iter()
            .map(|child| match (child, beside) {
                (Child::Narrow(el), Some(max)) => div().max_w(max).child(el).into_any_element(),
                (Child::Narrow(el) | Child::Wide(el), _) => el,
            });
        let side = self.side.map(|side| {
            let column = box_of(div().id("side"), &pane)
                .debug_selector(|| "side".to_string())
                .flex_none()
                .w(right);
            match side {
                Side::Plain(el) => column
                    .h_full()
                    .overflow_y_scroll()
                    .child(div().flex().flex_col().gap(space::s5(cx)).child(el)),
                // The edge under it is the column's own: `border-bottom: --tm-edge`, the colour
                // of its left edge.
                Side::Fit(el) => {
                    let mut column = column
                        .p(Pixels::ZERO)
                        .max_h_full()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .border_b(pane.border.left)
                        .child(div().flex().flex_col().min_h_0().child(el));
                    if let Some(edge) = pane.border_colour {
                        column = column.border_color(edge);
                    }
                    column
                }
            }
        });
        // Over the main column, the pane is placed at its top right; else it is a column.
        let (over_main, beside_main) = if over {
            (side.map(|column| column.absolute().top_0().right_0()), None)
        } else {
            (None, side)
        };
        let main_column = box_of(div().id("main"), &main)
            .debug_selector(|| "main".to_string())
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::s5(cx))
                    .children(children),
            );
        type_of(div(), &shell)
            .size_full()
            .flex()
            .items_start()
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
            .child(if over {
                // The pane lies over the right of the main column.
                div()
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(main_column)
                    .children(over_main)
                    .into_any_element()
            } else {
                main_column.into_any_element()
            })
            .children(beside_main)
    }
}
