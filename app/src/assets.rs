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

/// Every embedded file, by its path.
pub const FILES: &[(&str, &[u8])] = icons![
    "tick-circle",
    "refresh",
    "time",
    "pause",
    "issue",
    "ban-circle",
    "cross-circle",
    "edit",
    "cog",
    "eye-open",
    "comparison",
    "properties",
    "git-merge",
    "rocket",
    "plus",
    "minus",
    "link",
    "git-branch",
    "console",
    "database",
    "layout",
    "desktop",
    "box",
    "search",
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
pub const FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/archivo/Archivo-Regular.ttf"),
    include_bytes!("../assets/fonts/archivo/Archivo-Medium.ttf"),
    include_bytes!("../assets/fonts/archivo/Archivo-SemiBold.ttf"),
    include_bytes!("../assets/fonts/archivo/Archivo-Bold.ttf"),
    include_bytes!("../assets/fonts/big-shoulders-display/BigShouldersDisplay-SemiBold.ttf"),
    include_bytes!("../assets/fonts/big-shoulders-display/BigShouldersDisplay-Bold.ttf"),
    include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf"),
    include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Medium.ttf"),
    include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-SemiBold.ttf"),
];
