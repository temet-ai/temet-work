//! IssueTable (`temet-issuetable`): issues as a table of key, title, stage, agent and last
//! change, and the milestone when asked. Every table takes the theme's column widths, so columns
//! line up from panel to panel. A narrow table leaves out the Updated column, and a narrower one
//! the Agent column too. With no issues it draws no table and no head row, only "No issues".

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
    /// Its milestone, shown in the Milestone column when `show_milestone` is set.
    pub milestone: Option<SharedString>,
    /// The agent holding it, role and runtime; the table shows None when there is none.
    pub agent: Option<SharedString>,
    /// How long ago it changed, such as "4 min ago".
    pub updated: SharedString,
}

type OnSelect = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// How the table puts its issues under group rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupBy {
    /// A row per stage, from the furthest along (done) to the first (new), and no Stage
    /// column.
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
        // The furthest stage first, so the issues closest to shipping come first.
        GroupBy::Stage => Stage::ALL
            .into_iter()
            .rev()
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
    show_milestone: bool,
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
            show_milestone: false,
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

    /// Whether the Agent column shows. Left out, it shows unless the title would lose its room
    /// even without the Updated column. Set it false when no agent runs, so a column of None
    /// takes no room.
    pub fn show_agent(mut self, show: bool) -> Self {
        self.show_agent = Some(show);
        self
    }

    /// Shows a Milestone column after the title, for a table not grouped by milestone.
    pub fn show_milestone(mut self, show: bool) -> Self {
        self.show_milestone = show;
        self
    }
}

/// Which of the two optional columns a table has room for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Room {
    /// Agent and Updated.
    Both,
    /// Agent alone.
    Agent,
    /// Neither.
    Neither,
}

/// The room a table `width` wide has, with the Stage and Milestone columns as given: Updated
/// needs the title's least width beside it, Agent alone its narrow width.
pub fn room(width: Pixels, columns: &IssueColumns, show_stage: bool, show_milestone: bool) -> Room {
    let mut fixed = columns.key + columns.agent;
    if show_stage {
        fixed += columns.stage;
    }
    if show_milestone {
        fixed += columns.milestone;
    }
    if width >= fixed + columns.updated + columns.title_min {
        Room::Both
    } else if width >= fixed + columns.title_narrow {
        Room::Agent
    } else {
        Room::Neither
    }
}

/// Whether the Agent and Updated columns show: Agent as asked, else when there is room for it;
/// Updated when Agent is left out by request or there is room for both.
pub fn optional_columns(show_agent: Option<bool>, room: Room) -> (bool, bool) {
    let agent = show_agent.unwrap_or(room != Room::Neither);
    let updated = show_agent == Some(false) || room == Room::Both;
    (agent, updated)
}

/// The columns a table draws, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Columns {
    pub stage: bool,
    pub milestone: bool,
    pub agent: bool,
    pub updated: bool,
}

/// The column widths, `None` for the title, which takes what is left.
pub fn colgroup(widths: &IssueColumns, shown: Columns) -> Vec<Option<Pixels>> {
    let mut out = vec![Some(widths.key), None];
    for (show, width) in [
        (shown.stage, widths.stage),
        (shown.milestone, widths.milestone),
        (shown.agent, widths.agent),
        (shown.updated, widths.updated),
    ] {
        if show {
            out.push(Some(width));
        }
    }
    out
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
        let show_stage = self.group_by != Some(GroupBy::Stage);
        let show_milestone = self.show_milestone;
        let (show_agent, show_updated) = optional_columns(
            self.show_agent,
            room(measured, &columns, show_stage, show_milestone),
        );
        let shown = Columns {
            stage: show_stage,
            milestone: show_milestone,
            agent: show_agent,
            updated: show_updated,
        };

        let mut heads = vec![Th::new("Key"), Th::new("Title")];
        if show_stage {
            heads.push(Th::new("Stage"));
        }
        if show_milestone {
            heads.push(Th::new("Milestone"));
        }
        if show_agent {
            heads.push(Th::new("Agent"));
        }
        if show_updated {
            heads.push(Th::new("Updated"));
        }

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
            if show_milestone {
                row = row.child(Td::new(issue.milestone.unwrap_or_default()));
            }
            if show_agent {
                let agent = issue.agent.unwrap_or_else(|| "None".into());
                row = row.child(Td::new(agent).class_name(Class::IssueQuiet));
            }
            if show_updated {
                row = row.child(Td::new(issue.updated).class_name(Class::IssueQuiet));
            }
            row
        };

        let mut table = HTMLTable::new()
            .compact(true)
            .interactive(on_select.is_some())
            .class_name(Class::Issues)
            .colgroup(colgroup(&columns, shown))
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
                // A stage's count is StageCounts' to say; a group's is said only here.
                let tally = stage
                    .is_none()
                    .then(|| classed(Class::IssueCount, cx).child(count(issues.len())));
                let head = TableRow::new(SharedString::from(format!("group-{name}"))).head(
                    Th::element(classed(Class::IssueGroup, cx).child(label).children(tally)),
                );
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
