//! ListRow (`temet-listrow`): one row of a list, with a quiet rule above it. The title says
//! what the row is, `meta` adds muted detail, `end` holds its status or its one action.

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Styled,
    Window, div, px,
};

use super::{Class, ROW_GROUP, apply, baseline, box_of, to_baseline, type_of};
use crate::theme::look::{self, State};
use crate::theme::space;

#[derive(IntoElement)]
pub struct ListRow {
    title: AnyElement,
    strong: bool,
    meta: Option<AnyElement>,
    meta_first: bool,
    end: Option<AnyElement>,
    compact: bool,
    inset: bool,
    class_name: Vec<Class>,
    children: Vec<AnyElement>,
}

impl ListRow {
    /// A row. The title is text, or an element such as a check's clamped done-when.
    pub fn new(title: impl IntoElement) -> Self {
        Self {
            title: title.into_any_element(),
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
    /// line of its own under the title, whose items wrap as a row.
    pub fn meta(mut self, meta: impl IntoElement) -> Self {
        self.meta = Some(meta.into_any_element());
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
        let title_look = if self.strong {
            look::get(
                cx,
                "ListRow",
                "tm-row-main tm-row-strong tm-row-title",
                rest,
            )
        } else {
            look::get(cx, "ListRow", "tm-row-main tm-row-title", rest)
        };
        let meta_look = look::get(cx, "ListRow", "tm-row-main tm-row-meta", rest);
        // Beside the title, the meta sits on the title's baseline.
        let drop = to_baseline(cx, &title_look, &meta_look);
        let meta = self.meta.map(|meta| {
            let line = type_of(div(), &meta_look);
            if wraps {
                // Moved down to the title's baseline without growing the line, so a meta that
                // wraps to its own line does not push the row taller.
                line.flex_grow(1.)
                    .flex_shrink_0()
                    .max_w_full()
                    .relative()
                    .top(drop)
                    .line_height(meta_look.height)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(meta)
            } else if compact {
                line.flex_1()
                    .min_w_0()
                    .mt(drop)
                    .line_height(meta_look.height)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(meta)
            } else {
                // A line of its own, whose text wraps; a meta of several items, such as a
                // decision's, lays them out as a wrapping row of its own.
                line.child(meta)
            }
        });
        let untitled = self.class_name.contains(&Class::GroupsEmpty);
        let title = (!untitled).then(|| {
            let title = type_of(div(), &title_look).child(self.title);
            if compact {
                title.flex_none().line_height(title_look.height)
            } else {
                title
            }
        });
        let mut main = div().flex_1().min_w_0();
        if wraps {
            // The meta wraps under the title when the two do not fit on one line.
            main = main
                .flex()
                .flex_wrap()
                .items_start()
                .gap_x(space::s3(cx))
                .gap_y(Pixels::ZERO);
        } else if compact {
            main = main.flex().items_start().gap(space::s3(cx));
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
        // The group a revealed action (`.tm-reveal`) shows on.
        let mut row = box_of(div(), &look)
            .group(ROW_GROUP)
            .flex()
            .gap(space::s4(cx));
        for class in &self.class_name {
            row = apply(row, *class, cx);
        }
        row = if compact {
            // A compact row is one line at the side column's row height, the look's height.
            row.items_center().min_h(look.height)
        } else {
            row.items_start()
        };
        if compact && self.end.is_some() {
            // The end of a compact row is a tag, centred in the row as the main line is: move
            // the main line so its baseline meets the tag word's. GPUI rounds line heights to
            // whole pixels, snaps each box's edges to whole pixels (halves toward zero) and
            // rounds each glyph's baseline, so the two baselines are worked out on those
            // snapped positions; on the unsnapped ones they differ by less than a pixel and
            // still land on different rows.
            let tag = look::get(cx, "StatusTag", "minimal", rest);
            let word = tag.height - tag.padding.top - tag.padding.bottom;
            let inside = look.height - look.border.top - look.padding.top - look.padding.bottom;
            // Where a box of this height lands when centred in the row; halves go toward zero,
            // as GPUI snaps them.
            let centred = |height: Pixels| px((f32::from((inside - height) / 2.) - 0.5).ceil());
            let tag_word =
                centred(tag.height) + tag.padding.top + baseline(cx, tag.font_size, word).round();
            let title_word = centred(title_look.height.round())
                + baseline(cx, title_look.font_size, title_look.height).round();
            main = main.relative().top(tag_word - title_word);
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
