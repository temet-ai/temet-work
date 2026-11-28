//! Draws the window offscreen and writes each shot as a PNG, with no window, Wayland or
//! compositor: GPUI's headless app context renders the frame through its wgpu renderer into
//! memory. Run it with `cargo run --example shots --features shots -- DIR [NAME...]`.
//!
//! The `page-` shots draw the Project page as the design system's radar page stories draw it,
//! named after the story's id, such as `page-pages-temet-work-radar--board-dark`: the sample
//! (`sample.rs`, the stories' own data as a daemon view) through `view_map`, with the story's
//! View, Group by, selected issue and tab, at 1440 by 900 in each theme. The `live-` shots draw
//! the checked-in view fixture through `view_map`, as the window draws what the daemon sends,
//! and the problem and stale states of a daemon that does not answer; the `real-` shots draw the
//! view in the file `SHOTS_VIEW` names, such as `temet-work status --json` printed.
//!
//! The `part-` shots draw one part as one of the design system's stories draws it, named after
//! the story's id, such as `part-temet-stagetag--all-stages-dark`: the part on the page ground,
//! inset as Storybook insets a story, 1440 px wide.

use std::io::Write;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Font, FontWeight, HeadlessAppContext, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, div, font, px, size,
};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::components::Intent;
use temet_work_app::components::app_shell::AppShell;
use temet_work_app::components::cut_panel::CutPanel;
use temet_work_app::components::icon::IconName;
use temet_work_app::components::property_list::PropertyItem;
use temet_work_app::components::stage_tag::{Stage, StageTag};
use temet_work_app::components::status_tag::{Status, StatusTag};
use temet_work_app::components::tag::Tag;
use temet_work_app::components::form_group::FormGroup;
use temet_work_app::components::input_group::{InputGroup, Size as InputSize};
use temet_work_app::components::radio_group::{Radio, RadioGroup};
use temet_work_app::components::text_area::TextArea;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::decision_row::{
    Decision, DecisionOption, DecisionRow, OverturnDraft, UndoCost,
};
use temet_work_app::composites::group_list::{GroupList, GroupListAgent, GroupListGroup};
use temet_work_app::composites::issue_pane::{
    IssueDecision, IssuePane, IssuePlan, IssuePullRequest, IssueTab,
};
use temet_work_app::composites::issue_table::{GroupBy, IssueSummary, IssueTable};
use temet_work_app::composites::meter_list::{MeterList, MeterListItem};
use temet_work_app::composites::milestone_panel::MilestonePanel;
use temet_work_app::composites::needs_you::{NeedsYou, NeedsYouDecision};
use temet_work_app::composites::problem_list::{Problem as ListProblem, ProblemList};
use temet_work_app::composites::review_rounds::{ReviewPoint, ReviewRound};
use temet_work_app::composites::refine_dialog::{RefineCheck, RefineDialog};
use temet_work_app::composites::side_nav::{SideNav, SideNavArea, SideNavProject};
use temet_work_app::daemon::Problem;
use temet_work_app::pages::project::{Grouping, Project, ViewMode};
use temet_work_app::{launch, sample, theme, view_map};
use temet_work_core::View;

/// The view fixture the live shots draw: Discworld data from the core's tests.
const FIXTURE: &str = include_str!("../../crates/core/tests/fixtures/view.json");
/// The fixture's latest change, 1795000900000 ms, and five minutes on: the clock the shots use.
const NOW_MS: u64 = 1_795_001_200_000;
const SOCKET: &str = "/run/user/1000/temet-work.sock";

/// One of the radar page's stories, as the Project page draws it: the story's id, whether the
/// daemon runs, and the page's View, Group by, the issue open in the pane and its tab.
struct PageStory {
    id: &'static str,
    running: bool,
    mode: ViewMode,
    grouping: Grouping,
    selected: Option<&'static str>,
    tab: Option<IssueTab>,
}

const fn story(id: &'static str) -> PageStory {
    PageStory {
        id,
        running: false,
        mode: ViewMode::List,
        grouping: Grouping::Milestone,
        selected: Some("clacks#212"),
        tab: None,
    }
}

/// The stories `Radar.stories.tsx` writes for the screen this issue builds, with their args.
const PAGES: &[PageStory] = &[
    PageStory {
        mode: ViewMode::Board,
        selected: Some("clacks#219"),
        ..story("board")
    },
    PageStory {
        mode: ViewMode::Board,
        running: true,
        selected: Some("clacks#219"),
        ..story("board-agents-running")
    },
    PageStory {
        mode: ViewMode::Board,
        selected: None,
        ..story("board-pane-closed")
    },
    story("radar"),
    PageStory {
        running: true,
        ..story("agents-running")
    },
    PageStory {
        grouping: Grouping::Stage,
        ..story("grouped-by-stage")
    },
    PageStory {
        grouping: Grouping::Group,
        ..story("grouped-by-group")
    },
    PageStory {
        selected: None,
        ..story("pane-closed")
    },
    PageStory {
        tab: Some(IssueTab::Plan),
        ..story("plan-checks")
    },
];

/// What a `live-` or `real-` shot draws.
#[derive(Clone, Copy, PartialEq)]
enum Source {
    /// The fixture view, as a daemon sends it.
    Live,
    /// The fixture view, stale: the last ask got no answer.
    Stale,
    /// No view: nothing listens at the socket.
    Down,
    /// The view in the JSON file `SHOTS_VIEW` names, with the first issue that has review rounds
    /// selected. These shots are skipped when the variable is not set.
    File,
}

/// A `live-` or `real-` shot: its name, theme, the page's View and selected issue, and its
/// source.
struct Shot {
    name: &'static str,
    theme: &'static str,
    mode: ViewMode,
    selected: &'static str,
    source: Source,
}

const fn shot(name: &'static str, theme: &'static str, source: Source) -> Shot {
    Shot {
        name,
        theme,
        mode: ViewMode::List,
        selected: "watch#4",
        source,
    }
}

const SHOTS: &[Shot] = &[
    shot("live-dark-1440", "dark", Source::Live),
    shot("live-light-1440", "light", Source::Live),
    Shot {
        selected: "watch#5",
        ..shot("live-dark-1440-checks", "dark", Source::Live)
    },
    Shot {
        mode: ViewMode::Board,
        ..shot("live-dark-1440-board", "dark", Source::Live)
    },
    shot("live-stale-dark-1440", "dark", Source::Stale),
    shot("live-down-dark-1440", "dark", Source::Down),
    shot("live-down-light-1440", "light", Source::Down),
    shot("real-dark-1440", "dark", Source::File),
    shot("real-light-1440", "light", Source::File),
    Shot {
        mode: ViewMode::Board,
        ..shot("real-dark-1440-board", "dark", Source::File)
    },
];

fn main() -> gpui::Result<()> {
    // Draw with the app's text gamma and contrast, unless set (`temet_work_app::launch`).
    if let Some(code) = launch::with_text_rendering() {
        std::process::exit(code);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(dir) = args.first() else {
        anyhow_exit("usage: shots DIR [NAME...]");
    };
    let only = &args[1..];
    let wanted = |name: &str| only.is_empty() || only.iter().any(|o| o == name);
    for page_story in PAGES {
        for theme_name in ["dark", "light"] {
            let name = format!(
                "page-pages-temet-work-radar--{}-{theme_name}",
                page_story.id
            );
            if !wanted(&name) {
                continue;
            }
            let (mut cx, faces) = headless(theme_name)?;
            cx.update(|cx| cx.set_reduce_motion(true));
            let window = cx.open_window(size(px(1440.), px(900.)), |window, cx| {
                let page = cx.new(|_| {
                    let view = if page_story.running {
                        sample::running_view()
                    } else {
                        sample::project_view()
                    };
                    let mut page = Project::new(view, page_story.selected.map(Into::into));
                    page.set_mode(page_story.mode);
                    page.set_grouping(page_story.grouping);
                    if let Some(tab) = page_story.tab {
                        page.set_tab(tab);
                    }
                    page
                });
                cx.new(|cx| Root::new(page, window, cx))
            })?;
            capture(&mut cx, window.into(), dir, &name, &faces)?;
        }
    }
    for s in SHOTS {
        if !wanted(s.name) {
            continue;
        }
        let text = match s.source {
            Source::File => match std::env::var("SHOTS_VIEW") {
                Ok(path) => std::fs::read_to_string(path)?,
                Err(_) => continue,
            },
            _ => FIXTURE.to_string(),
        };
        let (mut cx, faces) = headless(s.theme)?;
        let mut fixture: View = serde_json::from_str(&text).expect("the view parses");
        if s.source != Source::File {
            // The window shows the first project, so the fixture's second is the only one.
            fixture.projects.retain(|p| p.id.0 == "watch");
        }
        // A file's selected issue: the first with review rounds, else the first.
        let chosen = fixture
            .issues
            .iter()
            .find(|i| !i.reviews.is_empty())
            .or(fixture.issues.first())
            .map(|i| i.id.clone());
        let selected = match (s.source, chosen) {
            (Source::File, Some(key)) => key,
            _ => s.selected.to_string(),
        };
        let path = std::path::Path::new(SOCKET);
        let window = cx.open_window(size(px(1440.), px(900.)), |window, cx| {
            let page = cx.new(|_| {
                let mut page = Project::blank(None);
                let view = (s.source != Source::Down).then_some(&fixture);
                let problem = match s.source {
                    Source::Stale => Some(Problem::NoAnswer {
                        within: std::time::Duration::from_secs(5),
                    }),
                    Source::Down => Some(Problem::NoSocket {
                        path: path.to_path_buf(),
                    }),
                    _ => None,
                };
                view_map::show(&mut page, view, problem.as_ref(), path, NOW_MS);
                page.select(&selected.clone().into());
                page.set_mode(s.mode);
                page
            });
            cx.new(|cx| Root::new(page, window, cx))
        })?;
        capture(&mut cx, window.into(), dir, s.name, &faces)?;
    }
    for (id, story, height) in PARTS.iter().chain(FORM_PARTS) {
        for theme_name in ["dark", "light"] {
            let name = format!("part-{id}-{theme_name}");
            if !wanted(&name) {
                continue;
            }
            let (mut cx, faces) = headless(theme_name)?;
            // The story shots are taken with reduced motion, which holds the hue edge still at
            // its start.
            cx.update(|cx| cx.set_reduce_motion(true));
            let window = cx.open_window(size(px(1440.), px(*height)), |window, cx| {
                let gallery = cx.new(|_| Gallery { story: *story });
                cx.new(|cx| Root::new(gallery, window, cx))
            })?;
            capture(&mut cx, window.into(), dir, &name, &faces)?;
        }
    }
    Ok(())
}

/// A headless app with the faces registered and `theme_name` in force; also says which faces
/// it found.
fn headless(theme_name: &str) -> gpui::Result<(HeadlessAppContext, String)> {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new("Archivo"));
    let mut cx = HeadlessAppContext::with_platform(
        text,
        Arc::new(Assets),
        gpui_platform::current_headless_renderer,
    );
    let faces = cx.update(|cx| -> gpui::Result<String> {
        gpui_component::init(cx);
        cx.text_system().add_fonts(assets::faces())?;
        theme::apply(cx, theme_name).map_err(std::io::Error::other)?;
        // Each weight of Archivo resolves to a face of its own: 400, 600 and 700.
        let at = |weight: f32| {
            cx.text_system().resolve_font(&Font {
                weight: FontWeight(weight),
                ..font("Archivo")
            })
        };
        let weights = [at(400.), at(600.), at(700.)];
        let distinct = weights[0] != weights[1] && weights[1] != weights[2];
        Ok(format!(
            "text {}, code {}, Archivo 400/600/700 distinct faces: {distinct}",
            theme::text::family(cx),
            theme::text::code_family(cx)
        ))
    })?;
    Ok((cx, faces))
}

/// Draws the window's frames and writes the last as a PNG. The issue tables and check lists
/// measure their width in the first frame and draw to it in the next.
fn capture(
    cx: &mut HeadlessAppContext,
    window: gpui::AnyWindowHandle,
    dir: &str,
    name: &str,
    faces: &str,
) -> gpui::Result<()> {
    // At a scale of 1, so a shot's pixels match the design system's story shots.
    cx.simulate_window_scale_factor_change(window, 1.);
    for _ in 0..3 {
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))?;
    }
    cx.run_until_parked();
    let image = cx.capture_screenshot(window)?;
    let path = format!("{dir}/{name}.png");
    write_png(&path, image.width(), image.height(), image.as_raw())?;
    println!("{path}: {}x{}; {faces}", image.width(), image.height());
    Ok(())
}

/// A story of the design system that a `part-` shot draws.
#[derive(Clone, Copy, PartialEq)]
enum Story {
    StageTagAllStages,
    StageTagWithRound,
    TagIntents,
    CheckListDefault,
    CheckListFailedAndTimed,
    CheckListNoChecks,
    CheckListNotRunYet,
    CheckListLongChecks,
    CutPanelPadded,
    CutPanelAttention,
    CutPanelHere,
    CutPanelHereWithAttention,
    CutPanelMachinesWithOneHere,
    /// One of the form parts' stories (`FORM_PARTS`).
    Form(Form),
    IssuePaneDecisions,
    IssuePanePlan,
    IssuePaneReview,
    IssuePanePullRequest,
    IssuePaneFactsOnly,
    IssuePaneNoDecisions,
    IssuePaneInTheShell,
    SideNavProjects,
    SideNavNoAreas,
    IssueTableByStageWithMilestone,
    MilestonePanelNotTaken,
    ProblemListDefault,
}

/// Each part shot: the story's id, the story, and the frame's height, the same as the story
/// shot's.
const PARTS: &[(&str, Story, f32)] = &[
    ("temet-stagetag--all-stages", Story::StageTagAllStages, 120.),
    ("temet-stagetag--with-round", Story::StageTagWithRound, 120.),
    ("temet-tag--intents", Story::TagIntents, 120.),
    ("temet-checklist--default", Story::CheckListDefault, 300.),
    (
        "temet-checklist--failed-and-timed",
        Story::CheckListFailedAndTimed,
        300.,
    ),
    ("temet-checklist--no-checks", Story::CheckListNoChecks, 300.),
    (
        "temet-checklist--not-run-yet",
        Story::CheckListNotRunYet,
        300.,
    ),
    (
        "temet-checklist--long-checks",
        Story::CheckListLongChecks,
        420.,
    ),
    ("temet-cutpanel--padded", Story::CutPanelPadded, 220.),
    ("temet-cutpanel--attention", Story::CutPanelAttention, 220.),
    ("temet-cutpanel--here", Story::CutPanelHere, 220.),
    (
        "temet-cutpanel--here-with-attention",
        Story::CutPanelHereWithAttention,
        220.,
    ),
    (
        "temet-cutpanel--machines-with-one-here",
        Story::CutPanelMachinesWithOneHere,
        220.,
    ),
    (
        "temet-issuepane--decisions",
        Story::IssuePaneDecisions,
        760.,
    ),
    ("temet-issuepane--plan", Story::IssuePanePlan, 560.),
    ("temet-issuepane--review", Story::IssuePaneReview, 460.),
    (
        "temet-issuepane--pull-request",
        Story::IssuePanePullRequest,
        420.,
    ),
    (
        "temet-issuepane--facts-only",
        Story::IssuePaneFactsOnly,
        320.,
    ),
    (
        "temet-issuepane--no-decisions",
        Story::IssuePaneNoDecisions,
        400.,
    ),
    (
        "temet-issuepane--in-the-shell",
        Story::IssuePaneInTheShell,
        900.,
    ),
    ("temet-sidenav--projects", Story::SideNavProjects, 300.),
    ("temet-sidenav--no-areas", Story::SideNavNoAreas, 120.),
    (
        "temet-issuetable--by-stage-with-milestone",
        Story::IssueTableByStageWithMilestone,
        460.,
    ),
    (
        "temet-milestonepanel--not-taken",
        Story::MilestonePanelNotTaken,
        280.,
    ),
    (
        "temet-problemlist--default",
        Story::ProblemListDefault,
        200.,
    ),
];

fn st(text: &str) -> SharedString {
    SharedString::from(text.to_string())
}

/// IssuePane's stories' issue, radar#44, with the data each story keeps.
fn pane(story: Story) -> IssuePane {
    let check = |done_when: &str, proven_by: &str, status: Status| PlanCheck {
        done_when: st(done_when),
        proven_by: st(proven_by),
        status: Some(status),
        status_text: None,
    };
    let plan = IssuePlan {
        folder: Some(st("omit:~/work/radar-2")),
        merge: Some(st("squash after CI and one approval")),
        checks: vec![
            check(
                "Signing in stores a user token in the system keyring",
                "cargo nextest run -p radar-slack sign_in",
                Status::Passed,
            ),
            check(
                "The token is read back after radar restarts",
                "restart radar, open #general",
                Status::Running,
            ),
            check(
                "A revoked token sends the user back to sign-in",
                "cargo nextest run -p radar-slack revoked",
                Status::Waiting,
            ),
        ],
    };
    let pull = IssuePullRequest {
        number: 212,
        repo: None,
        merged: None,
        branch: Some(st("radar-44-keyring-token")),
        ci: Some(Status::Passed),
        ci_text: Some(st("CI passed in 3 m 12 s")),
        lines: Some((184, 37)),
        files: Some(6),
    };
    let point = |text: &str, reply: Option<&str>, resolved: bool| ReviewPoint {
        text: st(text),
        reply: reply.map(st),
        resolved,
    };
    let rounds = vec![
        ReviewRound {
            round: 1,
            by: st("reviewer \u{b7} Codex"),
            status: Status::Blocked,
            points: vec![point(
                "The token is written to ~/.config in plain text when the keyring is locked.",
                Some("Now refuses to store and asks to unlock the keyring."),
                true,
            )],
        },
        ReviewRound {
            round: 2,
            by: st("reviewer \u{b7} Codex"),
            status: Status::InProgress,
            points: vec![point(
                "sign_in() swallows the keyring error and returns Ok(()).",
                None,
                false,
            )],
        },
    ];
    let option = |id: &str, label: &str| DecisionOption {
        id: st(id),
        label: st(label),
    };
    let decision = |id: &str,
                    by: &str,
                    at: &str,
                    took: &str,
                    weighed: &[&str],
                    why: &str,
                    options: Vec<DecisionOption>,
                    before: &str| IssueDecision {
        id: st(id),
        decision: Decision {
            question: None,
            took: st(took),
            by: st(by),
            issue: None,
            at: Some(st(at)),
            cost: None,
            weighed: weighed.iter().map(|w| st(w)).collect(),
            why: Some(st(why)),
            options,
            overturn_before: Some(st(before)),
            overturned: None,
            overturned_to: None,
            overturn_reason: None,
        },
        draft: None,
        pending: false,
        error: None,
    };
    let decisions = vec![
        decision(
            "d1",
            "author \u{b7} Claude",
            "10:41",
            "Store the token in the system keyring (Secret Service on Linux, Keychain on macOS)",
            &[
                "Keyring",
                "Encrypted file in ~/.config",
                "Ask on every start",
            ],
            "Survives restarts, needs no key of our own, and is where other desktop apps keep tokens.",
            vec![
                option("file", "Encrypted file in ~/.config"),
                option("ask", "Ask on every start"),
            ],
            "the pull request merges",
        ),
        decision(
            "d2",
            "arbiter \u{b7} Claude",
            "10:52",
            "Refuse to store when the keyring is locked, rather than fall back to a file",
            &["Refuse and ask", "Fall back to a file"],
            "A plain-text fallback is the thing round 1 blocked.",
            vec![option("file", "Fall back to a file")],
            "round 2 closes",
        ),
    ];
    let base = IssuePane::new(
        "radar#44",
        "Signing in from radar stores a user token that outlives a restart",
        Stage::InReview,
    )
    .on_overturn(|_, _, _, _| {})
    .on_close(|_, _| {});
    match story {
        Story::IssuePaneFactsOnly => IssuePane::new(
            "radar#47",
            "Show unread counts per channel in the side bar",
            Stage::Refining,
        )
        .on_overturn(|_, _, _, _| {})
        .on_close(|_, _| {})
        .facts(vec![
            PropertyItem::new("Milestone", "radar 0.4"),
            PropertyItem::new("Group", "radar sign-in"),
            PropertyItem::new("Agent", "refiner \u{b7} Claude"),
            PropertyItem::new("Updated", "20 min ago"),
        ]),
        _ => {
            let tab = match story {
                Story::IssuePanePlan => IssueTab::Plan,
                Story::IssuePaneReview => IssueTab::Review,
                Story::IssuePanePullRequest => IssueTab::Pull,
                _ => IssueTab::Decisions,
            };
            let decisions = if story == Story::IssuePaneNoDecisions {
                Vec::new()
            } else {
                decisions
            };
            base.plan(Some(plan))
                .pull(Some(pull))
                .rounds(Some(rounds))
                .decisions(Some(decisions))
                .tab(tab)
        }
    }
}

/// IssuePane's In The Shell story: the work composites in AppShell. Its Needs you is the
/// redesigned one by issue, which this window does not draw yet (temet-work#258): the window's
/// Needs you lists the same issues' decisions.
fn in_the_shell(cx: &App) -> AnyElement {
    let _ = cx;
    let agent = |id: &str, role: &str, place: &str, detail: &str, status| GroupListAgent {
        id: st(id),
        role: st(role),
        r#where: Some(st(place)),
        detail: Some(st(detail)),
        status,
    };
    let nav = div()
        .child(GroupList::new(vec![GroupListGroup {
            id: st("sign-in"),
            name: st("radar sign-in"),
            note: Some(st("radar 0.4")),
            agents: vec![
                agent(
                    "a1",
                    "author",
                    "omit \u{b7} sonnet",
                    "Claude \u{b7} omit \u{b7} sonnet",
                    Status::Running,
                ),
                agent(
                    "a2",
                    "reviewer",
                    "deus \u{b7} gpt-5-codex",
                    "Codex \u{b7} deus \u{b7} gpt-5-codex",
                    Status::Waiting,
                ),
                agent(
                    "a3",
                    "arbiter",
                    "omit \u{b7} opus",
                    "Claude \u{b7} omit \u{b7} opus",
                    Status::Idle,
                ),
            ],
            empty: None,
        }]))
        .child(MeterList::new(
            "Machines",
            vec![
                MeterListItem {
                    id: st("omit"),
                    label: st("omit"),
                    value: 0.75,
                    reading: st("3 / 4 slots"),
                    note: None,
                },
                MeterListItem {
                    id: st("deus"),
                    label: st("deus"),
                    value: 0.5,
                    reading: st("1 / 2 slots"),
                    note: None,
                },
            ],
        ));
    let need = |id: &str, issue: &str, took: &str, cost| NeedsYouDecision {
        id: st(id),
        decision: Decision {
            question: None,
            took: st(took),
            by: st("arbiter \u{b7} Claude"),
            issue: Some(st(issue)),
            at: None,
            cost: Some(cost),
            weighed: Vec::new(),
            why: None,
            options: Vec::new(),
            overturn_before: Some(st("merge")),
            overturned: None,
            overturned_to: None,
            overturn_reason: None,
        },
    };
    let issue = |key: &str, title: &str, stage, agent: &str, updated: &str| IssueSummary {
        key: st(key),
        title: st(title),
        stage,
        group: None,
        milestone: None,
        agent: Some(st(agent)),
        updated: st(updated),
    };
    AppShell::new(nav)
        .side_fit(pane(Story::IssuePaneDecisions))
        .child(NeedsYou::new(vec![
            need(
                "n1",
                "radar#51",
                "Slack channel list pages past 1,000 channels",
                UndoCost::High,
            ),
            need(
                "n2",
                "radar#44",
                "Signing in from radar stores a user token that outlives a restart",
                UndoCost::Medium,
            ),
        ]))
        .child(
            MilestonePanel::new("radar 0.4")
                .done(7)
                .total(12)
                .due("12 Dec")
                .child(
                    IssueTable::new(
                        "shell-issues",
                        vec![
                            issue(
                                "radar#44",
                                "Signing in from radar stores a user token that outlives a restart",
                                Stage::InReview,
                                "reviewer \u{b7} Codex",
                                "4 min ago",
                            ),
                            issue(
                                "radar#51",
                                "Slack channel list pages past 1,000 channels",
                                Stage::WithArbiter,
                                "arbiter \u{b7} Claude",
                                "11 min ago",
                            ),
                            issue(
                                "radar#47",
                                "Show unread counts per channel in the side bar",
                                Stage::Refining,
                                "refiner \u{b7} Claude",
                                "20 min ago",
                            ),
                        ],
                    )
                    .selected(Some(st("radar#44")))
                    .on_select(|_, _, _| {}),
                ),
        )
        .into_any_element()
}

/// The MilestonePanel stories' issues, radar 0.4's, in a table.
fn radar_issues(id: &'static str) -> IssueTable {
    let issue = |key: &str, title: &str, stage, agent: &str, updated: &str| IssueSummary {
        key: st(key),
        title: st(title),
        stage,
        group: None,
        milestone: None,
        agent: Some(st(agent)),
        updated: st(updated),
    };
    IssueTable::new(
        id,
        vec![
            issue(
                "radar#44",
                "Signing in from radar stores a user token that outlives a restart",
                Stage::InReview,
                "reviewer \u{b7} Codex",
                "4 min ago",
            ),
            issue(
                "radar#51",
                "Slack channel list pages past 1,000 channels",
                Stage::WithArbiter,
                "arbiter \u{b7} Claude",
                "11 min ago",
            ),
            issue(
                "radar#42",
                "Keep the cursor row when a message arrives",
                Stage::InReview,
                "reviewer \u{b7} Codex",
                "26 min ago",
            ),
        ],
    )
    .selected(Some(st("radar#44")))
    .on_select(|_, _, _| {})
}

/// The IssueTable story grouped by stage with the Milestone column.
fn by_stage_with_milestone() -> IssueTable {
    let issue = |key: &str, title: &str, stage, milestone: &str, updated: &str| IssueSummary {
        key: st(key),
        title: st(title),
        stage,
        group: None,
        milestone: Some(st(milestone)),
        agent: None,
        updated: st(updated),
    };
    IssueTable::new(
        "by-stage",
        vec![
            issue(
                "clacks#212",
                "Relays drop the GNU overhead when a message passes through more than three towers",
                Stage::Fixing,
                "0.4",
                "6 min ago",
            ),
            issue(
                "clacks#201",
                "Messages queued during fog are sent in the wrong order when the fog lifts",
                Stage::InReview,
                "0.5",
                "3 min ago",
            ),
            issue(
                "clacks#215",
                "Tower status page shows a relay as up for a minute after its shutters jam",
                Stage::Working,
                "0.4",
                "14 min ago",
            ),
            issue(
                "clacks#160",
                "Show which operator sent a message in the relay log",
                Stage::Ready,
                "Backlog",
                "4 h ago",
            ),
            issue(
                "clacks#228",
                "Translate the status page into \u{dc}berwaldean",
                Stage::New,
                "Backlog",
                "10 h ago",
            ),
        ],
    )
    .group_by(GroupBy::Stage)
    .show_milestone(true)
    .show_agent(false)
    .selected(Some(st("radar#44")))
    .on_select(|_, _, _| {})
}

/// The SideNav stories' areas and projects.
fn side_nav(story: Story) -> SideNav {
    let project = |id: &str, icon, waiting| SideNavProject {
        id: st(id),
        name: st(id),
        icon: Some(icon),
        waiting,
    };
    match story {
        Story::SideNavNoAreas => SideNav::new(Vec::new())
            .projects(vec![project("clacks", IconName::GitRepo, 33)])
            .selected("clacks"),
        _ => SideNav::new(vec![
            SideNavArea {
                id: st("temet"),
                name: st("temet"),
                projects: vec![
                    project("radar", IconName::Console, 2),
                    project("mike", IconName::Database, 1),
                    project("ask-kit", IconName::Search, 0),
                    project("tether", IconName::Link, 0),
                ],
            },
            SideNavArea {
                id: st("guild"),
                name: st("guild"),
                projects: vec![
                    project("guild-ui", IconName::Layout, 1),
                    project("guild-api", IconName::Desktop, 0),
                    project("guild-base", IconName::Box, 0),
                ],
            },
        ])
        .selected("radar"),
    }
}

/// A check as the CheckList stories write one.
fn check(done_when: &str, proven_by: &str, status: Option<Status>) -> PlanCheck {
    PlanCheck {
        done_when: done_when.to_string().into(),
        proven_by: proven_by.to_string().into(),
        status,
        status_text: None,
    }
}

/// The checks of a CheckList story, as the story writes them.
fn checks(story: Story) -> Vec<PlanCheck> {
    let sign_in = || {
        check(
            "Signing in stores a user token in the system keyring",
            "cargo nextest run -p radar-slack sign_in",
            Some(Status::Passed),
        )
    };
    let revoked = |status| {
        check(
            "A revoked token sends the user back to sign-in",
            "cargo nextest run -p radar-slack revoked",
            Some(status),
        )
    };
    match story {
        Story::CheckListDefault => vec![
            sign_in(),
            check(
                "The token is read back after radar restarts",
                "restart radar, open #general",
                Some(Status::Running),
            ),
            revoked(Status::Waiting),
        ],
        Story::CheckListFailedAndTimed => vec![
            PlanCheck {
                status_text: Some("Passed in 41 s".into()),
                ..sign_in()
            },
            revoked(Status::Failed),
        ],
        Story::CheckListNotRunYet => vec![
            check(
                "Night lamps switch on when the tower's own light sensor reads below the level \
                 its operator set.",
                "cargo nextest run -p clacks-lamps sensor_switches_on",
                None,
            ),
            check(
                "A failed sensor leaves the lamps on the clock, never on all day or off all \
                 night.",
                "cargo nextest run -p clacks-lamps failed_sensor_uses_clock",
                None,
            ),
        ],
        Story::CheckListLongChecks => vec![
            check(
                "A message sent from Ankh-Morpork to Genua over the eight towers of the coast \
                 line arrives with the X-Clacks-Overhead field exactly as the sending tower set \
                 it: the same value, in the same position in the header, with no extra spaces \
                 and no second copy of the field. This holds when the message is rerouted by \
                 the Hub after it has left the first tower, when it waits at any tower during \
                 fog and is sent on later, and when a tower on the way restarts while the \
                 message is in its queue.",
                "cargo nextest run -p clacks-network overhead_survives_eight_towers \
                 overhead_survives_reroute overhead_survives_fog overhead_survives_restart \
                 overhead_survives_signal_box",
                Some(Status::Passed),
            ),
            check(
                "The relay's log line for a hop shows the overhead at debug level and is absent \
                 at info. At info a tower's log grows by no more lines per message than it did \
                 before the change, measured over an hour of the recorded Ankh-Morpork traffic \
                 replayed through a test tower.",
                "scripts/replay-traffic --line ankh-morpork --hours 1 --level info, then compare \
                 line counts with the run on main; the counts must match to the line",
                Some(Status::Waiting),
            ),
            check(
                "No setting turns the copy off on any tower.",
                "grep for overhead in the tower config schema finds no key",
                Some(Status::Passed),
            ),
        ],
        _ => Vec::new(),
    }
}

/// The CutPanel stories' padded content: `<strong>radar 0.4</strong>` over a muted line.
fn milestone(cx: &App) -> AnyElement {
    div()
        .child(
            div()
                .font_weight(theme::text::bold_weight(cx))
                .child("radar 0.4"),
        )
        .child(
            div()
                .text_color(theme::colour::muted(cx))
                .child("7 / 12 done \u{b7} due 12 Dec"),
        )
        .into_any_element()
}

/// One story's part on the page ground, inset as Storybook insets a story.
struct Gallery {
    story: Story,
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use theme::{colour, space, text};
        let gap = space::s4(cx);
        let row = || div().flex().flex_wrap().gap(gap);
        let panel = |attention: bool, here: bool, cx: &App| {
            CutPanel::new()
                .padded(true)
                .attention(attention)
                .here(here)
                .child(milestone(cx))
        };
        let part: AnyElement = match self.story {
            Story::StageTagAllStages => row()
                .children(Stage::ALL.map(StageTag::new))
                .into_any_element(),
            Story::StageTagWithRound => row()
                .child(StageTag::new(Stage::InReview).round(Some(2)))
                .child(StageTag::new(Stage::Fixing).round(Some(1)))
                .child(StageTag::new(Stage::WithArbiter).round(Some(3)))
                .into_any_element(),
            Story::TagIntents => row()
                .child(Tag::new("Keyring"))
                .child(Tag::new("radar 0.4").intent(Intent::Primary))
                .child(Tag::new("+184").intent(Intent::Success))
                .child(
                    Tag::new("high to undo")
                        .intent(Intent::Warning)
                        .icon(IconName::WarningSign),
                )
                .child(Tag::new("\u{2212}37").intent(Intent::Danger))
                .into_any_element(),
            Story::CheckListDefault
            | Story::CheckListFailedAndTimed
            | Story::CheckListNoChecks
            | Story::CheckListNotRunYet
            | Story::CheckListLongChecks => div()
                .max_w(theme::size::col_right(cx, window.viewport_size().width))
                .child(CheckList::new("story", checks(self.story)))
                .into_any_element(),
            Story::Form(form) => form_part(form, window, cx),
            Story::CutPanelPadded => panel(false, false, cx).into_any_element(),
            Story::CutPanelAttention => panel(true, false, cx).into_any_element(),
            Story::CutPanelHere => panel(false, true, cx).into_any_element(),
            Story::CutPanelHereWithAttention => panel(true, true, cx).into_any_element(),
            Story::CutPanelMachinesWithOneHere => div()
                .flex()
                .flex_col()
                .children(
                    [
                        ("omit", Status::Running, false),
                        ("mac", Status::Running, true),
                        ("deus", Status::Idle, false),
                    ]
                    .map(|(name, status, here)| {
                        CutPanel::new().padded(true).here(here).child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(space::s2(cx))
                                .child(div().font_weight(text::bold_weight(cx)).child(name))
                                .child(StatusTag::new(status)),
                        )
                    }),
                )
                .into_any_element(),
            Story::IssuePaneDecisions
            | Story::IssuePanePlan
            | Story::IssuePaneReview
            | Story::IssuePanePullRequest
            | Story::IssuePaneFactsOnly
            | Story::IssuePaneNoDecisions => div()
                .max_w(theme::size::col_right(cx, window.viewport_size().width))
                .child(pane(self.story))
                .into_any_element(),
            Story::IssuePaneInTheShell => return in_the_shell(cx),
            Story::SideNavProjects => div()
                .max_w(px(260.))
                .child(side_nav(self.story))
                .into_any_element(),
            Story::SideNavNoAreas => side_nav(self.story).into_any_element(),
            Story::IssueTableByStageWithMilestone => CutPanel::new()
                .child(by_stage_with_milestone())
                .into_any_element(),
            Story::MilestonePanelNotTaken => MilestonePanel::new("Backlog")
                .done(1)
                .total(44)
                .note(Some(st("no group takes it")))
                .child(radar_issues("not-taken"))
                .into_any_element(),
            Story::ProblemListDefault => div()
                .max_w(px(720.))
                .child(ProblemList::new(vec![
                    ListProblem {
                        name: st("tracker"),
                        text: st("GitHub answered 502 three times; the next read is in 2 min"),
                    },
                    ListProblem {
                        name: st("start reviewer on clacks#203"),
                        text: st(
                            "the runner could not open a terminal: no free slot on local; it \
                             tries again when one frees",
                        ),
                    },
                ]))
                .into_any_element(),
        };
        // The CutPanel stories are framed 520 px wide.
        let framed = match self.story {
            Story::CutPanelPadded
            | Story::CutPanelAttention
            | Story::CutPanelHere
            | Story::CutPanelHereWithAttention
            | Story::CutPanelMachinesWithOneHere => div().max_w(px(520.)).child(part),
            _ => div().child(part),
        };
        div()
            .size_full()
            .bg(colour::page(cx))
            .text_color(colour::text(cx))
            .text_size(text::body(cx))
            .font_family(text::family(cx))
            .line_height(text::line_height(cx))
            .p(space::s6(cx))
            .child(framed)
            .into_any_element()
    }
}

fn anyhow_exit(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2)
}

/// Writes 8-bit RGBA pixels as a PNG, with the image data in stored (uncompressed) deflate
/// blocks, so the tool needs no image library of its own.
fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    let row = width as usize * 4;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in rgba.chunks(row) {
        raw.push(0);
        raw.extend_from_slice(line);
    }
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(u8::from(i + 1 == blocks.len()));
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut out = std::fs::File::create(path)?;
    out.write_all(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])?;
    for (kind, data) in [(b"IHDR", &header), (b"IDAT", &zlib), (b"IEND", &Vec::new())] {
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        let mut chunk = kind.to_vec();
        chunk.extend_from_slice(data);
        out.write_all(&chunk)?;
        out.write_all(&crc32(&chunk).to_be_bytes())?;
    }
    Ok(())
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// A story of the form parts: the text fields, radios and dialogs, and the parts built from
/// them.
#[derive(Clone, Copy, PartialEq)]
enum Form {
    TextAreaAutoResize,
    InputGroupLabelled,
    InputGroupSmall,
    InputGroupDisabled,
    RadioGroupOptions,
    RadioGroupInline,
    RadioGroupDisabled,
    RefineDialogDefault,
    RefineDialogNoPlan,
    DecisionShort,
    DecisionOverturnReady,
    DecisionNoOptions,
    DecisionWithQuestion,
    DecisionWithQuestionOverturning,
    DecisionOneOtherOption,
    DecisionSending,
    DecisionNotSent,
    DecisionSent,
}

/// Each form part shot: the story's id, the story, and the frame's height, the same as the
/// story shot's.
const FORM_PARTS: &[(&str, Story, f32)] = &[
    (
        "blueprint-textarea--auto-resize",
        Story::Form(Form::TextAreaAutoResize),
        300.,
    ),
    (
        "temet-inputgroup--labelled",
        Story::Form(Form::InputGroupLabelled),
        300.,
    ),
    (
        "temet-inputgroup--small",
        Story::Form(Form::InputGroupSmall),
        300.,
    ),
    (
        "temet-inputgroup--disabled",
        Story::Form(Form::InputGroupDisabled),
        300.,
    ),
    (
        "blueprint-radiogroup--options",
        Story::Form(Form::RadioGroupOptions),
        300.,
    ),
    (
        "blueprint-radiogroup--inline",
        Story::Form(Form::RadioGroupInline),
        300.,
    ),
    (
        "blueprint-radiogroup--disabled",
        Story::Form(Form::RadioGroupDisabled),
        300.,
    ),
    (
        "temet-refinedialog--default",
        Story::Form(Form::RefineDialogDefault),
        560.,
    ),
    (
        "temet-refinedialog--no-plan",
        Story::Form(Form::RefineDialogNoPlan),
        560.,
    ),
    ("temet-decisionrow--short", Story::Form(Form::DecisionShort), 420.),
    (
        "temet-decisionrow--overturn-ready",
        Story::Form(Form::DecisionOverturnReady),
        420.,
    ),
    (
        "temet-decisionrow--no-options",
        Story::Form(Form::DecisionNoOptions),
        420.,
    ),
    (
        "temet-decisionrow--with-question",
        Story::Form(Form::DecisionWithQuestion),
        420.,
    ),
    (
        "temet-decisionrow--with-question-overturning",
        Story::Form(Form::DecisionWithQuestionOverturning),
        420.,
    ),
    (
        "temet-decisionrow--one-other-option",
        Story::Form(Form::DecisionOneOtherOption),
        420.,
    ),
    (
        "temet-decisionrow--sending",
        Story::Form(Form::DecisionSending),
        420.,
    ),
    (
        "temet-decisionrow--not-sent",
        Story::Form(Form::DecisionNotSent),
        520.,
    ),
    ("temet-decisionrow--sent", Story::Form(Form::DecisionSent), 420.),
];

/// A check as the RefineDialog stories write one.
fn refine_check(done_when: &str, proven_by: &str) -> RefineCheck {
    RefineCheck {
        done_when: done_when.to_string().into(),
        proven_by: proven_by.to_string().into(),
    }
}

/// The DecisionRow stories' decision: a short one in their default args, or the one with the
/// agent's question.
fn decision(question: bool) -> Decision {
    let s = |text: &str| SharedString::from(text.to_string());
    let option = |id: &str, label: &str| DecisionOption {
        id: s(id),
        label: s(label),
    };
    if question {
        Decision {
            question: Some(s("How many towers should the end-to-end test send a message through?")),
            took: s("8"),
            by: s("refiner-66"),
            issue: None,
            at: None,
            cost: Some(UndoCost::Low),
            weighed: Vec::new(),
            why: Some(s(
                "Four is the smallest count that shows the bug, but the Ankh-Morpork to Genua line \
                 has eight towers and is the longest in use. Eight covers both without making the \
                 test slower than the rest of the suite; twelve added forty seconds and covered no \
                 line that exists.",
            )),
            options: vec![option("1", "4"), option("3", "12")],
            overturn_before: Some(s("merge")),
            overturned: None,
            overturned_to: None,
            overturn_reason: None,
        }
    } else {
        Decision {
            question: None,
            took: s("Page the channel list 200 at a time instead of loading all"),
            by: s("arbiter \u{b7} Claude"),
            issue: Some(s("radar#51")),
            at: None,
            cost: Some(UndoCost::High),
            weighed: Vec::new(),
            why: None,
            options: vec![
                option("all", "Load every channel at once"),
                option("page-1000", "Page the channel list 1,000 at a time"),
            ],
            overturn_before: Some(s("merge, in about 20 min")),
            overturned: None,
            overturned_to: None,
            overturn_reason: None,
        }
    }
}

/// The reason the DecisionRow stories with the question type.
const REASON: &str = "The \u{dc}berwald line has twelve towers from next month, so the test should cover it.";

/// A form story's part, framed as the story frames it.
fn form_part(form: Form, window: &mut Window, cx: &mut App) -> AnyElement {
    let col = theme::size::col_right(cx, window.viewport_size().width);
    let draft = |option: Option<&str>, reason: Option<&str>| {
        Some(OverturnDraft {
            option: option.map(|o| o.to_string().into()),
            reason: reason.map(|r| r.to_string().into()),
        })
    };
    let row = |d: Decision| DecisionRow::new("story", d).on_overturn(|_, _, _| {});
    // The short stories are framed twice the pane's width, the question stories at its width.
    let short = |row: DecisionRow| div().max_w(col * 2.).child(row).into_any_element();
    let asked = |row: DecisionRow| div().max_w(col).child(row).into_any_element();
    let radios = |names: &[&str]| names.iter().map(|n| Radio::new(*n, *n)).collect::<Vec<_>>();
    match form {
        Form::TextAreaAutoResize => div()
            .w(px(400.))
            .child(
                TextArea::new("story")
                    .fill(true)
                    .auto_resize(true)
                    .default_value(
                        "Reproduce: sign in, restart radar, open settings.\nThe token is still there.",
                    ),
            )
            .into_any_element(),
        Form::InputGroupLabelled => div()
            .max_w(px(320.))
            .child(
                FormGroup::new()
                    .label("Branch")
                    .child(InputGroup::new("story").default_value("radar-44-keyring-token")),
            )
            .into_any_element(),
        Form::InputGroupSmall => div()
            .max_w(px(320.))
            .child(
                InputGroup::new("story")
                    .size(InputSize::Small)
                    .left_icon(IconName::Search)
                    .default_value("omit"),
            )
            .into_any_element(),
        Form::InputGroupDisabled => div()
            .max_w(px(320.))
            .child(
                InputGroup::new("story")
                    .disabled(true)
                    .default_value("builder-2"),
            )
            .into_any_element(),
        Form::RadioGroupOptions => RadioGroup::new(
            "story",
            radios(&["refiner", "author", "reviewer", "arbiter"]),
        )
        .label("Role")
        .into_any_element(),
        Form::RadioGroupInline => RadioGroup::new("story", radios(&["omit", "deus", "mac"]))
            .label("Machine")
            .inline(true)
            .into_any_element(),
        Form::RadioGroupDisabled => {
            RadioGroup::new("story", radios(&["opus", "sonnet", "gpt-5-codex"]))
                .label("Model")
                .disabled(true)
                .into_any_element()
        }
        Form::RefineDialogDefault => RefineDialog::new(
            "story",
            "clacks#219",
            "Night lamps switch on by the clock instead of the light reading at each tower",
            vec![
                refine_check(
                    "Night lamps switch on when the tower's own light sensor reads below the level its operator set.",
                    "cargo nextest run -p clacks-lamps sensor_switches_on",
                ),
                refine_check(
                    "A failed sensor leaves the lamps on the clock, never on all day or off all night.",
                    "cargo nextest run -p clacks-lamps failed_sensor_uses_clock",
                ),
                refine_check(
                    "The six \u{dc}berwald towers with older controllers stay on the clock, and the plan says so.",
                    "docs/handbook/lamps.md, read in review",
                ),
            ],
        )
        .is_open(true)
        .into_any_element(),
        Form::RefineDialogNoPlan => RefineDialog::new(
            "story",
            "clacks#228",
            "Translate the status page into \u{dc}berwaldean",
            Vec::new(),
        )
        .is_open(true)
        .into_any_element(),
        Form::DecisionShort => short(row(decision(false))),
        Form::DecisionOverturnReady => short(row(decision(false)).default_draft(draft(
            Some("page-1000"),
            Some("200 at a time makes five requests where one would do."),
        ))),
        Form::DecisionNoOptions => short(row(Decision {
            options: Vec::new(),
            ..decision(false)
        })),
        Form::DecisionWithQuestion => asked(row(decision(true))),
        Form::DecisionWithQuestionOverturning => {
            asked(row(decision(true)).default_draft(draft(None, None)))
        }
        Form::DecisionOneOtherOption => asked(
            row(Decision {
                question: Some("Should the fix ship behind a setting so a tower can turn it off?".into()),
                took: "No setting".into(),
                cost: Some(UndoCost::Medium),
                why: Some(
                    "A setting would let one tower drop the overhead again, which is the bug.".into(),
                ),
                options: vec![DecisionOption {
                    id: "1".into(),
                    label: "Yes, on by default".into(),
                }],
                ..decision(true)
            })
            .default_draft(draft(None, None)),
        ),
        Form::DecisionSending => asked(
            row(decision(true))
                .default_draft(draft(Some("3"), Some(REASON)))
                .pending(true),
        ),
        Form::DecisionNotSent => asked(
            row(decision(true))
                .default_draft(draft(Some("3"), Some(REASON)))
                .error(Some(
                    "No temet-work daemon answers at ~/.local/state/temet-work/daemon.sock. Start \
                     the daemon, then press Overturn again."
                        .into(),
                )),
        ),
        Form::DecisionSent => asked(row(Decision {
            overturned: Some("Overturned just now".into()),
            overturned_to: Some("12".into()),
            overturn_reason: Some(REASON.into()),
            ..decision(true)
        })),
    }
}
