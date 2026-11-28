//! TextArea (`blueprint-textarea`): a text field of lines that wrap, as tall as its text
//! (`autoResize`), in the measured look; with `tm-mono` its text is in the code face, as a
//! RefineDialog's proven-by is (`.tm-refine-row textarea.tm-mono`).

use gpui::prelude::FluentBuilder;
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};
use gpui_component::Sizable;
use gpui_component::input::Textarea;

use super::input_group::FieldState;
use super::{Class, box_of};
use crate::theme::look::{self, State};
use crate::theme::text;

#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    state: FieldState,
    fill: bool,
    disabled: bool,
    class_name: Option<Class>,
}

impl TextArea {
    /// A text area drawing `state`, which should be a field of lines.
    pub fn new(id: impl Into<ElementId>, state: &FieldState) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            fill: false,
            disabled: false,
            class_name: None,
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

    /// A class of the design system's own: `Mono` for text in the code face.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }
}

impl RenderOnce for TextArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focused = self.state.is_focused(window, cx);
        let state = if focused { State::Focus } else { State::Rest };
        let area = look::get(cx, "TextArea", "fill", state);
        let mono = self.class_name == Some(Class::Mono);
        let (family, size) = if mono {
            (
                text::code_family(cx),
                look::get(
                    cx,
                    "RefineDialog",
                    ".tm-refine [class*='tm-']: tm-mono tm-refine-row",
                    State::Rest,
                )
                .font_size,
            )
        } else {
            (text::family(cx), area.font_size)
        };
        let selector = format!("{}", self.id);
        // The library pads a field of lines by its medium size; the look's padding is drawn by
        // pulling the field out by the difference at each side.
        let library = gpui_component::Size::Medium;
        let pull_x = area.padding.left - library.input_px();
        let pull_y = area.padding.top - library.input_py();
        let field = match &self.state {
            FieldState::Lines(state) => Textarea::new(state).with_size(library),
            FieldState::Line(_) => {
                return div().id(self.id).into_any_element();
            }
        }
        .appearance(false)
        .disabled(self.disabled)
        .mx(pull_x)
        .my(pull_y)
        .font_family(family)
        .text_size(size)
        .text_color(area.text)
        .line_height(text::line_height(cx));
        box_of(div().id(self.id), &area)
            .debug_selector(|| selector)
            .when(self.fill, |el| el.w_full())
            .p(gpui::Pixels::ZERO)
            .child(field)
            .into_any_element()
    }
}
