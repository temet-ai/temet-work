//! StageTag (`temet-stagetag`): an issue's stage as its icon and name, in the default colour. A
//! stage has no status colour; a status uses StatusTag.

use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use super::icon::IconName;
use super::tag::Tag;
use crate::theme;

/// The stages an issue goes through, in order, as the design system names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Refining,
    Working,
    InReview,
    WithArbiter,
    Checks,
    Merging,
    Deployed,
}

impl Stage {
    pub const ALL: [Stage; 7] = [
        Self::Refining,
        Self::Working,
        Self::InReview,
        Self::WithArbiter,
        Self::Checks,
        Self::Merging,
        Self::Deployed,
    ];

    /// The stage's name, such as `in review`; the tag shows it as it is.
    pub fn name(self) -> &'static str {
        match self {
            Self::Refining => "refining",
            Self::Working => "working",
            Self::InReview => "in review",
            Self::WithArbiter => "with arbiter",
            Self::Checks => "checks",
            Self::Merging => "merging",
            Self::Deployed => "deployed",
        }
    }

    /// The stage's icon, from the design system's map.
    pub fn icon(self) -> IconName {
        let view = theme::stage(self.name()).expect("every stage is in the design system's map");
        IconName::from_name(view.icon).expect("every stage icon is embedded")
    }
}

/// A minimal tag with the stage's icon and name.
#[derive(IntoElement)]
pub struct StageTag {
    stage: Stage,
}

impl StageTag {
    pub fn new(stage: Stage) -> Self {
        Self { stage }
    }
}

impl RenderOnce for StageTag {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().flex_none().child(
            Tag::new(self.stage.name())
                .minimal(true)
                .icon(self.stage.icon()),
        )
    }
}
