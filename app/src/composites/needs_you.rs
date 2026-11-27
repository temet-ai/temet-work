//! NeedsYou (`temet-needsyou`): the agents' decisions for the person to read, in one attention
//! panel at the top of the middle column, costliest to undo first. Nothing waits on them; each
//! can be overturned until its deadline. It draws nothing when there are no decisions.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::decision_row::{Decision, DecisionRow};
use crate::components::attention_text::AttentionText;
use crate::components::cut_panel::CutPanel;
use crate::components::heading::Heading;
use crate::components::icon::{Icon, IconName};
use crate::components::{Class, classed};

/// A decision in the list, with an id to tell it apart.
#[derive(Clone, Debug, PartialEq)]
pub struct NeedsYouDecision {
    pub id: SharedString,
    pub decision: Decision,
}

type OnOverturn = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct NeedsYou {
    decisions: Vec<NeedsYouDecision>,
    note: Option<SharedString>,
    on_overturn: Option<OnOverturn>,
}

impl NeedsYou {
    /// The decisions, in the order given: the costliest to undo first.
    pub fn new(decisions: Vec<NeedsYouDecision>) -> Self {
        Self {
            decisions,
            note: None,
            on_overturn: None,
        }
    }

    /// Muted text beside the heading, such as "4, costliest to undo first"; the count when
    /// left out.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Called with a decision's id by its Overturn button.
    pub fn on_overturn(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overturn = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for NeedsYou {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.decisions.is_empty() {
            // Nothing: out of the layout, so the column keeps no gap for it.
            return div().hidden().into_any_element();
        }
        let note = self
            .note
            .unwrap_or_else(|| self.decisions.len().to_string().into());
        let title = AttentionText::new()
            .class_name(Class::NeedsTitle)
            .child(Icon::new(IconName::Notifications))
            .child("Needs you");
        let rows = self.decisions.into_iter().map(|n| {
            let row = DecisionRow::new(n.id.clone(), n.decision).inset(true);
            match self.on_overturn.clone() {
                Some(handler) => {
                    let id = n.id;
                    row.on_overturn(move |window, cx| handler(&id, window, cx))
                }
                None => row,
            }
        });
        CutPanel::new()
            .attention(true)
            .child(
                classed(Class::NeedsHead, cx).child(
                    Heading::new(title)
                        .note(note)
                        .class_name(Class::NeedsHeading),
                ),
            )
            .children(rows)
            .into_any_element()
    }
}
