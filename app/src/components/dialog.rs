//! Dialog (`blueprint-dialog`): a modal window over the page, for a task that needs the
//! person's full attention, such as editing an issue's plan. A header with its icon, title and
//! close button, the body, and a footer with its actions at the right, over the backdrop. It
//! closes on its close button, on Escape and on a click on the backdrop, through `on_close`.

use std::rc::Rc;

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels, Point,
    RenderOnce, SharedString, Styled, Window, anchored, deferred, div,
};

use super::button::{Button, Variant};
use super::icon::{Icon, IconName};
use super::{Class, box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{colour, size, space, text};

type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;

/// Whether the dialog was open when last drawn, and the focus it takes on opening, so Escape
/// reaches it before the person clicks into it.
struct Opened {
    open: bool,
    focus: FocusHandle,
}

#[derive(IntoElement)]
pub struct Dialog {
    id: SharedString,
    is_open: bool,
    title: Option<SharedString>,
    icon: Option<IconName>,
    class_name: Option<Class>,
    on_close: Option<OnClose>,
    actions: Vec<AnyElement>,
    children: Vec<AnyElement>,
}

impl Dialog {
    /// A dialog. `id` names it in the window; its close button is drawn as `{id}-close`.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            title: None,
            icon: None,
            class_name: None,
            on_close: None,
            actions: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// The title, in the header.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// An icon before the title.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A class of the design system's own: `Refine` makes it RefineDialog's wide dialog.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// Called by the close button, Escape and a click on the backdrop.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    /// The footer's buttons, at its right, as Blueprint's `DialogFooter` takes `actions`.
    /// Without them there is no footer.
    pub fn actions(mut self, actions: Vec<AnyElement>) -> Self {
        self.actions = actions;
        self
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let opened = window.use_keyed_state(
            SharedString::from(format!("{}-opened", self.id)),
            cx,
            |_, cx| Opened {
                open: false,
                focus: cx.focus_handle(),
            },
        );
        let was_open = opened.read(cx).open;
        let focus = opened.read(cx).focus.clone();
        if was_open != self.is_open {
            opened.update(cx, |o, _| o.open = self.is_open);
            if self.is_open {
                let focus = focus.clone();
                window.defer(cx, move |window, cx| window.focus(&focus, cx));
            }
        }
        if !self.is_open {
            return div().hidden().into_any_element();
        }
        let rest = State::Rest;
        let refine = self.class_name == Some(Class::Refine);
        let card = if refine {
            look::get(cx, "RefineDialog", "tm-refine: tm-refine", rest)
        } else {
            look::get(cx, "Dialog", "dialog: default", rest)
        };
        let header = look::get(cx, "Dialog", "dialog-header: default", rest);
        let footer = look::get(cx, "Dialog", "dialog-footer: default", rest);
        let screen = window.viewport_size();
        // `.tm-refine { width: min(var(--tm-dialog-wide), calc(100vw - 2 * var(--tm-space-6))) }`.
        let width = if refine {
            size::dialog_wide(cx).min(screen.width - space::s6(cx) * 2.)
        } else {
            size::dialog(cx)
        };

        let close = Button::new(SharedString::from(format!("{}-close", self.id)))
            .variant(Variant::Minimal)
            .icon(IconName::X);
        let close = match self.on_close.clone() {
            Some(handler) => close.on_click(move |_, window, cx| handler(window, cx)),
            None => close,
        };
        // `.bp6-dialog-header .bp6-icon { margin-left: calc(var(--bp-surface-spacing) * -1);
        // margin-right: calc(var(--bp-surface-spacing) * 4 * 0.5) }`, in the muted colour.
        let icon = self.icon.map(|icon| {
            div()
                .flex()
                .flex_none()
                .ml(-size::label_gap(cx))
                .mr(size::label_gap(cx) * 2.)
                .child(Icon::new(icon).color(colour::muted(cx)))
        });
        // `.bp6-dialog-header .bp6-heading { font-size: 14px; line-height: inherit }`, a
        // Blueprint heading's bold weight; cut with an ellipsis.
        let title = self.title.map(|title| {
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_size(text::section(cx))
                .font_weight(text::bold_weight(cx))
                .child(title)
        });
        let head = box_of(type_of(div(), &header), &header)
            .flex()
            .flex_none()
            .items_center()
            .min_h(header.height)
            .children(icon)
            .children(title)
            .child(close);
        // `border-radius: var(--bp-surface-border-radius) var(--bp-surface-border-radius) 0 0`.
        let mut head = head;
        let corners = &mut head.style().corner_radii;
        corners.bottom_left = Some(Pixels::ZERO.into());
        corners.bottom_right = Some(Pixels::ZERO.into());
        let body = div()
            .flex_1()
            .m(size::form_gap(cx))
            .children(self.children);
        let foot = (!self.actions.is_empty()).then(|| {
            box_of(type_of(div(), &footer), &footer)
                .flex()
                .justify_end()
                .items_center()
                .gap(size::alert_button_gap(cx))
                .children(self.actions)
        });
        let card = box_of(type_of(div().id(self.id.clone()), &card), &card)
            .track_focus(&focus)
            .occlude()
            .flex()
            .flex_col()
            .w(width)
            .line_height(text::line_height(cx))
            .child(head)
            .child(body)
            .children(foot);

        let on_close = self.on_close;
        let on_escape = on_close.clone();
        let mut backdrop = div()
            .id(SharedString::from(format!("{}-backdrop", self.id)))
            .occlude()
            .w(screen.width)
            .h(screen.height)
            .flex()
            .items_center()
            .justify_center()
            .bg(colour::backdrop(cx))
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape"
                    && let Some(handler) = on_escape.as_ref()
                {
                    cx.stop_propagation();
                    handler(window, cx);
                }
            })
            .child(card);
        if let Some(handler) = on_close {
            // A click on the backdrop, not on the dialog, which occludes it.
            backdrop = backdrop.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                handler(window, cx)
            });
        }
        deferred(anchored().position(Point::default()).child(backdrop))
            .with_priority(1)
            .into_any_element()
    }
}
