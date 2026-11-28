//! RadioGroup and Radio (`blueprint-radiogroup`): options of which the person picks one, such
//! as the option to switch a decision to. The design system measured no look for a radio; its
//! circle and fill follow Blueprint's rules for `.bp6-control.bp6-radio` with the theme's
//! tokens (`docs/ui.md`).

use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::theme::{colour, shape, size, text};

/// One option: the value the group hands back, and its label.
#[derive(Clone, Debug, PartialEq)]
pub struct Radio {
    value: SharedString,
    label: SharedString,
    disabled: bool,
}

impl Radio {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

type OnChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct RadioGroup {
    id: SharedString,
    options: Vec<Radio>,
    selected_value: Option<SharedString>,
    label: Option<SharedString>,
    inline: bool,
    disabled: bool,
    on_change: Option<OnChange>,
}

impl RadioGroup {
    /// A group. `id` names it in the window: each radio is drawn as `{id}-option-{value}`.
    pub fn new(id: impl Into<SharedString>, options: Vec<Radio>) -> Self {
        Self {
            id: id.into(),
            options,
            selected_value: None,
            label: None,
            inline: false,
            disabled: false,
            on_change: None,
        }
    }

    /// The chosen option's value; none is chosen without it.
    pub fn selected_value(mut self, value: Option<SharedString>) -> Self {
        self.selected_value = value;
        self
    }

    /// A label over the options.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The options side by side.
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Called with an option's value when the person picks it.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (circle, gap) = size::control(cx);
        let dot = size::control_dot(cx);
        let ring = colour::control_ring(cx);
        let (fill, dot_colour, edge) = colour::control_checked(cx);
        let (off_fill, off_checked, off_dot) = colour::control_disabled(cx);
        let line = text::line_height(cx);
        let radios = self.options.into_iter().map(|radio| {
            let disabled = self.disabled || radio.disabled;
            let checked = self.selected_value.as_ref() == Some(&radio.value);
            // `.bp6-control .bp6-control-indicator`: a 16 px circle ringed by an inset line;
            // chosen, filled with the accent and a dot; disabled, a faint fill and no ring.
            let mut indicator = div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .size(circle)
                .rounded(circle / 2.);
            let line_width = shape::border(cx);
            indicator = match (checked, disabled) {
                (false, false) => indicator.border(line_width).border_color(ring),
                (true, false) => indicator.bg(fill).border(line_width).border_color(edge),
                (false, true) => indicator.bg(off_fill),
                (true, true) => indicator.bg(off_checked),
            };
            if checked {
                let colour = if disabled { off_dot } else { dot_colour };
                indicator = indicator.child(div().size(dot).rounded(dot / 2.).bg(colour));
            }
            let id = SharedString::from(format!("{}-option-{}", self.id, radio.value));
            let selector = id.to_string();
            let mut row = div()
                .id(ElementId::Name(id))
                .debug_selector(|| selector)
                .flex()
                .items_center()
                .gap(gap)
                .mb(gap)
                .line_height(line)
                .child(indicator)
                .child(radio.label);
            if self.inline {
                // `.bp6-control.bp6-inline { margin-inline-end: calc(var(--bp-surface-spacing) * 5) }`.
                row = row.mr(size::section_gap(cx));
            }
            if disabled {
                row = row.text_color(colour::disabled_text(cx));
            } else {
                row = row.cursor_pointer();
                if let Some(handler) = self.on_change.clone() {
                    let value = radio.value.clone();
                    row = row.on_click(move |_, window, cx| handler(&value, window, cx));
                }
            }
            row
        });
        let options = if self.inline {
            div().flex().flex_wrap().children(radios)
        } else {
            div().flex().flex_col().children(radios)
        };
        div()
            .flex()
            .flex_col()
            .children(
                self.label
                    .map(|label| div().mb(size::label_gap(cx)).child(label)),
            )
            .child(options)
    }
}
