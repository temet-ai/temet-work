//! PropertyList (`temet-propertylist`): labels and values in two columns, such as an issue's
//! milestone, agent and round. Labels are muted and share one column width, so lists line up
//! from panel to panel. A value that is a set of tags wraps under itself.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::{Class, apply};
use crate::theme::{colour, size, space};

/// A value: text, an element, or a set of elements laid out as a wrapping row.
pub enum PropertyValue {
    Text(SharedString),
    Element(AnyElement),
    Set(Vec<AnyElement>),
}

/// A label and its value.
pub struct PropertyItem {
    label: SharedString,
    value: PropertyValue,
    mono: bool,
}

impl PropertyItem {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Text(value.into()),
            mono: false,
        }
    }

    /// A value that is an element, such as a StatusTag.
    pub fn element(label: impl Into<SharedString>, value: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Element(value.into_any_element()),
            mono: false,
        }
    }

    /// A value that is a set, such as the tags of what a decision weighed.
    pub fn set(label: impl Into<SharedString>, values: Vec<AnyElement>) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Set(values),
            mono: false,
        }
    }

    /// Sets the value in the code face, for keys, branches and folders.
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }
}

/// The label column's width: `normal`, or `narrow` for short labels inside a row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelWidth {
    #[default]
    Normal,
    Narrow,
}

#[derive(IntoElement, Default)]
pub struct PropertyList {
    items: Vec<PropertyItem>,
    label_width: LabelWidth,
}

impl PropertyList {
    pub fn new(items: Vec<PropertyItem>) -> Self {
        Self {
            items,
            label_width: LabelWidth::Normal,
        }
    }

    pub fn label_width(mut self, width: LabelWidth) -> Self {
        self.label_width = width;
        self
    }
}

impl RenderOnce for PropertyList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = match self.label_width {
            LabelWidth::Normal => size::label(cx),
            LabelWidth::Narrow => size::label_narrow(cx),
        };
        // The list's labels have no measured look: `.tm-props dt { color: muted }`.
        let muted = colour::muted(cx);
        let rows = self.items.into_iter().map(|item| {
            let mut value = div().flex_1().min_w_0();
            if item.mono {
                value = apply(value, Class::Mono, cx);
            }
            let value = match item.value {
                PropertyValue::Text(text) => value.child(text),
                PropertyValue::Element(el) => value.flex().child(el),
                PropertyValue::Set(els) => {
                    value.flex().flex_wrap().gap(space::s2(cx)).children(els)
                }
            };
            div()
                .flex()
                .items_start()
                .gap(space::s4(cx))
                .child(
                    div()
                        .flex_none()
                        .w(label)
                        .text_color(muted)
                        .child(item.label),
                )
                .child(value)
        });
        div().flex().flex_col().gap(space::s3(cx)).children(rows)
    }
}
