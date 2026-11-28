//! CheckList (`temet-checklist`): the checks of an issue's plan: a line counting where they
//! stand, then for each what done means, what proves it and its status. A long done-when or
//! proven-by shows its first lines, with plus to open it whole and minus to close it. A check
//! that has not run has no status tag. With no checks it says "No checks yet".

use gpui::{
    App, ElementId, Entity, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window, canvas, div,
};

use crate::components::button::{Button, Size, Variant};
use crate::components::icon::IconName;
use crate::components::list_row::ListRow;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::{CHECK_LINES, CHECK_META_LINES, Class, check_lines, classed, spans};
use crate::theme::space;

/// One check in a plan.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanCheck {
    /// What has to be true for the work to be done. A long one shows its first lines until
    /// opened.
    pub done_when: SharedString,
    /// The command or action that shows it.
    pub proven_by: SharedString,
    /// Where the check stands. None for a check that has not run yet: the row has no status
    /// tag.
    pub status: Option<Status>,
    /// Text in place of the status word, such as "Passed in 41 s".
    pub status_text: Option<SharedString>,
}

/// The line over the checks, such as "3 checks: 2 passed, 1 waiting": how many there are, then
/// how many stand at each status, in the order the statuses first appear. None with no checks.
pub fn count_line(checks: &[PlanCheck]) -> Option<String> {
    if checks.is_empty() {
        return None;
    }
    let mut counts: Vec<(String, usize)> = Vec::new();
    for c in checks {
        let word = c.status.map_or_else(
            || "not run yet".to_string(),
            |s| s.view().word.to_lowercase(),
        );
        match counts.iter_mut().find(|(w, _)| *w == word) {
            Some((_, n)) => *n += 1,
            None => counts.push((word, 1)),
        }
    }
    let total = match checks.len() {
        1 => "1 check".to_string(),
        n => format!("{n} checks"),
    };
    let each: Vec<String> = counts.iter().map(|(w, n)| format!("{n} {w}")).collect();
    Some(format!("{total}: {}", each.join(", ")))
}

/// A row's state between frames: whether it is open, and its text's width, measured after
/// layout.
#[derive(Clone, Copy, Debug, Default)]
struct RowState {
    open: bool,
    width: Option<Pixels>,
}

#[derive(IntoElement)]
pub struct CheckList {
    id: ElementId,
    checks: Vec<PlanCheck>,
}

impl CheckList {
    /// A list. `id` is its identity in the window, which keeps each row open or closed between
    /// frames.
    pub fn new(id: impl Into<ElementId>, checks: Vec<PlanCheck>) -> Self {
        Self {
            id: id.into(),
            checks,
        }
    }
}

impl RenderOnce for CheckList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = count_line(&self.checks).map(|line| classed(Class::PaneCount, cx).child(line));
        let empty = self
            .checks
            .is_empty()
            .then(|| ListRow::new("No checks yet"));
        let mut rows = Vec::new();
        for (i, c) in self.checks.into_iter().enumerate() {
            let key = SharedString::from(format!("{}-check-{i}", self.id));
            let state: Entity<RowState> =
                window.use_keyed_state(key.clone(), cx, |_, _| RowState::default());
            let RowState { open, width } = *state.read(cx);

            let runs = vec![
                ("Proven by ".into(), None),
                (c.proven_by, Some(Class::Mono)),
            ];
            let lines = (!open).then_some(CHECK_META_LINES);
            let (meta, meta_cut) = spans(runs, width, lines, window, cx);
            let title_cut =
                width.is_some_and(|w| check_lines(&c.done_when, w, window, cx) > CHECK_LINES);
            let text = if open {
                div().child(c.done_when)
            } else {
                classed(Class::CheckClamp, cx).child(c.done_when)
            };
            // The title's width is the text's: measured after layout, for the next frame.
            let measured = state.clone();
            let measure = canvas(
                move |bounds, _, cx| {
                    measured.update(cx, |s, cx| {
                        if s.width != Some(bounds.size.width) {
                            s.width = Some(bounds.size.width);
                            cx.notify();
                        }
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full();
            let title = div().relative().child(text).child(measure);

            let toggle = Button::new(SharedString::from(format!("{key}-toggle")))
                .size(Size::Small)
                .variant(Variant::Minimal);
            let toggle = if open || title_cut || meta_cut {
                let toggled = state.clone();
                toggle
                    .icon(if open {
                        IconName::Minus
                    } else {
                        IconName::Plus
                    })
                    .on_click(move |_, _, cx| {
                        toggled.update(cx, |s, cx| {
                            s.open = !s.open;
                            cx.notify();
                        });
                    })
            } else {
                // Holds the button's place, so status tags line up whether a row opens or not.
                toggle
                    .icon(IconName::Plus)
                    .class_name(Class::CheckToggleNone)
            };
            let status = c.status.map(|status| match c.status_text {
                Some(text) => StatusTag::new(status).children(text),
                None => StatusTag::new(status),
            });
            let end = div()
                .flex()
                .items_center()
                .gap(space::s5(cx))
                .children(status)
                .child(toggle);
            rows.push(ListRow::new(title).meta(meta).end(end));
        }
        div().children(count).children(empty).children(rows)
    }
}
