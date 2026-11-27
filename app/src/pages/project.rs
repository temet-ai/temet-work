//! Project (`pages-temet-work-radar`): one project: the areas, projects, agent groups, machines
//! and limits on the left; what needs the person and the project's issues in the middle; the
//! selected issue on the right.

use std::collections::HashMap;

use gpui::{App, Context, IntoElement, ParentElement, Render, SharedString, Window};

use crate::components::Intent;
use crate::components::app_shell::AppShell;
use crate::components::segmented_control::{SegmentedControl, SegmentedControlOption};
use crate::composites::group_list::{GroupList, GroupListGroup};
use crate::composites::issue_pane::{
    IssueDecision, IssuePane, IssuePlan, IssuePullRequest, IssueTab,
};
use crate::composites::issue_table::{GroupBy, IssueSummary, IssueTable};
use crate::composites::meter_list::{MeterList, MeterListItem};
use crate::composites::milestone_panel::MilestonePanel;
use crate::composites::needs_you::{NeedsYou, NeedsYouDecision};
use crate::composites::page_header::{PageFact, PageHeader};
use crate::composites::review_rounds::ReviewRound;
use crate::composites::side_nav::{SideNav, SideNavArea};

/// What the Project page shows, in the shape of its parts' props.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectView {
    pub areas: Vec<SideNavArea>,
    /// The id of the project on screen.
    pub project: SharedString,
    pub groups: Vec<GroupListGroup>,
    pub machines: Vec<MeterListItem>,
    pub limits: Vec<MeterListItem>,
    /// The page title: the project's name.
    pub title: SharedString,
    /// Where the project's issues and code live.
    pub facts: Vec<PageFact>,
    /// The agents' decisions for the person to read, costliest to undo first.
    pub needs: Vec<NeedsYouDecision>,
    pub milestones: Vec<Milestone>,
    /// What each issue has so far beyond its row, by key.
    pub details: HashMap<SharedString, IssueDetail>,
}

/// A milestone and its open issues.
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub name: SharedString,
    pub due: Option<SharedString>,
    /// How many of its issues are done; they are not listed.
    pub done: usize,
    /// How many issues it has, done or not.
    pub total: usize,
    pub issues: Vec<IssueSummary>,
}

/// What an issue has beyond its row: its review round, and the parts of its pane it has so far.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IssueDetail {
    /// The review round it is in; 0 before its first review.
    pub round: usize,
    pub plan: Option<IssuePlan>,
    pub pull: Option<IssuePullRequest>,
    pub rounds: Option<Vec<ReviewRound>>,
    pub decisions: Option<Vec<IssueDecision>>,
}

/// How the middle column groups the issues: a panel per milestone, or one table grouped by
/// stage or agent group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    #[default]
    Milestone,
    Stage,
    Group,
}

impl Grouping {
    pub const ALL: [Grouping; 3] = [Self::Milestone, Self::Stage, Self::Group];

    /// The option's value and label in the Group by control.
    pub fn value(self) -> &'static str {
        match self {
            Self::Milestone => "milestone",
            Self::Stage => "stage",
            Self::Group => "group",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Milestone => "Milestone",
            Self::Stage => "Stage",
            Self::Group => "Group",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.value() == value)
    }

    /// The IssueTable grouping, for a single table.
    pub fn table(self) -> Option<GroupBy> {
        match self {
            Self::Milestone => None,
            Self::Stage => Some(GroupBy::Stage),
            Self::Group => Some(GroupBy::Group),
        }
    }
}

/// The issue selected in the page's tables: one at most, across every milestone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection(Option<SharedString>);

impl Selection {
    pub fn new(key: Option<SharedString>) -> Self {
        Self(key)
    }

    /// Selects the issue `key`. Returns whether the selection changed.
    pub fn select(&mut self, key: &SharedString) -> bool {
        if self.0.as_ref() == Some(key) {
            return false;
        }
        self.0 = Some(key.clone());
        true
    }

    /// The selected issue's key.
    pub fn key(&self) -> Option<&SharedString> {
        self.0.as_ref()
    }
}

/// The facts the pane shows for an issue: its milestone, group, agent, round and last change.
pub fn issue_facts(
    milestone: &Milestone,
    issue: &IssueSummary,
    detail: Option<&IssueDetail>,
) -> Vec<(SharedString, SharedString)> {
    let none = || SharedString::from("None");
    let round = match detail.map(|d| d.round).unwrap_or_default() {
        0 => "Not reviewed yet".to_string(),
        n => n.to_string(),
    };
    vec![
        ("Milestone".into(), milestone.name.clone()),
        ("Group".into(), issue.group.clone().unwrap_or_else(none)),
        ("Agent".into(), issue.agent.clone().unwrap_or_else(none)),
        ("Round".into(), round.into()),
        ("Updated".into(), issue.updated.clone()),
    ]
}

/// "7 issues across 3 milestones": the summary of the one panel that groups by stage or group.
pub fn all_issues_summary(milestones: &[Milestone]) -> String {
    let issues: usize = milestones.iter().map(|m| m.issues.len()).sum();
    let across = match milestones.len() {
        1 => "1 milestone".to_string(),
        n => format!("{n} milestones"),
    };
    let issues = match issues {
        1 => "1 issue".to_string(),
        n => format!("{n} issues"),
    };
    format!("{issues} across {across}")
}

/// The Project page: a window's view, holding what it shows and what the person has chosen.
pub struct Project {
    view: ProjectView,
    selection: Selection,
    grouping: Grouping,
    /// The open tab of the selected issue's pane; its first tab when none is chosen.
    tab: Option<IssueTab>,
}

impl Project {
    pub fn new(view: ProjectView, selected: Option<SharedString>) -> Self {
        Self {
            view,
            selection: Selection::new(selected),
            grouping: Grouping::Milestone,
            tab: None,
        }
    }

    /// Selects the issue `key` and opens it in the pane, on its first tab.
    pub fn select(&mut self, key: &SharedString) -> bool {
        let changed = self.selection.select(key);
        if changed {
            self.tab = None;
        }
        changed
    }

    pub fn set_grouping(&mut self, grouping: Grouping) {
        self.grouping = grouping;
    }

    pub fn set_tab(&mut self, tab: IssueTab) {
        self.tab = Some(tab);
    }
}

impl Render for Project {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = cx.entity().downgrade();
        let on_select = {
            let page = page.clone();
            move |key: &SharedString, _: &mut Window, cx: &mut App| {
                page.update(cx, |page, cx| {
                    if page.select(key) {
                        cx.notify();
                    }
                })
                .ok();
            }
        };
        let on_group = {
            let page = page.clone();
            move |value: &SharedString, _: &mut Window, cx: &mut App| {
                if let Some(grouping) = Grouping::from_value(value) {
                    page.update(cx, |page, cx| {
                        page.set_grouping(grouping);
                        cx.notify();
                    })
                    .ok();
                }
            }
        };
        let on_tab = move |tab: IssueTab, _: &mut Window, cx: &mut App| {
            page.update(cx, |page, cx| {
                page.set_tab(tab);
                cx.notify();
            })
            .ok();
        };
        // Overturning does nothing yet: the daemon has no command for it.
        let overturn = |_: &SharedString, _: &mut Window, _: &mut App| {};

        let view = &self.view;
        let selected = self.selection.key().cloned();
        let nav = SideNav::new(view.areas.clone())
            .selected(view.project.clone())
            .child(GroupList::new(view.groups.clone()))
            .child(MeterList::new("Machines", view.machines.clone()))
            .child(MeterList::new("Limits", view.limits.clone()));

        let group_by = SegmentedControl::new(
            "group-by",
            Grouping::ALL
                .into_iter()
                .map(|g| SegmentedControlOption {
                    label: g.label().into(),
                    value: g.value().into(),
                })
                .collect(),
        )
        .intent(Intent::Primary)
        .value(self.grouping.value())
        .on_value_change(on_group);
        let header = PageHeader::new(view.title.clone())
            .facts(view.facts.clone())
            .actions_label("Group by")
            .actions(group_by);
        let needs = NeedsYou::new(view.needs.clone())
            .note(format!("{}, costliest to undo first", view.needs.len()))
            .on_overturn(overturn);

        let mut shell = AppShell::new(nav).child(header).child(needs);
        match self.grouping.table() {
            None => {
                for m in &view.milestones {
                    let mut panel = MilestonePanel::new(m.name.clone())
                        .done(m.done)
                        .total(m.total);
                    if let Some(due) = &m.due {
                        panel = panel.due(due.clone());
                    }
                    shell = shell.child(
                        panel.child(
                            IssueTable::new(
                                SharedString::from(format!("issues-{}", m.name)),
                                m.issues.clone(),
                            )
                            .selected(selected.clone())
                            .on_select(on_select.clone()),
                        ),
                    );
                }
            }
            Some(by) => {
                let all = view
                    .milestones
                    .iter()
                    .flat_map(|m| m.issues.clone())
                    .collect();
                shell = shell.child(
                    MilestonePanel::new("All issues")
                        .summary(all_issues_summary(&view.milestones))
                        .child(
                            IssueTable::new("issues-all", all)
                                .group_by(by)
                                .selected(selected.clone())
                                .on_select(on_select),
                        ),
                );
            }
        }

        let found = selected.as_ref().and_then(|key| {
            view.milestones
                .iter()
                .find_map(|m| m.issues.iter().find(|i| &i.key == key).map(|i| (m, i)))
        });
        if let Some((milestone, issue)) = found {
            let detail = view.details.get(&issue.key).cloned().unwrap_or_default();
            let pane = IssuePane::new(issue.key.clone(), issue.title.clone(), issue.stage)
                .facts(issue_facts(milestone, issue, Some(&detail)))
                .plan(detail.plan)
                .pull(detail.pull)
                .rounds(detail.rounds)
                .decisions(detail.decisions)
                .tab(self.tab.unwrap_or(IssueTab::Plan))
                .on_tab_change(on_tab)
                .on_overturn(overturn);
            shell = shell.side(pane);
        }
        shell
    }
}
