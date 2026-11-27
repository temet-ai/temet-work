//! GroupList (`temet-grouplist`): agent groups in a side column, each group's name and
//! milestone over one row per agent with its role, where it runs and its status. A group with
//! no agents says so.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use crate::components::heading::Heading;
use crate::components::list_row::ListRow;
use crate::components::status_tag::{Status, StatusTag};
use crate::components::{Class, classed};
use crate::theme::space;

/// An agent in a group.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupListAgent {
    pub id: SharedString,
    /// The agent's role in its group, such as author or reviewer.
    pub role: SharedString,
    /// Where it runs, such as "omit · opus". It moves under the role when it does not fit
    /// beside it, and is cut with an ellipsis only when longer than the line.
    pub r#where: Option<SharedString>,
    pub status: Status,
}

/// A group and its agents, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupListGroup {
    pub id: SharedString,
    pub name: SharedString,
    /// Muted text at the right of the name, such as the milestone the group works on.
    pub note: Option<SharedString>,
    pub agents: Vec<GroupListAgent>,
}

#[derive(IntoElement)]
pub struct GroupList {
    title: SharedString,
    groups: Vec<GroupListGroup>,
}

impl GroupList {
    pub fn new(groups: Vec<GroupListGroup>) -> Self {
        Self {
            title: "Groups".into(),
            groups,
        }
    }

    /// The heading over all the groups; "Groups" when left out.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
}

impl RenderOnce for GroupList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let groups = self.groups.into_iter().enumerate().map(|(i, group)| {
            let head = classed(Class::GroupsHead, cx)
                .child(classed(Class::GroupsName, cx).child(group.name))
                .children(
                    group
                        .note
                        .map(|note| classed(Class::GroupsNote, cx).child(note)),
                );
            let empty = group.agents.is_empty().then(|| {
                ListRow::new("")
                    .compact(true)
                    .meta("No agents")
                    .class_name(Class::GroupsGroup)
                    .class_name(Class::GroupsEmpty)
            });
            let agents = group.agents.into_iter().map(|agent| {
                let row = ListRow::new(agent.role)
                    .compact(true)
                    .strong(true)
                    .class_name(Class::GroupsGroup)
                    .end(StatusTag::new(agent.status));
                match agent.r#where {
                    Some(place) => row.meta(place),
                    None => row,
                }
            });
            let block = div().child(head).children(empty).children(agents);
            if i > 0 {
                block.mt(space::s5(cx))
            } else {
                block
            }
        });
        div()
            .child(Heading::new(self.title).class_name(Class::ListTitle))
            .children(groups)
    }
}
