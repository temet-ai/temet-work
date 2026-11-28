//! RefineDialog (`temet-refinedialog`): an issue's title and its checks, for the person to edit.
//! Each check is what has to be true and what proves it, in a row with Delete; Add check adds an
//! empty row. Cancel, the close button, Escape and the backdrop close it; Save and "Save and send
//! to refiner" are disabled while the title is empty, and busy while what they sent is
//! answered. A refusal's reason, or why the daemon could not be asked, shows over the fields.
//!
//! What the person types lives in a [`RefineDraft`], which the page keeps while the dialog is
//! open.

use std::rc::Rc;

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};

use crate::components::button::{Button, Variant};
use crate::components::callout::Callout;
use crate::components::dialog::{Dialog, DialogBody, DialogFooter};
use crate::components::form_group::FormGroup;
use crate::components::icon::IconName;
use crate::components::input_group::{FieldState, InputGroup};
use crate::components::text_area::TextArea;
use crate::components::{Class, Intent, classed};
use crate::theme::{size, space};

/// One check as the person edits it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RefineCheck {
    /// What has to be true for the work to be done.
    pub done_when: SharedString,
    /// The command or action that shows it; may be empty.
    pub proven_by: SharedString,
}

impl RefineCheck {
    pub fn new(done_when: impl Into<SharedString>, proven_by: impl Into<SharedString>) -> Self {
        Self {
            done_when: done_when.into(),
            proven_by: proven_by.into(),
        }
    }
}

/// The fields of one check's row.
#[derive(Clone)]
pub struct RefineRow {
    pub done_when: FieldState,
    pub proven_by: FieldState,
}

/// What the person has typed so far: the title and each check's row.
#[derive(Clone)]
pub struct RefineDraft {
    title: FieldState,
    rows: Vec<RefineRow>,
}

impl RefineDraft {
    /// A draft holding `title` and `checks`, as the dialog opens.
    pub fn new(title: &str, checks: &[RefineCheck], window: &mut Window, cx: &mut App) -> Self {
        let title = FieldState::line(title.to_string(), window, cx);
        let rows = checks
            .iter()
            .map(|c| RefineRow {
                done_when: FieldState::lines(c.done_when.clone(), window, cx),
                proven_by: FieldState::lines(c.proven_by.clone(), window, cx),
            })
            .collect();
        Self { title, rows }
    }

    /// Adds an empty check at the end.
    pub fn add(&mut self, window: &mut Window, cx: &mut App) {
        self.rows.push(RefineRow {
            done_when: FieldState::lines("", window, cx),
            proven_by: FieldState::lines("", window, cx),
        });
    }

    /// Deletes the check in row `n`.
    pub fn delete(&mut self, n: usize) {
        if n < self.rows.len() {
            self.rows.remove(n);
        }
    }

    /// The title field.
    pub fn title_field(&self) -> &FieldState {
        &self.title
    }

    /// The fields of row `n`.
    pub fn row(&self, n: usize) -> Option<&RefineRow> {
        self.rows.get(n)
    }

    /// The title as typed.
    pub fn title(&self, cx: &App) -> String {
        self.title.value(cx)
    }

    /// Every row's check as typed, empty ones too.
    pub fn checks(&self, cx: &App) -> Vec<RefineCheck> {
        self.rows
            .iter()
            .map(|r| RefineCheck::new(r.done_when.value(cx), r.proven_by.value(cx)))
            .collect()
    }
}

/// Which of the two buttons sent what is being answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sending {
    Save,
    SaveAndSend,
}

/// Why what was sent was not done: a refusal, with the daemon's reason word for word, or what
/// went wrong asking it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotDone {
    Refused(SharedString),
    Failed(SharedString),
}

type OnAction = Rc<dyn Fn(&mut Window, &mut App)>;
type OnSave = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type OnDelete = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct RefineDialog {
    is_open: bool,
    issue_key: SharedString,
    draft: RefineDraft,
    sending: Option<Sending>,
    not_done: Option<NotDone>,
    focus: Option<FocusHandle>,
    on_close: Option<OnAction>,
    on_save: Option<OnSave>,
    on_add: Option<OnAction>,
    on_delete: Option<OnDelete>,
}

impl RefineDialog {
    /// The dialog for the issue `issue_key`, drawing `draft`.
    pub fn new(issue_key: impl Into<SharedString>, draft: &RefineDraft) -> Self {
        Self {
            is_open: true,
            issue_key: issue_key.into(),
            draft: draft.clone(),
            sending: None,
            not_done: None,
            focus: None,
            on_close: None,
            on_save: None,
            on_add: None,
            on_delete: None,
        }
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// What is being sent, while it is answered: its button is busy and the others disabled.
    pub fn sending(mut self, sending: Option<Sending>) -> Self {
        self.sending = sending;
        self
    }

    /// Why the last save was not done, shown over the fields.
    pub fn not_done(mut self, not_done: Option<NotDone>) -> Self {
        self.not_done = not_done;
        self
    }

    /// The focus the dialog holds while it is open.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Closes the dialog without saving: its close button, Escape, the backdrop and Cancel.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    /// Saves the draft; `send` is true for "Save and send to refiner".
    pub fn on_save(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_save = Some(Rc::new(handler));
        self
    }

    /// Add check.
    pub fn on_add(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add = Some(Rc::new(handler));
        self
    }

    /// Delete on the row given.
    pub fn on_delete(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_delete = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RefineDialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let busy = self.sending.is_some();
        let no_title = self.draft.title(cx).trim().is_empty();
        let (first, second) = size::refine_columns();

        let not_done = self.not_done.map(|n| {
            let callout = Callout::new().intent(Intent::Danger);
            let callout = match n {
                NotDone::Refused(why) => callout.title("Refused").child(why),
                NotDone::Failed(why) => callout.child(why),
            };
            div()
                .debug_selector(|| "refine-not-done".into())
                .mb(space::s5(cx))
                .child(callout)
        });
        let title = FormGroup::new().label("Title").child(
            InputGroup::new("refine-title", self.draft.title_field())
                .fill(true)
                .disabled(busy),
        );
        let head = classed(Class::RefineHead, cx)
            .child(div().flex_basis(first).min_w_0().child("Done when"))
            .child(div().flex_basis(second).min_w_0().child("Proven by"));
        let rows = self.draft.rows.iter().enumerate().map(|(n, row)| {
            let mut delete = Button::new(SharedString::from(format!("refine-check-{n}-delete")))
                .variant(Variant::Minimal)
                .icon(IconName::Trash)
                .disabled(busy);
            if let Some(handler) = self.on_delete.clone() {
                delete = delete.on_click(move |_, window, cx| handler(n, window, cx));
            }
            classed(Class::RefineRow, cx)
                .child(
                    div().flex_basis(first).min_w_0().child(
                        TextArea::new(
                            SharedString::from(format!("refine-check-{n}-done-when")),
                            &row.done_when,
                        )
                        .fill(true)
                        .disabled(busy),
                    ),
                )
                .child(
                    div().flex_basis(second).min_w_0().child(
                        TextArea::new(
                            SharedString::from(format!("refine-check-{n}-proven-by")),
                            &row.proven_by,
                        )
                        .class_name(Class::Mono)
                        .fill(true)
                        .disabled(busy),
                    ),
                )
                .child(delete)
        });
        let none = self
            .draft
            .rows
            .is_empty()
            .then(|| classed(Class::RefineNone, cx).child("No checks yet"));
        let mut add = Button::new("refine-add")
            .variant(Variant::Minimal)
            .icon(IconName::Add)
            .text("Add check")
            .disabled(busy);
        if let Some(handler) = self.on_add {
            add = add.on_click(move |_, window, cx| handler(window, cx));
        }
        let checks = div()
            .child(head)
            .children(rows)
            .children(none)
            .child(div().flex().child(add));

        let mut cancel = Button::new("refine-cancel").text("Cancel").disabled(busy);
        if let Some(handler) = self.on_close.clone() {
            cancel = cancel.on_click(move |_, window, cx| handler(window, cx));
        }
        let save_button = |id: &'static str, text: &'static str, send: bool| {
            let sends = if send {
                Sending::SaveAndSend
            } else {
                Sending::Save
            };
            let mut b = Button::new(id)
                .text(text)
                .loading(self.sending == Some(sends))
                .disabled(busy || no_title);
            if send {
                b = b.intent(Intent::Primary);
            }
            if let Some(handler) = self.on_save.clone() {
                b = b.on_click(move |_, window, cx| handler(send, window, cx));
            }
            b
        };
        let save = save_button("refine-save", "Save", false);
        let send = save_button("refine-send", "Save and send to refiner", true);
        let footer = DialogFooter::new().actions([
            cancel.into_any_element(),
            save.into_any_element(),
            send.into_any_element(),
        ]);

        let body: AnyElement = DialogBody::new()
            .children(not_done)
            .child(title)
            .child(checks)
            .into_any_element();
        let mut dialog = Dialog::new()
            .is_open(self.is_open)
            .title(format!("Refine {}", self.issue_key))
            .icon(IconName::Edit)
            .class_name(Class::Refine)
            .child(body)
            .child(footer);
        if let Some(focus) = &self.focus {
            dialog = dialog.track_focus(focus);
        }
        if let Some(handler) = self.on_close {
            dialog = dialog.on_close(move |window, cx| {
                if !busy {
                    handler(window, cx);
                }
            });
        }
        dialog
    }
}
