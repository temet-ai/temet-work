//! Heading (`temet-heading`): a heading at one of three sizes from the theme's type scale, with
//! an optional muted note at the right end of its line.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::{Class, apply};
use crate::theme;

/// The heading's size: `page`, the one title of a screen; `pane`, the title of a side pane;
/// `section`, a group of rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingSize {
    Page,
    Pane,
    #[default]
    Section,
}

#[derive(IntoElement)]
pub struct Heading {
    children: SharedString,
    size: HeadingSize,
    note: Option<SharedString>,
    class_name: Option<Class>,
}

impl Heading {
    pub fn new(children: impl Into<SharedString>) -> Self {
        Self {
            children: children.into(),
            size: HeadingSize::Section,
            note: None,
            class_name: None,
        }
    }

    pub fn size(mut self, size: HeadingSize) -> Self {
        self.size = size;
        self
    }

    /// Muted text at the right end of the line, such as a count.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// A class of the design system's own, such as `ListTitle` from GroupList.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }
}

impl RenderOnce for Heading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        use theme::{colour, space, text};
        let mut heading = div()
            .flex()
            .items_baseline()
            .justify_between()
            .gap(space::s4(cx))
            .font_weight(text::heading_weight(cx));
        heading = match self.size {
            HeadingSize::Page => heading
                .text_size(text::page(cx))
                .line_height(text::page_line_height(cx)),
            HeadingSize::Pane => heading
                .mb(space::s3(cx))
                .text_size(text::pane(cx))
                .line_height(text::pane_line_height(cx)),
            HeadingSize::Section => heading.mb(space::s3(cx)).text_size(text::section(cx)),
        };
        if let Some(class) = self.class_name {
            heading = apply(heading, class, cx);
        }
        heading
            .child(div().min_w_0().child(self.children))
            .children(self.note.map(|note| {
                div()
                    .flex_none()
                    .text_size(text::small(cx))
                    .font_weight(text::normal_weight(cx))
                    .text_color(colour::muted(cx))
                    .child(note)
            }))
    }
}
