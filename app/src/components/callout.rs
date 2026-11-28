//! Callout (`blueprint-callout`): a block of text on the faint ground of its intent, with an
//! optional title and an icon at the left. temet-work uses it where a screen has something to
//! tell the person that is not an issue, such as the daemon not answering.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::Intent;
use crate::theme::look::{self, State};
use crate::theme::text;

#[derive(IntoElement)]
pub struct Callout {
    title: Option<SharedString>,
    intent: Intent,
    icon: Option<IconName>,
    children: Vec<AnyElement>,
}

impl Callout {
    pub fn new() -> Self {
        Self {
            title: None,
            intent: Intent::None,
            icon: None,
            children: Vec::new(),
        }
    }

    /// The title, in bold above the contents.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }
}

impl Default for Callout {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Callout {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The icon Blueprint gives an intent when `icon` is not set: none without an intent.
fn default_icon(intent: Intent) -> Option<IconName> {
    match intent {
        Intent::None => None,
        Intent::Primary => Some(IconName::InfoSign),
        Intent::Success => Some(IconName::TickCircle),
        Intent::Warning => Some(IconName::WarningSign),
        Intent::Danger => Some(IconName::CrossCircle),
    }
}

impl RenderOnce for Callout {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let look = match self.intent {
            Intent::None => look::get(cx, "Callout", "default", rest),
            Intent::Primary => look::get(cx, "Callout", "intent-primary", rest),
            Intent::Success => look::get(cx, "Callout", "intent-success", rest),
            Intent::Warning => look::get(cx, "Callout", "intent-warning", rest),
            Intent::Danger => look::get(cx, "Callout", "intent-danger", rest),
        };
        // The look's top padding is also the icon's inset from the top and left edges.
        let inset = look.padding.top;
        let icon = self.icon.or_else(|| default_icon(self.intent));
        box_of(type_of(div(), &look), &look)
            .relative()
            .flex()
            .flex_col()
            .children(icon.map(|icon| {
                div()
                    .absolute()
                    .top(inset)
                    .left(inset)
                    .child(Icon::new(icon))
            }))
            .children(
                self.title
                    .map(|title| div().font_weight(text::bold_weight(cx)).child(title)),
            )
            .children(self.children)
    }
}
