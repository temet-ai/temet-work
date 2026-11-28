//! CheckList (`temet-checklist`): the checks of an issue's plan: for each, what done means, what
//! proves it and its status. A long done-when or proven-by shows its first lines; a click on its
//! row opens it whole and another closes it, and its plus or minus, for the keyboard, shows on the
//! row under the pointer and while the row is open. A check that has not run has no status tag.
//! With no checks it says "No checks yet".

use gpui::{
    App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div,
};

use crate::components::button::{Button, Size, Variant};
use crate::components::icon::IconName;
use crate::components::list_row::ListRow;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::{
    CHECK_LINES, CHECK_META_LINES, Class, check_lines, classed, reveal, spans,
};
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

            let openable = open || title_cut || meta_cut;
            let toggle = Button::new(SharedString::from(format!("{key}-toggle")))
                .size(Size::Small)
                .variant(Variant::Minimal);
            let toggle = if openable {
                let toggled = state.clone();
                let button = toggle
                    .icon(if open {
                        IconName::Minus
                    } else {
                        IconName::Plus
                    })
                    .on_click(move |_, _, cx| {
                        // The row's own click would toggle it back.
                        cx.stop_propagation();
                        toggled.update(cx, |s, cx| {
                            s.open = !s.open;
                            cx.notify();
                        });
                    });
                // Shown on the row under the pointer, and while it is open.
                if open {
                    div().child(button)
                } else {
                    reveal(button)
                }
            } else {
                // Holds the button's place, so status tags line up whether a row opens or not.
                div().child(
                    toggle
                        .icon(IconName::Plus)
                        .class_name(Class::CheckToggleNone),
                )
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
            let row = div()
                .id(key.clone())
                .child(ListRow::new(title).meta(meta).end(end));
            // A cut check opens with a click anywhere on its row.
            let row = if openable {
                let toggled = state.clone();
                row.cursor_pointer().on_click(move |_, _, cx| {
                    toggled.update(cx, |s, cx| {
                        s.open = !s.open;
                        cx.notify();
                    });
                })
            } else {
                row
            };
            rows.push(row);
        }
        div().children(empty).children(rows)
    }
}
