//! The design system's bans, as data, and the checks of the synced tokens against them.
//!
//! The Rules page of the design system says: no teal, cyan or brown; red never stands against
//! green, so success is blue; corners are 2 px, with no pills. A hue band here is where a colour
//! reads as one of those hues on the dark ground.

use crate::{Px, Rgba};

/// A band of hue that the design system bans, and the saturation and lightness within which a
/// colour in the band reads as that hue.
#[derive(Clone, Copy, Debug)]
pub struct HueBan {
    pub name: &'static str,
    /// From and to, in degrees.
    pub hue: (f32, f32),
    /// Below this a colour reads as a grey.
    pub min_saturation: f32,
    /// From and to, 0 to 1.
    pub lightness: (f32, f32),
}

/// The banned hues.
pub const HUES: [HueBan; 3] = [
    HueBan {
        name: "teal or cyan",
        hue: (160.0, 200.0),
        min_saturation: 0.2,
        lightness: (0.0, 1.0),
    },
    HueBan {
        name: "brown",
        hue: (15.0, 45.0),
        min_saturation: 0.2,
        lightness: (0.0, 0.45),
    },
    HueBan {
        name: "green, which would stand against red",
        hue: (75.0, 160.0),
        min_saturation: 0.2,
        lightness: (0.0, 1.0),
    },
];

/// The largest corner radius: tags, buttons, switches, inputs and progress bars are square
/// but for this.
pub const MAX_RADIUS: Px = Px(2.0);

// Every theme's corner radius stays within it, or the crate does not build.
const _: () = assert!(crate::tokens::DARK.bp_surface_border_radius.0 <= MAX_RADIUS.0);
const _: () = assert!(crate::tokens::LIGHT.bp_surface_border_radius.0 <= MAX_RADIUS.0);

/// The ban a colour falls under, if any. A colour under 10% alpha is a wash too faint to read
/// as a hue.
pub fn banned(colour: Rgba) -> Option<&'static HueBan> {
    if colour.a() < 26 {
        return None;
    }
    let (h, s, l) = colour.hsl();
    HUES.iter().find(|b| {
        h >= b.hue.0
            && h < b.hue.1
            && s >= b.min_saturation
            && l >= b.lightness.0
            && l <= b.lightness.1
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{Intent, tokens::*};

    #[test]
    fn the_bans_catch_what_they_name() {
        assert_eq!(
            banned(Rgba(0x00a396ff)).map(|b| b.name),
            Some("teal or cyan")
        );
        assert_eq!(banned(Rgba(0x77450dff)).map(|b| b.name), Some("brown"));
        assert!(banned(Rgba(0x238551ff)).is_some(), "Blueprint's green");
        assert!(banned(Rgba(0xd41b6fff)).is_none(), "the accent");
        assert!(banned(Rgba(0xdcbb2aff)).is_none(), "the warning yellow");
    }

    /// Banned colours the design system has been told about, by theme, token and value. Each
    /// one fails the test below once its value changes, so none outlives its fix.
    const REPORTED: [(&str, &str, Rgba); 0] = [];

    #[test]
    fn no_token_colour_is_banned_in_any_theme() {
        let mut found: Vec<String> = Vec::new();
        for (theme, t) in THEMES {
            for (name, c) in t.colours() {
                let reported = REPORTED.iter().any(|r| (r.0, r.1, r.2) == (theme, name, c));
                if let Some(b) = banned(c).filter(|_| !reported) {
                    found.push(format!("{theme}: {name} {:08x} is {}", c.0, b.name));
                }
            }
        }
        assert!(found.is_empty(), "{}", found.join("\n"));
        for (theme, name, value) in REPORTED {
            let (_, t) = THEMES.iter().find(|(n, _)| *n == theme).expect("a theme");
            let now = t
                .colours()
                .into_iter()
                .find(|(n, _)| *n == name)
                .map(|(_, c)| c);
            assert_eq!(
                now,
                Some(value),
                "{theme}: {name} changed, so take it off REPORTED"
            );
        }
    }

    #[test]
    fn the_accent_is_the_rules_magenta_in_every_theme() {
        for (theme, t) in THEMES {
            assert_eq!(
                t.bp_intent_primary_rest,
                Rgba(0xd41b6fff),
                "{theme}: the accent as a fill"
            );
            for (what, c) in [
                ("focus ring", t.bp_emphasis_focus_color),
                ("accent as text", t.bp_typography_color_primary_rest),
            ] {
                let (h, s, _) = c.hsl();
                assert!(
                    (320.0..345.0).contains(&h) && s > 0.5,
                    "{theme}: the {what} {:08x} is not magenta",
                    c.0
                );
            }
        }
        assert_eq!(DARK.bp_typography_color_primary_rest, Rgba(0xf77cb3ff));
    }

    #[test]
    fn success_is_blue_and_danger_red_in_every_theme() {
        for (theme, t) in THEMES {
            let (h, ..) = t.bp_intent_success_rest.hsl();
            assert!((200.0..250.0).contains(&h), "{theme}: success hue {h}");
            let (h, ..) = t.bp_intent_danger_rest.hsl();
            assert!(!(15.0..345.0).contains(&h), "{theme}: danger hue {h}");
        }
    }

    #[test]
    fn every_status_has_a_word_an_icon_and_an_intent() {
        for s in STATUS {
            assert!(!s.word.is_empty() && !s.icon.is_empty(), "{s:?}");
        }
    }

    #[test]
    fn statuses_that_share_an_icon_share_their_meaning() {
        let mut by_icon: HashMap<&str, Intent> = HashMap::new();
        for s in STATUS {
            let intent = *by_icon.entry(s.icon).or_insert(s.intent);
            assert_eq!(
                intent, s.intent,
                "{} shares its icon with another meaning",
                s.name
            );
        }
    }

    #[test]
    fn every_stage_has_its_own_icon() {
        let mut icons: Vec<&str> = STAGES.iter().map(|s| s.icon).collect();
        icons.sort_unstable();
        icons.dedup();
        assert_eq!(icons.len(), STAGES.len());
    }
}
