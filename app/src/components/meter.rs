//! Meter (`temet-meter`): a labelled figure over a bar, for how much of something is in use, such
//! as a machine's build slots or an account's limit. The figure states the amount in words or
//! numbers, so the bar never carries meaning by length alone.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::progress_bar::ProgressBar;
use super::{padded, type_of};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, space};

#[derive(IntoElement)]
pub struct Meter {
    label: SharedString,
    value: f32,
    reading: SharedString,
    note: Option<SharedString>,
}

impl Meter {
    /// A meter: what is measured, how full it is from 0 to 1, and the figure at the right.
    pub fn new(
        label: impl Into<SharedString>,
        value: f32,
        reading: impl Into<SharedString>,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            reading: reading.into(),
            note: None,
        }
    }

    /// Muted text after the reading, such as "resets 14:20".
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }
}

impl RenderOnce for Meter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let meter = look::get(cx, "Meter", "tm-meter", rest);
        let label = look::get(cx, "Meter", "tm-meter-head tm-meter-label", rest);
        let reading = look::get(cx, "Meter", "tm-meter-head tm-meter-reading", rest);
        let head = div()
            .flex()
            .justify_between()
            .gap(space::s4(cx))
            .mb(space::s2(cx))
            .child(
                type_of(div(), &label)
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.label),
            )
            .child(
                type_of(div(), &reading)
                    .flex()
                    .flex_none()
                    .whitespace_nowrap()
                    .child(self.reading)
                    .children(self.note.map(|note| {
                        // `.tm-meter-note { color: muted }`: no measured look.
                        div()
                            .text_color(colour::muted(cx))
                            .child(format!(" · {note}"))
                    })),
            );
        padded(div(), &meter)
            .child(head)
            .child(ProgressBar::new().intent(Intent::Primary).value(self.value))
    }
}
