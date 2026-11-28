//! PropertyList (`temet-propertylist`): labels and values in two columns, such as an issue's
//! milestone, agent and round. Labels are muted and share one column width, so lists line up
//! from panel to panel. A value that is a set of tags wraps under itself.

use std::rc::Rc;

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::{Class, apply, below_baseline};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size, space};

/// A value: text, an element, a set of elements laid out as a wrapping row, or links.
pub enum PropertyValue {
    Text(SharedString),
    Element(AnyElement),
    Set(Vec<AnyElement>),
    Links(Vec<PropertyLink>),
}

type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;

/// A link in a value, as the design system's pages write `<a>` in a PropertyList: in Blueprint's
/// link colour, underlined under the pointer, such as an issue key that opens that issue.
pub struct PropertyLink {
    id: SharedString,
    text: SharedString,
    mono: bool,
    on_click: Option<OnClick>,
}

impl PropertyLink {
    /// A link. `id` is its identity in the window, and its selector for a test.
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            mono: false,
            on_click: None,
        }
    }

    /// In the code face, as Blueprint's `Classes.MONOSPACE_TEXT` sets an issue key.
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

/// A label and its value.
pub struct PropertyItem {
    label: SharedString,
    value: PropertyValue,
    mono: bool,
    field: bool,
}

impl PropertyItem {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Text(value.into()),
            mono: false,
            field: false,
        }
    }

    /// A value that is an element, such as a StatusTag.
    pub fn element(label: impl Into<SharedString>, value: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Element(value.into_any_element()),
            mono: false,
            field: false,
        }
    }

    /// A value that is a set, such as the tags of what a decision weighed.
    pub fn set(label: impl Into<SharedString>, values: Vec<AnyElement>) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Set(values),
            mono: false,
            field: false,
        }
    }

    /// A value that is a text field, such as the reason in DecisionRow's overturn confirm:
    /// its label sits level with the field's middle, where `.tm-overturn-fields { align-items:
    /// baseline }` puts it beside a text area.
    pub fn field(mut self, field: bool) -> Self {
        self.field = field;
        self
    }

    /// A value that is one link or several, such as the issues an issue waits for.
    pub fn links(label: impl Into<SharedString>, links: Vec<PropertyLink>) -> Self {
        Self {
            label: label.into(),
            value: PropertyValue::Links(links),
            mono: false,
            field: false,
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = match self.label_width {
            LabelWidth::Normal => size::label(cx),
            LabelWidth::Narrow => size::label_narrow(cx),
        };
        // The list's labels have no measured look: `.tm-props dt { color: muted }`.
        let muted = colour::muted(cx);
        let tag = look::get(cx, "StatusTag", "minimal", State::Rest);
        // The story keeps that line's fraction, and the next row lands on the next whole
        // pixel; GPUI would round it away, so it is rounded up.
        let tag_line = (tag.padding.top + size::icon(cx) + below_baseline(window)).ceil();
        let rows = self.items.into_iter().map(|item| {
            let mut value = div().flex_1().min_w_0();
            if item.mono {
                value = apply(value, Class::Mono, cx);
            }
            let value = match item.value {
                PropertyValue::Text(text) => value.child(text),
                // A tag in a value sits in a line box as an inline block: its icon's foot on
                // the text's baseline, the line reaching on below it.
                PropertyValue::Element(el) => value.flex().items_start().min_h(tag_line).child(el),
                PropertyValue::Set(els) => {
                    value.flex().flex_wrap().gap(space::s2(cx)).children(els)
                }
                PropertyValue::Links(links) => value
                    .flex()
                    .flex_wrap()
                    .gap(space::s2(cx))
                    .children(links.into_iter().map(|link| anchor(link, cx))),
            };
            let row = div().flex().gap(space::s4(cx));
            let row = if item.field {
                row.items_center()
            } else {
                row.items_start()
            };
            row
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

/// A link as Blueprint draws `<a>`: the link colour, no underline until the pointer is over it.
fn anchor(link: PropertyLink, cx: &App) -> impl IntoElement {
    let selector = link.id.to_string();
    let mut el = div()
        .id(link.id)
        .debug_selector(|| selector)
        .text_color(colour::intent_text(cx, Intent::Primary))
        .cursor_pointer()
        .hover(|style| style.underline())
        .child(link.text);
    if link.mono {
        el = el.font_family(crate::theme::text::code_family(cx));
    }
    if let Some(handler) = link.on_click {
        el = el.on_click(move |_, window, cx| handler(window, cx));
    }
    el
}
