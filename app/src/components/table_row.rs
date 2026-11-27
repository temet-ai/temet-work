//! TableRow (`temet-tablerow`): a row of an HTMLTable that can be the selected one, drawn as an
//! accent wash with a line above and below. This file also holds the row's cells, `Th` and
//! `Td`, which the design system writes as `<th>` and `<td>`.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::{Class, apply, baseline, padded, type_of};
use crate::theme::look::{self, Look, State};
use crate::theme::{size, space};

/// A head cell, `<th>`: a column's head over the rows, or a group row's head across them.
pub struct Th {
    pub(crate) content: Content,
}

impl Th {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            content: Content::Text(text.into()),
        }
    }

    /// A head holding an element, such as a group row's stage and count.
    pub fn element(element: impl IntoElement) -> Self {
        Self {
            content: Content::Element(element.into_any_element()),
        }
    }
}

/// A cell, `<td>`: text, or an element such as a StageTag.
pub struct Td {
    content: Content,
    class_name: Vec<Class>,
    col_span: bool,
}

pub(crate) enum Content {
    Text(SharedString),
    Element(AnyElement),
}

impl Td {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            content: Content::Text(text.into()),
            class_name: Vec::new(),
            col_span: false,
        }
    }

    /// A cell holding an element. In an `Issues` table the element is a tag (IssueTable's
    /// cells hold no other), raised by `--tm-space-1` so its word lines up with the title's
    /// (`.tm-issues .tm-status`).
    pub fn element(element: impl IntoElement) -> Self {
        Self {
            content: Content::Element(element.into_any_element()),
            class_name: Vec::new(),
            col_span: false,
        }
    }

    /// A class of the design system's own, such as `Mono` and `IssueKey` on an issue's key.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name.push(class);
        self
    }

    /// Spans every column, as `colSpan` set to the column count.
    pub fn col_span(mut self, all: bool) -> Self {
        self.col_span = all;
        self
    }
}

/// What a row takes from its table: the column widths (`None` takes what is left), whether the
/// table is compact, interactive or an `Issues` table, and whether the row is the first.
#[derive(Clone, Default)]
pub(crate) struct Layout {
    pub columns: Vec<Option<Pixels>>,
    pub compact: bool,
    pub interactive: bool,
    pub issues: bool,
    pub first: bool,
}

impl Layout {
    /// A cell's box in column `i`, with the cell padding the design system measured: an
    /// `Issues` table's cells (`.tm-issues td`), or a compact or a default table's cells.
    pub(crate) fn cell(&self, i: usize, span: bool, cx: &App) -> Div {
        let rest = State::Rest;
        let look = if self.issues {
            look::get(cx, "IssueTable", "tm-issue-quiet tm-issues", rest)
        } else if self.compact {
            look::get(cx, "HTMLTable", "td: compact interactive", rest)
        } else {
            look::get(cx, "HTMLTable", "td: default", rest)
        };
        let cell = padded(type_of(div(), &look), &look);
        match self.columns.get(i) {
            Some(Some(width)) if !span => cell.flex_none().w(*width),
            _ => cell.flex_1().min_w_0(),
        }
    }

    /// A head's box: the measured head's type and its padding above and below, and the side
    /// padding of the table's cells; across every column when `span`.
    pub(crate) fn head(&self, i: usize, span: bool, cx: &App) -> Div {
        let rest = State::Rest;
        let head = if self.compact {
            look::get(cx, "HTMLTable", "th: compact interactive", rest)
        } else {
            look::get(cx, "HTMLTable", "th: default", rest)
        };
        type_of(self.cell(i, span, cx), &head)
            .pt(head.padding.top)
            .pb(head.padding.bottom)
    }

    /// The ground and lines of a row: selected, the first under the heads, or neither.
    fn row(&self, selected: bool, state: State, cx: &App) -> Look {
        match (self.issues, selected, self.first) {
            (true, true, true) => look::get(
                cx,
                "IssueTable",
                "first-row selected tm-issue-quiet tm-issues",
                state,
            ),
            (true, true, false) => {
                look::get(cx, "IssueTable", "selected tm-issue-quiet tm-issues", state)
            }
            (true, false, true) => look::get(
                cx,
                "IssueTable",
                "first-row tm-issue-quiet tm-issues",
                state,
            ),
            (true, false, false) => look::get(cx, "IssueTable", "tm-issue-quiet tm-issues", state),
            (false, true, _) => look::get(
                cx,
                "HTMLTable",
                "td: compact first-row interactive selected",
                state,
            ),
            (false, false, true) => look::get(cx, "HTMLTable", "td: first-row", state),
            (false, false, false) => look::get(cx, "HTMLTable", "td: compact interactive", state),
        }
    }
}

/// How far a line box reaches below its baseline, in the text style in force: the font's
/// descent plus half the leading. GPUI gives the descent as a negative offset from the
/// baseline, as fonts store it.
fn below_baseline(window: &Window) -> Pixels {
    let style = window.text_style();
    let rem = window.rem_size();
    let font_size = style.font_size.to_pixels(rem);
    let line = style.line_height_in_pixels(rem);
    let text = window.text_system();
    let font = text.resolve_font(&style.font());
    let ascent = text.ascent(font, font_size);
    let descent = text.descent(font, font_size).abs();
    (line - (ascent + descent)) / 2. + descent
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct TableRow {
    id: ElementId,
    selected: bool,
    on_click: Option<OnClick>,
    cells: Vec<Td>,
    head: Option<Th>,
    pub(crate) layout: Layout,
}

impl TableRow {
    /// A row. `id` is the row's identity in the window, as React's `key`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: false,
            on_click: None,
            cells: Vec::new(),
            head: None,
            layout: Layout::default(),
        }
    }

    /// Draws the row as the selected one.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// The next cell.
    pub fn child(mut self, cell: Td) -> Self {
        self.cells.push(cell);
        self
    }

    /// A head across every column, `<th colSpan scope="colgroup">`, for a group row.
    pub fn head(mut self, head: Th) -> Self {
        self.head = Some(head);
        self
    }
}

impl RenderOnce for TableRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let layout = self.layout;
        // A raised tag's line box: the tag above its baseline, which is its icon's foot, less
        // the raise; then the text's reach below the baseline.
        let tag = look::get(cx, "IssueTable", "minimal tm-issues tm-status", State::Rest);
        let tag_line = tag.padding.top + size::icon(cx) - space::s1(cx) + below_baseline(window);
        // The tag is raised so its word sits on the baseline of the text in the cells beside
        // it (`.tm-issues .tm-status`); the line of the tag's word is its height less its
        // padding.
        let text = look::get(cx, "IssueTable", "tm-issue-title tm-issues", State::Rest);
        let word = tag.height - tag.padding.top - tag.padding.bottom;
        let raise = tag.padding.top + baseline(cx, tag.font_size, word)
            - baseline(cx, text.font_size, text.height);
        let cells = self.cells.into_iter().enumerate().map(|(i, td)| {
            let mut cell = layout.cell(i, td.col_span, cx);
            for class in &td.class_name {
                cell = apply(cell, *class, cx);
            }
            match td.content {
                Content::Text(text) => cell.child(text),
                Content::Element(element) if layout.issues => cell.child(
                    div()
                        .h(tag_line)
                        .child(div().flex().mt(-raise).child(element)),
                ),
                Content::Element(element) => cell.child(element),
            }
        });

        // The row's ground and its lines (inset shadows): the selected row's accent wash with a
        // line above and below, or Blueprint's line under the column heads on the first row.
        let rest = layout.row(self.selected, State::Rest, cx);
        let grouping = self.head.is_some();
        let head = self.head.map(|th| {
            let cell = layout.head(0, true, cx);
            match th.content {
                Content::Text(text) => cell.child(text),
                Content::Element(element) => cell.child(element),
            }
        });
        let mut row = div()
            .id(self.id)
            .flex()
            .items_start()
            .w_full()
            .bg(rest.background)
            .children(head)
            .children(cells);
        if !rest.shadow.is_empty() {
            row = row.shadow(rest.shadow);
        }
        // A group row's head is a `th`, which Blueprint's interactive hover leaves alone.
        let interactive = layout.interactive && !grouping;
        if interactive {
            let hover = layout.row(self.selected, State::Hover, cx).background;
            row = row.cursor_pointer().hover(move |style| style.bg(hover));
        }
        if let Some(handler) = self.on_click {
            row = row.on_click(move |event, window, cx| handler(event, window, cx));
        }
        row
    }
}
