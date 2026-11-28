//! ReviewRounds (`temet-reviewrounds`): an issue's review rounds: for each round its reviewer
//! and status, then every point with the author's reply. Only a point in the other state than
//! most of its round's (open or resolved) says which it is in. With no rounds it says "Not
//! reviewed yet".

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

use crate::components::Class;
use crate::components::list_row::ListRow;
use crate::components::status_tag::{Status, StatusTag};
use crate::theme::space;

/// One point a reviewer raised.
#[derive(Clone, Debug, PartialEq)]
pub struct ReviewPoint {
    pub text: SharedString,
    /// The author's reply; left out while there is none.
    pub reply: Option<SharedString>,
    pub resolved: bool,
}

/// One round of review.
#[derive(Clone, Debug, PartialEq)]
pub struct ReviewRound {
    /// The round's number, from 1.
    pub round: usize,
    /// The reviewer: role and runtime, such as "reviewer · Codex".
    pub by: SharedString,
    pub status: Status,
    pub points: Vec<ReviewPoint>,
}

#[derive(IntoElement)]
pub struct ReviewRounds {
    rounds: Vec<ReviewRound>,
}

impl ReviewRounds {
    /// The rounds, oldest first.
    pub fn new(rounds: Vec<ReviewRound>) -> Self {
        Self { rounds }
    }
}

/// Whether most of a round's points are resolved; a tie counts as open.
pub fn mostly_resolved(points: &[ReviewPoint]) -> bool {
    points.iter().filter(|p| p.resolved).count() * 2 > points.len()
}

impl RenderOnce for ReviewRounds {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let empty = self
            .rounds
            .is_empty()
            .then(|| ListRow::new("Not reviewed yet"));
        let rounds = self.rounds.into_iter().map(|r| {
            let head = ListRow::new(SharedString::from(format!("Round {}", r.round)))
                .compact(true)
                .strong(true)
                .class_name(Class::RoundHead)
                .meta(r.by)
                .end(StatusTag::new(r.status));
            let usual = mostly_resolved(&r.points);
            let points = r.points.into_iter().map(|p| {
                let reply = match p.reply {
                    Some(reply) => format!("Author: {reply}"),
                    None => "No reply yet".to_string(),
                };
                let row = ListRow::new(p.text).meta(reply);
                // Only a point in the other state than most of its round's says which it is in.
                if p.resolved == usual {
                    row
                } else {
                    let status = if p.resolved {
                        Status::Resolved
                    } else {
                        Status::Open
                    };
                    row.end(StatusTag::new(status))
                }
            });
            div().child(head).children(points)
        });
        div()
            .flex()
            .flex_col()
            .gap(space::s5(cx))
            .children(empty)
            .children(rounds)
    }
}
