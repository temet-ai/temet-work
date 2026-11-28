//! Heading (`temet-heading`): a heading at one of three sizes from the theme's type scale, with
//! an optional muted note at the right end of its line.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::{Class, apply, baseline, to_baseline, type_of};
use crate::theme::look::{self, State};
use crate::theme::{size, space};

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
    children: AnyElement,
    size: HeadingSize,
    note: Option<SharedString>,
    class_name: Option<Class>,
}

impl Heading {
    /// A heading: its text, or an element such as AttentionText with an icon.
    pub fn new(children: impl IntoElement) -> Self {
        Self {
            children: children.into_any_element(),
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
        let rest = State::Rest;
        let look = match self.size {
            HeadingSize::Page => look::get(cx, "Heading", "tm-heading tm-heading-page", rest),
            HeadingSize::Pane => look::get(cx, "Heading", "tm-heading tm-heading-pane", rest),
            HeadingSize::Section => look::get(cx, "Heading", "tm-heading tm-heading-section", rest),
        };
        // The look's height is one line of the heading; NeedsYou's heading line is taller, to
        // hold its icon beside the title.
        let needs = self.class_name == Some(Class::NeedsHeading);
        let line = if needs {
            look::get(
                cx,
                "NeedsYou",
                "tm-heading tm-heading-section tm-needs-head",
                rest,
            )
        } else {
            look.clone()
        };
        let mut heading = type_of(div(), &look)
            .flex()
            .items_start()
            .justify_between()
            .gap(space::s4(cx))
            .line_height(line.height);
        if needs {
            // The line keeps its height when the title in it is a shorter box of its own.
            heading = heading.min_h(line.height);
        }
        if self.size != HeadingSize::Page {
            heading = heading.mb(space::s3(cx));
        }
        if let Some(class) = self.class_name {
            heading = apply(heading, class, cx);
        }
        heading
            .child(div().min_w_0().child(self.children))
            .children(self.note.map(|note| {
                let look = look::get(
                    cx,
                    "Heading",
                    "tm-heading tm-heading-note tm-heading-section",
                    rest,
                );
                let drop = if needs {
                    // NeedsYou's heading line takes its baseline from the title's first item,
                    // the bell, which has no text: the bell's foot, centred in the title's line.
                    // The note sits on that, below the words' baseline.
                    let title = look::get(
                        cx,
                        "NeedsYou",
                        "tm-attention-text tm-heading-text tm-needs-title",
                        rest,
                    );
                    (title.height + size::icon(cx)) / 2.
                        - baseline(cx, look.font_size, look.height).round()
                } else {
                    // The note sits on the title's baseline.
                    to_baseline(cx, &line, &look)
                };
                type_of(div(), &look)
                    .flex_none()
                    .line_height(look.height)
                    .mt(drop)
                    .child(note)
            }))
    }
}
