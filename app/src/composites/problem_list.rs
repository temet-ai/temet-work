//! ProblemList (`temet-problemlist`): what is wrong now, as the daemon reports it, in a panel
//! above Needs you: one row each with what it is about and what is wrong. It draws nothing when
//! there is no problem. Its rows' Send to session buttons come with temet-work#256.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use crate::components::cut_panel::CutPanel;
use crate::components::heading::Heading;
use crate::components::icon::{Icon, IconName};
use crate::components::list_row::ListRow;
use crate::components::{Class, classed};
use crate::theme::Intent;

/// One thing that is wrong now, as the daemon reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    /// What it is about, such as "deploy clacks#203".
    pub name: SharedString,
    /// What is wrong, such as "failed 3 times: the release was refused (exit 2)".
    pub text: SharedString,
}

#[derive(IntoElement)]
pub struct ProblemList {
    problems: Vec<Problem>,
}

impl ProblemList {
    /// The problems, in the daemon's order.
    pub fn new(problems: Vec<Problem>) -> Self {
        Self { problems }
    }
}

/// The heading's note: "1 problem" or "3 problems".
pub fn count(n: usize) -> String {
    if n == 1 {
        "1 problem".into()
    } else {
        format!("{n} problems")
    }
}

impl RenderOnce for ProblemList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.problems.is_empty() {
            // Nothing: out of the layout, so the column keeps no gap for it.
            return div().hidden().into_any_element();
        }
        let title = classed(Class::ProblemsTitle, cx)
            .child(Icon::new(IconName::CrossCircle).intent(Intent::Danger))
            .child("Problems");
        let rows = self.problems.into_iter().map(|p| {
            ListRow::new(classed(Class::Mono, cx).child(p.name))
                .meta(p.text)
                .inset(true)
        });
        CutPanel::new()
            .child(
                classed(Class::ProblemsHead, cx).child(Heading::new(title).note(count(rows.len()))),
            )
            .children(rows)
            .into_any_element()
    }
}
