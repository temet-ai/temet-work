//! StatusTag (`temet-statustag`): a status as icon, word and colour together, never colour
//! alone. It never shrinks, so the word is never cut short beside long text.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use super::icon::IconName;
use super::tag::draw;
use crate::theme::look::{self, State};
use crate::theme::{self, Intent};

/// The statuses, as the design system names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Passed,
    Resolved,
    Running,
    InProgress,
    Waiting,
    Idle,
    Open,
    Blocked,
    Failed,
}

impl Status {
    pub const ALL: [Status; 9] = [
        Self::Passed,
        Self::Resolved,
        Self::Running,
        Self::InProgress,
        Self::Waiting,
        Self::Idle,
        Self::Open,
        Self::Blocked,
        Self::Failed,
    ];

    /// The design system's name for the status, such as `inProgress`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Resolved => "resolved",
            Self::Running => "running",
            Self::InProgress => "inProgress",
            Self::Waiting => "waiting",
            Self::Idle => "idle",
            Self::Open => "open",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }

    /// How StatusTag draws the status: its word, icon and intent, from the design system's map.
    pub fn view(self) -> &'static theme::StatusView {
        theme::status(self.name()).expect("every status is in the design system's map")
    }

    /// The status's icon.
    pub fn icon(self) -> IconName {
        IconName::from_name(self.view().icon).expect("every status icon is embedded")
    }
}

/// A minimal tag with the status's icon, word and intent, at the medium size: the size the
/// design system measured.
#[derive(IntoElement)]
pub struct StatusTag {
    status: Status,
    children: Option<SharedString>,
}

impl StatusTag {
    pub fn new(status: Status) -> Self {
        Self {
            status,
            children: None,
        }
    }

    /// Text in place of the status word; the icon and colour stay.
    pub fn children(mut self, children: impl Into<SharedString>) -> Self {
        self.children = Some(children.into());
        self
    }
}

impl RenderOnce for StatusTag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = self.status.view();
        let word = self.children.unwrap_or_else(|| view.word.into());
        let rest = State::Rest;
        let look = match view.intent {
            Intent::None => look::get(cx, "StatusTag", "minimal", rest),
            Intent::Primary => look::get(cx, "StatusTag", "intent-primary minimal", rest),
            Intent::Success => look::get(cx, "StatusTag", "intent-success minimal", rest),
            Intent::Warning => look::get(cx, "StatusTag", "intent-warning minimal", rest),
            Intent::Danger => look::get(cx, "StatusTag", "intent-danger minimal", rest),
        };
        div()
            .flex()
            .flex_none()
            .child(draw(&look, Some(self.status.icon()), word, cx))
    }
}
