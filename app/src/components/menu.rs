//! Menu, MenuItem and MenuDivider (`blueprint-menu`): a list of actions, such as IssueActions
//! behind an issue's actions button. An item is an icon and a word in the measured look, washed
//! under the pointer; a divider is a rule between groups of items.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::icon::{Icon, IconName};
use super::{box_of, type_of};
use crate::theme::look::{self, State};
use crate::theme::{colour, size};

#[derive(IntoElement)]
pub struct Menu {
    children: Vec<AnyElement>,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ParentElement for Menu {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (pad, least) = size::menu(cx);
        div()
            .flex()
            .flex_col()
            .p(pad)
            .min_w(least)
            .children(self.children)
    }
}

type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct MenuItem {
    id: ElementId,
    text: SharedString,
    icon: Option<IconName>,
    on_click: Option<OnClick>,
}

impl MenuItem {
    /// An item. `id` is its identity in the window, as React's `key`.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let rest = look::get(cx, "MenuItem", "default", State::Rest);
        let hover = look::get(cx, "MenuItem", "default", State::Hover);
        let selector = format!("{}", self.id);
        // `.bp6-menu-item .bp6-icon` is muted, centred on the item's 22 px line.
        let icon = self.icon.map(|icon| {
            div()
                .flex()
                .items_center()
                .h(size::menu_item_line(cx))
                .child(Icon::new(icon).color(colour::muted(cx)))
        });
        let mut item = box_of(type_of(div().id(self.id), &rest), &rest)
            .debug_selector(|| selector)
            .flex()
            .items_start()
            .gap(size::menu_item_gap(cx))
            .line_height(size::menu_item_line(cx))
            .cursor_pointer()
            .hover(move |style| style.bg(hover.background).text_color(hover.text))
            .children(icon)
            .child(div().flex_1().min_w_0().child(self.text));
        if let Some(handler) = self.on_click {
            item = item.on_click(move |event, window, cx| handler(event, window, cx));
        }
        item
    }
}

#[derive(IntoElement, Default)]
pub struct MenuDivider;

impl MenuDivider {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for MenuDivider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.bp6-menu-divider { border-top: 1px solid; margin: var(--bp-surface-spacing)
        // calc(var(--bp-surface-spacing) * -1) }`.
        let (pad, _) = size::menu(cx);
        div()
            .border_t(crate::theme::shape::border(cx))
            .border_color(colour::edge(cx))
            .my(pad)
            .mx(-pad)
    }
}
