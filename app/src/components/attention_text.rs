//! AttentionText (`temet-attentionbutton`, its text): text in the attention colour, such as
//! "Overturn before merge". Colour on the text only, never a fill behind it.

use gpui::{AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use super::{Class, apply};
use crate::theme::colour;

#[derive(IntoElement, Default)]
pub struct AttentionText {
    class_name: Option<Class>,
    children: Vec<AnyElement>,
}

impl AttentionText {
    pub fn new() -> Self {
        Self::default()
    }

    /// A class of the design system's own, such as `NeedsTitle` from NeedsYou.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }
}

impl ParentElement for AttentionText {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AttentionText {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.tm-attention-text { color: var(--tm-attention) }`, the theme's role.
        let mut text = div().text_color(colour::attention(cx));
        if let Some(class) = self.class_name {
            text = apply(text, class, cx);
        }
        text.children(self.children)
    }
}
