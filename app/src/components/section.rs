//! Section and SectionCard (`temet-section`): Blueprint's Section without `collapsible`. In
//! temet's apps a Section sits in a CutPanel, which draws its border, so it has no ground or
//! border of its own.

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};

use super::type_of;
use crate::theme::{
    self,
    look::{self, State},
};

#[derive(IntoElement, Default)]
pub struct Section {
    title: Option<SharedString>,
    subtitle: Option<SharedString>,
    compact: bool,
    right_element: Option<AnyElement>,
    children: Vec<SectionCard>,
}

impl Section {
    pub fn new() -> Self {
        Self::default()
    }

    /// The title. The header is drawn only when there is one.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Muted small text under the title.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// What sits at the right of the header.
    pub fn right_element(mut self, element: impl IntoElement) -> Self {
        self.right_element = Some(element.into_any_element());
        self
    }

    /// A card of the section's body.
    pub fn child(mut self, card: SectionCard) -> Self {
        self.children.push(card);
        self
    }
}

impl RenderOnce for Section {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        use theme::{colour, shape, size, text};
        // The title has a measured look; the header's rule, height and padding and the
        // subtitle are Blueprint's and have none, so they take the theme's roles.
        let compact = self.compact;
        let rule = |el: gpui::Div| {
            // Blueprint's own rule, white at 20% in the dark mode and near black at 15% in the
            // light one; the nearest token is the panel edge.
            el.border_b(shape::border(cx))
                .border_color(colour::edge(cx))
        };
        let header = self.title.map(|title| {
            let left = div()
                .flex()
                .items_center()
                .gap(size::section_title_pad(cx))
                .py(size::section_title_pad(cx))
                .child(
                    div()
                        .child(
                            type_of(
                                div(),
                                &look::get(
                                    cx,
                                    "CutPanel",
                                    "tm-cut-body tm-section-title",
                                    State::Rest,
                                ),
                            )
                            .child(title),
                        )
                        .children(self.subtitle.map(|subtitle| {
                            div()
                                .mt(size::section_subtitle_gap(cx))
                                .text_size(text::body_small(cx))
                                .text_color(colour::muted(cx))
                                .child(subtitle)
                        })),
                );
            rule(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(size::section_gap(cx))
                    .w_full()
                    .min_h(size::section_head(cx, compact))
                    .px(size::section_pad(cx, compact)),
            )
            .child(left)
            .children(self.right_element.map(|right| {
                div()
                    .flex()
                    .items_center()
                    .gap(size::section_title_pad(cx))
                    .ml_auto()
                    .child(right)
            }))
        });
        let last = self.children.len().saturating_sub(1);
        let cards = self.children.into_iter().enumerate().map(|(i, card)| {
            let pad = card.padded.then(|| size::section_pad(cx, compact));
            let mut body = div().children(card.children);
            if let Some(pad) = pad {
                body = body.p(pad);
            }
            if i < last { rule(body) } else { body }
        });
        div()
            .w_full()
            .overflow_hidden()
            .children(header)
            .children(cards)
    }
}

/// A part of a Section's body, padded unless told not to be.
pub struct SectionCard {
    padded: bool,
    children: Vec<AnyElement>,
}

impl Default for SectionCard {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionCard {
    pub fn new() -> Self {
        Self {
            padded: true,
            children: Vec::new(),
        }
    }

    pub fn padded(mut self, padded: bool) -> Self {
        self.padded = padded;
        self
    }
}

impl ParentElement for SectionCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
