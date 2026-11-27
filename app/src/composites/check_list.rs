//! CheckList (`temet-checklist`): the checks of an issue's plan: for each, what done means, what
//! proves it and its status. With no checks it says "No checks yet".

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Window, div};

use crate::components::list_row::ListRow;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::{Class, spans};

/// One check in a plan.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanCheck {
    /// What has to be true for the work to be done.
    pub done_when: SharedString,
    /// The command or action that shows it.
    pub proven_by: SharedString,
    pub status: Status,
    /// Text in place of the status word, such as "Passed in 41 s".
    pub status_text: Option<SharedString>,
}

#[derive(IntoElement)]
pub struct CheckList {
    checks: Vec<PlanCheck>,
}

impl CheckList {
    pub fn new(checks: Vec<PlanCheck>) -> Self {
        Self { checks }
    }
}

impl RenderOnce for CheckList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let empty = self
            .checks
            .is_empty()
            .then(|| ListRow::new("No checks yet"));
        let rows = self.checks.into_iter().map(|c| {
            let status = match c.status_text {
                Some(text) => StatusTag::new(c.status).children(text),
                None => StatusTag::new(c.status),
            };
            ListRow::new(c.done_when)
                .strong(true)
                .meta(spans(
                    vec![
                        ("Proven by ".into(), None),
                        (c.proven_by, Some(Class::Mono)),
                    ],
                    cx,
                ))
                .end(status)
        });
        div().children(empty).children(rows)
    }
}
