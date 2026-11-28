//! The app's embedded files: Blueprint's icons as SVG (`app/assets/icons`, Apache-2.0, see its
//! NOTICE), served to GPUI by path, such as `icons/16/pause.svg`; and the design system's faces
//! as TTF (`app/assets/fonts`, SIL Open Font License 1.1, see its NOTICE), which the window
//! registers with GPUI's text system before it puts a theme in force.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        &[$(
            (
                concat!("icons/16/", $name, ".svg"),
                include_bytes!(concat!("../assets/icons/16/", $name, ".svg")) as &[u8],
            ),
            (
                concat!("icons/20/", $name, ".svg"),
                include_bytes!(concat!("../assets/icons/20/", $name, ".svg")) as &[u8],
            ),
        )*]
    };
}

/// A face, by its path under `app/assets/fonts/`, and its bytes.
macro_rules! font {
    ($path:literal) => {
        (
            $path,
            include_bytes!(concat!("../assets/fonts/", $path)) as &[u8],
        )
    };
}

/// Every embedded file, by its path.
pub const FILES: &[(&str, &[u8])] = icons![
    "tick-circle",
    "refresh",
    "time",
    "pause",
    "issue",
    "ban-circle",
    "cross-circle",
    "inbox",
    "edit",
    "clipboard",
    "code",
    "eye-open",
    "wrench",
    "take-action",
    "git-merge",
    "rocket",
    "lab-test",
    "tick",
    "plus",
    "minus",
    "link",
    "git-branch",
    "notifications",
    "warning-sign",
    "more",
    "arrow-right",
    "caret-right",
    "info-sign",
    "x",
    "console",
    "database",
    "layout",
    "desktop",
    "box",
    "search",
    "trash",
    "add",
    "person",
    "cross",
    "git-repo",
];

/// The embedded files as GPUI's asset source.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(FILES
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(FILES
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .map(|(p, _)| SharedString::from(*p))
            .collect())
    }
}

/// The design system's faces, each weight its type scale uses: Archivo for text, Big Shoulders
/// Display for figures, IBM Plex Mono for code.
pub const FONTS: &[(&str, &[u8])] = &[
    font!("archivo/Archivo-Regular.ttf"),
    font!("archivo/Archivo-Medium.ttf"),
    font!("archivo/Archivo-SemiBold.ttf"),
    font!("archivo/Archivo-Bold.ttf"),
    font!("big-shoulders-display/BigShouldersDisplay-SemiBold.ttf"),
    font!("big-shoulders-display/BigShouldersDisplay-Bold.ttf"),
    font!("ibm-plex-mono/IBMPlexMono-Regular.ttf"),
    font!("ibm-plex-mono/IBMPlexMono-Medium.ttf"),
    font!("ibm-plex-mono/IBMPlexMono-SemiBold.ttf"),
];

/// The faces as GPUI's text system takes them.
pub fn faces() -> Vec<Cow<'static, [u8]>> {
    FONTS
        .iter()
        .map(|(_, bytes)| Cow::Borrowed(*bytes))
        .collect()
}
