//! GitHub issues, milestones and pull requests.

use temet_work_tracker::Tracker;

/// The github adapter.
#[derive(Default)]
pub struct GitHub;

impl Tracker for GitHub {
    fn id(&self) -> &'static str {
        "github"
    }
}
