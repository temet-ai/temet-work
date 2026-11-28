//! SideNav (`temet-sidenav`): the left column's areas and their projects, with a count beside
//! each project that waits on the person, and below them what else the column holds, such as
//! a GroupList. An area opens and closes with plus and minus. It goes in AppShell's `nav`.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use crate::components::icon::IconName;
use crate::components::plus_minus_tree::{PlusMinusTree, TreeNode};
use crate::components::tag::Tag;
use crate::theme::space;

/// A project in an area.
#[derive(Clone, Debug, PartialEq)]
pub struct SideNavProject {
    /// The project's id, as `selected` names it.
    pub id: SharedString,
    pub name: SharedString,
    pub icon: Option<IconName>,
    /// How many things in the project wait on the person; shown as a count when above 0.
    pub waiting: usize,
}

/// An area and its projects, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct SideNavArea {
    pub id: SharedString,
    pub name: SharedString,
    pub projects: Vec<SideNavProject>,
}

#[derive(IntoElement)]
pub struct SideNav {
    areas: Vec<SideNavArea>,
    selected: Option<SharedString>,
    default_closed: Vec<SharedString>,
    children: Vec<AnyElement>,
}

impl SideNav {
    pub fn new(areas: Vec<SideNavArea>) -> Self {
        Self {
            areas,
            selected: None,
            default_closed: Vec::new(),
            children: Vec::new(),
        }
    }

    /// The id of the project on screen.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected = Some(id.into());
        self
    }

    /// The ids of the areas that start closed. Every other area starts open.
    pub fn default_closed(mut self, ids: impl IntoIterator<Item = SharedString>) -> Self {
        self.default_closed = ids.into_iter().collect();
        self
    }
}

impl ParentElement for SideNav {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The tree's id for an area, kept apart from project ids.
fn area_key(id: &str) -> SharedString {
    format!("area:{id}").into()
}

impl RenderOnce for SideNav {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self
            .areas
            .iter()
            .filter(|a| !self.default_closed.contains(&a.id))
            .map(|a| area_key(&a.id))
            .collect::<Vec<_>>();
        let selected = self.selected;
        let nodes = self.areas.into_iter().map(|area| {
            TreeNode::new(area_key(&area.id), area.name).child_nodes(area.projects.into_iter().map(
                |p| {
                    let mut node = TreeNode::new(p.id.clone(), p.name)
                        .is_selected(selected.as_ref() == Some(&p.id));
                    if let Some(icon) = p.icon {
                        node = node.icon(icon);
                    }
                    if p.waiting > 0 {
                        node = node.secondary_label(Tag::new(p.waiting.to_string()));
                    }
                    node
                },
            ))
        });
        div()
            .flex()
            .flex_col()
            .gap(space::s6(cx))
            .child(PlusMinusTree::new("side-nav", nodes).default_expanded(open))
            .children(self.children)
    }
}
