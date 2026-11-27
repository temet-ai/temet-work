//! ListRow (`temet-listrow`): one row of a list, with a quiet rule above it. The title says
//! what the row is, `meta` adds muted detail, `end` holds its status or its one action.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window,
    div,
};

use super::{Class, box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::space;

#[derive(IntoElement)]
pub struct ListRow {
    title: SharedString,
    strong: bool,
    meta: Option<SharedString>,
    meta_first: bool,
    end: Option<AnyElement>,
    compact: bool,
    inset: bool,
    class_name: Vec<Class>,
    children: Vec<AnyElement>,
}

impl ListRow {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            strong: false,
            meta: None,
            meta_first: false,
            end: None,
            compact: false,
            inset: false,
            class_name: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Sets the title in the heading weight.
    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }

    /// Muted small text: beside the title in a compact row, cut with an ellipsis; otherwise a
    /// line of its own under the title.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Puts the meta line above the title.
    pub fn meta_first(mut self, meta_first: bool) -> Self {
        self.meta_first = meta_first;
        self
    }

    /// What sits at the right end. It never shrinks.
    pub fn end(mut self, end: impl IntoElement) -> Self {
        self.end = Some(end.into_any_element());
        self
    }

    /// One line at the side column's row height.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Pads the sides to line up with a padded panel's content.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// A class of the design system's own, such as `GroupsEmpty` from GroupList; each call
    /// adds one.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name.push(class);
        self
    }
}

impl ParentElement for ListRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ListRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        let compact = self.compact;
        // In a group, a compact row's meta wraps under its title rather than being cut short.
        let wraps = compact && self.class_name.contains(&Class::GroupsGroup);
        let meta = self.meta.map(|meta| {
            let look = look::get(cx, "ListRow", "tm-row-main tm-row-meta", rest);
            let line = type_of(div(), &look);
            if wraps {
                line.flex_grow(1.)
                    .flex_shrink_0()
                    .max_w_full()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(meta)
            } else if compact {
                line.flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(meta)
            } else {
                line.flex()
                    .flex_wrap()
                    .items_center()
                    .gap_y(space::s1(cx))
                    .gap_x(space::s3(cx))
                    .child(meta)
            }
        });
        let untitled = self.class_name.contains(&Class::GroupsEmpty);
        let title = (!untitled).then(|| {
            let look = if self.strong {
                look::get(
                    cx,
                    "ListRow",
                    "tm-row-main tm-row-strong tm-row-title",
                    rest,
                )
            } else {
                look::get(cx, "ListRow", "tm-row-main tm-row-title", rest)
            };
            let title = type_of(div(), &look).child(self.title);
            if compact { title.flex_none() } else { title }
        });
        let mut main = div().flex_1().min_w_0();
        if wraps {
            main = main
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_x(space::s3(cx))
                .gap_y(Pixels::ZERO);
        } else if compact {
            main = main.flex().items_baseline().gap(space::s3(cx));
        }
        let (before, after) = if self.meta_first {
            (meta, None)
        } else {
            (None, meta)
        };
        main = main.children(before).children(title).children(after);
        if !self.children.is_empty() {
            main = main.child(div().mt(space::s3(cx)).children(self.children));
        }

        let look = if compact {
            look::get(cx, "ListRow", "tm-row tm-row-compact", rest)
        } else if self.inset {
            look::get(cx, "ListRow", "tm-cut-body tm-row tm-row-inset", rest)
        } else {
            look::get(cx, "ListRow", "tm-row", rest)
        };
        let mut row = box_of(div(), &look).flex().gap(space::s4(cx));
        row = if compact {
            // A compact row is one line at the side column's row height, the look's height.
            row.items_center().min_h(look.height)
        } else {
            row.items_start()
        };
        row.child(main).children(self.end.map(|end| {
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(space::s5(cx))
                .child(end)
        }))
    }
}
