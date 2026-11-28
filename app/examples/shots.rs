//! Draws the window offscreen and writes each shot as a PNG, with no window, Wayland or
//! compositor: GPUI's headless app context renders the frame through its wgpu renderer into
//! memory. Run it with `cargo run --example shots --features shots -- DIR [NAME...]`.
//!
//! Each shot is the Project page in one theme at one window size. Most draw the sample data, which
//! holds a selected row, a milestone with no issues, a group with no agents and long names. The
//! `live-` shots draw the checked-in view fixture through `view_map`, as the window draws what
//! the daemon sends, and the problem and stale states of a daemon that does not answer.
//!
//! The `part-` shots draw one part as one of the design system's stories draws it, named after
//! the story's id, such as `part-temet-stagetag--all-stages-dark`: the part on the page ground,
//! inset as Storybook insets a story, 1440 px wide.

use std::io::Write;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Context, Font, FontWeight, HeadlessAppContext, IntoElement,
    ParentElement, Render, Styled, Window, div, font, px, size,
};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::components::Intent;
use temet_work_app::components::cut_panel::CutPanel;
use temet_work_app::components::icon::IconName;
use temet_work_app::components::stage_tag::{Stage, StageTag};
use temet_work_app::components::status_tag::{Status, StatusTag};
use temet_work_app::components::tag::Tag;
use temet_work_app::composites::check_list::{CheckList, PlanCheck};
use temet_work_app::composites::issue_actions::{IssueActions, IssueActionsAgent};
use temet_work_app::composites::issue_pane::IssueTab;
use temet_work_app::composites::refine_dialog::{RefineCheck, RefineDialog, RefineDraft};
use temet_work_app::daemon::Problem;
use temet_work_app::pages::project::{Grouping, Project};
use temet_work_app::{launch, sample, theme, view_map};
use temet_work_core::View;

/// The view fixture the live shots draw: Discworld data from the core's tests.
const FIXTURE: &str = include_str!("../../crates/core/tests/fixtures/view.json");
/// The fixture's latest change, 1795000900000 ms, and five minutes on: the clock the shots use.
const NOW_MS: u64 = 1_795_001_200_000;
const SOCKET: &str = "/run/user/1000/temet-work.sock";

/// What a shot draws.
#[derive(Clone, Copy, PartialEq)]
enum Source {
    Sample,
    /// The fixture view, as a daemon sends it.
    Live,
    /// The fixture view, stale: the last ask got no answer.
    Stale,
    /// No view: nothing listens at the socket.
    Down,
    /// The view in the JSON file `SHOTS_VIEW` names, such as `temet-work status --json`
    /// printed, with the first issue that has review rounds selected. These shots are skipped
    /// when the variable is not set.
    File,
}

/// One shot: its name, theme, window size, and the page's state.
struct Shot {
    name: &'static str,
    theme: &'static str,
    width: f32,
    height: f32,
    tab: IssueTab,
    grouping: Grouping,
    needs: bool,
    selected: &'static str,
    overturn: Overturn,
    source: Source,
    /// Whether the selected issue's actions menu is open.
    actions: bool,
    refine: Refine,
}

/// How far the shot has gone with refining the selected issue.
#[derive(Clone, Copy, PartialEq)]
enum Refine {
    None,
    /// RefineDialog is open.
    Open,
    /// The daemon refused what it sent.
    Refused,
}

/// How far the shot has gone with overturning WATCH-51's decision in Needs you.
#[derive(Clone, Copy, PartialEq)]
enum Overturn {
    None,
    /// The confirm question is open.
    Asking,
    /// Confirmed: the toast is on show.
    Done,
}

const fn shot(name: &'static str, theme: &'static str, width: f32, height: f32) -> Shot {
    Shot {
        name,
        theme,
        width,
        height,
        tab: IssueTab::Plan,
        grouping: Grouping::Milestone,
        needs: true,
        selected: "WATCH-44",
        overturn: Overturn::None,
        source: Source::Sample,
        actions: false,
        refine: Refine::None,
    }
}

/// Every shot: each theme at three widths, each tab of the pane, the issues grouped by stage
/// and by group, nothing waiting on the person, and an issue with only its facts.
const SHOTS: &[Shot] = &[
    shot("dark-1440", "dark", 1440., 900.),
    shot("light-1440", "light", 1440., 900.),
    shot("dark-1280", "dark", 1280., 800.),
    shot("light-1280", "light", 1280., 800.),
    shot("dark-1700", "dark", 1700., 1000.),
    shot("light-1700", "light", 1700., 1000.),
    Shot {
        tab: IssueTab::Pull,
        ..shot("dark-1440-pull", "dark", 1440., 900.)
    },
    Shot {
        tab: IssueTab::Review,
        ..shot("dark-1440-review", "dark", 1440., 900.)
    },
    Shot {
        tab: IssueTab::Decisions,
        ..shot("dark-1440-decisions", "dark", 1440., 900.)
    },
    Shot {
        tab: IssueTab::Decisions,
        ..shot("light-1440-decisions", "light", 1440., 900.)
    },
    Shot {
        grouping: Grouping::Stage,
        ..shot("dark-1440-by-stage", "dark", 1440., 900.)
    },
    Shot {
        grouping: Grouping::Group,
        ..shot("dark-1440-by-group", "dark", 1440., 900.)
    },
    Shot {
        needs: false,
        ..shot("dark-1440-no-needs", "dark", 1440., 900.)
    },
    Shot {
        needs: false,
        ..shot("light-1440-no-needs", "light", 1440., 900.)
    },
    Shot {
        selected: "WATCH-47",
        ..shot("dark-1440-facts-only", "dark", 1440., 900.)
    },
    Shot {
        overturn: Overturn::Asking,
        ..shot("dark-1440-overturn-asking", "dark", 1440., 900.)
    },
    Shot {
        overturn: Overturn::Asking,
        ..shot("light-1440-overturn-asking", "light", 1440., 900.)
    },
    Shot {
        overturn: Overturn::Done,
        ..shot("dark-1440-overturn-done", "dark", 1440., 900.)
    },
    Shot {
        overturn: Overturn::Done,
        ..shot("light-1440-overturn-done", "light", 1440., 900.)
    },
    Shot {
        actions: true,
        ..shot("dark-1440-actions", "dark", 1440., 900.)
    },
    Shot {
        actions: true,
        ..shot("light-1440-actions", "light", 1440., 900.)
    },
    Shot {
        refine: Refine::Open,
        ..shot("dark-1440-refine", "dark", 1440., 900.)
    },
    Shot {
        refine: Refine::Open,
        ..shot("light-1440-refine", "light", 1440., 900.)
    },
    Shot {
        refine: Refine::Refused,
        ..shot("dark-1440-refine-refused", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Live,
        selected: "4",
        ..shot("live-dark-1440", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Live,
        selected: "4",
        actions: true,
        ..shot("live-dark-1440-actions", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Live,
        selected: "4",
        ..shot("live-light-1440", "light", 1440., 900.)
    },
    Shot {
        source: Source::Live,
        selected: "5",
        ..shot("live-dark-1440-checks", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Live,
        selected: "4",
        tab: IssueTab::Review,
        ..shot("live-dark-1440-review", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Stale,
        selected: "4",
        ..shot("live-stale-dark-1440", "dark", 1440., 900.)
    },
    Shot {
        source: Source::File,
        ..shot("real-dark-1440", "dark", 1440., 900.)
    },
    Shot {
        source: Source::File,
        ..shot("real-light-1440", "light", 1440., 900.)
    },
    Shot {
        source: Source::Down,
        ..shot("live-down-dark-1440", "dark", 1440., 900.)
    },
    Shot {
        source: Source::Down,
        ..shot("live-down-light-1440", "light", 1440., 900.)
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
    for s in SHOTS {
        if !only.is_empty() && !only.iter().any(|o| o == s.name) {
            continue;
        }
        let (mut cx, faces) = headless(s.theme)?;
        let sample_view = if s.needs {
            sample::project_view()
        } else {
            sample::without_needs(sample::project_view())
        };
        let text = match s.source {
            Source::File => match std::env::var("SHOTS_VIEW") {
                Ok(path) => std::fs::read_to_string(path)?,
                Err(_) => continue,
            },
            _ => FIXTURE.to_string(),
        };
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
            .map(|i| i.id.rsplit('#').next().unwrap_or_default().to_string());
        let selected = match (s.source, chosen) {
            (Source::File, Some(key)) => key,
            _ => s.selected.to_string(),
        };
        let path = std::path::Path::new(SOCKET);
        let window = cx.open_window(size(px(s.width), px(s.height)), |window, cx| {
            let page = cx.new(|_| {
                let mut page = match s.source {
                    Source::Sample => Project::new(sample_view, Some(s.selected.into())),
                    live => {
                        let mut page = Project::blank(None);
                        let view = (live != Source::Down).then_some(&fixture);
                        let problem = match live {
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
                        page
                    }
                };
                page.set_grouping(s.grouping);
                page.set_tab(s.tab);
                if s.overturn != Overturn::None {
                    page.ask_need(&"n1".into());
                }
                if s.overturn == Overturn::Done {
                    page.confirm();
                }
                page.set_actions_open(s.actions);
                page
            });
            if s.refine != Refine::None {
                page.update(cx, |page, cx| {
                    page.open_refine(&selected.clone().into(), window, cx);
                    if s.refine == Refine::Refused {
                        page.set_refine_not_done(
                            "the daemon is held: resume it, or Save without send",
                        );
                    }
                });
            }
            cx.new(|cx| Root::new(page, window, cx))
        })?;
        capture(&mut cx, window.into(), dir, s.name, &faces)?;
    }
    for (id, story, height) in PARTS {
        for theme_name in ["dark", "light"] {
            let name = format!("part-{id}-{theme_name}");
            if !only.is_empty() && !only.contains(&name) {
                continue;
            }
            let (mut cx, faces) = headless(theme_name)?;
            // The story shots are taken with reduced motion, which holds the hue edge still at
            // its start.
            cx.update(|cx| cx.set_reduce_motion(true));
            let window = cx.open_window(size(px(1440.), px(*height)), |window, cx| {
                let draft = (*story == Story::RefineDialogDefault)
                    .then(|| RefineDraft::new(REFINE_TITLE, &refine_checks(), window, cx));
                let gallery = cx.new(|_| Gallery {
                    story: *story,
                    draft,
                });
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
    IssueActionsInPane,
    RefineDialogDefault,
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
        "temet-issueactions--in-pane",
        Story::IssueActionsInPane,
        200.,
    ),
    (
        "temet-refinedialog--default",
        Story::RefineDialogDefault,
        900.,
    ),
];

/// RefineDialog's story: clacks#219's title and three checks.
const REFINE_TITLE: &str =
    "Night lamps switch on by the clock instead of the light reading at each tower";

fn refine_checks() -> Vec<RefineCheck> {
    vec![
        RefineCheck::new(
            "Night lamps switch on when the tower's own light sensor reads below the level its \
             operator set.",
            "cargo nextest run -p clacks-lamps sensor_switches_on",
        ),
        RefineCheck::new(
            "A failed sensor leaves the lamps on the clock, never on all day or off all night.",
            "cargo nextest run -p clacks-lamps failed_sensor_uses_clock",
        ),
        RefineCheck::new(
            "The six \u{dc}berwald towers with older controllers stay on the clock, and the plan \
             says so.",
            "docs/handbook/lamps.md, read in review",
        ),
    ]
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
    /// RefineDialog's draft, for its story.
    draft: Option<RefineDraft>,
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
            // The story's issue is ready; the window offers Refine and the one move the daemon
            // takes from ready, back to new, and has neither Assign nor Unassign.
            Story::IssueActionsInPane => div()
                .flex()
                .child(
                    IssueActions::new(Stage::Ready)
                        .on_refine(|_, _| {})
                        .move_to(vec![Stage::New])
                        .on_move(|_, _, _| {})
                        .assignable(vec![IssueActionsAgent {
                            id: "refiner-82".into(),
                            label: "refiner-82".into(),
                            detail: Some("clacks#215".into()),
                        }])
                        .on_assign(|_, _, _| {})
                        .assigned(vec![IssueActionsAgent {
                            id: "author-71".into(),
                            label: "author-71".into(),
                            detail: Some("clacks#219".into()),
                        }])
                        .on_unassign(|_, _, _| {}),
                )
                .into_any_element(),
            Story::CutPanelPadded => panel(false, false, cx).into_any_element(),
            Story::CutPanelAttention => panel(true, false, cx).into_any_element(),
            Story::CutPanelHere => panel(false, true, cx).into_any_element(),
            Story::CutPanelHereWithAttention => panel(true, true, cx).into_any_element(),
            Story::RefineDialogDefault => match &self.draft {
                Some(draft) => RefineDialog::new("clacks#219", draft).into_any_element(),
                None => div().into_any_element(),
            },
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
