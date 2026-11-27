//! IssueTable (`temet-issuetable`): issues as a table of key, title, stage, agent and last
//! change. Every table takes the theme's column widths, so columns line up from panel to panel.
//! A narrow table leaves out the Agent column. With no issues it says "No issues".

use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window, canvas, div,
};

use crate::components::Class;
use crate::components::html_table::HTMLTable;
use crate::components::stage_tag::{Stage, StageTag};
use crate::components::table_row::{TableRow, Td, Th};
use crate::theme::size::{self, IssueColumns};

/// One issue as a row of the table.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueSummary {
    /// The issue's key, such as WATCH-12.
    pub key: SharedString,
    /// The title; it takes up to two lines, then ends in an ellipsis.
    pub title: SharedString,
    pub stage: Stage,
    /// The agent holding it, role and runtime; the table shows None when there is none.
    pub agent: Option<SharedString>,
    /// How long ago it changed, such as "4 min ago".
    pub updated: SharedString,
}

type OnSelect = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct IssueTable {
    id: ElementId,
    issues: Vec<IssueSummary>,
    selected: Option<SharedString>,
    on_select: Option<OnSelect>,
    show_agent: Option<bool>,
}

impl IssueTable {
    /// A table. `id` is its identity in the window, which keeps its measured width between
    /// frames.
    pub fn new(id: impl Into<ElementId>, issues: Vec<IssueSummary>) -> Self {
        Self {
            id: id.into(),
            issues,
            selected: None,
            on_select: None,
            show_agent: None,
        }
    }

    /// The key of the selected issue.
    pub fn selected(mut self, key: Option<SharedString>) -> Self {
        self.selected = key;
        self
    }

    /// Called with an issue's key when its row is clicked. Without it rows cannot be selected.
    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Whether the Agent column shows. Left out, it shows when the table is wide enough to
    /// leave the title its room.
    pub fn show_agent(mut self, show: bool) -> Self {
        self.show_agent = Some(show);
        self
    }
}

/// Whether a table `width` wide leaves the title its least width with the Agent column shown.
pub fn agent_fits(width: Pixels, columns: &IssueColumns, show_stage: bool) -> bool {
    let stage = if show_stage {
        columns.stage
    } else {
        Pixels::ZERO
    };
    width >= columns.key + stage + columns.agent + columns.updated + columns.title_min
}

/// The column widths, `None` for the title, which takes what is left.
pub fn colgroup(columns: &IssueColumns, show_agent: bool) -> Vec<Option<Pixels>> {
    let mut widths = vec![Some(columns.key), None, Some(columns.stage)];
    if show_agent {
        widths.push(Some(columns.agent));
    }
    widths.push(Some(columns.updated));
    widths
}

impl RenderOnce for IssueTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let columns = size::issue_columns(cx);
        // The table's width, measured after layout; until then it is taken as wide.
        let width: Entity<Pixels> = window.use_keyed_state(self.id.clone(), cx, |_, _| Pixels::MAX);
        let measured = *width.read(cx);
        let show_agent = self
            .show_agent
            .unwrap_or_else(|| agent_fits(measured, &columns, true));

        let mut heads = vec![Th::new("Key"), Th::new("Title"), Th::new("Stage")];
        if show_agent {
            heads.push(Th::new("Agent"));
        }
        heads.push(Th::new("Updated"));
        let span = heads.len();

        let rows: Vec<TableRow> = if self.issues.is_empty() {
            vec![TableRow::new("none").child(Td::new("No issues").col_span(span > 0))]
        } else {
            self.issues
                .into_iter()
                .map(|issue| {
                    let mut row = TableRow::new(SharedString::from(format!("issue-{}", issue.key)))
                        .selected(self.selected.as_ref() == Some(&issue.key));
                    if let Some(on_select) = self.on_select.clone() {
                        let key = issue.key.clone();
                        row = row.on_click(move |_, window, cx| on_select(&key, window, cx));
                    }
                    row = row
                        .child(
                            Td::new(issue.key)
                                .class_name(Class::Mono)
                                .class_name(Class::IssueKey),
                        )
                        .child(Td::new(issue.title).class_name(Class::IssueTitle))
                        .child(Td::element(StageTag::new(issue.stage)));
                    if show_agent {
                        row = row.child(Td::new(issue.agent.unwrap_or_else(|| "None".into())));
                    }
                    row.child(Td::new(issue.updated))
                })
                .collect()
        };

        let table = HTMLTable::new()
            .compact(true)
            .interactive(self.on_select.is_some())
            .class_name(Class::Issues)
            .colgroup(colgroup(&columns, show_agent))
            .thead(heads)
            .tbody(rows);
        let measure = canvas(
            move |bounds, _, cx| {
                width.update(cx, |w, cx| {
                    if *w != bounds.size.width {
                        *w = bounds.size.width;
                        cx.notify();
                    }
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        div().relative().w_full().child(table).child(measure)
    }
}
