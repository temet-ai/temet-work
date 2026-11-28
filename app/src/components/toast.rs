//! Toast and OverlayToaster (`blueprint-toast`): a short message at the window's bottom right,
//! with an icon in its intent's colour and a close button, such as "Overturned on WATCH-51:
//! ...". It stays until it is closed: a timeout would need a duration the design system's
//! tokens do not have.

use std::rc::Rc;

use gpui::{
    App, IntoElement, ParentElement, Point, RenderOnce, SharedString, Styled, Window, anchored,
    deferred, div,
};

use super::button::{Button, Variant};
use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size};

type OnDismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// One toast: its message, icon and close button.
#[derive(IntoElement)]
pub struct Toast {
    message: SharedString,
    icon: Option<IconName>,
    intent: Intent,
    on_dismiss: Option<OnDismiss>,
}

impl Toast {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            icon: None,
            intent: Intent::None,
            on_dismiss: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The icon's colour.
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Called by the close button.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Every intent's toast was measured with the same ground, text and corner.
        let toast = match self.intent {
            Intent::Success => look::get(cx, "Toast", "intent-success", State::Rest),
            Intent::Warning => look::get(cx, "Toast", "intent-warning", State::Rest),
            Intent::Danger => look::get(cx, "Toast", "intent-danger", State::Rest),
            Intent::Primary => look::get(cx, "Toast", "intent-primary", State::Rest),
            Intent::None => {
                // No toast without an intent was measured. It has the intent toasts' ground and
                // text, and the plain ring of the theme's overlays, measured on the Dialog: in
                // the light theme an intent toast's ring takes the intent's colour.
                let mut plain = look::get(cx, "Toast", "intent-primary", State::Rest);
                plain.shadow = look::get(cx, "Dialog", "dialog: default", State::Rest).shadow;
                plain
            }
        };
        let (narrowest, widest) = size::toast_width(cx);
        let mut close = Button::new("toast-close")
            .variant(Variant::Minimal)
            .icon(IconName::X);
        if let Some(handler) = self.on_dismiss {
            close = close.on_click(move |_, window, cx| handler(window, cx));
        }
        // The story's close button has no fill but keeps a plain button's edge, the inset ring;
        // no look was measured for it, so the ring is the plain button's look's.
        let ring = look::get(cx, "Button", "default", State::Rest);
        let close = div().rounded(ring.radius).shadow(ring.shadow).child(close);
        // `.bp6-toast > .bp6-icon { margin: 12px; margin-right: 0 }`, in the intent's colour, or
        // muted without one.
        let tint = match self.intent {
            Intent::None => colour::muted(cx),
            intent => colour::intent_text(cx, intent),
        };
        let icon = self.icon.map(|icon| {
            div()
                .flex()
                .m(size::alert_footer_gap(cx))
                .mr(gpui::Pixels::ZERO)
                .child(Icon::new(icon).color(tint))
        });
        box_of(type_of(div(), &toast), &toast)
            .flex()
            .items_start()
            .min_h(toast.height)
            .min_w(narrowest)
            .max_w(widest)
            .children(icon)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .p(size::toast_message_pad(cx))
                    .child(self.message),
            )
            .child(
                div()
                    .flex_none()
                    .p(size::toast_button_pad(cx))
                    .pl(gpui::Pixels::ZERO)
                    .child(close),
            )
    }
}

/// Where toasts show: the window's bottom right (`position: bottom-right`), the newest at the
/// bottom.
#[derive(IntoElement, Default)]
pub struct OverlayToaster {
    toasts: Vec<Toast>,
}

impl OverlayToaster {
    pub fn new() -> Self {
        Self::default()
    }

    /// A toast on show.
    pub fn child(mut self, toast: Toast) -> Self {
        self.toasts.push(toast);
        self
    }
}

impl RenderOnce for OverlayToaster {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.toasts.is_empty() {
            return div().hidden().into_any_element();
        }
        let screen = window.viewport_size();
        let pad = size::overlay_pad(cx);
        deferred(
            anchored().position(Point::default()).child(
                div()
                    .w(screen.width)
                    .h(screen.height)
                    .flex()
                    .flex_col()
                    .justify_end()
                    .items_end()
                    .gap(pad)
                    .px(pad)
                    .pb(pad)
                    .children(self.toasts),
            ),
        )
        .with_priority(2)
        .into_any_element()
    }
}
