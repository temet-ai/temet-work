//! Tag (`temet-tag`): Blueprint's Tag without `round`, since temet has no pills. A tag that
//! shows a status is a StatusTag. temet-work uses the minimal tag only (`docs/ui.md`), the
//! variant the design system measured.

use gpui::{App, Div, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::look::{self, Look, State};
use crate::theme::{Intent, size};

/// A minimal tag: an optional icon, then its text, on the faint ground of its intent.
#[derive(IntoElement)]
pub struct Tag {
    children: SharedString,
    intent: Intent,
    icon: Option<IconName>,
    multiline: bool,
}

impl Tag {
    pub fn new(children: impl Into<SharedString>) -> Self {
        Self {
            children: children.into(),
            intent: Intent::None,
            icon: None,
            multiline: false,
        }
    }

    /// Lets the text take several lines rather than end in an ellipsis.
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// An icon before the text.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A tag drawn in `look`: StatusTag and StageTag draw theirs with their own looks.
pub(crate) fn draw(look: &Look, icon: Option<IconName>, text: SharedString, cx: &App) -> Div {
    draw_lines(look, icon, text, false, cx)
}

/// A tag drawn in `look`, its text on as many lines as it needs when `multiline`.
fn draw_lines(
    look: &Look,
    icon: Option<IconName>,
    text: SharedString,
    multiline: bool,
    cx: &App,
) -> Div {
    // The look's height is one line of the tag's word inside its padding.
    let line = look.height - look.padding.top - look.padding.bottom;
    box_of(type_of(div(), look), look)
        .flex()
        .items_center()
        .max_w_full()
        .gap(size::tag_gap(cx))
        .min_h(look.height)
        .min_w(look.height)
        .line_height(line)
        .children(icon.map(Icon::new))
        .child(if multiline {
            div().min_w_0().child(text)
        } else {
            div()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(text)
        })
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let look = match self.intent {
            Intent::None => look::get(cx, "Tag", "minimal", rest),
            Intent::Primary => look::get(cx, "Tag", "intent-primary minimal", rest),
            Intent::Success => look::get(cx, "Tag", "intent-success minimal", rest),
            Intent::Warning => look::get(cx, "Tag", "intent-warning minimal", rest),
            Intent::Danger => look::get(cx, "Tag", "intent-danger minimal", rest),
        };
        draw_lines(&look, self.icon, self.children, self.multiline, cx)
    }
}
