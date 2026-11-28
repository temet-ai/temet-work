//! Alert (`blueprint-alert`): a modal question with a confirm and a cancel button, for an action
//! that is hard to undo, such as Overturn. It covers the window with the backdrop and centres the
//! question over it.

use std::rc::Rc;

use gpui::{
    AnyElement, App, InteractiveElement, IntoElement, ParentElement, Point, RenderOnce,
    SharedString, Styled, Window, anchored, deferred, div,
};

use super::button::Button;
use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size};

type OnAnswer = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Alert {
    is_open: bool,
    icon: Option<IconName>,
    intent: Intent,
    cancel_button_text: Option<SharedString>,
    confirm_button_text: SharedString,
    on_cancel: Option<OnAnswer>,
    on_confirm: Option<OnAnswer>,
    children: Vec<AnyElement>,
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Alert {
    pub fn new() -> Self {
        Self {
            is_open: false,
            icon: None,
            intent: Intent::None,
            cancel_button_text: None,
            confirm_button_text: "OK".into(),
            on_cancel: None,
            on_confirm: None,
            children: Vec::new(),
        }
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// An icon beside the question, at the large size.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The confirm button's intent.
    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// The cancel button's text; without it there is no cancel button.
    pub fn cancel_button_text(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_button_text = Some(text.into());
        self
    }

    pub fn confirm_button_text(mut self, text: impl Into<SharedString>) -> Self {
        self.confirm_button_text = text.into();
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }

    pub fn on_confirm(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_confirm = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Alert {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Alert {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().hidden().into_any_element();
        }
        // The Alert is a Blueprint dialog: its box is the measured dialog's; its padding, width
        // and footer follow `.bp6-alert`, which has no look of its own.
        let dialog = look::get(cx, "Dialog", "dialog: default", State::Rest);
        let confirm = {
            let mut b = Button::new("alert-confirm")
                .text(self.confirm_button_text)
                .intent(self.intent);
            if let Some(handler) = self.on_confirm {
                b = b.on_click(move |_, window, cx| handler(window, cx));
            }
            b
        };
        let cancel = self.cancel_button_text.map(|text| {
            let mut b = Button::new("alert-cancel").text(text);
            if let Some(handler) = self.on_cancel {
                b = b.on_click(move |_, window, cx| handler(window, cx));
            }
            b
        });
        // The question is a paragraph, which keeps Blueprint's paragraph margin under it.
        let body = div()
            .flex()
            .gap(size::overlay_pad(cx))
            .children(self.icon.map(|icon| {
                Icon::new(icon)
                    .large_at(size::alert_icon(cx))
                    .color(colour::intent_text(cx, self.intent))
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pb(size::paragraph_gap(cx))
                    .children(self.children),
            );
        let footer = div()
            .flex()
            .flex_row_reverse()
            .gap(size::alert_button_gap(cx))
            .mt(size::alert_footer_gap(cx))
            .child(confirm)
            .children(cancel);
        let card = box_of(type_of(div().id("alert"), &dialog), &dialog)
            .occlude()
            .p(size::overlay_pad(cx))
            .max_w(size::alert_width(cx))
            .child(body)
            .child(footer);
        let screen = window.viewport_size();
        deferred(
            anchored().position(Point::default()).child(
                div()
                    .id("alert-backdrop")
                    .occlude()
                    .w(screen.width)
                    .h(screen.height)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(colour::backdrop(cx))
                    .child(card),
            ),
        )
        .with_priority(1)
        .into_any_element()
    }
}
