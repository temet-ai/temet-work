//! InputGroup (`temet-inputgroup`): a one-line text field, as the design system's `InputGroup`
//! draws Blueprint's: a sunken field in the measured look, ringed in the focus colour while the
//! person types in it.
//!
//! This file also holds [`FieldState`]: the text a field holds while the person edits it, kept
//! by whoever owns the draft (a page) across frames. An InputGroup or a TextArea draws it.

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, ElementId, Entity, Focusable, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div,
};
use gpui_component::input::{Input, InputState, Textarea, TextareaState};

use super::{box_of, type_of};
use crate::theme::look::{self, Look, State};
use crate::theme::text;

/// The most lines a text area grows to before it scrolls.
const MOST_LINES: usize = 12;

/// The text of one field, as the person edits it. Cloning it gives another handle on the same
/// text.
#[derive(Clone)]
pub enum FieldState {
    /// One line, for an InputGroup.
    Line(Entity<InputState>),
    /// Lines that wrap, growing with the text, for a TextArea.
    Lines(Entity<TextareaState>),
}

impl FieldState {
    /// A one-line field holding `value`.
    pub fn line(value: impl Into<SharedString>, window: &mut Window, cx: &mut App) -> Self {
        let value = value.into();
        Self::Line(cx.new(|cx| InputState::new(window, cx).default_value(value)))
    }

    /// A field of lines that wrap, holding `value`; it is as tall as its text.
    pub fn lines(value: impl Into<SharedString>, window: &mut Window, cx: &mut App) -> Self {
        let value = value.into();
        Self::Lines(cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, MOST_LINES)
                .default_value(value)
        }))
    }

    /// The text the field holds now.
    pub fn value(&self, cx: &App) -> String {
        match self {
            Self::Line(state) => state.read(cx).value().to_string(),
            Self::Lines(state) => state.read(cx).value().to_string(),
        }
    }

    /// Replaces the text, as the person would by selecting it all and typing.
    pub fn set_value(&self, value: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
        let value: SharedString = value.into();
        match self {
            Self::Line(state) => state.update(cx, |s, cx| s.set_value(value, window, cx)),
            Self::Lines(state) => state.update(cx, |s, cx| s.set_value(value, window, cx)),
        }
    }

    /// Puts the person's typing in the field.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        match self {
            Self::Line(state) => state.update(cx, |s, cx| s.focus(window, cx)),
            Self::Lines(state) => state.update(cx, |s, cx| s.focus(window, cx)),
        }
    }

    /// Whether the person's typing goes to the field.
    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        match self {
            Self::Line(state) => state.read(cx).focus_handle(cx).is_focused(window),
            Self::Lines(state) => state.read(cx).focus_handle(cx).is_focused(window),
        }
    }
}

/// The library's field drawing its text as `field` sets it: one line of the look's height, with
/// the look's padding at each side.
fn field_text<E: Styled>(el: E, field: &Look, cx: &App) -> E {
    el.h(field.height)
        .pl(field.padding.left)
        .pr(field.padding.right)
        .text_size(field.font_size)
        .text_color(field.text)
        .line_height(text::line_height(cx))
}

#[derive(IntoElement)]
pub struct InputGroup {
    id: ElementId,
    state: FieldState,
    fill: bool,
    disabled: bool,
}

impl InputGroup {
    /// A field drawing `state`. `id` is its identity in the window, as the story's `id`.
    pub fn new(id: impl Into<ElementId>, state: &FieldState) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            fill: false,
            disabled: false,
        }
    }

    /// Takes the width of its container.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = if self.disabled {
            State::Disabled
        } else if self.state.is_focused(window, cx) {
            State::Focus
        } else {
            State::Rest
        };
        let field = look::get(cx, "InputGroup", "default", state);
        let selector = format!("{}", self.id);
        // The look's box holds the field, which draws the text with the look's padding.
        let input = match &self.state {
            FieldState::Line(state) => field_text(
                Input::new(state).appearance(false).disabled(self.disabled),
                &field,
                cx,
            )
            .into_any_element(),
            FieldState::Lines(state) => field_text(
                Textarea::new(state)
                    .appearance(false)
                    .disabled(self.disabled),
                &field,
                cx,
            )
            .into_any_element(),
        };
        box_of(type_of(div().id(self.id), &field), &field)
            .debug_selector(|| selector)
            .when(self.fill, |el| el.w_full())
            .pl(gpui::Pixels::ZERO)
            .pr(gpui::Pixels::ZERO)
            .h(field.height)
            .flex()
            .items_center()
            .child(input)
    }
}
