//! The style guide: the design system's tokens, the ban list, and the check used when reviewing
//! UI work. `docs/ui.md` says which part is built from which.
//!
//! The tokens come from the design system, @temet-ai/design, through `scripts/sync-tokens`. This
//! crate holds them as plain values; the front end turns them into its UI library's types.

use temet_work_core::Part;

pub mod ban;
pub mod review;
pub mod tokens;

/// The style part.
#[derive(Default)]
pub struct Style;

impl Part for Style {
    fn id(&self) -> &'static str {
        "style"
    }
}

/// A colour as `0xRRGGBBAA`, in sRGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub u32);

impl Rgba {
    pub const fn r(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub const fn g(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub const fn b(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub const fn a(self) -> u8 {
        self.0 as u8
    }

    /// Hue in degrees (0 to 360), saturation and lightness (0 to 1), ignoring alpha.
    pub fn hsl(self) -> (f32, f32, f32) {
        let [r, g, b] = [self.r(), self.g(), self.b()].map(|c| f32::from(c) / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let d = max - min;
        if d == 0.0 {
            return (0.0, 0.0, l);
        }
        let s = d / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == r {
            60.0 * ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        (h, s, l)
    }
}

/// A length in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Px(pub f32);

/// A cubic Bézier easing curve, as CSS writes it: `cubic-bezier(x1, y1, x2, y2)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve(pub f32, pub f32, pub f32, pub f32);

/// The design system's intents. In the temet theme success is blue, so red never stands
/// against green.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intent {
    None,
    Primary,
    Success,
    Warning,
    Danger,
}

/// How StatusTag draws one status: always an icon, a word and an intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatusView {
    /// The status as the design system names it, such as `inProgress`.
    pub name: &'static str,
    /// The word shown, unless the tag is given its own text.
    pub word: &'static str,
    /// The Blueprint icon, such as `tick-circle`.
    pub icon: &'static str,
    pub intent: Intent,
}

/// How StageTag draws one stage of an issue: its icon and name, in the default colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageView {
    pub name: &'static str,
    /// The Blueprint icon, such as `git-merge`.
    pub icon: &'static str,
}

/// One layer of a box shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// Drawn inside the box.
    pub inset: bool,
    pub x: Px,
    pub y: Px,
    pub blur: Px,
    pub spread: Px,
    pub colour: Rgba,
}

/// The state a part is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Rest,
    Hover,
    Active,
    Focus,
    Disabled,
}

/// A part as the design system's stories draw it, in one variant and state: what the design
/// system's build measured on the rendered element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// The part, as the design system names it: `Tag`, `ListRow`.
    pub component: &'static str,
    /// The element's variant classes, such as `intent-primary minimal` or `tm-row tm-row-compact`,
    /// or `default`.
    pub variant: &'static str,
    pub state: State,
    /// Fully transparent when the element has no fill.
    pub background: Rgba,
    pub text: Rgba,
    /// Border widths: top, right, bottom, left.
    pub border: [Px; 4],
    pub border_colour: Option<Rgba>,
    pub radius: Px,
    /// The rendered height. For an element whose height follows its content, it is the story's.
    pub height: Px,
    /// Top, right, bottom, left.
    pub padding: [Px; 4],
    pub font_size: Px,
    pub font_weight: f32,
    pub shadow: &'static [Shadow],
}

/// A font file the app embeds, under `app/assets/fonts/`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontFile {
    pub family: &'static str,
    pub weight: f32,
    /// Its path under `app/assets/fonts/`.
    pub file: &'static str,
    /// Its licence's path under `app/assets/fonts/`.
    pub licence: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_splits_into_channels() {
        let c = Rgba(0xd41b6f80);
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (0xd4, 0x1b, 0x6f, 0x80));
    }

    #[test]
    fn hsl_of_known_colours() {
        let near = |(h, s, l): (f32, f32, f32), want: (f32, f32, f32)| {
            (h - want.0).abs() < 1.0 && (s - want.1).abs() < 0.01 && (l - want.2).abs() < 0.01
        };
        assert!(near(Rgba(0xff0000ff).hsl(), (0.0, 1.0, 0.5)));
        assert!(near(Rgba(0x00ffffff).hsl(), (180.0, 1.0, 0.5)));
        assert!(near(Rgba(0x808080ff).hsl(), (0.0, 0.0, 0.502)));
        assert!(near(Rgba(0x2d72d2ff).hsl(), (215.0, 0.647, 0.5)));
    }
}
