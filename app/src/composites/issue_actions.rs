//! IssueActions (`temet-issueactions`): what a person can do to an issue, as one menu: Open,
//! Refine, Move to a stage, Assign an agent and Unassign one. Built in temet-work#253; this file
//! holds its props until the part lands.

use gpui::SharedString;

/// An agent in the Assign or Unassign list.
#[derive(Clone, Debug, PartialEq)]
pub struct IssueActionsAgent {
    /// The agent's id, handed back to on_assign or on_unassign.
    pub id: SharedString,
    /// The agent as the list names it, such as author-71.
    pub label: SharedString,
    /// Muted text at the right, such as the issue it is on now or "free".
    pub detail: Option<SharedString>,
}
