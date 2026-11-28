//! CutPanel (`temet-cutpanel`): a panel with its corners cut at top left and bottom right, the
//! border following the cut. Every panel and card in temet's apps is one.

use gpui::{
    AnyElement, App, Bounds, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, RenderOnce,
    Styled, Window, canvas, div, point,
};

use crate::theme::look::{self, State};
use crate::theme::shape;

#[derive(IntoElement, Default)]
pub struct CutPanel {
    attention: bool,
    padded: bool,
    children: Vec<AnyElement>,
}

impl CutPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws the border in the attention colour, for the one panel on screen that asks for the
    /// person.
    pub fn attention(mut self, attention: bool) -> Self {
        self.attention = attention;
        self
    }

    /// Pads the body. Leave it off when the child is a Section, which pads itself.
    pub fn padded(mut self, padded: bool) -> Self {
        self.padded = padded;
        self
    }
}

impl ParentElement for CutPanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The panel's outline with each cut corner `cut` deep, inset by `inset` on every side.
fn outline(b: Bounds<Pixels>, inset: Pixels, cut: Pixels) -> Option<gpui::Path> {
    let (l, t) = (b.left() + inset, b.top() + inset);
    let (r, bt) = (b.right() - inset, b.bottom() - inset);
    let mut p = PathBuilder::fill();
    p.move_to(point(l + cut, t));
    p.line_to(point(r, t));
    p.line_to(point(r, bt - cut));
    p.line_to(point(r - cut, bt));
    p.line_to(point(l, bt));
    p.line_to(point(l, t + cut));
    p.close();
    p.build().ok()
}

impl RenderOnce for CutPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        // The outer layer is the border's colour showing through its padding around the body.
        let outer = if self.attention {
            look::get(cx, "CutPanel", "tm-attention tm-cut", rest)
        } else {
            look::get(cx, "CutPanel", "tm-cut", rest)
        };
        let body = if self.padded {
            look::get(cx, "CutPanel", "tm-cut tm-cut-body tm-padded", rest)
        } else {
            look::get(cx, "CutPanel", "tm-cut tm-cut-body", rest)
        };
        let border = outer.padding.top;
        let cut = shape::cut(cx);
        // The body's cut is smaller by the border's diagonal, 1 px x (sqrt 2 - 1), so the
        // border keeps its width along the cut (temet.css, CutPanel).
        let inner = cut - border * (std::f32::consts::SQRT_2 - 1.);
        let line: Hsla = outer.background;
        let fill = body.background;

        let back = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                if let Some(path) = outline(b, Pixels::ZERO, cut) {
                    window.paint_path(path, line);
                }
                if let Some(path) = outline(b, border, inner) {
                    window.paint_path(path, fill);
                }
            },
        )
        .absolute()
        .size_full();
        let p = body.padding;
        let content = div().relative().p(border).flex().flex_col().child(
            div()
                .pt(p.top)
                .pr(p.right)
                .pb(p.bottom)
                .pl(p.left)
                .children(self.children),
        );
        div().relative().w_full().child(back).child(content)
    }
}
