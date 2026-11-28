//! FormGroup (`blueprint-formgroup`): a field with its label over it, as the RefineDialog's
//! Title. Blueprint's rules place it: the label a spacing step above the field, and four steps
//! under the group (`.bp6-form-group`, `label.bp6-label`).

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::theme::{colour, size, text};

#[derive(IntoElement)]
pub struct FormGroup {
    label: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl Default for FormGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl FormGroup {
    pub fn new() -> Self {
        Self {
            label: None,
            children: Vec::new(),
        }
    }

    /// The field's name, over it.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl ParentElement for FormGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FormGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (under, label_gap) = size::form_group(cx);
        let label = self.label.map(|label| {
            div()
                .mb(label_gap)
                .text_color(colour::text(cx))
                .text_size(text::body(cx))
                .line_height(text::line_height(cx))
                .child(label)
        });
        div()
            .flex()
            .flex_col()
            .mb(under)
            .children(label)
            .children(self.children)
    }
}
