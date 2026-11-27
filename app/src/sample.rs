//! Sample data for the window until the daemon sends its view: one project with made-up
//! Discworld names, already in the shape of the parts' props. This is the one place that maps
//! data into props; the daemon's view will be mapped here too.

use gpui::SharedString;

use crate::components::icon::IconName;
use crate::components::stage_tag::Stage;
use crate::components::status_tag::Status;
use crate::composites::group_list::{GroupListAgent, GroupListGroup};
use crate::composites::issue_table::IssueSummary;
use crate::composites::page_header::PageFact;
use crate::composites::side_nav::{SideNavArea, SideNavProject};
use crate::pages::project::{Milestone, ProjectView};

fn s(text: &str) -> SharedString {
    SharedString::from(text.to_string())
}

fn project(id: &str, icon: IconName, waiting: usize) -> SideNavProject {
    SideNavProject {
        id: s(id),
        name: s(id),
        icon: Some(icon),
        waiting,
    }
}

fn agent(id: &str, role: &str, place: &str, status: Status) -> GroupListAgent {
    GroupListAgent {
        id: s(id),
        role: s(role),
        r#where: Some(s(place)),
        status,
    }
}

fn issue(key: &str, title: &str, stage: Stage, agent: Option<&str>, updated: &str) -> IssueSummary {
    IssueSummary {
        key: s(key),
        title: s(title),
        stage,
        agent: agent.map(s),
        updated: s(updated),
    }
}

/// The issue selected when the window opens.
pub fn selected() -> Option<SharedString> {
    Some(s("WATCH-44"))
}

/// The watch project: two areas, four agent groups (one with no agents, one with long names),
/// and three milestones, the last with no issues yet.
pub fn project_view() -> ProjectView {
    let areas = vec![
        SideNavArea {
            id: s("ankh-morpork"),
            name: s("ankh-morpork"),
            projects: vec![
                project("watch", IconName::Console, 2),
                project("post-office", IconName::Database, 1),
                project("mint", IconName::Search, 0),
                project("unseen-university", IconName::Link, 0),
            ],
        },
        SideNavArea {
            id: s("lancre"),
            name: s("lancre"),
            projects: vec![
                project("castle", IconName::Layout, 1),
                project("coven", IconName::Desktop, 0),
                project("bridge", IconName::Box, 0),
            ],
        },
    ];

    let groups = vec![
        GroupListGroup {
            id: s("night-watch"),
            name: s("night watch"),
            note: Some(s("watch 0.4")),
            agents: vec![
                agent("nw-1", "refiner", "omit · opus", Status::Idle),
                agent("nw-2", "author", "omit · sonnet", Status::Running),
                agent("nw-3", "reviewer", "deus · gpt-5-codex", Status::Waiting),
                agent("nw-4", "arbiter", "omit · opus", Status::Idle),
            ],
        },
        GroupListGroup {
            id: s("yard"),
            name: s("pseudopolis yard clacks paging and unread counts"),
            note: Some(s("watch 0.5")),
            agents: vec![
                agent(
                    "py-1",
                    "reviewer",
                    "deus · claude-sonnet-5 · watch-44-keyring-token",
                    Status::Waiting,
                ),
                agent(
                    "py-2",
                    "author",
                    "omit · claude-opus-5 · watch-51-clacks-pages",
                    Status::Blocked,
                ),
            ],
        },
        GroupListGroup {
            id: s("mint-audit"),
            name: s("mint audit"),
            note: Some(s("mint 1.2")),
            agents: vec![
                agent("ma-1", "author", "deus · sonnet", Status::Passed),
                agent("ma-2", "reviewer", "deus · opus", Status::Failed),
            ],
        },
        GroupListGroup {
            id: s("day-watch"),
            name: s("day watch"),
            note: Some(s("watch Q1")),
            agents: Vec::new(),
        },
    ];

    let milestones = vec![
        Milestone {
            name: s("watch 0.4"),
            due: Some(s("12 Dec")),
            done: 7,
            total: 12,
            issues: vec![
                issue(
                    "WATCH-44",
                    "Signing in at the Watch House stores a token that outlives a restart",
                    Stage::InReview,
                    Some("reviewer · Codex"),
                    "4 min ago",
                ),
                issue(
                    "WATCH-51",
                    "Clacks tower list pages past 1,000 towers",
                    Stage::WithArbiter,
                    Some("arbiter · Claude"),
                    "11 min ago",
                ),
                issue(
                    "WATCH-47",
                    "Show unread counts per beat in the side bar",
                    Stage::Refining,
                    Some("refiner · Claude"),
                    "20 min ago",
                ),
                issue(
                    "WATCH-39",
                    "Resize no longer clears the duty board",
                    Stage::Deployed,
                    None,
                    "2 h ago",
                ),
                issue(
                    "WATCH-42",
                    "Keep the cursor row when a report arrives from Pseudopolis Yard while \
                     Sergeant Colon is scrolling through the night shift's long list of \
                     reports, so that he does not lose his place every time a constable \
                     files one",
                    Stage::Checks,
                    Some("author · Claude"),
                    "26 min ago",
                ),
            ],
        },
        Milestone {
            name: s("watch 0.5"),
            due: Some(s("16 Jan")),
            done: 0,
            total: 2,
            issues: vec![
                issue(
                    "WATCH-55",
                    "Search reports across every connected clacks tower",
                    Stage::Working,
                    Some("author · Claude"),
                    "1 min ago",
                ),
                issue(
                    "WATCH-58",
                    "Selecting text copies it over the semaphore",
                    Stage::Merging,
                    None,
                    "8 min ago",
                ),
            ],
        },
        Milestone {
            name: s("watch 0.6"),
            due: Some(s("20 Feb")),
            done: 0,
            total: 0,
            issues: Vec::new(),
        },
    ];

    ProjectView {
        areas,
        project: s("watch"),
        groups,
        title: s("watch"),
        facts: vec![
            PageFact {
                label: s("Issues"),
                value: s("tracker.example.com WATCH"),
                icon: Some(IconName::Link),
            },
            PageFact {
                label: s("Code"),
                value: s("git.example.com/ankh-morpork/watch"),
                icon: Some(IconName::GitBranch),
            },
        ],
        milestones,
    }
}
