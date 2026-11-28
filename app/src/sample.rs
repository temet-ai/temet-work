//! Sample data for the window's `--sample` mode, the tests and the screenshot tool: one project
//! with made-up Discworld names, already in the shape of the parts' props. The daemon's view is
//! mapped into the same shape in `view_map.rs`.

use std::collections::HashMap;

use gpui::SharedString;

use crate::components::icon::IconName;
use crate::components::stage_tag::Stage;
use crate::components::status_tag::Status;
use crate::composites::check_list::PlanCheck;
use crate::composites::decision_row::{Decision, UndoCost};
use crate::composites::group_list::{GroupListAgent, GroupListGroup};
use crate::composites::issue_pane::{IssueDecision, IssuePlan, IssuePullRequest};
use crate::composites::issue_table::IssueSummary;
use crate::composites::meter_list::MeterListItem;
use crate::composites::needs_you::NeedsYouDecision;
use crate::composites::page_header::PageFact;
use crate::composites::review_rounds::{ReviewPoint, ReviewRound};
use crate::composites::side_nav::{SideNavArea, SideNavProject};
use crate::pages::project::{IssueDetail, Milestone, ProjectView};

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

fn issue(
    key: &str,
    title: &str,
    stage: Stage,
    group: &str,
    agent: Option<&str>,
    updated: &str,
) -> IssueSummary {
    IssueSummary {
        key: s(key),
        title: s(title),
        stage,
        group: Some(s(group)),
        agent: agent.map(s),
        updated: s(updated),
    }
}

fn meter(label: &str, value: f32, reading: &str, note: Option<&str>) -> MeterListItem {
    MeterListItem {
        id: s(label),
        label: s(label),
        value,
        reading: s(reading),
        note: note.map(s),
    }
}

fn check(done_when: &str, proven_by: &str, status: Status) -> PlanCheck {
    PlanCheck {
        done_when: s(done_when),
        proven_by: s(proven_by),
        status: Some(status),
        status_text: None,
    }
}

/// A decision with only what NeedsYou shows: who, on which issue, its cost and deadline.
fn need(
    id: &str,
    issue: &str,
    by: &str,
    cost: UndoCost,
    took: &str,
    before: &str,
) -> NeedsYouDecision {
    NeedsYouDecision {
        id: s(id),
        decision: Decision {
            took: s(took),
            by: s(by),
            issue: Some(s(issue)),
            at: None,
            cost: Some(cost),
            weighed: Vec::new(),
            why: None,
            overturn_before: Some(s(before)),
            overturned: None,
        },
    }
}

/// The issue selected when the window opens.
pub fn selected() -> Option<SharedString> {
    Some(s("WATCH-44"))
}

/// The watch project: two areas, four agent groups (one with no agents, one with long names),
/// three machines (one full) and three limits, four decisions that need the person, and three
/// milestones, the last with no issues yet. WATCH-44 has a plan, a pull request, two review
/// rounds and two decisions; WATCH-47 has only its facts.
pub fn project_view() -> ProjectView {
    let areas = vec![
        SideNavArea {
            id: s("ankh-morpork"),
            name: s("ankh-morpork"),
            projects: vec![
                project("watch", IconName::Console, 2),
                project("post-office", IconName::Database, 1),
                project("mint", IconName::Search, 1),
                project("unseen-university", IconName::Link, 0),
            ],
        },
        SideNavArea {
            id: s("lancre"),
            name: s("lancre"),
            projects: vec![
                project("castle", IconName::Layout, 0),
                project("coven", IconName::Desktop, 0),
                project("bridge", IconName::Box, 0),
            ],
        },
    ];

    let groups = vec![
        GroupListGroup {
            id: s("night-watch"),
            name: s("night watch"),
            note: Some(s("watch 0.4, 0.5")),
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
            note: Some(s("watch 0.4, 0.5")),
            agents: vec![
                agent(
                    "py-1",
                    "reviewer",
                    "deus · claude-sonnet-5 · watch-58-semaphore-copy",
                    Status::Waiting,
                ),
                agent(
                    "py-2",
                    "author",
                    "omit · claude-opus-5 · watch-42-cursor-row",
                    Status::Blocked,
                ),
            ],
        },
        GroupListGroup {
            id: s("mint-audit"),
            name: s("mint audit"),
            note: Some(s("mint 1.2")),
            agents: vec![
                agent("ma-1", "author", "mac · sonnet", Status::Passed),
                agent("ma-2", "reviewer", "mac · opus", Status::Failed),
            ],
        },
        GroupListGroup {
            id: s("day-watch"),
            name: s("day watch"),
            note: Some(s("watch 0.6")),
            agents: Vec::new(),
        },
    ];

    // Each agent holds a slot on the machine it runs on, the first word of where it runs.
    let machines = [("omit", 6), ("deus", 4), ("mac", 2)]
        .into_iter()
        .map(|(machine, slots)| {
            let used = groups
                .iter()
                .flat_map(|g| &g.agents)
                .filter(|a| {
                    a.r#where
                        .as_ref()
                        .is_some_and(|w| w.split(" · ").next() == Some(machine))
                })
                .count();
            meter(
                machine,
                used as f32 / slots as f32,
                &format!("{used} / {slots} slots"),
                None,
            )
        })
        .collect();
    let limits = vec![
        meter("Claude 5 h", 0.62, "62%", Some("resets 14:20")),
        meter("Claude 7 d", 0.41, "41%", Some("resets Mon 09:00")),
        meter("Codex 5 h", 0.18, "18%", Some("resets 15:05")),
    ];

    let needs = vec![
        need(
            "n1",
            "WATCH-51",
            "arbiter · Claude",
            UndoCost::High,
            "Page the tower list 200 at a time instead of loading all",
            "merge, in about 20 min",
        ),
        need(
            "n2",
            "MINT-209",
            "reviewer · Claude",
            UndoCost::High,
            "Skip the flaky coin-count end-to-end test for this pull request",
            "deploy to staging",
        ),
        need(
            "n3",
            "WATCH-44",
            "arbiter · Claude",
            UndoCost::Medium,
            "Refuse to store the token when the keyring is locked",
            "round 2 closes",
        ),
        need(
            "n4",
            "POST-318",
            "author · Claude",
            UndoCost::Low,
            "Keep the old /letters route as a redirect for one release",
            "the next release",
        ),
    ];

    let milestones = vec![
        Milestone {
            name: s("watch 0.4"),
            due: Some(s("12 Dec")),
            done: 1,
            total: 5,
            issues: vec![
                issue(
                    "WATCH-44",
                    "Signing in at the Watch House stores a token that outlives a restart",
                    Stage::InReview,
                    "night watch",
                    Some("reviewer · Codex"),
                    "4 min ago",
                ),
                issue(
                    "WATCH-51",
                    "Clacks tower list pages past 1,000 towers",
                    Stage::WithArbiter,
                    "night watch",
                    Some("arbiter · Claude"),
                    "11 min ago",
                ),
                issue(
                    "WATCH-47",
                    "Show unread counts per beat in the side bar",
                    Stage::Refining,
                    "night watch",
                    Some("refiner · Claude"),
                    "20 min ago",
                ),
                issue(
                    "WATCH-39",
                    "Resize no longer clears the duty board",
                    Stage::Done,
                    "pseudopolis yard clacks paging and unread counts",
                    None,
                    "2 h ago",
                ),
                issue(
                    "WATCH-42",
                    "Keep the cursor row when a report arrives from Pseudopolis Yard while \
                     Sergeant Colon is scrolling through the night shift's long list of \
                     reports, so that he does not lose his place every time a constable \
                     files one",
                    Stage::Verifying,
                    "pseudopolis yard clacks paging and unread counts",
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
                    "night watch",
                    Some("author · Claude"),
                    "1 min ago",
                ),
                issue(
                    "WATCH-58",
                    "Selecting text copies it over the semaphore",
                    Stage::Merging,
                    "pseudopolis yard clacks paging and unread counts",
                    Some("reviewer · Claude"),
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

    let decision =
        |id: &str, by: &str, at: &str, took: &str, weighed: &[&str], why: &str, before: &str| {
            IssueDecision {
                id: s(id),
                decision: Decision {
                    took: s(took),
                    by: s(by),
                    issue: None,
                    at: Some(s(at)),
                    cost: None,
                    weighed: weighed.iter().map(|w| s(w)).collect(),
                    why: Some(s(why)),
                    overturn_before: Some(s(before)),
                    overturned: None,
                },
            }
        };
    let watch_44 = IssueDetail {
        id: Some(temet_work_core::ids::IssueId::new("watch", "WATCH-44")),
        moves: vec![Stage::New, Stage::Ready],
        round: 2,
        plan: Some(IssuePlan {
            folder: Some(s("omit:~/work/watch-2")),
            merge: Some(s("squash after CI and one approval")),
            checks: vec![
                check(
                    "Signing in stores a token in the system keyring",
                    "cargo nextest run -p watch-house sign_in",
                    Status::Passed,
                ),
                check(
                    "The token is read back after the Watch House restarts",
                    "cargo nextest run -p watch-house token_survives_restart",
                    Status::Running,
                ),
                check(
                    "A revoked token sends the constable back to sign-in",
                    "cargo nextest run -p watch-house revoked",
                    Status::Waiting,
                ),
            ],
        }),
        pull: Some(IssuePullRequest {
            number: 212,
            repo: None,
            merged: None,
            branch: Some(s("watch-44-keyring-token")),
            ci: Some(Status::Passed),
            ci_text: Some(s("CI passed in 3 m 12 s")),
            lines: Some((184, 37)),
            files: Some(6),
        }),
        rounds: Some(vec![
            ReviewRound {
                round: 1,
                by: s("reviewer · Codex"),
                status: Status::Blocked,
                points: vec![
                    ReviewPoint {
                        text: s(
                            "The token is written to ~/.config in plain text when the keyring is locked.",
                        ),
                        reply: Some(s("Now refuses to store and asks to unlock the keyring.")),
                        resolved: true,
                    },
                    ReviewPoint {
                        text: s("No test for a revoked token."),
                        reply: Some(s("Added watch-house revoked.")),
                        resolved: true,
                    },
                ],
            },
            ReviewRound {
                round: 2,
                by: s("reviewer · Codex"),
                status: Status::InProgress,
                points: vec![ReviewPoint {
                    text: s("sign_in() swallows the keyring error and returns Ok(())."),
                    reply: None,
                    resolved: false,
                }],
            },
        ]),
        decisions: Some(vec![
            decision(
                "d1",
                "author · Claude",
                "10:41",
                "Store the token in the system keyring (Secret Service on Linux, Keychain on macOS)",
                &[
                    "Keyring",
                    "Encrypted file in ~/.config",
                    "Ask on every start",
                ],
                "Survives restarts, needs no key of our own, and is where other desktop apps keep tokens.",
                "the pull request merges",
            ),
            decision(
                "d2",
                "arbiter · Claude",
                "10:52",
                "Refuse to store the token when the keyring is locked",
                &["Refuse and ask", "Fall back to a file"],
                "A plain-text fallback is the thing round 1 blocked.",
                "round 2 closes",
            ),
        ]),
    };
    let details = HashMap::from([
        (s("WATCH-44"), watch_44),
        (
            s("WATCH-51"),
            IssueDetail {
                round: 3,
                ..IssueDetail::default()
            },
        ),
        (
            s("WATCH-39"),
            IssueDetail {
                round: 1,
                ..IssueDetail::default()
            },
        ),
        (
            s("WATCH-42"),
            IssueDetail {
                round: 1,
                ..IssueDetail::default()
            },
        ),
        (
            s("WATCH-58"),
            IssueDetail {
                round: 2,
                ..IssueDetail::default()
            },
        ),
    ]);

    // Which project each decision that needs the person belongs to, and the decision in an
    // issue's pane that is the same one, so overturning either overturns both.
    let need_project = HashMap::from([
        (s("n1"), s("watch")),
        (s("n2"), s("mint")),
        (s("n3"), s("watch")),
        (s("n4"), s("post-office")),
    ]);
    let same_decision = HashMap::from([(s("n3"), (s("WATCH-44"), s("d2")))]);

    ProjectView {
        need_project,
        same_decision,
        areas,
        project: s("watch"),
        groups,
        machines,
        limits,
        title: s("watch"),
        facts: vec![
            PageFact {
                label: s("Issues"),
                value: s("tracker.example.com/projects/WATCH"),
                icon: Some(IconName::Link),
            },
            PageFact {
                label: s("Code"),
                value: s("git.example.com/ankh-morpork/watch"),
                icon: Some(IconName::GitBranch),
            },
        ],
        needs,
        milestones,
        details,
    }
}

/// The same project with nothing waiting on the person: no decisions to read, and no project
/// counts in the side column.
pub fn without_needs(mut view: ProjectView) -> ProjectView {
    view.needs.clear();
    for area in &mut view.areas {
        for project in &mut area.projects {
            project.waiting = 0;
        }
    }
    view
}
