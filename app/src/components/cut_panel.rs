//! CutPanel (`temet-cutpanel`): a panel with its corners cut at top left and bottom right, the
//! border following the cut. Every panel and card in temet's apps is one.

use gpui::{
    AnyElement, App, Bounds, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, RenderOnce,
    Styled, Window, canvas, div, point,
};

use crate::theme;

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

/// A cut corner's triangle outside the outline and the border along its diagonal, at the top
/// left (`top`) or bottom right.
fn corner(
    b: Bounds<Pixels>,
    top: bool,
    border: Pixels,
    cut: Pixels,
    inner: Pixels,
) -> (Option<gpui::Path>, Option<gpui::Path>) {
    // Mirror the top-left corner's points for the bottom right.
    let at = |x: Pixels, y: Pixels| {
        if top {
            point(b.left() + x, b.top() + y)
        } else {
            point(b.right() - x, b.bottom() - y)
        }
    };
    let zero = Pixels::ZERO;
    let mut outside = PathBuilder::fill();
    outside.move_to(at(zero, zero));
    outside.line_to(at(cut, zero));
    outside.line_to(at(zero, cut));
    outside.close();
    let mut edge = PathBuilder::fill();
    edge.move_to(at(cut, zero));
    edge.line_to(at(border + inner, border));
    edge.line_to(at(border, border + inner));
    edge.line_to(at(zero, cut));
    edge.close();
    (outside.build().ok(), edge.build().ok())
}

impl RenderOnce for CutPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        use theme::{colour, shape, size};
        let border = shape::border(cx);
        let cut = shape::cut(cx);
        // The body's cut is smaller by the border's diagonal, 1 px x (sqrt 2 - 1), so the
        // border keeps its width along the cut (temet.css, CutPanel).
        let inner = cut - border * (std::f32::consts::SQRT_2 - 1.);
        let line: Hsla = if self.attention {
            colour::attention(cx)
        } else {
            colour::edge(cx)
        };
        let body = colour::panel(cx);
        let ground = colour::page(cx);

        let back = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                if let Some(path) = outline(b, Pixels::ZERO, cut) {
                    window.paint_path(path, line);
                }
                if let Some(path) = outline(b, border, inner) {
                    window.paint_path(path, body);
                }
            },
        )
        .absolute()
        .size_full();
        // Content cannot be clipped to the cut, so the corners are drawn again over it: the
        // page's ground outside the cut, then the border along it. A CutPanel sits on the page.
        let front = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                for top in [true, false] {
                    let (outside, edge) = corner(b, top, border, cut, inner);
                    if let Some(path) = outside {
                        window.paint_path(path, ground);
                    }
                    if let Some(path) = edge {
                        window.paint_path(path, line);
                    }
                }
            },
        )
        .absolute()
        .size_full();

        let mut content = div().relative().p(border).flex().flex_col();
        if self.padded {
            content = content.child(div().p(size::panel_pad(cx)).children(self.children));
        } else {
            content = content.children(self.children);
        }
        div()
            .relative()
            .w_full()
            .child(back)
            .child(content)
            .child(front)
    }
}
