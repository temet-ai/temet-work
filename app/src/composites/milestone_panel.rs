//! MilestonePanel (`temet-milestonepanel`): a cut-corner panel for a milestone and its issues,
//! with the name, how many are done and when it is due, a progress bar, and an IssueTable
//! below. Without `done` and `total` it has no bar.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::components::cut_panel::CutPanel;
use crate::components::progress_bar::ProgressBar;
use crate::components::section::{Section, SectionCard};
use crate::theme::{Intent, size};

#[derive(IntoElement)]
pub struct MilestonePanel {
    name: SharedString,
    due: Option<SharedString>,
    done: Option<usize>,
    total: Option<usize>,
    summary: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl MilestonePanel {
    /// A panel titled with a milestone's name, or with what the issues are grouped by.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            due: None,
            done: None,
            total: None,
            summary: None,
            children: Vec::new(),
        }
    }

    /// When the milestone is due, such as 12 Dec.
    pub fn due(mut self, due: impl Into<SharedString>) -> Self {
        self.due = Some(due.into());
        self
    }

    /// How many of its issues are done. With `total`, draws the figure and a progress bar.
    pub fn done(mut self, done: usize) -> Self {
        self.done = Some(done);
        self
    }

    /// How many issues it has.
    pub fn total(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }

    /// Text under the title in place of the done figure and due date.
    pub fn summary(mut self, summary: impl Into<SharedString>) -> Self {
        self.summary = Some(summary.into());
        self
    }
}

impl ParentElement for MilestonePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The line under the panel's title: the summary, or how many are done and when it is due,
/// as "7 / 12 done · due 12 Dec".
pub fn subtitle(
    done: Option<usize>,
    total: Option<usize>,
    due: Option<&str>,
    summary: Option<&str>,
) -> Option<String> {
    if let Some(summary) = summary {
        return Some(summary.to_string());
    }
    let counted = done.zip(total).map(|(d, t)| format!("{d} / {t} done"));
    let due = due.map(|d| format!("due {d}"));
    let line = counted
        .into_iter()
        .chain(due)
        .collect::<Vec<_>>()
        .join(" · ");
    (!line.is_empty()).then_some(line)
}

/// How much of the milestone is done, from 0 to 1; 0 when it has no issues.
pub fn progress(done: usize, total: usize) -> f32 {
    if total == 0 {
        0.
    } else {
        done as f32 / total as f32
    }
}

impl RenderOnce for MilestonePanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let line = subtitle(
            self.done,
            self.total,
            self.due.as_deref(),
            self.summary.as_deref(),
        );
        let mut section = Section::new().compact(true).title(self.name);
        if let Some(line) = line {
            section = section.subtitle(line);
        }
        if let Some((done, total)) = self.done.zip(self.total) {
            section = section.right_element(
                div().w(size::meter_inline(cx)).child(
                    ProgressBar::new()
                        .intent(Intent::Primary)
                        .value(progress(done, total)),
                ),
            );
        }
        CutPanel::new()
            .child(section.child(SectionCard::new().padded(false).children(self.children)))
    }
}
