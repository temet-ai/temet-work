//! InputGroup (`temet-inputgroup`): a single-line field, as Blueprint's InputGroup without
//! `round`, such as an issue's title in RefineDialog or a search. Its text is edited by
//! gpui-component's input, held in the window between frames under the field's id; its box is
//! the design system's.

use std::rc::Rc;

use gpui::{
    App, Focusable, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{Sizable, Size as LibrarySize};

use super::icon::{Icon, IconName};
use super::text_area::{Field, OnChange, field_box};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size, text};

/// The field's size: medium, or small, as a filter in a panel's header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Small,
    #[default]
    Medium,
}

#[derive(IntoElement)]
pub struct InputGroup {
    id: ElementId,
    value: Option<SharedString>,
    default_value: SharedString,
    placeholder: Option<SharedString>,
    left_icon: Option<IconName>,
    size: Size,
    intent: Intent,
    fill: bool,
    disabled: bool,
    on_change: Option<OnChange>,
}

impl InputGroup {
    /// A field. `id` is its identity in the window, which keeps its text and caret between
    /// frames.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: SharedString::default(),
            placeholder: None,
            left_icon: None,
            size: Size::Medium,
            intent: Intent::None,
            fill: false,
            disabled: false,
            on_change: None,
        }
    }

    /// The text, kept by the caller from `on_change`: when it differs from the field's, the
    /// field shows it.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The text the field starts with, when the caller does not keep it.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// An icon inside the field at its left, such as Search.
    pub fn left_icon(mut self, icon: IconName) -> Self {
        self.left_icon = Some(icon);
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// The danger intent outlines a value that cannot be used; the others draw as none.
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Takes the full width of its container.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Called with the text each time the person changes it.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = self
            .value
            .or(Some(self.default_value).filter(|v| !v.is_empty()));
        let placeholder = self.placeholder.clone();
        let input: Entity<InputState> = Field::keyed(
            self.id.clone(),
            value,
            self.on_change,
            window,
            cx,
            move |window, cx| {
                let state = InputState::new(window, cx);
                match placeholder {
                    Some(p) => state.placeholder(p),
                    None => state,
                }
            },
        );
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let state = if self.disabled {
            State::Disabled
        } else if focused {
            State::Focus
        } else {
            State::Rest
        };
        // The small field was measured with its left icon, the danger one at medium; the
        // disabled look is the default field's.
        let look = match (self.size, self.intent, state) {
            (_, _, State::Disabled) => look::get(cx, "InputGroup", "default", State::Disabled),
            (Size::Small, _, state) => look::get(cx, "InputGroup", "small", state),
            (Size::Medium, Intent::Danger, state) => {
                look::get(cx, "InputGroup", "intent-danger", state)
            }
            (Size::Medium, _, state) => look::get(cx, "InputGroup", "default", state),
        };
        let mut look = look;
        // The text's inset from the left: past the icon with one, as the sides without.
        look.padding.left = match (self.left_icon.is_some(), self.size) {
            (false, _) => look.padding.right,
            (true, Size::Small) => look.padding.left,
            (true, Size::Medium) => size::input_icon_pad(cx),
        };
        let (_, top) = super::text_area::inner_pad();
        let line = look.height - look.padding.top - look.padding.bottom - top - top;
        let field = Input::new(&input)
            .appearance(false)
            .with_size(LibrarySize::XSmall)
            .disabled(self.disabled)
            .h(line)
            .text_color(look.text)
            .text_size(look.font_size)
            .font_family(text::family(cx));
        // The icon sits in the middle of the inset it makes: `.bp6-input-group > .bp6-icon
        // { margin: 7px }`, 4 px on a small field.
        let icon = self.left_icon.map(|icon| {
            let inset = (look.padding.left - size::icon(cx)) / 2.;
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(inset)
                .flex()
                .items_center()
                .child(Icon::new(icon).color(colour::muted(cx)))
        });
        let selector = format!("{}", self.id);
        let mut el = field_box(div().id(self.id), &look)
            .debug_selector(|| selector)
            .relative()
            .flex()
            .items_center()
            .h(look.height)
            .min_w_0()
            .children(icon)
            .child(field);
        if self.fill {
            el = el.w_full();
        }
        el
    }
}
