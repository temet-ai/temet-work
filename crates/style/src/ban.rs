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
        assert!(banned(Rgba(0x5b21b6ff)).is_none(), "the violet accent");
        assert!(banned(Rgba(0xd41b6fff)).is_none(), "the magenta accent");
        assert!(banned(Rgba(0xdcbb2aff)).is_none(), "the warning yellow");
    }

    /// Banned colours the design system has been told about, by theme, token and value. Each
    /// one fails the test below once its value changes, so none outlives its fix.
    const REPORTED: [(&str, &str, Rgba); 0] = [];

    /// The hue edge of the one panel the person is in (CutPanel's `here`, Theme/Colours) runs
    /// through six hues, green among them; it marks where the person is and stands for no
    /// meaning, so the ban on green against red does not reach it.
    fn hue_edge(name: &str) -> bool {
        name.starts_with("--tm-here-")
    }

    #[test]
    fn no_token_colour_is_banned_in_any_theme() {
        let mut found: Vec<String> = Vec::new();
        for (theme, t) in THEMES {
            for (name, c) in t.colours().into_iter().filter(|(n, _)| !hue_edge(n)) {
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
    fn the_accent_is_its_setting_in_every_theme() {
        for (theme, t) in THEMES {
            let accent = [
                t.tm_accent_1,
                t.tm_accent_2,
                t.tm_accent_3,
                t.tm_accent_4,
                t.tm_accent_5,
                t.tm_accent_6,
            ];
            let ramp = [
                t.bp_palette_blue_1,
                t.bp_palette_blue_2,
                t.bp_palette_blue_3,
                t.bp_palette_blue_4,
                t.bp_palette_blue_5,
                t.bp_palette_blue_6,
            ];
            assert_eq!(ramp, accent, "{theme}: the blue ramp reads the accent");
            assert_eq!(
                t.bp_intent_primary_rest, t.tm_accent_3,
                "{theme}: the accent as a fill"
            );
            for (what, c) in [
                ("focus ring", t.bp_emphasis_focus_color),
                ("accent as text", t.bp_typography_color_primary_rest),
            ] {
                assert!(
                    accent.contains(&c),
                    "{theme}: the {what} {:08x} is not a step of the accent",
                    c.0
                );
            }
        }
        for step in 1..=6 {
            let name = format!("--tm-accent-{step}");
            assert!(SETTINGS.contains(&name.as_str()), "{name} is a setting");
        }
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

    /// Every colour a look draws with, named for the message.
    fn look_colours(l: &crate::Look) -> Vec<(String, Rgba)> {
        let mut out = vec![
            ("background".to_string(), l.background),
            ("text".to_string(), l.text),
        ];
        out.extend(l.border_colour.map(|c| ("border".to_string(), c)));
        out.extend(
            l.shadow
                .iter()
                .enumerate()
                .map(|(i, s)| (format!("shadow {i}"), s.colour)),
        );
        out
    }

    #[test]
    fn no_part_draws_a_banned_colour_or_a_round_corner_in_any_theme() {
        let mut found: Vec<String> = Vec::new();
        for (theme, t) in THEMES {
            for l in t.looks {
                let at = format!("{theme}: {} {} {:?}", l.component, l.variant, l.state);
                for (what, c) in look_colours(l) {
                    if let Some(b) = banned(c) {
                        found.push(format!("{at}: {what} {:08x} is {}", c.0, b.name));
                    }
                }
                if l.radius.0 > MAX_RADIUS.0 {
                    found.push(format!("{at}: a {} px corner", l.radius.0));
                }
            }
        }
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    #[test]
    fn every_theme_measures_the_same_parts_variants_and_states() {
        let keys = |t: &Tokens| -> Vec<(&str, &str, crate::State)> {
            let mut k: Vec<_> = t
                .looks
                .iter()
                .map(|l| (l.component, l.variant, l.state))
                .collect();
            k.sort_by_key(|(c, v, s)| (*c, *v, format!("{s:?}")));
            k
        };
        let (first, rest) = THEMES.split_first().expect("a theme");
        for (theme, t) in rest {
            assert_eq!(keys(t), keys(first.1), "{theme} and {}", first.0);
        }
    }

    #[test]
    fn every_setting_is_a_colour_of_every_theme() {
        for (theme, t) in THEMES {
            for name in SETTINGS {
                assert!(
                    t.colours().iter().any(|(n, _)| *n == name),
                    "{theme}: the setting {name} is not one of its colours"
                );
            }
        }
    }
}
