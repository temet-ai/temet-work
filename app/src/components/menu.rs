//! Menu, MenuItem and MenuDivider (`blueprint-menu`): a list of actions on the raised surface,
//! each item with its icon, its text and an optional muted label at the right, and an item with items of its own opening them as a submenu
//! beside it, with a caret at its end, as Blueprint's `<MenuItem>` with children does. A divider
//! is a rule between groups of items.
//!
//! No look was measured for a menu, so it follows Blueprint's rules with the theme's roles: the
//! menu on the raised surface (`--tm-surface-raised`, which menus take) in the text colour, an
//! item's icon and caret muted, an item under the pointer on the hover surface. A submenu opens on
//! a click on its item, and to the item's left: IssueActions sits in the pane at the window's
//! right edge, where Blueprint's popover turns a submenu that has no room on the right to the
//! left. It stays open until its item is clicked again or the menu closes.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, anchored, div,
};

use super::icon::{Icon, IconName};
use crate::theme::look::{self, State};
use crate::theme::{colour, shape, size, text};

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A menu: its items and dividers, in order.
#[derive(IntoElement, Default)]
pub struct Menu {
    items: Vec<AnyElement>,
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, item: MenuItem) -> Self {
        self.items.push(item.into_any_element());
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items
            .extend(items.into_iter().map(IntoElement::into_any_element));
        self
    }

    /// A rule after the items so far, before the next group.
    pub fn divider(mut self) -> Self {
        self.items.push(MenuDivider.into_any_element());
        self
    }
}

/// A rule between groups of items: `.bp6-menu-divider { border-top: 1px solid; margin:
/// var(--bp-surface-spacing) calc(var(--bp-surface-spacing) * -1) }`, in the panel edge.
#[derive(IntoElement)]
pub struct MenuDivider;

impl RenderOnce for MenuDivider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pad = size::menu_pad(cx);
        div()
            .border_t(shape::border(cx))
            .border_color(colour::edge(cx))
            .my(pad)
            .mx(-pad)
    }
}

/// The menu's box: the raised surface, padded, with the theme's overlay edge, as Blueprint draws
/// a menu in a popover; the edge is the one measured on the Dialog, as the Toast's is.
fn menu_box(cx: &App) -> gpui::Div {
    let edge = look::get(cx, "Dialog", "dialog: default", State::Rest).shadow;
    div()
        .flex()
        .flex_col()
        .min_w(size::menu_width(cx))
        .p(size::menu_pad(cx))
        .bg(colour::raised(cx))
        .text_color(colour::text(cx))
        .text_size(text::body(cx))
        .rounded(shape::radius(cx))
        .shadow(edge)
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        menu_box(cx).children(self.items)
    }
}

/// One item: its icon, its text, and either a click or items of its own.
#[derive(IntoElement)]
pub struct MenuItem {
    id: SharedString,
    text: SharedString,
    icon: Option<IconName>,
    label: Option<SharedString>,
    items: Vec<MenuItem>,
    on_click: Option<OnClick>,
}

impl MenuItem {
    /// An item. `id` is its identity in the window, as React's `key`.
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
            label: None,
            items: Vec::new(),
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Muted text at the right, as Blueprint's `labelElement`, such as the issue an agent is on.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// An item of its submenu.
    pub fn child(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
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

impl RenderOnce for MenuItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (pad_y, pad_x) = size::menu_item_pad(cx);
        let line = size::menu_item_line(cx);
        let muted = colour::muted(cx);
        let hover = colour::hover(cx);
        let submenu = !self.items.is_empty();
        let open = window.use_keyed_state(
            SharedString::from(format!("{}-submenu", self.id)),
            cx,
            |_, _| false,
        );
        let is_open = submenu && *open.read(cx);
        // `.bp6-menu-item .bp6-menu-item-icon { height: 22px }`, the icon centred in the line.
        let icon = self.icon.map(|icon| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .h(line)
                .child(Icon::new(icon).color(muted))
        });
        let label = self
            .label
            .map(|label| div().flex_none().text_color(muted).child(label));
        let caret = submenu.then(|| {
            div()
                .flex()
                .flex_none()
                .items_center()
                .h(line)
                .child(Icon::new(IconName::CaretRight).color(muted))
        });
        let selector = self.id.to_string();
        let mut row = div()
            .id(self.id.clone())
            .debug_selector(|| selector)
            .relative()
            .flex()
            .items_start()
            .gap(pad_x)
            .px(pad_x)
            .py(pad_y)
            .line_height(line)
            .rounded(shape::radius(cx))
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .children(icon)
            .child(div().flex_1().min_w_0().child(self.text))
            .children(label)
            .children(caret);
        if is_open {
            // Blueprint's open submenu item keeps the hover surface.
            row = row.bg(hover);
        }
        if submenu {
            let open = open.clone();
            row = row.on_click(move |_, _, cx| {
                open.update(cx, |open, cx| {
                    *open = !*open;
                    cx.notify();
                });
            });
        } else if let Some(handler) = self.on_click {
            row = row.on_click(move |event, window, cx| handler(event, window, cx));
        }
        if !is_open {
            return row.into_any_element();
        }
        // The submenu's first item level with this one, and its right edge on the menu's left
        // edge: up by the menu's padding, and left by it from the item's own left edge.
        let pad = size::menu_pad(cx);
        let items: Vec<AnyElement> = self
            .items
            .into_iter()
            .map(IntoElement::into_any_element)
            .collect();
        row.child(
            div().absolute().left_0().top(-pad).child(
                anchored()
                    .anchor(gpui::Anchor::TopRight)
                    .offset(gpui::point(-pad, gpui::Pixels::ZERO))
                    .child(menu_box(cx).occlude().children(items)),
            ),
        )
        .into_any_element()
    }
}
