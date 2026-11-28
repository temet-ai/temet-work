//! Opens temet-work's window on the Project page, with sample data until the daemon sends its
//! view. The theme is dark unless `--theme light` or `TEMET_WORK_THEME=light` says otherwise.
//! The design system's faces are embedded and registered before the theme is put in force.

use gpui::{AppContext, TitlebarOptions, WindowOptions};
use gpui_component::Root;
use temet_work_app::assets::{self, Assets};
use temet_work_app::pages::project::Project;
use temet_work_app::{launch, sample, theme};

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

fn main() {
    // Run again with the text gamma and contrast set, unless the person set them (`launch`).
    if let Some(code) = launch::with_text_rendering() {
        std::process::exit(code);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let name = theme_name(&args, std::env::var("TEMET_WORK_THEME").ok());
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
                let page = cx.new(|_| Project::new(sample::project_view(), sample::selected()));
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
    use super::theme_name;

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
