//! IssueTable (`temet-issuetable`): issues as a table of key, title, stage, agent and last
//! change. Every table takes the theme's column widths, so columns line up from panel to panel.
//! A narrow table leaves out the Agent column. With no issues it draws no table and no head
//! row, only "No issues".

use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window, canvas, div,
};

use crate::components::html_table::HTMLTable;
use crate::components::stage_tag::{Stage, StageTag};
use crate::components::table_row::{TableRow, Td, Th};
use crate::components::{Class, classed};
use crate::theme::size::{self, IssueColumns};

/// One issue as a row of the table.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueSummary {
    /// The issue's key, such as WATCH-12.
    pub key: SharedString,
    /// The title; it takes up to two lines, then ends in an ellipsis.
    pub title: SharedString,
    pub stage: Stage,
    /// The agent group working on it, for grouping by group.
    pub group: Option<SharedString>,
    /// The agent holding it, role and runtime; the table shows None when there is none.
    pub agent: Option<SharedString>,
    /// How long ago it changed, such as "4 min ago".
    pub updated: SharedString,
}

type OnSelect = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// How the table puts its issues under group rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupBy {
    /// A row per stage, in the order of the stages, and no Stage column.
    Stage,
    /// A row per agent group, in the order the groups first appear.
    Group,
}

/// The issues under each group row, in order: the stage, or the group's name ("No group" for
/// an issue without one).
pub fn grouped(
    issues: Vec<IssueSummary>,
    by: GroupBy,
) -> Vec<(Option<Stage>, SharedString, Vec<IssueSummary>)> {
    match by {
        GroupBy::Stage => Stage::ALL
            .into_iter()
            .map(|stage| {
                let these: Vec<IssueSummary> = issues
                    .iter()
                    .filter(|i| i.stage == stage)
                    .cloned()
                    .collect();
                (Some(stage), SharedString::from(stage.name()), these)
            })
            .filter(|(_, _, these)| !these.is_empty())
            .collect(),
        GroupBy::Group => {
            let name = |i: &IssueSummary| i.group.clone().unwrap_or_else(|| "No group".into());
            let mut order: Vec<SharedString> = Vec::new();
            for issue in &issues {
                let n = name(issue);
                if !order.contains(&n) {
                    order.push(n);
                }
            }
            order
                .into_iter()
                .map(|n| {
                    let these = issues.iter().filter(|i| name(i) == n).cloned().collect();
                    (None, n, these)
                })
                .collect()
        }
    }
}

/// A group row's count, as "1 issue" or "3 issues".
pub fn count(n: usize) -> String {
    if n == 1 {
        "1 issue".into()
    } else {
        format!("{n} issues")
    }
}

#[derive(IntoElement)]
pub struct IssueTable {
    id: ElementId,
    issues: Vec<IssueSummary>,
    group_by: Option<GroupBy>,
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
            group_by: None,
            selected: None,
            on_select: None,
            show_agent: None,
        }
    }

    /// Puts the issues under a row per stage or per agent group. To group by milestone, give
    /// each MilestonePanel its own table.
    pub fn group_by(mut self, by: GroupBy) -> Self {
        self.group_by = Some(by);
        self
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
pub fn colgroup(columns: &IssueColumns, show_stage: bool, show_agent: bool) -> Vec<Option<Pixels>> {
    let mut widths = vec![Some(columns.key), None];
    if show_stage {
        widths.push(Some(columns.stage));
    }
    if show_agent {
        widths.push(Some(columns.agent));
    }
    widths.push(Some(columns.updated));
    widths
}

impl RenderOnce for IssueTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // With no issues it is not a table, so it draws no head row, only "No issues".
        if self.issues.is_empty() {
            return classed(Class::NoIssues, cx)
                .child("No issues")
                .into_any_element();
        }
        let columns = size::issue_columns(cx);
        // The table's width, measured after layout; until then it is taken as wide.
        let width: Entity<Pixels> = window.use_keyed_state(self.id.clone(), cx, |_, _| Pixels::MAX);
        let measured = *width.read(cx);
        let show_agent = self
            .show_agent
            .unwrap_or_else(|| agent_fits(measured, &columns, true));

        let show_stage = self.group_by != Some(GroupBy::Stage);
        let mut heads = vec![Th::new("Key"), Th::new("Title")];
        if show_stage {
            heads.push(Th::new("Stage"));
        }
        if show_agent {
            heads.push(Th::new("Agent"));
        }
        heads.push(Th::new("Updated"));

        let selected = self.selected;
        let on_select = self.on_select;
        let row = |issue: IssueSummary| {
            let mut row = TableRow::new(SharedString::from(format!("issue-{}", issue.key)))
                .selected(selected.as_ref() == Some(&issue.key));
            if let Some(on_select) = on_select.clone() {
                let key = issue.key.clone();
                row = row.on_click(move |_, window, cx| on_select(&key, window, cx));
            }
            row = row
                .child(
                    Td::new(issue.key)
                        .class_name(Class::Mono)
                        .class_name(Class::IssueKey),
                )
                .child(Td::new(issue.title).class_name(Class::IssueTitle));
            if show_stage {
                row = row.child(Td::element(StageTag::new(issue.stage)));
            }
            if show_agent {
                let agent = issue.agent.unwrap_or_else(|| "None".into());
                row = row.child(Td::new(agent).class_name(Class::IssueQuiet));
            }
            row.child(Td::new(issue.updated).class_name(Class::IssueQuiet))
        };

        let mut table = HTMLTable::new()
            .compact(true)
            .interactive(on_select.is_some())
            .class_name(Class::Issues)
            .colgroup(colgroup(&columns, show_stage, show_agent))
            .thead(heads);
        if let Some(by) = self.group_by {
            // Each group is a body of its own, under a row with its stage or name and count.
            for (stage, name, issues) in grouped(self.issues, by) {
                let label = match stage {
                    Some(stage) => StageTag::new(stage)
                        .class_name(Class::IssuesStatus)
                        .into_any_element(),
                    None => name.clone().into_any_element(),
                };
                let head =
                    TableRow::new(SharedString::from(format!("group-{name}"))).head(Th::element(
                        classed(Class::IssueGroup, cx)
                            .child(label)
                            .child(classed(Class::IssueCount, cx).child(count(issues.len()))),
                    ));
                table = table.tbody(std::iter::once(head).chain(issues.into_iter().map(row)));
            }
        } else {
            table = table.tbody(self.issues.into_iter().map(row));
        }
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
        div()
            .relative()
            .w_full()
            .child(table)
            .child(measure)
            .into_any_element()
    }
}
