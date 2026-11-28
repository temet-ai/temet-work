//! Spinner (`blueprint-spinner`): something is under way, such as a loading button's action
//! being sent. A ring, its track faint and a quarter of it, the head, drawn strong.
//!
//! Blueprint turns the head once every 500 ms; no motion token holds that duration yet, so the
//! head stands still at the top, where Blueprint's turn starts.

use gpui::{
    App, Bounds, IntoElement, PathBuilder, Pixels, RenderOnce, Styled, Window, canvas, point,
};

use crate::theme::colour;

/// Blueprint's spinner path: a circle of radius 45 in a box of 100, stroked 16 wide at the
/// small size, in a view box grown by the stroke so it is not cut (`getViewBox`).
const RADIUS: f32 = 45.;
const BOX: f32 = 100.;
const STROKE: f32 = 16.;

#[derive(IntoElement)]
pub struct Spinner {
    side: Option<Pixels>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self { side: None }
    }

    /// The spinner's side, such as `SpinnerSize.SMALL` in a button.
    pub fn size(mut self, side: Pixels) -> Self {
        self.side = Some(side);
        self
    }
}

/// The arc from `from` to `to`, in turns clockwise from the top, around the centre of `bounds`.
fn arc(bounds: Bounds<Pixels>, from: f32, to: f32) -> Option<gpui::Path> {
    let side = bounds.size.width.min(bounds.size.height);
    let scale = side / (BOX + STROKE);
    let radius = scale * RADIUS;
    let centre = bounds.center();
    let at = |turn: f32| {
        let angle = turn * std::f32::consts::TAU;
        point(
            centre.x + radius * angle.sin(),
            centre.y - radius * angle.cos(),
        )
    };
    let mut path = PathBuilder::stroke(scale * STROKE);
    path.move_to(at(from));
    // An arc of at most half a turn at a time, so each one's sweep is unambiguous.
    let mut turn = from;
    while turn < to {
        let next = (turn + 0.5).min(to);
        path.arc_to(point(radius, radius), Pixels::ZERO, false, true, at(next));
        turn = next;
    }
    path.build().ok()
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let side = self
            .side
            .unwrap_or_else(|| crate::theme::size::spinner_small(cx));
        let (head, track) = colour::spinner(cx);
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                if let Some(path) = arc(bounds, 0., 1.) {
                    window.paint_path(path, track);
                }
                if let Some(path) = arc(bounds, 0., 0.25) {
                    window.paint_path(path, head);
                }
            },
        )
        .size(side)
        .flex_none()
    }
}
