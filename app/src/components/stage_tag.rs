//! StageTag (`temet-stagetag`): an issue's stage as its icon and name, in the default colour. A
//! stage has no status colour; a status uses StatusTag. The eleven stages and their order are the
//! daemon's.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::icon::IconName;
use super::tag::draw;
use super::{Class, apply};
use crate::theme::{
    self,
    look::{self, State},
};

/// The stages an issue goes through, in the daemon's order, as the design system names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    New,
    Refining,
    Ready,
    Working,
    InReview,
    Fixing,
    WithArbiter,
    Merging,
    Deploying,
    Verifying,
    Done,
}

impl Stage {
    pub const ALL: [Stage; 11] = [
        Self::New,
        Self::Refining,
        Self::Ready,
        Self::Working,
        Self::InReview,
        Self::Fixing,
        Self::WithArbiter,
        Self::Merging,
        Self::Deploying,
        Self::Verifying,
        Self::Done,
    ];

    /// The stage's name, such as `in review`; the tag shows it as it is.
    pub fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Refining => "refining",
            Self::Ready => "ready",
            Self::Working => "working",
            Self::InReview => "in review",
            Self::Fixing => "fixing",
            Self::WithArbiter => "with arbiter",
            Self::Merging => "merging",
            Self::Deploying => "deploying",
            Self::Verifying => "verifying",
            Self::Done => "done",
        }
    }

    /// Whether the stage counts review rounds, so the tag shows the round with it: in review,
    /// fixing and with arbiter (the design system's `ROUND_STAGES`).
    pub fn has_round(self) -> bool {
        matches!(self, Self::InReview | Self::Fixing | Self::WithArbiter)
    }

    /// The stage's icon, from the design system's map.
    pub fn icon(self) -> IconName {
        let view = theme::stage(self.name()).expect("every stage is in the design system's map");
        IconName::from_name(view.icon).expect("every stage icon is embedded")
    }
}

/// A minimal tag with the stage's icon and name, and the review round after it on the stages
/// that have one.
#[derive(IntoElement)]
pub struct StageTag {
    stage: Stage,
    round: Option<usize>,
    class_name: Option<Class>,
}

impl StageTag {
    pub fn new(stage: Stage) -> Self {
        Self {
            stage,
            round: None,
            class_name: None,
        }
    }

    /// The review round, shown after the stage as the daemon writes it ("in review, round 2").
    /// Only in review, fixing and with arbiter show it, and only from round 1. Leave it out in a
    /// table column, where the tag must stay short.
    pub fn round(mut self, round: Option<usize>) -> Self {
        self.round = round;
        self
    }

    /// A class of the design system's own, such as `IssuesStatus` in a group row.
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// The tag's words: the stage, with its round where it has one.
    pub fn text(&self) -> SharedString {
        match self.round {
            Some(round) if round > 0 && self.stage.has_round() => {
                format!("{}, round {round}", self.stage.name()).into()
            }
            _ => self.stage.name().into(),
        }
    }
}

impl RenderOnce for StageTag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = look::get(cx, "StageTag", "minimal", State::Rest);
        let mut tag = draw(&look, Some(self.stage.icon()), self.text(), cx);
        if let Some(class) = self.class_name {
            tag = apply(tag, class, cx);
        }
        div().flex().flex_none().child(tag)
    }
}
