//! DecisionRow (`temet-decisionrow`): one decision an agent made, with the deadline for
//! overturning it and the Overturn button. With only the decision it is a short row with the
//! button at the right, as in NeedsYou; with what was weighed or why, it lists them, and the
//! deadline and button sit under the list, as in an issue's pane.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use crate::components::attention_button::AttentionButton;
use crate::components::attention_text::AttentionText;
use crate::components::icon::IconName;
use crate::components::list_row::ListRow;
use crate::components::property_list::{LabelWidth, PropertyItem, PropertyList};
use crate::components::tag::Tag;
use crate::components::{Class, classed};
use crate::theme::{Intent, space};

/// How costly a decision is to undo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndoCost {
    High,
    Medium,
    Low,
}

impl UndoCost {
    fn word(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// A decision an agent has already made and acted on.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// What was decided, in one sentence.
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
    /// The point after which it can no longer be overturned, such as "round 2 closes".
    pub overturn_before: Option<SharedString>,
    /// Set once the decision is overturned; it replaces the action.
    pub overturned: Option<SharedString>,
}

type OnOverturn = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct DecisionRow {
    id: SharedString,
    decision: Decision,
    inset: bool,
    on_overturn: Option<OnOverturn>,
}

impl DecisionRow {
    /// A row. `id` is the decision's identity in the window.
    pub fn new(id: impl Into<SharedString>, decision: Decision) -> Self {
        Self {
            id: id.into(),
            decision,
            inset: false,
            on_overturn: None,
        }
    }

    /// Pads the sides to line up with a padded panel's content.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Called by the Overturn button. Without it there is no button.
    pub fn on_overturn(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_overturn = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for DecisionRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let d = self.decision;
        let detailed = !d.weighed.is_empty() || d.why.is_some();
        let deadline = d
            .overturned
            .is_none()
            .then_some(d.overturn_before.clone())
            .flatten()
            .map(|before| format!("Overturn before {before}"));
        let attention = |text: String| AttentionText::new().child(text);
        let action = match (&d.overturned, self.on_overturn) {
            (Some(done), _) => Some(
                classed(Class::DecisionDone, cx)
                    .child(done.clone())
                    .into_any_element(),
            ),
            (None, Some(handler)) => Some(
                AttentionButton::new(SharedString::from(format!("overturn-{}", self.id)))
                    .text("Overturn")
                    .on_click(move |_, window, cx| handler(window, cx))
                    .into_any_element(),
            ),
            (None, None) => None,
        };
        let by = match &d.at {
            Some(at) => format!("{} · {at}", d.by),
            None => d.by.to_string(),
        };
        let cost = d.cost.map(|cost| {
            let (intent, icon) = match cost {
                UndoCost::Low => (Intent::None, IconName::InfoSign),
                _ => (Intent::Warning, IconName::WarningSign),
            };
            Tag::new(format!("{} to undo", cost.word()))
                .intent(intent)
                .icon(icon)
        });
        let meta = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_y(space::s1(cx))
            .gap_x(space::s3(cx))
            .children(
                d.issue
                    .map(|key| classed(Class::DecisionIssue, cx).child(key)),
            )
            .child(by)
            .children(cost)
            .children(
                (!detailed)
                    .then(|| deadline.clone().map(attention))
                    .flatten(),
            );
        let mut row = ListRow::new(d.took)
            .meta_first(true)
            .strong(detailed)
            .meta(meta)
            .inset(self.inset);
        if detailed {
            let mut items = Vec::new();
            if !d.weighed.is_empty() {
                let tags = d
                    .weighed
                    .into_iter()
                    .map(|w| Tag::new(w).multiline(true).into_any_element())
                    .collect();
                items.push(PropertyItem::set("Weighed", tags));
            }
            if let Some(why) = d.why {
                items.push(PropertyItem::new("Why", why));
            }
            row = row.child(PropertyList::new(items).label_width(LabelWidth::Narrow));
            if deadline.is_some() || action.is_some() {
                row = row.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .justify_between()
                        .items_center()
                        .gap(space::s4(cx))
                        .mt(space::s4(cx))
                        .children(deadline.map(attention))
                        .children(action),
                );
            }
        } else if let Some(action) = action {
            row = row.end(action);
        }
        row
    }
}
