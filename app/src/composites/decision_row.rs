//! DecisionRow (`temet-decisionrow`): one decision an agent made, with the deadline for
//! overturning it and the Overturn button. With only the decision it is a short row with the
//! button at the right, as in NeedsYou. With the agent's question, what was weighed or why, it
//! lists them, and the deadline and button sit under the list, as in an issue's pane.
//!
//! Overturn opens a confirm in the row, its labels in the same column as Took and Why: the other
//! options as radios (the only one as text, already chosen), a reason, then Cancel and
//! Overturn. Overturn stays disabled until both are given; Escape or Cancel closes the confirm.
//! While the app sends it the controls are disabled and Overturn is busy; a send that failed
//! shows why above the buttons and keeps what was typed.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};

use crate::components::attention_button::AttentionButton;
use crate::components::button::{Button, Size, Variant};
use crate::components::callout::Callout;
use crate::components::icon::IconName;
use crate::components::list_row::ListRow;
use crate::components::property_list::{LabelWidth, PropertyItem, PropertyList};
use crate::components::radio_group::{Radio, RadioGroup};
use crate::components::tag::Tag;
use crate::components::text_area::TextArea;
use crate::components::{Class, Intent, classed};
use crate::theme::space;

/// How costly a decision is to undo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoCost {
    High,
    Medium,
    Low,
}

impl UndoCost {
    /// The cost tag's words, such as "Medium undo cost".
    fn words(self) -> &'static str {
        match self {
            Self::High => "High undo cost",
            Self::Medium => "Medium undo cost",
            Self::Low => "Low undo cost",
        }
    }
}

/// Another way a decision could have gone, which the person can switch it to.
#[derive(Clone, Debug, PartialEq)]
pub struct DecisionOption {
    /// What the app gets back when this option is chosen: the daemon's number for it, from 1.
    pub id: SharedString,
    /// The option in words.
    pub label: SharedString,
}

/// What the person gives to overturn a decision.
#[derive(Clone, Debug, PartialEq)]
pub struct OverturnChoice {
    /// The id of the option to switch to.
    pub option: SharedString,
    /// Why, without leading or trailing spaces.
    pub reason: SharedString,
}

/// What an overturn's confirm holds so far: the option chosen and the reason typed, either or
/// both left out. A row given one opens its confirm at first with them filled in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverturnDraft {
    pub option: Option<SharedString>,
    pub reason: Option<SharedString>,
}

/// A decision an agent has already made and acted on.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// The question the agent answered, in a line. When given it is the row's title, and
    /// `took` and the other options show under it.
    pub question: Option<SharedString>,
    /// What was decided: the option taken. Without `question` it is the row's title.
    pub took: SharedString,
    /// Who decided: role and runtime, such as "arbiter · Claude".
    pub by: SharedString,
    /// The issue's key; left out inside that issue's own pane.
    pub issue: Option<SharedString>,
    /// When it was decided, such as 10:41.
    pub at: Option<SharedString>,
    pub cost: Option<UndoCost>,
    /// The options that were weighed.
    pub weighed: Vec<SharedString>,
    /// Why this one was taken.
    pub why: Option<SharedString>,
    /// The other options, without the one taken. With none, the decision cannot be overturned.
    pub options: Vec<DecisionOption>,
    /// The point after which it can no longer be overturned, such as "merge".
    pub overturn_before: Option<SharedString>,
    /// Set once the decision is overturned, such as "Overturned"; it replaces the action.
    pub overturned: Option<SharedString>,
    /// The option the person switched to, once overturned: shown as "Switched to".
    pub overturned_to: Option<SharedString>,
    /// Why the person overturned it: shown as "Reason".
    pub overturn_reason: Option<SharedString>,
}

type OnOverturn = Rc<dyn Fn(&OverturnChoice, &mut Window, &mut App)>;

/// The row's confirm between frames: what it holds while open, none while closed.
struct Confirm {
    draft: Option<OverturnDraft>,
}

#[derive(IntoElement)]
pub struct DecisionRow {
    id: SharedString,
    decision: Decision,
    inset: bool,
    on_overturn: Option<OnOverturn>,
    default_draft: Option<OverturnDraft>,
    pending: bool,
    error: Option<SharedString>,
}

impl DecisionRow {
    /// A row. `id` is the decision's identity in the window: its Overturn button is drawn as
    /// `overturn-{id}`, and its confirm keeps what was typed under it between frames.
    pub fn new(id: impl Into<SharedString>, decision: Decision) -> Self {
        Self {
            id: id.into(),
            decision,
            inset: false,
            on_overturn: None,
            default_draft: None,
            pending: false,
            error: None,
        }
    }

    /// Pads the sides to line up with a padded panel's content.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Called by the confirm's Overturn with the option chosen and the reason. The row stays as
    /// it is until the app sets `overturned`. Without it, or without options, there is no
    /// button.
    pub fn on_overturn(
        mut self,
        handler: impl Fn(&OverturnChoice, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overturn = Some(Rc::new(handler));
        self
    }

    /// Opens the confirm at first with these filled in, such as a draft the app kept. Read the
    /// first time the row is drawn.
    pub fn default_draft(mut self, draft: Option<OverturnDraft>) -> Self {
        self.default_draft = draft;
        self
    }

    /// Set while the app sends the overturn: the confirm stays open with its controls disabled
    /// and Overturn busy.
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    /// Why the last send failed, such as the daemon's reply: shown in the confirm above its
    /// buttons, with the option and reason kept, so Overturn sends again.
    pub fn error(mut self, error: Option<SharedString>) -> Self {
        self.error = error;
        self
    }
}

/// With one other option there is nothing to pick, so it starts chosen.
fn start(mut draft: OverturnDraft, options: &[DecisionOption]) -> OverturnDraft {
    if draft.option.is_none()
        && let [only] = options
    {
        draft.option = Some(only.id.clone());
    }
    draft
}

/// Changes the confirm and draws again.
fn change(confirm: &Entity<Confirm>, cx: &mut App, f: impl FnOnce(&mut Confirm)) {
    confirm.update(cx, |c, cx| {
        f(c);
        cx.notify();
    });
}

/// The confirm under a decision: the other options, the reason, why the last send failed, then
/// Cancel and Overturn.
struct Form<'a> {
    id: &'a SharedString,
    options: &'a [DecisionOption],
    draft: OverturnDraft,
    confirm: &'a Entity<Confirm>,
    pending: bool,
    error: Option<SharedString>,
    on_overturn: Option<OnOverturn>,
}

impl Form<'_> {
    fn draw(self, cx: &App) -> AnyElement {
        let Form {
            id,
            options,
            draft,
            confirm,
            pending,
            error,
            on_overturn,
        } = self;
        let prefix = SharedString::from(format!("overturn-{id}"));
        let switch: AnyElement = match options {
            [only] => div().child(only.label.clone()).into_any_element(),
            many => {
                let choosing = confirm.clone();
                RadioGroup::new(
                    prefix.clone(),
                    many.iter()
                        .map(|o| Radio::new(o.id.clone(), o.label.clone()))
                        .collect(),
                )
                .selected_value(draft.option.clone())
                .disabled(pending)
                .on_change(move |value, _, cx| {
                    let value = value.clone();
                    change(&choosing, cx, |c| {
                        if let Some(d) = c.draft.as_mut() {
                            d.option = Some(value);
                        }
                    });
                })
                .into_any_element()
            }
        };
        let typing = confirm.clone();
        let reason_field = TextArea::new(SharedString::from(format!("{prefix}-reason")))
            .fill(true)
            .auto_resize(true)
            .disabled(pending)
            .value(draft.reason.clone().unwrap_or_default())
            .on_change(move |text, _, cx| {
                let text = text.clone();
                change(&typing, cx, |c| {
                    if let Some(d) = c.draft.as_mut() {
                        d.reason = Some(text);
                    }
                });
            });
        let fields = PropertyList::new(vec![
            PropertyItem::element("Switch to", switch),
            PropertyItem::element("Reason", reason_field).field(true),
        ]);
        let error = error.map(|error| {
            div().mb(space::s4(cx)).child(
                Callout::new()
                    .intent(Intent::Danger)
                    .compact(true)
                    .title("Not sent")
                    .child(error),
            )
        });
        let cancel = {
            let closing = confirm.clone();
            Button::new(SharedString::from(format!("{prefix}-cancel")))
                .size(Size::Small)
                .variant(Variant::Minimal)
                .text("Cancel")
                .disabled(pending)
                .on_click(move |_, _, cx| change(&closing, cx, |c| c.draft = None))
        };
        let option = options
            .iter()
            .find(|o| Some(&o.id) == draft.option.as_ref())
            .map(|o| o.id.clone());
        let reason = draft
            .reason
            .as_ref()
            .map(|r| SharedString::from(r.trim().to_string()))
            .filter(|r| !r.is_empty());
        let confirm_id = SharedString::from(format!("{prefix}-confirm"));
        let overturn: AnyElement = match (option, reason, on_overturn) {
            (Some(option), Some(reason), Some(handler)) => {
                let choice = OverturnChoice { option, reason };
                AttentionButton::new(confirm_id)
                    .text("Overturn")
                    .loading(pending)
                    .on_click(move |_, window, cx| handler(&choice, window, cx))
                    .into_any_element()
            }
            // Without the attention colour, which would hide that it is disabled.
            _ => Button::new(confirm_id)
                .size(Size::Small)
                .variant(Variant::Outlined)
                .text("Overturn")
                .disabled(true)
                .into_any_element(),
        };
        let escaping = confirm.clone();
        div()
            .id(SharedString::from(format!("{prefix}-form")))
            .mt(space::s4(cx))
            .on_key_down(move |event, _, cx| {
                if event.keystroke.key == "escape" && !pending {
                    cx.stop_propagation();
                    change(&escaping, cx, |c| c.draft = None);
                }
            })
            .child(div().mb(space::s4(cx)).child(fields))
            .children(error)
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(space::s4(cx))
                    .child(cancel)
                    .child(overturn),
            )
            .into_any_element()
    }
}

impl RenderOnce for DecisionRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let d = self.decision;
        let id = self.id.clone();
        let first = self
            .default_draft
            .clone()
            .map(|draft| start(draft, &d.options));
        let confirm: Entity<Confirm> = window.use_keyed_state(
            SharedString::from(format!("decision-{id}-confirm")),
            cx,
            |_, _| Confirm { draft: first },
        );
        let detailed = d.question.is_some() || !d.weighed.is_empty() || d.why.is_some();
        let can = d.overturned.is_none() && self.on_overturn.is_some() && !d.options.is_empty();
        let draft = confirm.read(cx).draft.clone().filter(|_| can);
        let open = draft.is_some();

        let deadline = d
            .overturned
            .is_none()
            .then_some(d.overturn_before.clone())
            .flatten()
            .map(|before| {
                classed(Class::DecisionDeadline, cx).child(format!("Overturn before {before}"))
            });
        let action: Option<AnyElement> = if let Some(done) = &d.overturned {
            Some(
                classed(Class::DecisionDone, cx)
                    .child(done.clone())
                    .into_any_element(),
            )
        } else if can && !open {
            let opening = confirm.clone();
            let options = d.options.clone();
            Some(
                Button::new(SharedString::from(format!("overturn-{id}")))
                    .size(Size::Small)
                    .variant(Variant::Outlined)
                    .text("Overturn")
                    .on_click(move |_, _, cx| {
                        let draft = start(OverturnDraft::default(), &options);
                        change(&opening, cx, |c| c.draft = Some(draft));
                    })
                    .into_any_element(),
            )
        } else {
            None
        };
        let form = draft.map(|draft| {
            Form {
                id: &id,
                options: &d.options,
                draft,
                confirm: &confirm,
                pending: self.pending,
                error: self.error.clone(),
                on_overturn: self.on_overturn.clone(),
            }
            .draw(cx)
        });

        let by = match &d.at {
            Some(at) => format!("{} · {at}", d.by),
            None => d.by.to_string(),
        };
        let cost = d.cost.map(|cost| {
            let icon = match cost {
                UndoCost::Low => IconName::InfoSign,
                _ => IconName::WarningSign,
            };
            Tag::new(cost.words()).icon(icon)
        });
        let (meta_deadline, act_deadline) = if detailed {
            (None, deadline)
        } else {
            (deadline, None)
        };
        let meta = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_y(space::s1(cx))
            .gap_x(space::s3(cx))
            .children(
                d.issue
                    .clone()
                    .map(|key| classed(Class::DecisionIssue, cx).child(key)),
            )
            .child(by)
            .children(cost)
            .children(meta_deadline);

        let title = d.question.clone().unwrap_or_else(|| d.took.clone());
        let mut row = ListRow::new(title)
            .meta_first(true)
            .strong(detailed)
            .meta(meta)
            .inset(self.inset);
        if !detailed {
            if let Some(action) = action {
                row = row.end(action);
            }
            return row.children(form);
        }
        let mut items = Vec::new();
        if d.question.is_some() {
            items.push(PropertyItem::new("Took", d.took.clone()));
        }
        let tags = |words: Vec<SharedString>| {
            words
                .into_iter()
                .map(|w| Tag::new(w).multiline(true).into_any_element())
                .collect()
        };
        if !d.weighed.is_empty() {
            items.push(PropertyItem::set("Weighed", tags(d.weighed.clone())));
        } else if d.question.is_some() && !d.options.is_empty() {
            let others = d.options.iter().map(|o| o.label.clone()).collect();
            items.push(PropertyItem::set("Others", tags(others)));
        }
        if let Some(why) = d.why.clone() {
            items.push(PropertyItem::new("Why", why));
        }
        if d.overturned.is_some() {
            if let Some(to) = d.overturned_to.clone() {
                items.push(PropertyItem::new("Switched to", to));
            }
            if let Some(reason) = d.overturn_reason.clone().filter(|r| !r.is_empty()) {
                items.push(PropertyItem::new("Reason", reason));
            }
        }
        let width = if d.question.is_some() {
            LabelWidth::Normal
        } else {
            LabelWidth::Narrow
        };
        row = row.child(PropertyList::new(items).label_width(width));
        if act_deadline.is_some() || action.is_some() {
            // `.tm-decision-act`: the deadline at the left, the action kept at the right.
            row = row.child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_center()
                    .gap(space::s4(cx))
                    .mt(space::s4(cx))
                    .children(act_deadline)
                    .children(action.map(|a| div().ml_auto().child(a))),
            );
        }
        row.children(form)
    }
}
