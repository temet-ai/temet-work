//! HTMLTable (`blueprint-htmltable`): Blueprint's table, with column heads over rows of cells.
//! Its rows are TableRows. The columns take the widths given in `colgroup`, as the design
//! system's tables do with `table-layout: fixed`.

use gpui::{App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window, div};

use super::Class;
use super::table_row::{Layout, TableRow, Th};

#[derive(IntoElement, Default)]
pub struct HTMLTable {
    compact: bool,
    interactive: bool,
    class_name: Option<Class>,
    colgroup: Vec<Option<Pixels>>,
    thead: Vec<Th>,
    tbody: Vec<TableRow>,
}

impl HTMLTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Less padding above and below each cell.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Rows answer the pointer.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// A class of the design system's own, such as `Issues` from IssueTable.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// The column widths, `<colgroup>`: a width, or `None` for a column that takes what is left.
    pub fn colgroup(mut self, columns: impl IntoIterator<Item = Option<Pixels>>) -> Self {
        self.colgroup = columns.into_iter().collect();
        self
    }

    /// The column heads, `<thead>`.
    pub fn thead(mut self, heads: impl IntoIterator<Item = Th>) -> Self {
        self.thead = heads.into_iter().collect();
        self
    }

    /// The rows, `<tbody>`.
    pub fn tbody(mut self, rows: impl IntoIterator<Item = TableRow>) -> Self {
        self.tbody = rows.into_iter().collect();
        self
    }
}

impl RenderOnce for HTMLTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let layout = Layout {
            columns: self.colgroup,
            compact: self.compact,
            interactive: self.interactive,
            issues: self.class_name == Some(Class::Issues),
            first: false,
        };
        let heads = self
            .thead
            .into_iter()
            .enumerate()
            .map(|(i, th)| layout.head(i, cx).child(th.text));
        let rows = self.tbody.into_iter().enumerate().map(|(i, mut row)| {
            row.layout = Layout {
                first: i == 0,
                ..layout.clone()
            };
            row
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(div().flex().items_start().w_full().children(heads))
            .children(rows)
    }
}
