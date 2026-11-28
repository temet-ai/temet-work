//! Button (`blueprint-button`): an action, as text, an icon, or both, in a variant, an intent
//! and a size. A disabled button takes no click; a loading one is disabled and draws a Spinner
//! over its content, which keeps its place.

use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, ClickEvent, CursorStyle, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::icon::{Icon, IconName};
use super::spinner::Spinner;
use super::{Class, apply, box_of, type_of};
use crate::theme::look::{self, Look, State};
use crate::theme::{Intent, colour, size};

/// How the button is drawn: a filled button, text only, or an outline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Solid,
    Minimal,
    Outlined,
}

/// The button's size: medium, or small, as a check's plus and minus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Small,
    #[default]
    Medium,
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    text: Option<SharedString>,
    icon: Option<IconName>,
    intent: Intent,
    variant: Variant,
    size: Size,
    class_name: Option<Class>,
    disabled: bool,
    loading: bool,
    focus: Option<FocusHandle>,
    on_click: Option<OnClick>,
}

impl Button {
    /// A button. `id` is its identity in the window, as React's `key`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            text: None,
            icon: None,
            intent: Intent::None,
            variant: Variant::Solid,
            size: Size::Medium,
            class_name: None,
            disabled: false,
            loading: false,
            focus: None,
            on_click: None,
        }
    }

    /// The action, such as Overturn.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// A class of the design system's own, such as `CheckToggleNone` on the hidden button that
    /// holds a check's place.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// A disabled button is drawn muted and takes no click.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A loading button is disabled, and a Spinner turns over its content, as while the
    /// button's action is sent.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// The focus the button holds when the person moves to it, so the focus can be put back on
    /// it, as on the button that opened a menu once the dialog its item opened has closed.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
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

/// The button's measured look in `state`. A small button has the small button's height,
/// padding and type (`.bp6-small`) and its variant's colours.
fn button_look(cx: &App, variant: Variant, intent: Intent, size: Size, state: State) -> Look {
    let look = variant_look(cx, variant, intent, state);
    match size {
        Size::Medium => look,
        Size::Small => {
            let small = look::get(cx, "Button", "small", state);
            Look {
                height: small.height,
                padding: small.padding,
                font_size: small.font_size,
                ..look
            }
        }
    }
}

/// A disabled button's look: the primary button's measured disabled look; any other solid
/// button in Blueprint's disabled fill, and a minimal or outlined one with no fill, each with
/// the disabled word and no shadow (`.bp6-button:not([class*=bp6-intent-]):disabled`,
/// `.bp6-button.bp6-minimal:disabled`).
fn disabled_look(cx: &App, variant: Variant, intent: Intent, size: Size) -> Look {
    if variant == Variant::Solid && intent == Intent::Primary {
        return button_look(cx, variant, intent, size, State::Disabled);
    }
    let rest = button_look(cx, variant, intent, size, State::Rest);
    let fill = match variant {
        Variant::Solid => colour::disabled_fill(cx),
        Variant::Minimal | Variant::Outlined => gpui::transparent_black(),
    };
    Look {
        background: fill,
        text: colour::disabled_text(cx),
        shadow: Vec::new(),
        ..rest
    }
}

/// The medium button's measured look in a variant, intent and state.
fn variant_look(cx: &App, variant: Variant, intent: Intent, state: State) -> Look {
    match (variant, intent) {
        (Variant::Solid, Intent::None) => look::get(cx, "Button", "default", state),
        (Variant::Solid, Intent::Primary) => look::get(cx, "Button", "intent-primary", state),
        (Variant::Solid, Intent::Success) => look::get(cx, "Button", "intent-success", state),
        (Variant::Solid, Intent::Warning) => look::get(cx, "Button", "intent-warning", state),
        (Variant::Solid, Intent::Danger) => look::get(cx, "Button", "intent-danger", state),
        (Variant::Minimal, Intent::None) => look::get(cx, "Button", "minimal", state),
        (Variant::Minimal, Intent::Primary) => {
            look::get(cx, "Button", "intent-primary minimal", state)
        }
        (Variant::Minimal, Intent::Success) => {
            look::get(cx, "Button", "intent-success minimal", state)
        }
        (Variant::Minimal, Intent::Warning) => {
            look::get(cx, "Button", "intent-warning minimal", state)
        }
        (Variant::Minimal, Intent::Danger) => {
            look::get(cx, "Button", "intent-danger minimal", state)
        }
        (Variant::Outlined, Intent::None) => look::get(cx, "Button", "outlined", state),
        (Variant::Outlined, Intent::Primary) => {
            look::get(cx, "Button", "intent-primary outlined", state)
        }
        (Variant::Outlined, Intent::Success) => {
            look::get(cx, "Button", "intent-success outlined", state)
        }
        (Variant::Outlined, Intent::Warning) => {
            look::get(cx, "Button", "intent-warning outlined", state)
        }
        (Variant::Outlined, Intent::Danger) => {
            look::get(cx, "Button", "intent-danger outlined", state)
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled || self.loading;
        let rest = if disabled {
            disabled_look(cx, self.variant, self.intent, self.size)
        } else {
            button_look(cx, self.variant, self.intent, self.size, State::Rest)
        };
        let hover = button_look(cx, self.variant, self.intent, self.size, State::Hover);
        let active = button_look(cx, self.variant, self.intent, self.size, State::Active);
        let selector = format!("{}", self.id);
        let icon_only = self.text.is_none();
        // `.bp6-button .bp6-icon` is muted; an intent's button draws its icon in its text colour.
        let intent = self.intent;
        let icon = self.icon.map(|icon| match intent {
            _ if disabled => Icon::new(icon),
            Intent::None => Icon::new(icon).color(colour::muted(cx)),
            _ => Icon::new(icon),
        });
        // `.bp6-button.bp6-loading > :not(.bp6-button-spinner) { visibility: hidden }`: the
        // content keeps the button's width under the spinner.
        let content = div()
            .flex()
            .items_center()
            .gap(size::tag_gap(cx))
            .when(self.loading, |content| content.invisible())
            .children(icon)
            .children(self.text);
        let mut button = box_of(type_of(div().id(self.id), &rest), &rest)
            .debug_selector(|| selector)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(rest.height)
            .min_w(rest.height)
            .child(content);
        if self.loading {
            button = button.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Spinner::new().size(size::spinner_small(cx))),
            );
        }
        if icon_only {
            // An icon alone sits in a square button, `.bp6-button:empty`.
            button = button.px(gpui::Pixels::ZERO);
        }
        if let Some(class) = self.class_name {
            button = apply(button, class, cx);
        }
        if let Some(focus) = &self.focus {
            button = button.track_focus(focus);
        }
        if disabled {
            return button.cursor(CursorStyle::OperationNotAllowed);
        }
        button = button
            .cursor_pointer()
            .hover(move |style| style.bg(hover.background).text_color(hover.text))
            .active(move |style| style.bg(active.background));
        if let Some(handler) = self.on_click {
            button = button.on_click(move |event, window, cx| handler(event, window, cx));
        }
        button
    }
}
