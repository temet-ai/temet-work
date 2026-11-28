//! Dialog, DialogBody and DialogFooter (`blueprint-dialog`): a modal window over the page, with
//! a header holding its icon, its title and a close button, a body that scrolls when it is
//! taller than the window allows, and a footer with its actions at the end. The backdrop shades
//! the page; a click on it, Escape or the close button closes the dialog.

use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AbsoluteLength, AnyElement, App, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, Point, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, anchored, deferred, div,
};

use super::button::{Button, Variant};
use super::icon::{Icon, IconName};
use super::{Class, box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{colour, size, text};

type OnClose = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Dialog {
    is_open: bool,
    title: SharedString,
    icon: Option<IconName>,
    class_name: Option<Class>,
    focus: Option<FocusHandle>,
    on_close: Option<OnClose>,
    children: Vec<AnyElement>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            title: SharedString::default(),
            icon: None,
            class_name: None,
            focus: None,
            on_close: None,
            children: Vec::new(),
        }
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// The title in the header, such as "Refine WATCH-12".
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// The icon before the title.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A class of the design system's own: `Refine` for RefineDialog's width and look.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// The focus the dialog holds while it is open, so Escape reaches it.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Called by the close button, Escape and a click on the backdrop.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
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
        if !self.is_open {
            return div().hidden().into_any_element();
        }
        let screen = window.viewport_size();
        let refine = self.class_name == Some(Class::Refine);
        let card = if refine {
            look::get(cx, "RefineDialog", "tm-refine: tm-refine", State::Rest)
        } else {
            look::get(cx, "Dialog", "dialog: default", State::Rest)
        };
        let head = look::get(cx, "Dialog", "dialog-header: default", State::Rest);
        let (pull, after) = size::dialog_icon(cx);
        let close = {
            let mut b = Button::new("dialog-close")
                .icon(IconName::X)
                .variant(Variant::Minimal);
            if let Some(handler) = self.on_close.clone() {
                b = b.on_click(move |_, window, cx| handler(window, cx));
            }
            b
        };
        let header = box_of(type_of(div(), &head), &head)
            .flex()
            .flex_none()
            .items_center()
            .min_h(size::dialog_header(cx))
            .children(self.icon.map(|icon| {
                div()
                    .flex_none()
                    .ml(-pull)
                    .mr(after)
                    .child(Icon::new(icon).color(colour::muted(cx)))
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_size(text::section(cx))
                    .font_weight(text::heading_weight(cx))
                    .child(self.title),
            )
            .child(close);
        // Rounded at the top only, as the dialog's own corners are.
        let mut header = header;
        let square = Some(AbsoluteLength::Pixels(Pixels::ZERO));
        header.style().corner_radii.bottom_left = square;
        header.style().corner_radii.bottom_right = square;
        let on_key = self.on_close.clone();
        let mut dialog = box_of(type_of(div().id("dialog"), &card), &card)
            .occlude()
            .flex()
            .flex_col()
            .max_h(screen.height - size::dialog_margin(cx) * 2.)
            .line_height(text::line_height(cx))
            .when(refine, |d| d.w(size::dialog_wide(cx, screen.width)))
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape"
                    && let Some(handler) = &on_key
                {
                    handler(window, cx);
                }
            })
            .child(header)
            .children(self.children);
        if let Some(focus) = &self.focus {
            dialog = dialog.track_focus(focus);
        }
        let on_backdrop = self.on_close;
        deferred(
            anchored().position(Point::default()).child(
                div()
                    .id("dialog-backdrop")
                    .occlude()
                    .w(screen.width)
                    .h(screen.height)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(colour::backdrop(cx))
                    .on_click(move |_, window, cx| {
                        if let Some(handler) = &on_backdrop {
                            handler(window, cx);
                        }
                    })
                    .child(dialog),
            ),
        )
        .with_priority(1)
        .into_any_element()
    }
}

/// A dialog's body: its fields, with Blueprint's margin around them, scrolling once it is taller
/// than the dialog's room (`.bp6-dialog-body-scroll-container { max-height: 70vh; overflow: auto;
/// padding: calc(var(--bp-surface-spacing) * 4) }`).
#[derive(IntoElement)]
pub struct DialogBody {
    children: Vec<AnyElement>,
}

impl Default for DialogBody {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogBody {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ParentElement for DialogBody {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for DialogBody {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id("dialog-body")
            .flex_1()
            .min_h_0()
            .max_h(size::dialog_body_max(window.viewport_size().height))
            .overflow_y_scroll()
            .p(size::dialog_body(cx))
            .children(self.children)
    }
}

/// A dialog's footer: its actions at the end, on the footer's measured ground under a rule.
#[derive(IntoElement)]
pub struct DialogFooter {
    actions: Vec<AnyElement>,
}

impl Default for DialogFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogFooter {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// The buttons at the end, in order.
    pub fn actions(mut self, actions: impl IntoIterator<Item = AnyElement>) -> Self {
        self.actions.extend(actions);
        self
    }
}

impl RenderOnce for DialogFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let foot = look::get(cx, "Dialog", "dialog-footer: default", State::Rest);
        box_of(type_of(div(), &foot), &foot)
            .flex()
            .flex_none()
            .justify_end()
            .gap(size::dialog_button_gap(cx))
            .children(self.actions)
    }
}
