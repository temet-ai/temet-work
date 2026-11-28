//! Menu, MenuItem and MenuDivider (`blueprint-menu`): a list of actions, each a row with an
//! icon, its text and a muted label at the right. An item with children opens them as a submenu
//! at its right, with Blueprint's caret. Picking an item without children closes the popover the
//! menu is in, as Blueprint's MenuItem does.
//!
//! Blueprint opens a submenu when the pointer rests on its item; here a click opens it, and a
//! press outside it closes it.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, anchored, deferred, div,
};
use gpui_base::actions::Cancel;
use gpui_base::{DeferredPopover, GlobalState, POPUP_PRIORITY};

use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{colour, shape, size};

/// A list of items and dividers, on a raised surface.
#[derive(IntoElement, Default)]
pub struct Menu {
    children: Vec<AnyElement>,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    /// An item or a divider, after those already given.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_any_element));
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .min_w(size::menu_width(cx))
            .p(size::menu_pad(cx))
            .bg(colour::raised(cx))
            .text_color(colour::text(cx))
            .rounded(shape::radius(cx))
            .children(self.children)
    }
}

/// A rule between groups of items.
#[derive(IntoElement, Default)]
pub struct MenuDivider;

impl MenuDivider {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for MenuDivider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pad = size::menu_pad(cx);
        div()
            .my(pad)
            .mx(-pad)
            .border_t(shape::border(cx))
            .border_color(colour::edge(cx))
    }
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Whether an item's submenu is open, and its place among the open popovers while it is, so the
/// popover around the menu takes a press in the submenu as one inside it.
#[derive(Default)]
struct Submenu {
    open: Option<DeferredPopover>,
}

/// One action in a menu, or with children, the item that opens them as a submenu.
#[derive(IntoElement)]
pub struct MenuItem {
    id: ElementId,
    text: SharedString,
    icon: Option<IconName>,
    label: Option<SharedString>,
    on_click: Option<OnClick>,
    children: Vec<MenuItem>,
}

impl MenuItem {
    /// An item. `id` is its identity in the window, as React's `key`, and keeps a submenu open
    /// between frames.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
            label: None,
            on_click: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Muted text at the right, such as the issue an agent is on.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// An item of its submenu.
    pub fn child(mut self, item: MenuItem) -> Self {
        self.children.push(item);
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.children.extend(items);
        self
    }
}

impl RenderOnce for MenuItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let has_submenu = !self.children.is_empty();
        let submenu: Option<Entity<Submenu>> = has_submenu.then(|| {
            window.use_keyed_state(
                SharedString::from(format!("{}-submenu", self.id)),
                cx,
                |_, _| Submenu::default(),
            )
        });
        let open = submenu.as_ref().is_some_and(|s| s.read(cx).open.is_some());
        // The item whose submenu is open is drawn active (`.bp6-active`).
        let variant = if open { "active" } else { "default" };
        let rest = look::get(cx, "MenuItem", variant, State::Rest);
        let hover = look::get(cx, "MenuItem", variant, State::Hover);
        let gap = size::menu_item_gap(cx);
        let muted = colour::muted(cx);
        let selector = format!("{}", self.id);

        let icon = self
            .icon
            .map(|icon| div().flex().mr(gap).child(Icon::new(icon).color(muted)));
        let label = self.label.map(|label| {
            div()
                .flex_none()
                .ml(gap)
                .text_color(if open { rest.text } else { muted })
                .child(label)
        });
        let caret = has_submenu.then(|| {
            div()
                .flex()
                .ml(gap)
                .child(Icon::new(IconName::CaretRight).color(muted))
        });
        let mut row = box_of(type_of(div().id(self.id.clone()), &rest), &rest)
            .debug_selector(|| selector)
            .relative()
            .flex()
            .items_center()
            .min_h(rest.height)
            .cursor_pointer()
            .hover(move |style| style.bg(hover.background).text_color(hover.text))
            .children(icon)
            .child(div().flex_1().min_w_0().child(self.text))
            .children(label)
            .children(caret);

        match submenu {
            None => {
                let handler = self.on_click;
                row = row.on_click(move |event, window, cx| {
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    // Closes the popover the menu is in: the popover takes the focus when it
                    // opens and closes on Cancel.
                    window.dispatch_action(Box::new(Cancel), cx);
                });
            }
            Some(state) => {
                let opener = state.clone();
                row = row.on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    opener.update(cx, |s, cx| {
                        if s.open.is_none() {
                            s.open = Some(GlobalState::register_deferred_popover(cx));
                            cx.notify();
                        }
                    });
                });
                if open {
                    let closer = state.clone();
                    let items = Menu::new().children(self.children).into_any_element();
                    // It takes the pointer from what lies under it, as a popover's content does.
                    let popup = div()
                        .occlude()
                        .rounded(shape::radius(cx))
                        .on_mouse_down_out(move |_, _, cx| {
                            closer.update(cx, |s, cx| {
                                s.open = None;
                                cx.notify();
                            });
                        })
                        .child(items);
                    // `placement: right-start`, moved up by the menu's padding so the
                    // submenu's first item meets this one.
                    row = row.child(
                        div().absolute().left_full().top(-size::menu_pad(cx)).child(
                            deferred(
                                anchored()
                                    .snap_to_window_with_margin(size::menu_pad(cx))
                                    .child(popup),
                            )
                            // Over the popover the menu is in.
                            .with_priority(POPUP_PRIORITY + 1),
                        ),
                    );
                }
            }
        }
        row
    }
}
