//! Tabs and Tab (`blueprint-tabs`): a row of tabs over panels that share one place, such as an
//! issue's Plan, Pull request, Review and Decisions. Only the open tab's panel is drawn
//! (`renderActiveTabPanelOnly`). A tab may carry a count after its title (`tagContent`), a
//! minimal tag in the accent while the tab is open.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::tag::Tag;
use crate::theme::look::{self, State};
use crate::theme::{Intent, colour, size};

/// One tab: its id, its title and its panel.
pub struct Tab {
    id: SharedString,
    title: SharedString,
    tag_content: Option<SharedString>,
    panel: Option<AnyElement>,
}

impl Tab {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            tag_content: None,
            panel: None,
        }
    }

    /// A count after the title, such as how many checks a plan has, in a minimal tag.
    pub fn tag_content(mut self, content: Option<impl Into<SharedString>>) -> Self {
        self.tag_content = content.map(Into::into);
        self
    }

    /// What the tab shows when it is open.
    pub fn panel(mut self, panel: impl IntoElement) -> Self {
        self.panel = Some(panel.into_any_element());
        self
    }
}

type OnChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    selected_tab_id: Option<SharedString>,
    on_change: Option<OnChange>,
    tabs: Vec<Tab>,
}

impl Tabs {
    /// Tabs. `id` is their identity in the window.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected_tab_id: None,
            on_change: None,
            tabs: Vec::new(),
        }
    }

    /// The open tab's id; the first tab when it names none of them.
    pub fn selected_tab_id(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_tab_id = Some(id.into());
        self
    }

    /// Called with a tab's id when its title is clicked.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn child(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The open tab's title and the line under it are the accent as text; a tab's count is a
        // minimal tag, primary while its tab is open (Blueprint's TabTitle).
        let plain = look::get(cx, "Tab", "default", State::Rest);
        let chosen = look::get(cx, "Tab", "selected", State::Rest);
        let accent = chosen.text;
        let indicator = colour::intent_text(cx, Intent::Primary);
        let open = self
            .selected_tab_id
            .filter(|id| self.tabs.iter().any(|t| &t.id == id))
            .or_else(|| self.tabs.first().map(|t| t.id.clone()));
        let mut panel = None;
        let mut titles = Vec::new();
        for (i, tab) in self.tabs.into_iter().enumerate() {
            let is_open = open.as_ref() == Some(&tab.id);
            let selector = format!("tab-{}", tab.id);
            let mut title = div()
                .id(("tab", i))
                .debug_selector(|| selector)
                .relative()
                .flex()
                .items_center()
                .flex_none()
                .line_height(size::tab_line(cx))
                .text_size(plain.font_size)
                .text_color(plain.text)
                .whitespace_nowrap()
                .cursor_pointer()
                .child(tab.title)
                .children(tab.tag_content.map(|count| {
                    let intent = if is_open {
                        Intent::Primary
                    } else {
                        Intent::None
                    };
                    div()
                        .ml(size::tab_tag_gap(cx))
                        .child(Tag::new(count).intent(intent))
                }));
            if is_open {
                panel = tab.panel;
                title = title.text_color(accent).child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(size::tab_indicator(cx))
                        .bg(indicator),
                );
            } else {
                title = title.hover(move |style| style.text_color(accent));
                if let Some(handler) = self.on_change.clone() {
                    let id = tab.id.clone();
                    title = title.on_click(move |_, window, cx| handler(&id, window, cx));
                }
            }
            titles.push(title);
        }
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(size::tab_gap(cx))
                    .children(titles),
            )
            .children(panel.map(|panel| div().mt(size::tab_gap(cx)).child(panel)))
    }
}
