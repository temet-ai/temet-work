//! Draws the window offscreen and writes each shot as a PNG, with no window, Wayland or
//! compositor: GPUI's headless app context renders the frame through its wgpu renderer into
//! memory. Run it with `cargo run --example shots --features shots -- DIR [NAME...]`.
//!
//! Each shot is the Project page in one theme at one window size. Most draw the sample data, which
//! holds a selected row, a milestone with no issues, a group with no agents and long names. The
//! `live-` shots draw the checked-in view fixture through `view_map`, as the window draws what
//! the daemon sends, and the problem and stale states of a daemon that does not answer.

use std::io::Write;
use std::sync::Arc;

use gpui::{AppContext, Font, FontWeight, HeadlessAppContext, font, px, size};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::composites::issue_pane::IssueTab;
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
        source: Source::Live,
        selected: "4",
        ..shot("live-dark-1440", "dark", 1440., 900.)
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
        let text = Arc::new(gpui_wgpu::CosmicTextSystem::new("Archivo"));
        let mut cx = HeadlessAppContext::with_platform(
            text,
            Arc::new(Assets),
            gpui_platform::current_headless_renderer,
        );
        let faces = cx.update(|cx| -> gpui::Result<String> {
            gpui_component::init(cx);
            cx.text_system().add_fonts(assets::faces())?;
            theme::apply(cx, s.theme).map_err(std::io::Error::other)?;
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
                page
            });
            cx.new(|cx| Root::new(page, window, cx))
        })?;
        // At a scale of 1, so a shot's pixels match the design system's story shots.
        cx.simulate_window_scale_factor_change(window.into(), 1.);
        // Two frames: the issue tables measure their width in the first.
        for _ in 0..2 {
            cx.run_until_parked();
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))?;
        }
        cx.run_until_parked();
        let image = cx.capture_screenshot(window.into())?;
        let path = format!("{dir}/{}.png", s.name);
        write_png(&path, image.width(), image.height(), image.as_raw())?;
        println!("{path}: {}x{}; {faces}", image.width(), image.height());
    }
    Ok(())
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
