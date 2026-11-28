//! RefineDialog (`temet-refinedialog`): an issue's title and the checks of its plan, for the
//! person to edit. Each check is what has to be true and what proves it; checks are added and
//! deleted in place. Save keeps the edit; Save and send to refiner also hands the issue to a
//! refiner. Its close button, Escape and Cancel close it unsaved. Each time it opens it starts
//! from the title and checks it is given. While what a button sent is answered its buttons are
//! busy and its fields take no edit; a refusal's reason shows over the fields.

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Entity, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, div,
};

use crate::components::button::{Button, Variant};
use crate::components::callout::Callout;
use crate::components::dialog::Dialog;
use crate::components::form_group::FormGroup;
use crate::components::icon::IconName;
use crate::components::input_group::InputGroup;
use crate::components::text_area::TextArea;
use crate::components::{Class, Intent, classed};
use crate::theme::space;

/// One check of the plan as the person edits it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RefineCheck {
    /// What has to be true for the work to be done.
    pub done_when: SharedString,
    /// The command or action that shows it.
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

/// Which of the two buttons sent what is being answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sending {
    Save,
    SaveAndSend,
}

type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;
type OnSave = Rc<dyn Fn(&SharedString, &[RefineCheck], bool, &mut Window, &mut App)>;

/// The edit so far, and what it started from: the issue it is for and whether the dialog was
/// open when last drawn.
#[derive(Default)]
struct Draft {
    open: bool,
    issue_key: SharedString,
    title: SharedString,
    checks: Vec<RefineCheck>,
}

#[derive(IntoElement)]
pub struct RefineDialog {
    id: SharedString,
    issue_key: SharedString,
    title: SharedString,
    checks: Vec<RefineCheck>,
    is_open: bool,
    sending: Option<Sending>,
    not_done: Option<SharedString>,
    on_close: Option<OnClose>,
    on_save: Option<OnSave>,
}

impl RefineDialog {
    /// A dialog for the issue `issue_key`. `id` is its identity in the window, which keeps the
    /// edit between frames.
    pub fn new(
        id: impl Into<SharedString>,
        issue_key: impl Into<SharedString>,
        title: impl Into<SharedString>,
        checks: Vec<RefineCheck>,
    ) -> Self {
        Self {
            id: id.into(),
            issue_key: issue_key.into(),
            title: title.into(),
            checks,
            is_open: false,
            sending: None,
            not_done: None,
            on_close: None,
            on_save: None,
        }
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Which button's save is being answered: its button is busy, and every button and field
    /// takes nothing until the answer.
    pub fn sending(mut self, sending: Option<Sending>) -> Self {
        self.sending = sending;
        self
    }

    /// Why the last save was not done, shown over the fields: the daemon's refusal word for word,
    /// or why it could not be asked.
    pub fn not_done(mut self, not_done: Option<SharedString>) -> Self {
        self.not_done = not_done;
        self
    }

    /// Closes it without saving: its close button, Escape and Cancel.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    /// Called by Save and by Save and send to refiner, with the title without leading or
    /// trailing spaces, the checks that have a done-when, and whether to send it to a refiner.
    pub fn on_save(
        mut self,
        handler: impl Fn(&SharedString, &[RefineCheck], bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_save = Some(Rc::new(handler));
        self
    }
}

/// Changes the draft and draws again.
fn edit(draft: &Entity<Draft>, cx: &mut App, change: impl FnOnce(&mut Draft)) {
    draft.update(cx, |d, cx| {
        change(d);
        cx.notify();
    });
}

/// A column of the checks' grid: `grid-template-columns: minmax(0, 3fr) minmax(0, 2fr) auto`.
fn column(share: f32) -> gpui::Div {
    div().flex_grow(share).flex_basis(Pixels::ZERO).min_w_0()
}

impl RenderOnce for RefineDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let draft: Entity<Draft> =
            window.use_keyed_state(self.id.clone(), cx, |_, _| Draft::default());
        // A new issue, or the same one opened again, starts from what it has now.
        let (open, key) = {
            let d = draft.read(cx);
            (d.open, d.issue_key.clone())
        };
        if self.is_open && (!open || key != self.issue_key) {
            draft.update(cx, |d, _| {
                d.issue_key = self.issue_key.clone();
                d.title = self.title.clone();
                d.checks = self.checks.clone();
            });
        }
        if open != self.is_open {
            draft.update(cx, |d, _| d.open = self.is_open);
        }
        let (title, checks) = {
            let d = draft.read(cx);
            (d.title.clone(), d.checks.clone())
        };
        let blank = title.trim().is_empty();
        let busy = self.sending.is_some();
        let not_done = self.not_done.clone().map(|why| {
            div()
                .debug_selector(|| "refine-not-done".into())
                .mb(space::s5(cx))
                .child(Callout::new().intent(Intent::Danger).child(why))
        });

        let title_field = {
            let draft = draft.clone();
            InputGroup::new("refine-title")
                .fill(true)
                .disabled(busy)
                .value(title.clone())
                .on_change(move |text, _, cx| {
                    let text = text.clone();
                    edit(&draft, cx, |d| d.title = text);
                })
        };
        // The head shares the rows' grid and leaves the delete button's column empty.
        let head = classed(Class::RefineHead, cx)
            .flex()
            .gap(space::s3(cx))
            .child(column(3.).child("Done when"))
            .child(column(2.).child("Proven by"))
            .child(div());
        let rows = checks.iter().enumerate().map(|(n, c)| {
            let done_when = {
                let draft = draft.clone();
                TextArea::new(SharedString::from(format!("refine-check-{n}-done-when")))
                    .fill(true)
                    .auto_resize(true)
                    .disabled(busy)
                    .value(c.done_when.clone())
                    .on_change(move |text, _, cx| {
                        let text = text.clone();
                        edit(&draft, cx, |d| {
                            if let Some(c) = d.checks.get_mut(n) {
                                c.done_when = text;
                            }
                        });
                    })
            };
            let proven_by = {
                let draft = draft.clone();
                TextArea::new(SharedString::from(format!("refine-check-{n}-proven-by")))
                    .fill(true)
                    .auto_resize(true)
                    .class_name(Class::Mono)
                    .disabled(busy)
                    .value(c.proven_by.clone())
                    .on_change(move |text, _, cx| {
                        let text = text.clone();
                        edit(&draft, cx, |d| {
                            if let Some(c) = d.checks.get_mut(n) {
                                c.proven_by = text;
                            }
                        });
                    })
            };
            let delete = {
                let draft = draft.clone();
                Button::new(SharedString::from(format!("refine-check-{n}-delete")))
                    .variant(Variant::Minimal)
                    .icon(IconName::Trash)
                    .disabled(busy)
                    .on_click(move |_, _, cx| {
                        edit(&draft, cx, |d| {
                            if n < d.checks.len() {
                                d.checks.remove(n);
                            }
                        });
                    })
            };
            div()
                .flex()
                .items_start()
                .gap(space::s3(cx))
                .mb(space::s3(cx))
                .child(column(3.).child(done_when))
                .child(column(2.).child(proven_by))
                .child(delete)
        });
        let none = checks
            .is_empty()
            .then(|| classed(Class::RefineNone, cx).child("No checks yet"));
        let add = {
            let draft = draft.clone();
            Button::new("refine-add")
                .variant(Variant::Minimal)
                .icon(IconName::Add)
                .text("Add check")
                .disabled(busy)
                .on_click(move |_, _, cx| {
                    edit(&draft, cx, |d| d.checks.push(RefineCheck::default()));
                })
        };

        let save = |send: bool| {
            let draft = draft.clone();
            let on_save = self.on_save.clone();
            move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                let (title, checks) = {
                    let d = draft.read(cx);
                    let checks: Vec<RefineCheck> = d
                        .checks
                        .iter()
                        .filter(|c| !c.done_when.trim().is_empty())
                        .cloned()
                        .collect();
                    (SharedString::from(d.title.trim().to_string()), checks)
                };
                if let Some(handler) = on_save.as_ref() {
                    handler(&title, &checks, send, window, cx);
                }
            }
        };
        let cancel = {
            let on_close = self.on_close.clone();
            Button::new("refine-cancel")
                .text("Cancel")
                .disabled(busy)
                .on_click(move |_, window, cx| {
                    if let Some(handler) = on_close.as_ref() {
                        handler(window, cx);
                    }
                })
        };
        let actions = vec![
            cancel.into_any_element(),
            Button::new("refine-save")
                .text("Save")
                .loading(self.sending == Some(Sending::Save))
                .disabled(blank || busy)
                .on_click(save(false))
                .into_any_element(),
            Button::new("refine-send")
                .text("Save and send to refiner")
                .intent(Intent::Primary)
                .loading(self.sending == Some(Sending::SaveAndSend))
                .disabled(blank || busy)
                .on_click(save(true))
                .into_any_element(),
        ];

        let mut dialog = Dialog::new("refine")
            .is_open(self.is_open)
            .title(format!("Refine {}", self.issue_key))
            .icon(IconName::Edit)
            .class_name(Class::Refine)
            .actions(actions)
            .children(not_done)
            .child(FormGroup::new().label("Title").child(title_field))
            .child(
                div()
                    .child(head)
                    .children(rows)
                    .children(none)
                    .child(div().flex().child(add)),
            );
        if let Some(handler) = self.on_close {
            dialog = dialog.on_close(move |window, cx| handler(window, cx));
        }
        dialog
    }
}
