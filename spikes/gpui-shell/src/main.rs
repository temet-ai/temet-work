//! Spike: a projects sidebar and a virtual list of issue rows on gpui-component.
//! The issues are made-up sample data.

use std::{ops::Range, rc::Rc};

use gpui::{
    App, AppContext as _, Bounds, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Size, Styled as _, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Theme, ThemeMode, VirtualListScrollHandle, h_flex,
    scroll::Scrollbar,
    sidebar::{Sidebar, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex, v_virtual_list,
};

const APP_ID: &str = "temet-work-gpui-shell";
const PROJECTS: [&str; 5] = ["mike", "tether", "radar", "gg", "holdfast"];
const ROW_HEIGHT: f32 = 30.;

struct Issue {
    project: &'static str,
    number: u32,
    title: &'static str,
    stage: &'static str,
    agent: &'static str,
}

macro_rules! issues {
    ($(($p:literal, $n:literal, $t:literal, $s:literal, $a:literal)),* $(,)?) => {
        &[$(Issue { project: $p, number: $n, title: $t, stage: $s, agent: $a }),*]
    };
}

#[rustfmt::skip]
const ISSUES: &[Issue] = issues![
    ("mike", 601, "Keep a seat's mail when its session restarts", "refine", "-"),
    ("mike", 598, "Show queued builds in the fleet digest", "work", "author"),
    ("mike", 597, "Verdict floor runs twice after a rebase", "review", "reviewer"),
    ("mike", 592, "Record why a lesson was ignored", "work", "author"),
    ("mike", 588, "Roster marks an idle seat stale too early", "merge", "-"),
    ("mike", 583, "Retrospective job times out on long runs", "refine", "refiner"),
    ("mike", 579, "Filter rule dry run ignores the project", "deploy", "-"),
    ("mike", 571, "List tool errors by command", "done", "-"),
    ("mike", 566, "Spawned session loses its goal", "review", "arbiter"),
    ("mike", 560, "Search notes by data field", "refine", "-"),
    ("tether", 214, "Mail arrives after the turn has ended", "work", "author"),
    ("tether", 209, "Roster shows ended sessions as running", "review", "reviewer"),
    ("tether", 204, "Checkpoint a thread from a hook", "refine", "-"),
    ("tether", 198, "Wrap long goals in the side panel", "done", "-"),
    ("tether", 195, "Lesson recall is slow on large projects", "work", "author"),
    ("tether", 190, "Mark a correction as superseded", "merge", "-"),
    ("tether", 183, "Keep inbox order across restarts", "refine", "refiner"),
    ("radar", 342, "Source fetch fails on a redirect loop", "work", "author"),
    ("radar", 339, "Group duplicate stories from two feeds", "review", "reviewer"),
    ("radar", 334, "Keyboard shortcut to hide a source", "refine", "-"),
    ("radar", 331, "Digest repeats yesterday's top item", "deploy", "-"),
    ("radar", 327, "Setup screen loses the typed API key", "work", "author"),
    ("radar", 320, "Cache images between launches", "done", "-"),
    ("radar", 316, "Narrow layout clips the score column", "review", "arbiter"),
    ("gg", 88, "Diff view skips renamed files", "work", "author"),
    ("gg", 85, "Stage a single hunk from the list", "refine", "-"),
    ("gg", 81, "Branch picker sorts by name, not date", "review", "reviewer"),
    ("gg", 77, "Crash on a repository with no commits", "merge", "-"),
    ("gg", 72, "Show ahead and behind counts", "done", "-"),
    ("gg", 69, "Remember the last opened repository", "refine", "refiner"),
    ("holdfast", 131, "Restore skips files with long paths", "work", "author"),
    ("holdfast", 128, "Snapshot size is reported twice", "review", "reviewer"),
    ("holdfast", 124, "Schedule backups per folder", "refine", "-"),
    ("holdfast", 119, "Verify a snapshot after upload", "deploy", "-"),
    ("holdfast", 112, "Resume an interrupted upload", "work", "author"),
    ("holdfast", 107, "Exclude build folders by default", "done", "-"),
];

struct Shell {
    /// `None` shows every project.
    project: Option<&'static str>,
    /// Indices into `ISSUES` of the rows the list shows.
    rows: Vec<usize>,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    scroll: VirtualListScrollHandle,
    logged_range: Range<usize>,
}

impl Shell {
    fn new() -> Self {
        let mut shell = Self {
            project: None,
            rows: Vec::new(),
            row_sizes: Rc::default(),
            scroll: VirtualListScrollHandle::new(),
            logged_range: 0..0,
        };
        shell.select(None);
        shell
    }

    fn select(&mut self, project: Option<&'static str>) {
        self.project = project;
        self.rows = (0..ISSUES.len())
            .filter(|&ix| project.is_none_or(|p| ISSUES[ix].project == p))
            .collect();
        self.row_sizes = Rc::new(vec![size(px(0.), px(ROW_HEIGHT)); self.rows.len()]);
    }

    fn menu_item(&self, project: Option<&'static str>, cx: &mut Context<Self>) -> SidebarMenuItem {
        let count = ISSUES
            .iter()
            .filter(|i| project.is_none_or(|p| i.project == p))
            .count();
        let shell = cx.entity();
        SidebarMenuItem::new(project.unwrap_or("All projects"))
            .active(self.project == project)
            .suffix(move |_, cx| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(count.to_string())
            })
            .on_click(move |_, _, cx| {
                shell.update(cx, |this, cx| {
                    this.select(project);
                    cx.notify();
                })
            })
    }

    fn issue_row(&self, ix: usize, cx: &App) -> impl IntoElement + use<> {
        let issue = &ISSUES[ix];
        let muted = cx.theme().muted_foreground;
        row_cells(
            format!("#{}", issue.number),
            issue.title,
            issue.project,
            issue.stage,
            issue.agent,
        )
        .id(("issue", ix))
        .h(px(ROW_HEIGHT))
        .border_b_1()
        .border_color(cx.theme().border)
        .text_color(if issue.stage == "done" {
            muted
        } else {
            cx.theme().foreground
        })
    }
}

/// The column layout shared by the header and the rows.
fn row_cells(
    number: impl Into<SharedString>,
    title: &'static str,
    project: &'static str,
    stage: &'static str,
    agent: &'static str,
) -> gpui::Div {
    h_flex()
        .w_full()
        .px_3()
        .gap_3()
        .child(div().w(px(56.)).flex_none().child(number.into()))
        .child(div().flex_1().min_w_0().truncate().child(title))
        .child(div().w(px(80.)).flex_none().child(project))
        .child(div().w(px(72.)).flex_none().child(stage))
        .child(div().w(px(80.)).flex_none().child(agent))
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut items = vec![self.menu_item(None, cx)];
        for project in PROJECTS {
            items.push(self.menu_item(Some(project), cx));
        }
        let sidebar = Sidebar::new("projects")
            .w(px(200.))
            .header(SidebarHeader::new().child("temet-work"))
            .child(SidebarMenu::new().children(items));

        let title = format!(
            "{}: {} issues (sample data)",
            self.project.unwrap_or("All projects"),
            self.rows.len()
        );

        let list = v_virtual_list(
            cx.entity(),
            "issues",
            self.row_sizes.clone(),
            |this, range, _, cx| {
                // Evidence that only the visible rows are built. The list also
                // calls this for one row to measure it; skip that call.
                if range.len() > 1 && this.logged_range != range {
                    eprintln!("built rows {range:?} of {}", this.rows.len());
                    this.logged_range = range.clone();
                }
                range
                    .map(|row| this.issue_row(this.rows[row], cx))
                    .collect()
            },
        )
        .track_scroll(&self.scroll);

        h_flex()
            .size_full()
            .items_stretch()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .child(sidebar)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(div().px_3().py_2().child(title))
                    .child(
                        row_cells("Issue", "Title", "Project", "Stage", "Agent")
                            .py_1()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .border_b_1()
                            .border_color(cx.theme().border),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .min_h_0()
                            .child(list)
                            .child(Scrollbar::vertical(&self.scroll)),
                    ),
            )
    }
}

fn open_window(cx: &mut App) {
    let options = WindowOptions {
        app_id: Some(APP_ID.into()),
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1100.), px(700.)),
            cx,
        ))),
        ..Default::default()
    };
    cx.open_window(options, |window, cx| {
        window.set_window_title("temet-work");
        let shell: Entity<Shell> = cx.new(|_| Shell::new());
        cx.new(|cx| Root::new(shell, window, cx))
    })
    .expect("failed to open the window");
}

fn main() {
    gpui_platform::application()
        .with_assets(gpui_kit_assets::Assets)
        .run(|cx| {
            gpui_component::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);
            cx.on_window_closed(|cx, _| cx.quit()).detach();
            open_window(cx);
        });
}
