//! AttentionButton (`temet-attentionbutton`): an outlined button in the attention colour, for
//! the action on something that asks for the person, such as Overturn on an agent's decision.

use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::{box_of, type_of};
use crate::theme::look::{self, State};

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The small outlined button, the size temet-work uses and the design system measured.
#[derive(IntoElement)]
pub struct AttentionButton {
    id: ElementId,
    text: SharedString,
    on_click: Option<OnClick>,
}

impl AttentionButton {
    /// A button. `id` is its identity in the window, as React's `key`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            text: SharedString::default(),
            on_click: None,
        }
    }

    /// The action, such as Overturn.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AttentionButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = look::get(cx, "AttentionButton", "outlined small", State::Rest);
        let hover = look::get(cx, "AttentionButton", "outlined small", State::Hover);
        let active = look::get(cx, "AttentionButton", "outlined small", State::Active);
        let selector = format!("{}", self.id);
        let mut button = box_of(type_of(div().id(self.id), &rest), &rest)
            .debug_selector(|| selector)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(rest.height)
            .cursor_pointer()
            .hover(move |style| style.bg(hover.background))
            .active(move |style| style.bg(active.background))
            .child(self.text);
        if let Some(handler) = self.on_click {
            button = button.on_click(move |event, window, cx| handler(event, window, cx));
        }
        button
    }
}
