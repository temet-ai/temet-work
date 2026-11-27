//! ProgressBar (`blueprint-progressbar`): how far along something is, as a filled bar. Drawn
//! without stripes or animation, as every story in the design system sets it.

use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, relative};

use crate::theme::look::{self, State};
use crate::theme::{Intent, colour};

#[derive(IntoElement)]
pub struct ProgressBar {
    value: f32,
    intent: Intent,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            value: 0.,
            intent: Intent::None,
        }
    }

    /// How far along, from 0 to 1; values outside are taken as 0 or 1.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(0., 1.);
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let track = look::get(cx, "ProgressBar", "progress-bar: default", rest);
        let meter = look::get(cx, "ProgressBar", "progress-meter: default", rest);
        // The design system measured the meter with no intent; an intent's meter is that
        // intent's fill.
        let fill = match self.intent {
            Intent::None => meter.background,
            intent => colour::intent_fill(cx, intent),
        };
        div()
            .w_full()
            .h(track.height)
            .overflow_hidden()
            .rounded(track.radius)
            .bg(track.background)
            .child(
                div()
                    .h_full()
                    .w(relative(self.value))
                    .rounded(meter.radius)
                    .bg(fill),
            )
    }
}
