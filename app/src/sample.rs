//! Sample data for the window's `--sample` mode, the tests and the screenshot tool: the same data
//! as the design system's radar page stories (`ui/src/pages/radar/data.ts`, made-up Discworld
//! names), as a daemon view that the window maps through `view_map` as it maps the daemon's.
//!
//! `assets/sample-view.json` is written by `scripts/sample-view` from the design system's data:
//! the stopped daemon's view, as the page's default stories draw it, with no agent running and
//! 324 ended. [`running`] starts the data's seven agents on it, with the two issues their
//! refiners took and the daemon's two problems, as the stories with the daemon running draw it.

use gpui::SharedString;
use temet_work_core::View;
use temet_work_core::ids::{AgentId, GroupId};
use temet_work_core::view::{StageWord, StateWord};

use crate::pages::project::ProjectView;
use crate::view_map;

/// The view the design system's data has, written by `scripts/sample-view`.
const SAMPLE_VIEW: &str = include_str!("../assets/sample-view.json");

/// The sample's clock: 28 Nov 2026 13:30 UTC, in milliseconds since 1970, the same as
/// `scripts/sample-view` takes. Each issue changed the data's "6 min ago" before it.
pub const NOW_MS: u64 = 1_795_872_600_000;

/// The stopped daemon's view: no agent runs.
pub fn view() -> View {
    serde_json::from_str(SAMPLE_VIEW).expect("the sample view parses")
}

/// The agents the data runs while the daemon runs: id, issue, model, state, and what each is
/// doing as the daemon says it.
const RUNNING: [(&str, &str, &str, StateWord, &str); 7] = [
    (
        "author-71",
        "clacks#212",
        "opus",
        StateWord::Working,
        "fixing round 1",
    ),
    (
        "investigator-6",
        "clacks#212",
        "sonnet",
        StateWord::Working,
        "reading the Quirm logs",
    ),
    ("author-69", "clacks#215", "opus", StateWord::Working, ""),
    (
        "reviewer-74",
        "clacks#201",
        "opus",
        StateWord::Working,
        "round 2",
    ),
    ("reviewer-75", "clacks#203", "sonnet", StateWord::Idle, ""),
    ("refiner-82", "clacks#224", "opus", StateWord::Working, ""),
    (
        "refiner-83",
        "clacks#226",
        "sonnet",
        StateWord::Blocked,
        "the issue names no tower",
    ),
];

/// The daemon's problems while it runs, as the data has them.
const PROBLEMS: [(&str, &str); 2] = [
    (
        "tracker",
        "GitHub answered 502 three times; the next read is in 2 min",
    ),
    (
        "start reviewer on clacks#203",
        "the runner could not open a terminal: no free slot on local; it tries again when one frees",
    ),
];

/// `view` with the daemon running: the data's seven agents at work, the two new issues their
/// refiners took now refining in the group, changed just now, and the daemon's problems.
pub fn running(mut view: View) -> View {
    let group = view
        .groups
        .first()
        .map_or_else(|| GroupId::new("grand-trunk"), |g| g.id.clone());
    for (id, issue, model, state, doing) in RUNNING {
        let Some(agent) = view.agents.iter_mut().find(|a| a.id.0 == id) else {
            continue;
        };
        agent.issue = Some(issue.to_string());
        agent.model = Some(model.to_string());
        agent.state = state;
        agent.state_text = doing.to_string();
        if let Some(i) = view.issues.iter_mut().find(|i| i.id == issue) {
            let agent = AgentId::new(id);
            if !i.agents.contains(&agent) {
                i.agents.push(agent);
            }
            if id.starts_with("refiner") && i.stage == StageWord::New {
                i.stage = StageWord::Refining;
                i.stage_text = "refining".into();
                i.group = Some(group.clone());
                i.updated_ms = Some(NOW_MS);
            }
        }
    }
    view.problems = PROBLEMS
        .iter()
        .map(|(name, text)| (name.to_string(), text.to_string()))
        .collect();
    view
}

/// The page's view of the stopped daemon's sample.
pub fn project_view() -> ProjectView {
    view_map::project_view(&view(), None, NOW_MS).expect("the sample has a project")
}

/// The page's view of the sample with the daemon running.
pub fn running_view() -> ProjectView {
    view_map::project_view(&running(view()), None, NOW_MS).expect("the sample has a project")
}

/// The issue open in the pane when the window opens, as the radar page opens it.
pub fn selected() -> Option<SharedString> {
    Some("clacks#212".into())
}
