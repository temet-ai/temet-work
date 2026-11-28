//! CutPanel (`temet-cutpanel`): a panel with its corners cut at top left and bottom right, the
//! border following the cut. Every panel and card in temet's apps is one. `here` marks the one
//! panel for the item the person is in with a moving hue edge; `attention` marks the one panel
//! that asks for the person, and wins when both are set.

use std::sync::OnceLock;
use std::time::Instant;

use gpui::{
    AnyElement, App, Bounds, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, Point,
    RenderOnce, Rgba, Styled, Window, canvas, div, linear_color_stop, linear_gradient, point,
};

use crate::theme::look::{self, State};
use crate::theme::{colour, motion, shape};

#[derive(IntoElement, Default)]
pub struct CutPanel {
    attention: bool,
    here: bool,
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

    /// Marks the panel as the item the person is in, such as the shown space or the current
    /// machine: a slowly moving hue edge in place of the border, still under reduced motion. Set
    /// it on one panel on screen; `attention` wins when both are set.
    pub fn here(mut self, here: bool) -> Self {
        self.here = here;
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

/// The panel's outline with each cut corner `cut` deep, inset by `inset` on every side, as its
/// corners from the top left's cut, clockwise.
fn corners(b: Bounds<Pixels>, inset: Pixels, cut: Pixels) -> [Point<Pixels>; 6] {
    let (l, t) = (b.left() + inset, b.top() + inset);
    let (r, bt) = (b.right() - inset, b.bottom() - inset);
    [
        point(l + cut, t),
        point(r, t),
        point(r, bt - cut),
        point(r - cut, bt),
        point(l, bt),
        point(l, t + cut),
    ]
}

/// A closed path through `points`.
fn path(points: &[Point<Pixels>]) -> Option<gpui::Path> {
    let (first, rest) = points.split_first()?;
    let mut p = PathBuilder::fill();
    p.move_to(*first);
    for at in rest {
        p.line_to(*at);
    }
    p.close();
    p.build().ok()
}

/// The part of the convex outline `points` between `from` and `to` across.
fn band(points: &[Point<Pixels>], from: Pixels, to: Pixels) -> Vec<Point<Pixels>> {
    // Clip to one side of a vertical line at a time (Sutherland and Hodgman).
    let clip = |points: Vec<Point<Pixels>>, x: Pixels, keep_right: bool| {
        let inside = |p: &Point<Pixels>| if keep_right { p.x >= x } else { p.x <= x };
        let mut out = Vec::new();
        for (i, a) in points.iter().enumerate() {
            let b = &points[(i + 1) % points.len()];
            if inside(a) {
                out.push(*a);
            }
            if inside(a) != inside(b) {
                let share = (x - a.x) / (b.x - a.x);
                out.push(point(x, a.y + (b.y - a.y) * share));
            }
        }
        out
    };
    clip(clip(points.to_vec(), from, true), to, false)
}

/// A colour `share` of the way from `a` to `b`, mixed in sRGB as a CSS gradient mixes it.
fn mix(a: Hsla, b: Hsla, share: f32) -> Hsla {
    let (a, b) = (Rgba::from(a), Rgba::from(b));
    let at = |x: f32, y: f32| x + (y - x) * share;
    Rgba {
        r: at(a.r, b.r),
        g: at(a.g, b.g),
        b: at(a.b, b.b),
        a: at(a.a, b.a),
    }
    .into()
}

/// When the first panel marked here was drawn: the edge's drift counts from it.
static DRIFT_FROM: OnceLock<Instant> = OnceLock::new();

/// How far along its strip the hue edge has drifted, from 0 to 1: one way over `--tm-d-drift`
/// and back, as `animation: tm-here-drift var(--tm-d-drift) var(--tm-ease-linear) infinite
/// alternate` moves it. Still at 0 under reduced motion.
fn drift(cx: &App) -> f32 {
    if cx.reduce_motion() {
        return 0.;
    }
    let period = motion::drift(cx).as_secs_f32();
    let gone = DRIFT_FROM.get_or_init(Instant::now).elapsed().as_secs_f32();
    if period <= 0. {
        return 0.;
    }
    let lap = gone / period;
    let share = lap.fract();
    let share = if (lap as u64).is_multiple_of(2) {
        share
    } else {
        1. - share
    };
    motion::drift_ease(cx)(share)
}

/// The hue edge: a strip twice the panel's width, through the six colours and back to the first,
/// slid `drift` of a panel's width to the left, painted inside `outer` in bands of one colour
/// step each (`.tm-cut.tm-here`).
fn paint_here(
    window: &mut Window,
    b: Bounds<Pixels>,
    outer: &[Point<Pixels>],
    hues: &[Hsla; 6],
    drift: f32,
) {
    let width = b.size.width;
    let mut stops: Vec<Hsla> = hues.to_vec();
    stops.push(hues[0]);
    // Each step of the strip is a third of the panel wide: six steps over twice its width.
    let step = width * 2. / (stops.len() - 1) as f32;
    let shift = width * drift;
    for (i, pair) in stops.windows(2).enumerate() {
        let start = b.left() + step * i as f32 - shift;
        let (from, to) = (start.max(b.left()), (start + step).min(b.right()));
        if from >= to {
            continue;
        }
        let colour = |x: Pixels| mix(pair[0], pair[1], (x - start) / step);
        let fill = linear_gradient(
            90.,
            linear_color_stop(colour(from), 0.),
            linear_color_stop(colour(to), 1.),
        );
        if let Some(p) = path(&band(outer, from, to)) {
            window.paint_path(p, fill);
        }
    }
}

impl RenderOnce for CutPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        // `attention` wins: a panel with both has the attention border and no hue edge.
        let here = self.here && !self.attention;
        // The outer layer is the border's colour showing through its padding around the body.
        let plain = look::get(cx, "CutPanel", "tm-cut", rest);
        let outer = if self.attention {
            look::get(cx, "CutPanel", "tm-attention tm-cut", rest)
        } else if here {
            look::get(cx, "CutPanel", "tm-cut tm-here", rest)
        } else {
            plain.clone()
        };
        let body = match (self.padded, here) {
            (true, true) => look::get(cx, "CutPanel", "tm-cut tm-cut-body tm-here tm-padded", rest),
            (true, false) => look::get(cx, "CutPanel", "tm-cut tm-cut-body tm-padded", rest),
            (false, _) => look::get(cx, "CutPanel", "tm-cut tm-cut-body", rest),
        };
        let border = outer.padding.top;
        // The here edge is wider and grows outward, so the content and the panels around it stay
        // where they are (`margin: calc(1px - var(--tm-here-width))`).
        let grow = border - plain.padding.top;
        let cut = shape::cut(cx);
        // The body's cut is smaller by (2 - sqrt 2) x the border's width, so the border keeps
        // its width along the cut (temet.css, CutPanel).
        let inner = cut - border * (2. - std::f32::consts::SQRT_2);
        let line: Hsla = outer.background;
        let fill = body.background;
        let hues = here.then(|| (colour::here(cx), drift(cx)));

        let back = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                let outer = corners(b, Pixels::ZERO, cut);
                if let Some(p) = path(&outer) {
                    window.paint_path(p, line);
                }
                if let Some((hues, drift)) = &hues {
                    paint_here(window, b, &outer, hues, *drift);
                    // The edge keeps moving: draw the next frame.
                    window.request_animation_frame();
                }
                if let Some(p) = path(&corners(b, border, inner)) {
                    window.paint_path(p, fill);
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
        let panel = div().relative().child(back).child(content);
        if here {
            div().w_full().flex().flex_col().child(panel.m(-grow))
        } else {
            panel.w_full()
        }
    }
}
