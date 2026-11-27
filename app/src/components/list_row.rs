//! ListRow (`temet-listrow`): one row of a list, with a quiet rule above it. The title says
//! what the row is, `meta` adds muted detail, `end` holds its status or its one action.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window,
    div,
};

use super::Class;
use crate::theme;

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
        use theme::{colour, shape, size, space, text};
        let compact = self.compact;
        // In a group, a compact row's meta wraps under its title rather than being cut short.
        let wraps = compact && self.class_name.contains(&Class::GroupsGroup);
        let meta = self.meta.map(|meta| {
            let line = div()
                .text_size(text::small(cx))
                .text_color(colour::muted(cx));
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
            let title = div().child(self.title);
            let title = if compact { title.flex_none() } else { title };
            if self.strong {
                title.font_weight(text::heading_weight(cx))
            } else {
                title
            }
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

        let mut row = div()
            .flex()
            .gap(space::s4(cx))
            .border_t(shape::border(cx))
            .border_color(colour::edge_quiet(cx));
        row = if compact {
            row.items_center()
                .min_h(size::row_side(cx))
                .py(space::s1(cx))
                .px(space::s2(cx))
        } else {
            row.items_start().py(space::s4(cx))
        };
        if self.inset {
            row = row.px(space::s5(cx));
        }
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
