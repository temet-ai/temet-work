//! StageBoard (`temet-stageboard`): temet-work's issues as a board: a column per stage, a lane
//! per milestone, a card per issue. Built in temet-work#253; this file holds its props until the
//! part lands.

use gpui::SharedString;

use super::issue_card::IssueCardAgent;
use crate::components::stage_tag::Stage;

/// One issue on the board.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardIssue {
    /// The issue's key.
    pub key: SharedString,
    pub title: SharedString,
    /// Its stage: the column it sits in.
    pub stage: Stage,
    /// The agents on it now.
    pub agents: Vec<IssueCardAgent>,
    /// How many of its decisions are open.
    pub decisions: usize,
    /// How long ago it changed.
    pub updated: Option<SharedString>,
}

/// A row of the board, such as a milestone, with its issues.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardLane {
    /// The lane's id, such as the milestone's name.
    pub id: SharedString,
    /// The lane's heading.
    pub title: SharedString,
    /// Muted text after the count, such as "no group takes it".
    pub note: Option<SharedString>,
    /// Whether the lane starts open.
    pub default_open: bool,
    /// Its issues, in order.
    pub issues: Vec<BoardIssue>,
}
