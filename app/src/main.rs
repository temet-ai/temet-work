//! Opens temet-work's window on the Project page, drawn from the daemon's view, which the window
//! asks for once a second. With no answer the page says why and shows nothing else. `--sample`
//! or `TEMET_WORK_SAMPLE=1` opens it on made-up sample data instead, with no daemon. The theme
//! is dark unless `--theme light` or `TEMET_WORK_THEME=light` says otherwise. The design
//! system's faces are embedded and registered before the theme is put in force.

use gpui::{AppContext, TitlebarOptions, WindowOptions};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::daemon::{self, Daemon};
use temet_work_app::pages::project::Project;
use temet_work_app::{install, launch, sample, theme, view_map};
use temet_work_core::command::VIEW_PERIOD;

/// The window's app id, which the compositor shows as its class.
const APP_ID: &str = "temet-work";

/// The theme named by `--theme NAME` or `TEMET_WORK_THEME`, else the dark one.
fn theme_name(args: &[String], env: Option<String>) -> String {
    args.iter()
        .position(|a| a == "--theme")
        .and_then(|i| args.get(i + 1).cloned())
        .or(env)
        .unwrap_or_else(|| "dark".into())
}

/// Whether `--sample` or `TEMET_WORK_SAMPLE=1` asks for the sample data.
fn sample_mode(args: &[String], env: Option<String>) -> bool {
    args.iter().any(|a| a == "--sample") || env.is_some_and(|v| v == "1")
}

/// What `--version`, `--build` or `--desktop-entry` prints, when one of them is the first
/// argument; the app then exits without opening a window.
fn answer(args: &[String]) -> Option<String> {
    match args.first().map(String::as_str) {
        Some("--version") => Some(install::version_line()),
        Some("--build") => Some(install::BUILD.to_string()),
        Some("--desktop-entry") => install::started_from().map(|exe| install::desktop_entry(&exe)),
        _ => None,
    }
}

/// How often the window reads the build installed where it was started from: sixty view
/// periods, a minute.
const INSTALLED_EVERY: std::time::Duration = VIEW_PERIOD.saturating_mul(60);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(text) = answer(&args) {
        print!("{}", text.trim_end_matches('\n'));
        println!();
        return;
    }
    // Run again with the text gamma and contrast set, unless the person set them (`launch`).
    if let Some(code) = launch::with_text_rendering() {
        std::process::exit(code);
    }
    let name = theme_name(&args, std::env::var("TEMET_WORK_THEME").ok());
    let sample = sample_mode(&args, std::env::var("TEMET_WORK_SAMPLE").ok());
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            // The design system's faces, so the window never depends on what is installed.
            if let Err(why) = cx.text_system().add_fonts(assets::faces()) {
                eprintln!("could not load the design system's faces: {why}");
            }
            if let Err(why) = theme::apply(cx, &name) {
                eprintln!("{why}");
                let _ = theme::apply(cx, "dark");
            }
            eprintln!(
                "faces in force: text {}, code {}",
                theme::text::family(cx),
                theme::text::code_family(cx)
            );
            cx.on_window_closed(|cx, _| cx.quit()).detach();
            let options = WindowOptions {
                app_id: Some(APP_ID.into()),
                titlebar: Some(TitlebarOptions {
                    title: Some(APP_ID.into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let opened = cx.open_window(options, |window, cx| {
                let page = if sample {
                    cx.new(|_| Project::new(sample::project_view(), sample::selected()))
                } else {
                    // The page follows the daemon entity: each new view or problem is mapped
                    // onto it, and what the person chose stays.
                    let page = cx.new(|_| Project::blank(None));
                    let daemon = cx.new(Daemon::new);
                    page.update(cx, |page, cx| view_map::sync(page, daemon.read(cx)));
                    // A person's commands go to the socket the view is read from.
                    let socket = daemon.read(cx).path().to_path_buf();
                    page.update(cx, |page, _| {
                        page.set_sender(move |command, cx| {
                            let path = socket.clone();
                            let executor = cx.background_executor().clone();
                            cx.spawn(async move |_| {
                                daemon::send(path, command, &executor)
                                    .await
                                    .map_err(|problem| problem.reason())
                            })
                        })
                    });
                    let shown = page.clone();
                    // The observer holds the reader for as long as the window lives. An entity
                    // nobody holds is dropped, and its loop of asks with it, so the page would
                    // wait for the daemon for good.
                    let reader = daemon.clone();
                    cx.observe(&daemon, move |daemon, cx| {
                        let _ = &reader;
                        shown.update(cx, |page, cx| {
                            view_map::sync(page, daemon.read(cx));
                            cx.notify();
                        });
                    })
                    .detach();
                    // A newer build installed where this one was started from: the page says a
                    // restart picks it up.
                    if let Some(exe) = install::started_from() {
                        let shown = page.downgrade();
                        cx.spawn(async move |cx| {
                            loop {
                                cx.background_executor().timer(INSTALLED_EVERY).await;
                                let path = exe.clone();
                                let installed = cx
                                    .background_executor()
                                    .spawn(async move { install::installed_build(&path) })
                                    .await;
                                let update = installed
                                    .filter(|b| b != install::BUILD)
                                    .map(|b| install::newer_build_notice(&b, install::BUILD));
                                if shown
                                    .update(cx, |page, cx| {
                                        if page.newer_build() != update.as_ref() {
                                            page.set_newer_build(update.clone());
                                            cx.notify();
                                        }
                                    })
                                    .is_err()
                                {
                                    return;
                                }
                            }
                        })
                        .detach();
                    }
                    page
                };
                cx.new(|cx| Root::new(page, window, cx))
            });
            if let Err(why) = opened {
                eprintln!("could not open the window: {why}");
                cx.quit();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{sample_mode, theme_name};

    #[test]
    fn the_sample_is_asked_for_by_flag_or_by_the_environment() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(sample_mode(&args(&["--sample"]), None));
        assert!(sample_mode(&args(&[]), Some("1".into())));
        assert!(!sample_mode(&args(&[]), Some("0".into())));
        assert!(!sample_mode(&args(&["--theme", "light"]), None));
    }

    #[test]
    fn the_argument_wins_over_the_environment_and_dark_is_the_default() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            theme_name(&args(&["--theme", "light"]), Some("dark".into())),
            "light"
        );
        assert_eq!(theme_name(&args(&[]), Some("light".into())), "light");
        assert_eq!(theme_name(&args(&[]), None), "dark");
        assert_eq!(theme_name(&args(&["--theme"]), None), "dark");
    }
}
