//! RefineDialog (`temet-refinedialog`): an issue's title and the checks of its plan, for the
//! person to edit. Built in temet-work#253; this file holds its props until the part lands.

use gpui::SharedString;

/// One check of the plan as the person edits it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RefineCheck {
    /// What has to be true for the work to be done.
    pub done_when: SharedString,
    /// The command or action that shows it.
    pub proven_by: SharedString,
}
