//! Tag (`temet-tag`): Blueprint's Tag without `round`, since temet has no pills. A tag that
//! shows a status is a StatusTag.

use gpui::{App, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};
use gpui_component::tag as library;

use super::icon::{Icon, IconName};
use crate::theme::{self, Intent};

/// A tag's size: `medium` beside body text, `large` in a page header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    #[default]
    Medium,
    Large,
}

/// A tag: an optional icon, then its text, on a ground in its intent.
#[derive(IntoElement)]
pub struct Tag {
    children: SharedString,
    minimal: bool,
    intent: Intent,
    icon: Option<IconName>,
    size: Size,
}

impl Tag {
    pub fn new(children: impl Into<SharedString>) -> Self {
        Self {
            children: children.into(),
            minimal: false,
            intent: Intent::None,
            icon: None,
            size: Size::Medium,
        }
    }

    /// Minimal styles: the intent's colour as text on a faint ground of it.
    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
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

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        use theme::{colour, shape, size, text};
        let large = self.size == Size::Large;
        let (fg, ground) = if self.minimal {
            (
                colour::intent_text(cx, self.intent),
                colour::tag_wash(cx, self.intent),
            )
        } else {
            (
                colour::intent_foreground(cx, self.intent),
                colour::intent_fill(cx, self.intent),
            )
        };
        let outline = self
            .minimal
            .then(|| colour::tag_outline(cx, self.intent))
            .flatten();
        // The library's tag always has a border: clear unless the tag is outlined, and taken
        // out of the padding so the tag keeps Blueprint's size. Its corner is the theme's.
        let border = shape::border(cx);
        library::Tag::new()
            .max_w_full()
            .gap(size::tag_gap(cx, large))
            .min_h(size::tag_min(cx, large))
            .min_w(size::tag_min(cx, large))
            .px(size::tag_pad_x(cx, large) - border)
            .py(size::tag_pad_y(cx, large) - border)
            .bg(ground)
            .border_color(outline.unwrap_or(Hsla::transparent_black()))
            .text_color(fg)
            .text_size(if large {
                text::body_medium(cx)
            } else {
                text::body_small(cx)
            })
            .line_height(size::tag_line(cx, large))
            .children(self.icon.map(Icon::new))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.children),
            )
    }
}
