//! PageHeader (`temet-pageheader`): the head of a screen's main column, the page title and a
//! wrapping line of facts about what is shown.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use crate::components::heading::{Heading, HeadingSize};
use crate::components::icon::{Icon, IconName};
use crate::components::{Class, classed};
use crate::theme::space;

/// A short labelled fact under the title, such as where the issues live.
#[derive(Clone, Debug, PartialEq)]
pub struct PageFact {
    /// What the value is, such as Issues or Code; shown muted.
    pub label: SharedString,
    pub value: SharedString,
    /// An icon before the label.
    pub icon: Option<IconName>,
}

#[derive(IntoElement)]
pub struct PageHeader {
    title: SharedString,
    facts: Vec<PageFact>,
}

impl PageHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            facts: Vec::new(),
        }
    }

    pub fn facts(mut self, facts: Vec<PageFact>) -> Self {
        self.facts = facts;
        self
    }
}

impl RenderOnce for PageHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let facts =
            (!self.facts.is_empty()).then(|| {
                div()
                    .flex()
                    .flex_wrap()
                    .gap_y(space::s1(cx))
                    .gap_x(space::s6(cx))
                    .mt(space::s2(cx))
                    .children(self.facts.into_iter().map(|fact| {
                        div()
                            .flex()
                            .items_center()
                            .gap(space::s3(cx))
                            .min_w_0()
                            .children(fact.icon.map(|icon| {
                                classed(Class::PageheadIcon, cx).child(Icon::new(icon))
                            }))
                            .child(classed(Class::PageheadMuted, cx).child(fact.label))
                            .child(div().child(fact.value))
                    }))
            });
        div()
            .flex()
            .flex_wrap()
            .justify_between()
            .items_start()
            .gap_y(space::s4(cx))
            .gap_x(space::s6(cx))
            .child(
                div()
                    .min_w_0()
                    .child(Heading::new(self.title).size(HeadingSize::Page))
                    .children(facts),
            )
    }
}
