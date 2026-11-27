//! PlusMinusTree (`temet-plusminustree`): Blueprint's Tree with plus and minus in place of its
//! chevron caret. A group opens and closes by clicking its row, and its label is set as a muted
//! heading. The selected node is an accent wash with a line above and below.

use std::collections::HashSet;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use super::icon::{Icon, IconName};
use crate::theme;

/// One node: a group when it has child nodes, an item otherwise.
pub struct TreeNode {
    id: SharedString,
    label: SharedString,
    icon: Option<IconName>,
    is_selected: bool,
    secondary_label: Option<AnyElement>,
    child_nodes: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            is_selected: false,
            secondary_label: None,
            child_nodes: Vec::new(),
        }
    }

    /// The icon before an item's label. A group shows plus or minus in its place.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn is_selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    /// What sits at the end of the row, such as a count.
    pub fn secondary_label(mut self, label: impl IntoElement) -> Self {
        self.secondary_label = Some(label.into_any_element());
        self
    }

    pub fn child_nodes(mut self, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        self.child_nodes = nodes.into_iter().collect();
        self
    }
}

type OnNodeClick = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct PlusMinusTree {
    id: ElementId,
    contents: Vec<TreeNode>,
    default_expanded: Vec<SharedString>,
    on_node_click: Option<OnNodeClick>,
}

impl PlusMinusTree {
    /// A tree. `id` is its identity in the window, which keeps the open groups between frames.
    pub fn new(id: impl Into<ElementId>, contents: impl IntoIterator<Item = TreeNode>) -> Self {
        Self {
            id: id.into(),
            contents: contents.into_iter().collect(),
            default_expanded: Vec::new(),
            on_node_click: None,
        }
    }

    /// The ids of the groups open at first; the tree holds which are open after that.
    pub fn default_expanded(mut self, ids: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_expanded = ids.into_iter().collect();
        self
    }

    /// Called with an item's id when its row is clicked.
    pub fn on_node_click(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_node_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for PlusMinusTree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let initial = self.default_expanded;
        let open = window.use_keyed_state(self.id.clone(), cx, move |_, _| {
            initial.into_iter().collect::<HashSet<SharedString>>()
        });
        let mut rows = Vec::new();
        for node in self.contents {
            push_rows(&mut rows, node, 0, &open, self.on_node_click.clone(), cx);
        }
        div().id(self.id).flex().flex_col().w_full().children(rows)
    }
}

/// The rows of `node` and, when it is an open group, of its child nodes.
fn push_rows(
    rows: &mut Vec<AnyElement>,
    node: TreeNode,
    depth: usize,
    open: &gpui::Entity<HashSet<SharedString>>,
    on_click: Option<OnNodeClick>,
    cx: &App,
) {
    use theme::{colour, shape, size, text};
    let group = !node.child_nodes.is_empty();
    let is_open = group && open.read(cx).contains(&node.id);
    let icon = if group {
        Some(if is_open {
            IconName::Minus
        } else {
            IconName::Plus
        })
    } else {
        node.icon
    };
    let icon_colour = if node.is_selected {
        colour::intent_text(cx, theme::Intent::Primary)
    } else {
        colour::muted(cx)
    };
    let mut row = div()
        .id(SharedString::from(format!("node-{}", node.id)))
        .relative()
        .flex()
        .items_center()
        .w_full()
        .h(size::tree_row(cx))
        .pr(size::tree_end(cx))
        .pl(size::tree_indent(cx) * depth as f32)
        .children(icon.map(|icon| {
            div()
                .flex()
                .mr(size::tree_icon_gap(cx))
                .child(Icon::new(icon).color(icon_colour))
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(node.label),
        )
        .children(
            node.secondary_label
                .map(|label| div().flex().flex_none().px(size::tree_end(cx)).child(label)),
        );
    if group {
        row = row
            .text_color(colour::muted(cx))
            .font_weight(text::bold_weight(cx));
    }
    if node.is_selected {
        let line = colour::selected_line(cx);
        let rule = || {
            div()
                .absolute()
                .left_0()
                .right_0()
                .h(shape::border(cx))
                .bg(line)
        };
        row = row
            .bg(colour::selected_fill(cx))
            .child(rule().top_0())
            .child(rule().bottom_0());
    } else {
        let hover = colour::hover(cx);
        row = row.hover(move |style| style.bg(hover));
    }
    if group {
        let open = open.clone();
        let id = node.id.clone();
        row = row.cursor_pointer().on_click(move |_, _, cx| {
            open.update(cx, |open, cx| {
                if !open.remove(&id) {
                    open.insert(id.clone());
                }
                cx.notify();
            });
        });
    } else if let Some(handler) = on_click.clone() {
        let id = node.id.clone();
        row = row
            .cursor_pointer()
            .on_click(move |_, window, cx| handler(&id, window, cx));
    }
    rows.push(row.into_any_element());
    if is_open {
        for child in node.child_nodes {
            push_rows(rows, child, depth + 1, open, on_click.clone(), cx);
        }
    }
}
