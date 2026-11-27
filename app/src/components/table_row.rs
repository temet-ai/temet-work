//! TableRow (`temet-tablerow`): a row of an HTMLTable that can be the selected one, drawn as an
//! accent wash with a line above and below. This file also holds the row's cells, `Th` and
//! `Td`, which the design system writes as `<th>` and `<td>`.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::{Class, apply};
use crate::theme;

/// A column head, `<th>`: bold text.
pub struct Th {
    pub(crate) text: SharedString,
}

impl Th {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

/// A cell, `<td>`: text, or an element such as a StageTag.
pub struct Td {
    content: Content,
    class_name: Vec<Class>,
    col_span: bool,
}

enum Content {
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
    /// A cell's box in column `i`, with Blueprint's cell padding (`table.bp6-html-table td`),
    /// or `--tm-space-4` at the sides in an `Issues` table (blueprint-overrides.css).
    pub(crate) fn cell(&self, i: usize, span: bool, cx: &App) -> Div {
        use theme::{size, space};
        let pad_x = if self.issues {
            space::s4(cx)
        } else {
            size::cell_pad_x(cx)
        };
        let cell = div().px(pad_x).py(size::cell_pad_y(cx, self.compact));
        match self.columns.get(i) {
            Some(Some(width)) if !span => cell.flex_none().w(*width),
            _ => cell.flex_1().min_w_0(),
        }
    }
}

/// How far a line box reaches below its baseline, in the text style in force: the font's
/// descent plus half the leading.
fn below_baseline(window: &Window) -> Pixels {
    let style = window.text_style();
    let rem = window.rem_size();
    let font_size = style.font_size.to_pixels(rem);
    let line = style.line_height_in_pixels(rem);
    let text = window.text_system();
    let font = text.resolve_font(&style.font());
    let ascent = text.ascent(font, font_size);
    let descent = text.descent(font, font_size);
    (line - (ascent + descent)) / 2. + descent
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct TableRow {
    id: ElementId,
    selected: bool,
    on_click: Option<OnClick>,
    cells: Vec<Td>,
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
}

impl RenderOnce for TableRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use theme::{colour, shape, size, space};
        let layout = self.layout;
        // A raised tag's line box: the tag above its baseline, which is its icon's foot, less
        // the raise; then the text's reach below the baseline.
        let tag_line =
            size::tag_pad_y(cx, false) + size::icon(cx) - space::s1(cx) + below_baseline(window);
        let cells = self.cells.into_iter().enumerate().map(|(i, td)| {
            let mut cell = layout.cell(i, td.col_span, cx);
            for class in &td.class_name {
                cell = apply(cell, *class, cx);
                if *class == Class::IssueKey && layout.issues {
                    cell = cell.pr(space::s1(cx));
                }
            }
            match td.content {
                Content::Text(text) => cell.child(text),
                Content::Element(element) if layout.issues => cell.child(
                    div()
                        .h(tag_line)
                        .child(div().flex().mt(-space::s1(cx)).child(element)),
                ),
                Content::Element(element) => cell.child(element),
            }
        });

        let line = |top: bool, colour| {
            let rule = div()
                .absolute()
                .left_0()
                .right_0()
                .h(shape::border(cx))
                .bg(colour);
            if top { rule.top_0() } else { rule.bottom_0() }
        };
        let mut row = div()
            .id(self.id)
            .relative()
            .flex()
            .items_start()
            .w_full()
            .children(cells);
        if self.selected {
            let fill = colour::selected_fill(cx);
            let edge = colour::selected_line(cx);
            row = row
                .bg(fill)
                .child(line(true, edge))
                .child(line(false, edge));
        } else {
            if layout.interactive {
                let hover = colour::hover(cx);
                row = row.hover(move |style| style.bg(hover));
            }
            if layout.first {
                // `tbody tr:first-child td { box-shadow: inset 0 1px 0 0 ... }`: Blueprint's
                // rule under the column heads, whose nearest token is the panel edge.
                row = row.child(line(true, colour::edge(cx)));
            }
        }
        if layout.interactive {
            row = row.cursor_pointer();
        }
        if let Some(handler) = self.on_click {
            row = row.on_click(move |event, window, cx| handler(event, window, cx));
        }
        row
    }
}
