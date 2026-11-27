//! SegmentedControl (`blueprint-segmentedcontrol`): a row of options of which one is chosen,
//! such as Group by Milestone, Stage or Group.

use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size};

/// One option: what it says and the value it stands for.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentedControlOption {
    pub label: SharedString,
    pub value: SharedString,
}

type OnValueChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// The small control in the primary intent, as the design system's page header uses it.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    options: Vec<SegmentedControlOption>,
    value: Option<SharedString>,
    intent: Intent,
    on_value_change: Option<OnValueChange>,
}

impl SegmentedControl {
    /// A control. `id` is its identity in the window, as React's `key`.
    pub fn new(id: impl Into<ElementId>, options: Vec<SegmentedControlOption>) -> Self {
        Self {
            id: id.into(),
            options,
            value: None,
            intent: Intent::None,
            on_value_change: None,
        }
    }

    /// The chosen option's value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The chosen option's colour.
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn on_value_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = State::Rest;
        // The chosen option is a small button; the others are minimal small buttons.
        let chosen = look::get(cx, "SegmentedControl", "small", rest);
        let other = look::get(cx, "SegmentedControl", "minimal small", rest);
        let other_hover = look::get(cx, "SegmentedControl", "minimal small", State::Hover);
        // In an intent the chosen button takes the intent's solid button colours.
        let primary = look::get(cx, "Button", "intent-primary", rest);
        let options = self.options.into_iter().enumerate().map(|(i, option)| {
            let on = self.value.as_ref() == Some(&option.value);
            let look = if on { &chosen } else { &other };
            let selector = format!("option-{}", option.value);
            let mut button = box_of(type_of(div().id(("option", i)), look), look)
                .debug_selector(|| selector)
                .flex()
                .items_center()
                .h(look.height)
                .child(option.label);
            if on && self.intent == Intent::Primary {
                button = button.bg(primary.background).text_color(primary.text);
            }
            if !on {
                let hover = other_hover.background;
                button = button.cursor_pointer().hover(move |style| style.bg(hover));
                if let Some(handler) = self.on_value_change.clone() {
                    let value = option.value.clone();
                    button = button.on_click(move |_, window, cx| handler(&value, window, cx));
                }
            }
            button
        });
        // The ground behind the buttons has no measured look: Blueprint's
        // `.bp6-segmented-control`, a 2 px inset and gap on the theme's ground.
        div()
            .id(self.id)
            .flex()
            .flex_none()
            .p(size::segmented_inset(cx))
            .gap(size::segmented_inset(cx))
            .rounded(chosen.radius)
            .bg(colour::segmented_ground(cx))
            .children(options)
    }
}
