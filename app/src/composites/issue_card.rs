//! IssueCard (`temet-issuecard`): an issue as a card on a StageBoard: its key, its title, the
//! agents on it and how long ago it changed, with the Needs-you bell and count when it has open
//! decisions. Built in temet-work#253; this file holds its props until the part lands.

use gpui::SharedString;

use crate::components::status_tag::Status;

/// An agent on the card.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueCardAgent {
    /// The agent's id.
    pub id: SharedString,
    /// The agent as the card names it, such as author-71.
    pub label: SharedString,
    /// The agent's state, as a StatusTag.
    pub status: Status,
}
