//! Starting the app with the text rendering the design system's stories are drawn with.
//!
//! GPUI's renderer reads its text gamma and contrast only from the environment, once, when it
//! starts (`ZED_FONTS_GAMMA` and `ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST`, read by
//! gpui-pre-wgpu's `RenderingParameters::new`). Setting a variable inside a running process is
//! unsafe in edition 2024, and the app forbids unsafe code, so when either is unset the app runs
//! itself again with it set.

use std::ffi::OsString;
use std::process::Command;

/// The variable GPUI's renderer reads its text gamma from.
pub const FONTS_GAMMA_VAR: &str = "ZED_FONTS_GAMMA";

/// The text gamma the app runs with. With the contrast below, 1.0 gives each glyph the ink
/// Chromium gives it, so the window reads like the design system's stories: measured on the
/// same words in both themes, within 3%. GPUI's default of 1.8 draws text about 10% heavier.
pub const FONTS_GAMMA: &str = "1.0";

/// The variable GPUI's renderer reads the extra contrast of grayscale text from.
pub const FONTS_CONTRAST_VAR: &str = "ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST";

/// The extra contrast the app runs with: none. GPUI's default of 1.0 thickens text further; with
/// gamma 1.0 alone, text measured 4 to 7% heavier than Chromium's in the dark theme and 14 to
/// 17% in the light one.
pub const FONTS_CONTRAST: &str = "0";

/// The variables the app sets, each with its value.
pub const TEXT_RENDERING: [(&str, &str); 2] = [
    (FONTS_GAMMA_VAR, FONTS_GAMMA),
    (FONTS_CONTRAST_VAR, FONTS_CONTRAST),
];

/// The variables to set by running again: those the person has not set. A value the person
/// has set wins.
pub fn to_set(get: impl Fn(&str) -> Option<OsString>) -> Vec<(&'static str, &'static str)> {
    TEXT_RENDERING
        .into_iter()
        .filter(|(var, _)| get(var).is_none())
        .collect()
}

/// When a text rendering variable is unset, runs this executable again with the same arguments
/// and the unset variables set, waits for it, and returns its exit code. `None` when every
/// variable is set already, or when the executable cannot be run again, so the caller goes on.
pub fn with_text_rendering() -> Option<i32> {
    let set = to_set(|var| std::env::var_os(var));
    if set.is_empty() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    let status = Command::new(exe)
        .args(std::env::args_os().skip(1))
        .envs(set)
        .status()
        .ok()?;
    Some(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    #[test]
    fn values_the_person_set_are_left_and_unset_ones_are_set_by_running_again() {
        let none = |_: &str| None;
        assert_eq!(
            to_set(none),
            TEXT_RENDERING.to_vec(),
            "nothing set: run again with both"
        );

        let both = |_: &str| Some(OsString::from("2.2"));
        assert!(
            to_set(both).is_empty(),
            "both set: leave them, no second run"
        );

        let gamma_only = |var: &str| (var == FONTS_GAMMA_VAR).then(|| OsString::from("1.4"));
        assert_eq!(
            to_set(gamma_only),
            [(FONTS_CONTRAST_VAR, FONTS_CONTRAST)],
            "the person's gamma stays; only the contrast is set"
        );
    }
}
