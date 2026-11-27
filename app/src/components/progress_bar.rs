//! ProgressBar (`blueprint-progressbar`): how far along something is, as a filled bar. Drawn
//! without stripes or animation, as every story in the design system sets it.

use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, relative};

use crate::theme::{self, Intent};

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
        use theme::{colour, size};
        // The track: `.bp6-dark .bp6-progress-bar { background: var(--tm-edge-quiet) }`
        // (blueprint-overrides.css), the same role in the light theme.
        div()
            .w_full()
            .h(size::progress(cx))
            .overflow_hidden()
            .bg(colour::edge_quiet(cx))
            .child(
                div()
                    .h_full()
                    .w(relative(self.value))
                    .bg(colour::intent_fill(cx, self.intent)),
            )
    }
}
