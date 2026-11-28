//! FormGroup (`blueprint-formgroup`): a field with its label above it, such as RefineDialog's
//! Title. The design system measured no look for it; it follows Blueprint's rules for
//! `.bp6-form-group` and `label.bp6-label`.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::theme::size;

#[derive(IntoElement)]
pub struct FormGroup {
    label: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl FormGroup {
    pub fn new() -> Self {
        Self {
            label: None,
            children: Vec::new(),
        }
    }

    /// The field's label, over it.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Default for FormGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for FormGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FormGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .mb(size::form_gap(cx))
            .children(
                self.label
                    .map(|label| div().mb(size::label_gap(cx)).child(label)),
            )
            .children(self.children)
    }
}
