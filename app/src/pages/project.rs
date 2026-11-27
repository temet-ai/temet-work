//! Project (`pages-temet-work-radar`): one project, with the areas, projects and agent groups
//! on the left and the project's milestones and their issues in the middle. The right column
//! (the selected issue), Needs you, Machines and Limits come later.

use gpui::{App, Context, IntoElement, ParentElement, Render, SharedString, Window};

use crate::components::app_shell::AppShell;
use crate::composites::group_list::{GroupList, GroupListGroup};
use crate::composites::issue_table::{IssueSummary, IssueTable};
use crate::composites::milestone_panel::MilestonePanel;
use crate::composites::page_header::{PageFact, PageHeader};
use crate::composites::side_nav::{SideNav, SideNavArea};

/// What the Project page shows, in the shape of its parts' props.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectView {
    pub areas: Vec<SideNavArea>,
    /// The id of the project on screen.
    pub project: SharedString,
    pub groups: Vec<GroupListGroup>,
    /// The page title: the project's name.
    pub title: SharedString,
    /// Where the project's issues and code live.
    pub facts: Vec<PageFact>,
    pub milestones: Vec<Milestone>,
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

/// The Project page: a window's view, holding what it shows and which issue is selected.
pub struct Project {
    view: ProjectView,
    selection: Selection,
}

impl Project {
    pub fn new(view: ProjectView, selected: Option<SharedString>) -> Self {
        Self {
            view,
            selection: Selection::new(selected),
        }
    }
}

impl Render for Project {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = cx.entity().downgrade();
        let on_select = move |key: &SharedString, _: &mut Window, cx: &mut App| {
            page.update(cx, |page, cx| {
                if page.selection.select(key) {
                    cx.notify();
                }
            })
            .ok();
        };
        let view = &self.view;
        let selected = self.selection.key().cloned();
        let milestones = view.milestones.iter().map(|m| {
            let mut panel = MilestonePanel::new(m.name.clone())
                .done(m.done)
                .total(m.total);
            if let Some(due) = &m.due {
                panel = panel.due(due.clone());
            }
            panel.child(
                IssueTable::new(
                    SharedString::from(format!("issues-{}", m.name)),
                    m.issues.clone(),
                )
                .selected(selected.clone())
                .on_select(on_select.clone()),
            )
        });
        AppShell::new(
            SideNav::new(view.areas.clone())
                .selected(view.project.clone())
                .child(GroupList::new(view.groups.clone())),
        )
        .child(PageHeader::new(view.title.clone()).facts(view.facts.clone()))
        .children(milestones)
    }
}
