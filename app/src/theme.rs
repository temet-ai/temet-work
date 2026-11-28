//! The theme module: the one file in the app that reads `temet-work-style`. It holds the theme in
//! force, one of the design system's themes, and turns its tokens into GPUI values in two ways:
//!
//! - gpui-component's `Theme`, so the library's own widgets draw in the temet theme;
//! - the modules below, which the app's parts read: `colour`, `text`, `shape` and `motion` for
//!   components, `space` and `size` for composites (`docs/ui.md`). Each function names a role
//!   from the design system's Theme pages and returns the active theme's token for it.
//!
//! Every value here is a token; the roles follow the design system's Theme/Colours page. Some
//! library slots are a component's own look (a switch track, a slider thumb, a tab); until the
//! design system's token export carries each component's look, those take the nearest role.

use gpui::{App, Global, Hsla, Pixels, SharedString, px, rems, rgba};
use gpui_base::{Easing, Spring};
use gpui_component::{MotionTokens, Theme, ThemeColor, ThemeMode};
use temet_work_style::tokens::{self, Tokens};
use temet_work_style::{Curve, Rgba};

/// The design system's intents, and how StatusTag and StageTag draw each status and stage. The
/// parts name these through the theme module, the one file that reads `temet-work-style`.
pub use temet_work_style::{Intent, StageView, StatusView};

/// StatusTag's map: a status's word, icon and intent, by the design system's name for it.
pub fn status(name: &str) -> Option<&'static StatusView> {
    tokens::STATUS.iter().find(|s| s.name == name)
}

/// StageTag's map: a stage's icon, by its name.
pub fn stage(name: &str) -> Option<&'static StageView> {
    tokens::STAGES.iter().find(|s| s.name == name)
}

/// Every status in StatusTag's map, in the design system's order.
pub fn statuses() -> &'static [StatusView] {
    &tokens::STATUS
}

/// The design system's font files, by their path under `app/assets/fonts/`.
pub fn font_files() -> impl Iterator<Item = &'static str> {
    tokens::FONTS.iter().map(|f| f.file)
}

/// Every stage in StageTag's map, in the order an issue goes through them.
pub fn stages() -> &'static [StageView] {
    &tokens::STAGES
}

/// Whether a theme, by name, is the light one. Blueprint's styles have two modes; every other
/// theme builds on the dark one (Theme/Colours).
fn is_light(name: &str) -> bool {
    name == "light"
}

/// The theme in force: one of the design system's themes, by name, and its tokens. Set by
/// [`apply`]; the parts read it through the modules below.
#[derive(Clone, Copy, Debug)]
pub struct Active {
    /// The theme's name, as the design system names it.
    pub name: &'static str,
    tokens: Tokens,
}

impl Global for Active {}

/// The design system's themes, by name.
pub fn names() -> impl Iterator<Item = &'static str> {
    tokens::THEMES.iter().map(|(name, _)| *name)
}

/// The tokens of the theme in force.
fn active(cx: &App) -> &Tokens {
    &cx.global::<Active>().tokens
}

/// A token's colour as the library's colour.
fn c(token: Rgba) -> Hsla {
    rgba(token.0).into()
}

/// gpui-component's colour slots, each set from the token for its role. `base` matters only to
/// the tests: every slot is overwritten.
pub fn colours(t: &Tokens, base: ThemeColor) -> ThemeColor {
    let mut s = base;

    // Surfaces, edges and text (Theme/Colours: Surfaces, Text).
    let page = c(t.tm_surface_page);
    let panel = c(t.tm_surface_panel);
    let hover = c(t.tm_surface_hover);
    let raised = c(t.tm_surface_raised);
    let edge = c(t.tm_edge);
    let quiet = c(t.tm_edge_quiet);
    let strong = c(t.bp_surface_border_color_strong);
    let text = c(t.bp_typography_color_default_rest);
    let muted = c(t.bp_typography_color_muted);
    let clear = Hsla::transparent_black();

    // The accent, the selected row and the focus ring (Theme/Colours: Accent and attention).
    let accent_text = c(t.bp_typography_color_primary_rest);
    let selected = c(t.tm_selected_fill);
    let selected_line = c(t.tm_selected_line);
    let focus = c(t.bp_emphasis_focus_color);

    s.background = page;
    s.foreground = text;
    s.border = edge;
    s.input = strong;
    s.muted = raised;
    s.muted_foreground = muted;
    s.accent = hover;
    s.accent_foreground = text;
    s.overlay = c(t.bp_surface_color_code);
    s.window_border = edge;
    s.ring = focus;
    s.caret = accent_text;
    s.selection = selected;
    s.drag_border = focus;
    s.drop_target = selected;

    // Intents. The design system has no info intent; info takes the primary one.
    s.primary = c(t.bp_intent_primary_rest);
    s.primary_hover = c(t.bp_intent_primary_hover);
    s.primary_active = c(t.bp_intent_primary_active);
    s.primary_foreground = c(t.bp_intent_primary_foreground);
    s.info = s.primary;
    s.info_hover = s.primary_hover;
    s.info_active = s.primary_active;
    s.info_foreground = s.primary_foreground;
    s.success = c(t.bp_intent_success_rest);
    s.success_hover = c(t.bp_intent_success_hover);
    s.success_active = c(t.bp_intent_success_active);
    s.success_foreground = c(t.bp_intent_success_foreground);
    s.warning = c(t.bp_intent_warning_rest);
    s.warning_hover = c(t.bp_intent_warning_hover);
    s.warning_active = c(t.bp_intent_warning_active);
    s.warning_foreground = c(t.bp_intent_warning_foreground);
    s.danger = c(t.bp_intent_danger_rest);
    s.danger_hover = c(t.bp_intent_danger_hover);
    s.danger_active = c(t.bp_intent_danger_active);
    s.danger_foreground = c(t.bp_intent_danger_foreground);
    s.secondary = panel;
    s.secondary_hover = hover;
    s.secondary_active = raised;
    s.secondary_foreground = text;

    // Buttons: Button's solid variant in each intent.
    s.button = c(t.bp_intent_default_rest);
    s.button_hover = c(t.bp_intent_default_hover);
    s.button_active = c(t.bp_intent_default_active);
    s.button_foreground = c(t.bp_intent_default_foreground);
    s.button_primary = s.primary;
    s.button_primary_hover = s.primary_hover;
    s.button_primary_active = s.primary_active;
    s.button_primary_foreground = s.primary_foreground;
    s.button_info = s.info;
    s.button_info_hover = s.info_hover;
    s.button_info_active = s.info_active;
    s.button_info_foreground = s.info_foreground;
    s.button_success = s.success;
    s.button_success_hover = s.success_hover;
    s.button_success_active = s.success_active;
    s.button_success_foreground = s.success_foreground;
    s.button_warning = s.warning;
    s.button_warning_hover = s.warning_hover;
    s.button_warning_active = s.warning_active;
    s.button_warning_foreground = s.warning_foreground;
    s.button_danger = s.danger;
    s.button_danger_hover = s.danger_hover;
    s.button_danger_active = s.danger_active;
    s.button_danger_foreground = s.danger_foreground;
    s.button_secondary = s.secondary;
    s.button_secondary_hover = s.secondary_hover;
    s.button_secondary_active = s.secondary_active;
    s.button_secondary_foreground = s.secondary_foreground;

    // Links.
    s.link = accent_text;
    s.link_hover = c(t.bp_typography_color_primary_hover);
    s.link_active = c(t.bp_typography_color_primary_active);

    // Containers on panels: side columns, menus and popovers, title and status bars.
    s.sidebar = panel;
    s.sidebar_foreground = text;
    s.sidebar_border = edge;
    s.sidebar_accent = hover;
    s.sidebar_accent_foreground = text;
    s.sidebar_primary = s.primary;
    s.sidebar_primary_foreground = s.primary_foreground;
    s.popover = raised;
    s.popover_foreground = text;
    s.accordion = panel;
    s.group_box = panel;
    s.group_box_foreground = text;
    s.description_list_label = clear;
    s.description_list_label_foreground = muted;
    s.title_bar = panel;
    s.title_bar_border = edge;
    s.status_bar = panel;
    s.status_bar_border = edge;

    // Lists and tables. Rows are never striped; the selected row is an accent wash with a line
    // above and below (TableRow).
    s.list = clear;
    s.list_even = clear;
    s.list_head = panel;
    s.list_hover = hover;
    s.list_active = selected;
    s.list_active_border = selected_line;
    s.list_highlight_foreground = accent_text;
    s.table = clear;
    s.table_even = clear;
    s.table_head = clear;
    s.table_head_foreground = text;
    s.table_foot = clear;
    s.table_foot_foreground = text;
    s.table_hover = hover;
    s.table_active = selected;
    s.table_active_border = selected_line;
    s.table_row_border = quiet;

    // Tabs: the selected tab's word in the accent.
    s.tab = clear;
    s.tab_foreground = text;
    s.tab_active = clear;
    s.tab_active_foreground = accent_text;
    s.tab_bar = clear;
    s.tab_bar_segmented = hover;

    // Controls with a track: ProgressBar and Slider fill in the accent.
    s.progress_bar = s.primary;
    s.slider_bar = s.primary;
    s.slider_thumb = text;
    s.switch = raised;
    s.switch_thumb = text;
    s.skeleton = hover;

    // Scroll bars: an edge-coloured thumb on a clear track.
    s.scrollbar = clear;
    s.scrollbar_thumb = edge;
    s.scrollbar_thumb_hover = strong;

    // The library's base hues. The design system has no green, teal or cyan: those slots take
    // its blue, the success colour, so red never stands against green.
    s.red = c(t.bp_palette_red_3);
    s.red_light = c(t.bp_palette_red_5);
    s.yellow = c(t.bp_palette_orange_3);
    s.yellow_light = c(t.bp_palette_orange_5);
    s.magenta = c(t.bp_palette_blue_3);
    s.magenta_light = c(t.bp_palette_blue_5);
    s.blue = c(t.bp_palette_green_3);
    s.blue_light = c(t.bp_palette_green_5);
    s.green = s.blue;
    s.green_light = s.blue_light;
    s.cyan = s.blue;
    s.cyan_light = s.blue_light;

    // Charts. The map has no chart part and the design system no chart palette, so a chart
    // draws in the muted text colour until it has one. Rising and falling are success and
    // danger: blue against red.
    s.chart_1 = muted;
    s.chart_2 = muted;
    s.chart_3 = muted;
    s.chart_4 = muted;
    s.chart_5 = muted;
    s.chart_bullish = s.success;
    s.chart_bearish = s.danger;
    s.chart_grid = quiet;

    s
}

/// The library's movements, from the design system's motion tokens (Theme/Motion).
pub fn motion(t: &Tokens, base: MotionTokens) -> MotionTokens {
    let curve =
        |t: Curve, fallback: Easing| Easing::cubic_bezier(t.0, t.1, t.2, t.3).unwrap_or(fallback);
    let mut m = base;
    m.duration_instant = t.tm_d_state;
    m.duration_fast = t.tm_d_small;
    m.duration_normal = t.tm_d_panel;
    m.duration_slow = t.tm_d_stage;
    m.easing_enter = curve(t.tm_ease_out, m.easing_enter);
    m.easing_exit = curve(t.tm_ease_in, m.easing_exit);
    m.easing_move = curve(t.tm_ease_state, m.easing_move);
    m.spring_control = Spring::new(t.tm_d_small);
    m.spring_move = Spring::new(t.tm_d_panel);
    let body = t.tm_text_body.0;
    m.distance_short = rems(t.tm_rise.0 / body);
    m.distance_medium = rems(t.tm_slide.0 / body);
    m
}

/// The first family of a token's font stack that is installed. A stack also names CSS
/// generics, such as `sans-serif`, which match no installed font.
fn installed(stack: &[&str], fonts: &[String]) -> Option<SharedString> {
    stack
        .iter()
        .find(|f| fonts.iter().any(|name| name == *f))
        .map(|f| SharedString::from(f.to_string()))
}

/// Puts the design system's theme `name` in force: the parts' values and the library's theme.
/// Call it after `gpui_component::init`, and again to switch themes. An unknown name keeps the
/// theme in force and says so.
pub fn apply(cx: &mut App, name: &str) -> Result<(), String> {
    let Some((name, tokens)) = tokens::THEMES.iter().find(|(n, _)| *n == name) else {
        return Err(format!(
            "no theme named {name}; the design system has {}",
            names().collect::<Vec<_>>().join(", ")
        ));
    };
    let t = **tokens;
    cx.set_global(Active { name, tokens: t });
    let fonts = cx.text_system().all_font_names();
    Theme::update(cx, |lib| {
        lib.mode = if is_light(name) {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        lib.colors = colours(&t, lib.colors);
        lib.motion = motion(&t, lib.motion.clone());
        lib.radius = px(t.bp_surface_border_radius.0);
        lib.radius_lg = px(t.bp_surface_border_radius.0);
        lib.font_size = px(t.tm_text_body.0);
        lib.mono_font_size = px(t.tm_text_code.0);
        if let Some(family) = installed(t.tm_font_text, &fonts) {
            lib.font_family = family;
        }
        if let Some(family) = installed(t.tm_font_code, &fonts) {
            lib.mono_font_family = family;
        }
    });
    cx.refresh_windows();
    Ok(())
}

/// A size token as GPUI's size.
fn size_of(token: temet_work_style::Px) -> Pixels {
    px(token.0)
}

/// Colours by role (Theme/Colours). For components only.
pub mod colour {
    use gpui::{App, Hsla};

    use super::{Active, Intent, active, c, is_light};

    /// The shade over the page behind a modal dialog, such as an Alert (`--tm-backdrop`).
    pub fn backdrop(cx: &App) -> Hsla {
        c(active(cx).tm_backdrop)
    }
    /// The window's ground.
    pub fn page(cx: &App) -> Hsla {
        c(active(cx).tm_surface_page)
    }
    /// A panel on the ground: the side columns, a CutPanel's body.
    pub fn panel(cx: &App) -> Hsla {
        c(active(cx).tm_surface_panel)
    }
    /// A row or control under the pointer.
    pub fn hover(cx: &App) -> Hsla {
        c(active(cx).tm_surface_hover)
    }
    /// A surface raised above a panel: menus, popovers.
    pub fn raised(cx: &App) -> Hsla {
        c(active(cx).tm_surface_raised)
    }
    /// A panel's border.
    pub fn edge(cx: &App) -> Hsla {
        c(active(cx).tm_edge)
    }
    /// The rule between rows.
    pub fn edge_quiet(cx: &App) -> Hsla {
        c(active(cx).tm_edge_quiet)
    }
    /// Body text.
    pub fn text(cx: &App) -> Hsla {
        c(active(cx).bp_typography_color_default_rest)
    }
    /// Muted text: notes, meta lines, column heads.
    pub fn muted(cx: &App) -> Hsla {
        c(active(cx).bp_typography_color_muted)
    }
    /// The selected row's wash.
    pub fn selected_fill(cx: &App) -> Hsla {
        c(active(cx).tm_selected_fill)
    }
    /// The lines above and below the selected row.
    pub fn selected_line(cx: &App) -> Hsla {
        c(active(cx).tm_selected_line)
    }
    /// The focus ring.
    pub fn focus(cx: &App) -> Hsla {
        c(active(cx).bp_emphasis_focus_color)
    }
    /// What asks for the person: labels, borders and buttons. A theme setting.
    pub fn attention(cx: &App) -> Hsla {
        c(active(cx).tm_attention)
    }
    /// The six colours of the hue edge on the one panel the person is in (`--tm-here-1` to
    /// `-6`), in the order the edge runs through them.
    pub fn here(cx: &App) -> [Hsla; 6] {
        let t = active(cx);
        [
            t.tm_here_1,
            t.tm_here_2,
            t.tm_here_3,
            t.tm_here_4,
            t.tm_here_5,
            t.tm_here_6,
        ]
        .map(c)
    }
    /// A segmented control's ground: `.bp6-dark .bp6-segmented-control { background-color:
    /// var(--bp-palette-dark-gray-2) }`, and `var(--bp-palette-light-gray-5)` in Blueprint's
    /// light mode.
    pub fn segmented_ground(cx: &App) -> Hsla {
        let a = cx.global::<Active>();
        c(if is_light(a.name) {
            a.tokens.bp_palette_light_gray_5
        } else {
            a.tokens.bp_palette_dark_gray_2
        })
    }
    /// Text in an intent's colour, as a minimal tag or a link shows it.
    pub fn intent_text(cx: &App, intent: Intent) -> Hsla {
        let t = active(cx);
        c(match intent {
            Intent::None => t.bp_typography_color_default_rest,
            Intent::Primary => t.bp_typography_color_primary_rest,
            Intent::Success => t.bp_typography_color_success_rest,
            Intent::Warning => t.bp_typography_color_warning_rest,
            Intent::Danger => t.bp_typography_color_danger_rest,
        })
    }
    /// Disabled text: a disabled button's word, `--bp-typography-color-default-disabled`.
    pub fn disabled_text(cx: &App) -> Hsla {
        c(active(cx).bp_typography_color_default_disabled)
    }
    /// A disabled solid button's fill: `color-mix(in srgb, var(--bp-intent-default-disabled)
    /// 20%, transparent)`, and 4% in Blueprint's dark mode.
    pub fn disabled_fill(cx: &App) -> Hsla {
        let a = cx.global::<Active>();
        let fill = c(a.tokens.bp_intent_default_disabled);
        let share = if is_light(a.name) { 0.2 } else { 0.04 };
        Hsla {
            a: fill.a * share,
            ..fill
        }
    }
    /// A spinner's turning head and its track: `.bp6-spinner .bp6-spinner-head { stroke:
    /// color-mix(in srgb, var(--bp-palette-gray-1) 80%, transparent) }` and the track at 20%; in
    /// Blueprint's dark mode the head is `--bp-palette-gray-3` and the track black at 50%.
    pub fn spinner(cx: &App) -> (Hsla, Hsla) {
        let a = cx.global::<Active>();
        let mixed = |token, share: f32| {
            let colour = c(token);
            Hsla {
                a: colour.a * share,
                ..colour
            }
        };
        if is_light(a.name) {
            (
                mixed(a.tokens.bp_palette_gray_1, 0.8),
                mixed(a.tokens.bp_palette_gray_1, 0.2),
            )
        } else {
            (
                c(a.tokens.bp_palette_gray_3),
                mixed(a.tokens.bp_palette_black, 0.5),
            )
        }
    }
    /// An intent's solid fill, as a solid button or tag shows it.
    pub fn intent_fill(cx: &App, intent: Intent) -> Hsla {
        let t = active(cx);
        c(match intent {
            Intent::None => t.bp_intent_default_rest,
            Intent::Primary => t.bp_intent_primary_rest,
            Intent::Success => t.bp_intent_success_rest,
            Intent::Warning => t.bp_intent_warning_rest,
            Intent::Danger => t.bp_intent_danger_rest,
        })
    }
}

/// Type (Theme/Type). For components only.
pub mod text {
    use gpui::{App, DefiniteLength, FontWeight, Pixels, SharedString, relative};
    use gpui_component::Theme;

    use super::{active, size_of};

    /// Body text and controls.
    pub fn body(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_body)
    }
    /// Small text: meta lines, notes, tags.
    pub fn small(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_small)
    }
    /// Code and keys, in the mono face.
    pub fn code(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_code)
    }
    /// A section's heading.
    pub fn section(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_section)
    }
    /// A side pane's heading.
    pub fn pane(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_pane)
    }
    /// A screen's one title.
    pub fn page(cx: &App) -> Pixels {
        size_of(active(cx).tm_text_page)
    }
    /// The weight of every heading and of a strong row's title.
    pub fn heading_weight(cx: &App) -> FontWeight {
        FontWeight(active(cx).tm_weight_heading)
    }
    /// The weight of body text: a group row's count.
    pub fn normal_weight(cx: &App) -> FontWeight {
        FontWeight(active(cx).bp_typography_weight_default)
    }
    /// Blueprint's bold: a table's column heads, a tree's group labels.
    pub fn bold_weight(cx: &App) -> FontWeight {
        FontWeight(active(cx).bp_typography_weight_bold)
    }
    /// The line height of body text and controls.
    pub fn line_height(cx: &App) -> DefiniteLength {
        relative(active(cx).bp_typography_line_height_default)
    }
    /// Blueprint's small text, `--bp-typography-size-body-small`: a tag's word, a section's
    /// subtitle (`.bp6-text-small`).
    pub fn body_small(cx: &App) -> Pixels {
        size_of(active(cx).bp_typography_size_body_small)
    }
    /// The text face in force: the theme's `--tm-font-text`, or the system's face when it is
    /// not installed.
    pub fn family(cx: &App) -> SharedString {
        Theme::global(cx).font_family.clone()
    }
    /// The code face in force: the theme's `--tm-font-code`, or the system's mono face.
    pub fn code_family(cx: &App) -> SharedString {
        Theme::global(cx).mono_font_family.clone()
    }
}

/// Corners and borders (Theme/Colours, Rules). For components only.
pub mod shape {
    use gpui::{App, Pixels};

    use super::{active, size_of};

    /// Every corner: 2 px at most, no pills.
    pub fn radius(cx: &App) -> Pixels {
        size_of(active(cx).bp_surface_border_radius)
    }
    /// A border's width.
    pub fn border(cx: &App) -> Pixels {
        size_of(active(cx).bp_surface_border_width)
    }
    /// A CutPanel's cut corner.
    pub fn cut(cx: &App) -> Pixels {
        size_of(active(cx).tm_cut)
    }
}

/// Movements (Theme/Motion). For components only.
pub mod motion {
    use std::time::Duration;

    use gpui::App;

    use super::active;

    /// How long the hue edge of the panel the person is in takes to drift one way, before it
    /// drifts back (`--tm-d-drift`).
    pub fn drift(cx: &App) -> Duration {
        active(cx).tm_d_drift
    }
    /// The drift's pace, `--tm-ease-linear`: how far along it is for the share of its time gone,
    /// each from 0 to 1.
    pub fn drift_ease(cx: &App) -> impl Fn(f32) -> f32 + use<> {
        let c = active(cx).tm_ease_linear;
        gpui_base::animation::cubic_bezier(c.0, c.1, c.2, c.3)
    }
}

/// Spacing steps (`--tm-space-1` to `-6`). For components and composites.
pub mod space {
    use gpui::{App, Pixels};

    use super::{active, size_of};

    /// 2 px.
    pub fn s1(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_1)
    }
    /// 4 px.
    pub fn s2(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_2)
    }
    /// 6 px.
    pub fn s3(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_3)
    }
    /// 8 px.
    pub fn s4(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_4)
    }
    /// 12 px.
    pub fn s5(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_5)
    }
    /// 16 px.
    pub fn s6(cx: &App) -> Pixels {
        size_of(active(cx).tm_space_6)
    }
}

/// Layout sizes: columns, rows, labels. For components and composites.
pub mod size {
    use gpui::{App, Pixels, px, relative};

    use super::{active, size_of};

    /// The left column at a window this wide.
    pub fn col_left(cx: &App, window: Pixels) -> Pixels {
        let t = active(cx);
        size_of(if window < size_of(t.tm_screen_medium) {
            t.tm_col_left_narrow
        } else if window < size_of(t.tm_screen_wide) {
            t.tm_col_left_medium
        } else {
            t.tm_col_left_wide
        })
    }
    /// The right column at a window this wide.
    pub fn col_right(cx: &App, window: Pixels) -> Pixels {
        let t = active(cx);
        size_of(if window < size_of(t.tm_screen_medium) {
            t.tm_col_right_narrow
        } else if window < size_of(t.tm_screen_wide) {
            t.tm_col_right_medium
        } else {
            t.tm_col_right_wide
        })
    }
    /// A row in a side column.
    pub fn row_side(cx: &App) -> Pixels {
        size_of(active(cx).tm_row_side)
    }
    /// A label column, as in a PropertyList.
    pub fn label(cx: &App) -> Pixels {
        size_of(active(cx).tm_label_width)
    }
    /// A bar set inline at the end of a line, such as a milestone's progress.
    pub fn meter_inline(cx: &App) -> Pixels {
        size_of(active(cx).tm_meter_inline_width)
    }
    /// A PropertyList's label column: `--tm-label-width`, or `--tm-label-width-narrow`.
    pub fn label_narrow(cx: &App) -> Pixels {
        size_of(active(cx).tm_label_width_narrow)
    }
    /// The issue table's column widths, the same in every table.
    pub fn issue_columns(cx: &App) -> IssueColumns {
        IssueColumns::of(active(cx))
    }

    /// `n` of Blueprint's spacing steps, `--bp-surface-spacing`.
    fn steps(cx: &App, n: f32) -> Pixels {
        px(active(cx).bp_surface_spacing.0 * n)
    }

    // Blueprint 6 writes the sizes of its controls as multiples of `--bp-surface-spacing`; each
    // function below quotes the rule it follows.

    /// An icon: Blueprint's `IconSize.STANDARD`, the 16 px grid, four spacing steps.
    pub fn icon(cx: &App) -> Pixels {
        steps(cx, 4.)
    }
    /// A large icon: `IconSize.LARGE`, the 20 px grid, five spacing steps.
    pub fn icon_large(cx: &App) -> Pixels {
        steps(cx, 5.)
    }
    /// A segmented control's inset and the gap between its buttons: `.bp6-segmented-control {
    /// gap: calc(var(--bp-surface-spacing) * 0.5); padding: calc(var(--bp-surface-spacing) * 0.5) }`.
    pub fn segmented_inset(cx: &App) -> Pixels {
        steps(cx, 0.5)
    }
    /// An Alert's padding, and the space between a toast and the window's edge or the toast
    /// above it: `.bp6-alert { padding: calc(var(--bp-surface-spacing) * 5) }`,
    /// `.bp6-toast-container { padding: 0 calc(var(--bp-surface-spacing) * 5)
    /// calc(var(--bp-surface-spacing) * 5) }`.
    pub fn overlay_pad(cx: &App) -> Pixels {
        steps(cx, 5.)
    }
    /// An Alert's icon: `.bp6-alert-body .bp6-icon { font-size: $pt-icon-size-large * 2 }`.
    pub fn alert_icon(cx: &App) -> Pixels {
        icon_large(cx) * 2.
    }
    /// The space under a paragraph: Blueprint's reset, `p { margin-bottom:
    /// calc(var(--bp-surface-spacing) * 2) }`.
    pub fn paragraph_gap(cx: &App) -> Pixels {
        steps(cx, 2.)
    }
    /// An Alert's widest: `.bp6-alert { max-width: calc(var(--bp-surface-spacing) * 100) }`.
    pub fn alert_width(cx: &App) -> Pixels {
        steps(cx, 100.)
    }
    /// The space above an Alert's buttons: `.bp6-alert-footer { margin-top:
    /// calc(var(--bp-surface-spacing) * 3) }`; and around a toast's icon, written as `margin:
    /// 12px` in Blueprint's rules, three spacing steps.
    pub fn alert_footer_gap(cx: &App) -> Pixels {
        steps(cx, 3.)
    }
    /// The space between an Alert's buttons: `.bp6-alert-footer .bp6-button { margin-left:
    /// calc(var(--bp-surface-spacing) * 2) }`.
    pub fn alert_button_gap(cx: &App) -> Pixels {
        steps(cx, 2.)
    }
    /// A toast's narrowest and widest: `min-width: min(var(--bp-surface-spacing) * 75, 100%);
    /// max-width: min(var(--bp-surface-spacing) * 125, 100%)`.
    pub fn toast_width(cx: &App) -> (Pixels, Pixels) {
        (steps(cx, 75.), steps(cx, 125.))
    }
    /// A toast message's padding, written as `padding: 11px` in Blueprint's rules: the same as
    /// a table cell's side padding, `(--bp-surface-spacing * 10 - 18px) / 2`.
    pub fn toast_message_pad(cx: &App) -> Pixels {
        px((active(cx).bp_surface_spacing.0 * 10. - 18.) * 0.5)
    }
    /// The padding around a toast's close button, written as `padding: 5px` in Blueprint's
    /// rules: one and a quarter spacing steps.
    pub fn toast_button_pad(cx: &App) -> Pixels {
        steps(cx, 1.25)
    }
    /// A menu's padding: `.bp6-menu { padding: var(--bp-surface-spacing) }`; also the space
    /// between a popover and its target.
    pub fn menu_pad(cx: &App) -> Pixels {
        steps(cx, 1.)
    }
    /// A menu's narrowest: `.bp6-menu { min-width: calc(var(--bp-surface-spacing) * 45) }`.
    pub fn menu_width(cx: &App) -> Pixels {
        steps(cx, 45.)
    }
    /// A menu item's padding above and below, and at each side: `.bp6-menu-item { padding:
    /// var(--bp-surface-spacing) calc(var(--bp-surface-spacing) * 2) }`. The side padding is
    /// also the space after its icon and its text: `.bp6-menu-item > * { margin-right:
    /// calc(var(--bp-surface-spacing) * 2) }`.
    pub fn menu_item_pad(cx: &App) -> (Pixels, Pixels) {
        (steps(cx, 1.), steps(cx, 2.))
    }
    /// A menu item's line: `.bp6-menu-item { line-height: 22px }`, five and a half spacing steps.
    pub fn menu_item_line(cx: &App) -> Pixels {
        steps(cx, 5.5)
    }
    /// A tab's line: `.bp6-tab { line-height: 30px }`, seven and a half spacing steps.
    pub fn tab_line(cx: &App) -> Pixels {
        steps(cx, 7.5)
    }
    /// The space between tabs, and above the open tab's panel: `.bp6-tab-list { column-gap:
    /// calc(var(--bp-surface-spacing) * 5) }`, `.bp6-tab-panel { margin-top:
    /// calc(var(--bp-surface-spacing) * 5) }`.
    pub fn tab_gap(cx: &App) -> Pixels {
        steps(cx, 5.)
    }
    /// The open tab's line under its title: `box-shadow: inset 0 calc(var(--bp-surface-spacing)
    /// * 0.75 * -1) 0`.
    pub fn tab_indicator(cx: &App) -> Pixels {
        steps(cx, 0.75)
    }
    /// The space after a tag's icon: `.bp6-tag > * { margin-right: var(--bp-surface-spacing) }`.
    pub fn tag_gap(cx: &App) -> Pixels {
        steps(cx, 1.)
    }
    /// A tree row: `.bp6-tree-node-content { height: calc(var(--bp-surface-spacing) * 7.5) }`.
    pub fn tree_row(cx: &App) -> Pixels {
        steps(cx, 7.5)
    }
    /// A tree level's indent: `.bp6-tree-node-content-1 { padding-left:
    /// calc((var(--bp-surface-spacing) * 7.5 - var(--bp-surface-spacing) * 2) * 1) }`.
    pub fn tree_indent(cx: &App) -> Pixels {
        steps(cx, 7.5 - 2.)
    }
    /// The space after a tree row's icon: `.bp6-tree-node-icon { margin-right:
    /// calc(var(--bp-surface-spacing) * 2) }`.
    pub fn tree_icon_gap(cx: &App) -> Pixels {
        steps(cx, 2.)
    }
    /// A tree row's end padding, and its secondary label's padding at each side:
    /// `padding-right: var(--bp-surface-spacing)`, `padding: 0 var(--bp-surface-spacing)`.
    pub fn tree_end(cx: &App) -> Pixels {
        steps(cx, 1.)
    }
    /// A section header's least height: `min-height: calc(var(--bp-surface-spacing) * 12.5)`;
    /// compact, `calc(var(--bp-surface-spacing) * 10)`.
    pub fn section_head(cx: &App, compact: bool) -> Pixels {
        steps(cx, if compact { 10. } else { 12.5 })
    }
    /// A section header's side padding and a padded card's padding:
    /// `padding: 0 calc(var(--bp-surface-spacing) * 5)`; compact, `* 4`.
    pub fn section_pad(cx: &App, compact: bool) -> Pixels {
        steps(cx, if compact { 4. } else { 5. })
    }
    /// The space between a section header's two ends: `gap: calc(var(--bp-surface-spacing) * 5)`.
    pub fn section_gap(cx: &App) -> Pixels {
        steps(cx, 5.)
    }
    /// A section title's padding above and below, and the gap between the header's items:
    /// `.bp6-section-header-left { padding: calc(var(--bp-surface-spacing) * 2) 0 }`.
    pub fn section_title_pad(cx: &App) -> Pixels {
        steps(cx, 2.)
    }
    /// The space above a section's subtitle: `margin-top: calc(var(--bp-surface-spacing) * 0.5)`.
    pub fn section_subtitle_gap(cx: &App) -> Pixels {
        steps(cx, 0.5)
    }

    /// A wide dialog, such as RefineDialog: `--tm-dialog-wide`, at most the window less
    /// `--tm-space-6` at each side.
    pub fn dialog_wide(cx: &App, window: Pixels) -> Pixels {
        let t = active(cx);
        size_of(t.tm_dialog_wide).min(window - size_of(t.tm_space_6) * 2.)
    }
    /// The room a dialog leaves above and below it: `.bp6-dialog { margin:
    /// calc(var(--bp-surface-spacing) * 8) 0 }`.
    pub fn dialog_margin(cx: &App) -> Pixels {
        steps(cx, 8.)
    }
    /// A dialog body's tallest, in a window this tall: `.bp6-dialog-body-scroll-container {
    /// max-height: 70vh }`.
    pub fn dialog_body_max(window: Pixels) -> Pixels {
        window * 0.7
    }
    /// RefineDialog's two columns of fields: `.tm-refine-row { grid-template-columns: minmax(0,
    /// 3fr) minmax(0, 2fr) auto }`, as shares of the row. Laid out as flex bases that shrink by
    /// their share, they leave the same widths once the gaps and the Delete button are taken.
    pub fn refine_columns() -> (gpui::DefiniteLength, gpui::DefiniteLength) {
        let fr = |share: f32| relative(share);
        (fr(3. / 5.), fr(2. / 5.))
    }
    /// A dialog's header, at least: `.bp6-dialog-header { min-height: calc(30px +
    /// var(--bp-surface-spacing) * 2) }`, a medium button and a spacing step above and below it.
    pub fn dialog_header(cx: &App) -> Pixels {
        steps(cx, 7.5 + 2.)
    }
    /// The space after a dialog header's icon, and before it the icon's pull to the left:
    /// `.bp6-dialog-header .bp6-icon { margin-left: calc(var(--bp-surface-spacing) * -1);
    /// margin-right: calc(var(--bp-surface-spacing) * 4 * 0.5) }`.
    pub fn dialog_icon(cx: &App) -> (Pixels, Pixels) {
        (steps(cx, 1.), steps(cx, 2.))
    }
    /// The margin around a dialog's body: `.bp6-dialog-body { margin:
    /// calc(var(--bp-surface-spacing) * 4) }`.
    pub fn dialog_body(cx: &App) -> Pixels {
        steps(cx, 4.)
    }
    /// The space between a dialog footer's buttons: `.bp6-dialog-footer-actions .bp6-button {
    /// margin-left: calc(var(--bp-surface-spacing) * 2) }`.
    pub fn dialog_button_gap(cx: &App) -> Pixels {
        steps(cx, 2.)
    }
    /// A form group's space under it and its label's: `.bp6-form-group { margin: 0 0
    /// calc(var(--bp-surface-spacing) * 4) }`, `.bp6-form-group label.bp6-label { margin-bottom:
    /// var(--bp-surface-spacing) }`.
    pub fn form_group(cx: &App) -> (Pixels, Pixels) {
        (steps(cx, 4.), steps(cx, 1.))
    }
    /// A small spinner, as a loading button draws it: `SpinnerSize.SMALL`, five spacing steps.
    pub fn spinner_small(cx: &App) -> Pixels {
        steps(cx, 5.)
    }

    /// The issue table's column widths (`--tm-issue-col-*`). The title takes what is left, and
    /// needs at least `title_min` before the table shows its Agent column.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct IssueColumns {
        pub key: Pixels,
        pub stage: Pixels,
        pub agent: Pixels,
        pub updated: Pixels,
        pub title_min: Pixels,
    }

    impl IssueColumns {
        pub(super) fn of(t: &super::Tokens) -> Self {
            Self {
                key: size_of(t.tm_issue_col_key),
                stage: size_of(t.tm_issue_col_stage),
                agent: size_of(t.tm_issue_col_agent),
                updated: size_of(t.tm_issue_col_updated),
                title_min: size_of(t.tm_issue_col_title_min),
            }
        }
    }
}

/// The parts' looks, as the design system's stories draw them: each part, variant and state
/// measured by the design system's build. For components only. Ask with the part's name and its
/// variant's classes as string literals, as the design system names them
/// (`look::get(cx, "Tag", "minimal", State::Rest)`), so the theme's test can find every look the
/// app asks for and check that each theme has it.
pub mod look {
    use gpui::{App, BoxShadow, Edges, FontWeight, Hsla, Pixels, point};
    pub use temet_work_style::State;

    use super::{active, c, size_of};

    /// A look in GPUI's values.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Look {
        /// Fully transparent when the part has no fill.
        pub background: Hsla,
        pub text: Hsla,
        pub border: Edges<Pixels>,
        pub border_colour: Option<Hsla>,
        pub radius: Pixels,
        /// The height the story drew it at; a part whose height follows its content reads it
        /// only for a single line.
        pub height: Pixels,
        pub padding: Edges<Pixels>,
        pub font_size: Pixels,
        pub font_weight: FontWeight,
        pub shadow: Vec<BoxShadow>,
    }

    fn edges(e: [temet_work_style::Px; 4]) -> Edges<Pixels> {
        Edges {
            top: size_of(e[0]),
            right: size_of(e[1]),
            bottom: size_of(e[2]),
            left: size_of(e[3]),
        }
    }

    fn of(l: &temet_work_style::Look) -> Look {
        Look {
            background: c(l.background),
            text: c(l.text),
            border: edges(l.border),
            border_colour: l.border_colour.map(c),
            radius: size_of(l.radius),
            height: size_of(l.height),
            padding: edges(l.padding),
            font_size: size_of(l.font_size),
            font_weight: FontWeight(l.font_weight),
            shadow: l
                .shadow
                .iter()
                .map(|s| BoxShadow {
                    color: c(s.colour),
                    offset: point(size_of(s.x), size_of(s.y)),
                    blur_radius: size_of(s.blur),
                    spread_radius: size_of(s.spread),
                    inset: s.inset,
                })
                .collect(),
        }
    }

    /// The look of `component` in `variant` and `state` in the theme in force. A state the
    /// design system did not measure for that variant takes the variant's rest look. A variant
    /// it did not measure at all is a bug the theme's test catches; at run time it panics.
    pub fn get(cx: &App, component: &str, variant: &str, state: State) -> Look {
        let t = active(cx);
        let found = t
            .look(component, variant, state)
            .or_else(|| t.look(component, variant, State::Rest));
        match found {
            Some(l) => of(l),
            None => panic!("the design system has no look for {component} `{variant}`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};
    use temet_work_style::ban::banned;

    use super::*;

    /// The slots as JSON: each a name and an `#rrggbbaa` string.
    fn slots(colours: &ThemeColor) -> Map<String, Value> {
        match serde_json::to_value(colours).expect("the slots serialize") {
            Value::Object(map) => map,
            other => panic!("the slots serialize as {other}"),
        }
    }

    fn rgba_of(hex: &str) -> Rgba {
        let h = hex.trim_start_matches('#');
        let full = if h.len() == 6 {
            format!("{h}ff")
        } else {
            h.to_string()
        };
        Rgba(u32::from_str_radix(&full, 16).expect("a hex colour"))
    }

    #[test]
    fn every_slot_comes_from_a_token_in_every_theme() {
        // Fill every slot with one colour no token has, then set them: a slot left unset keeps it.
        let mut base = slots(&ThemeColor::default());
        for v in base.values_mut() {
            *v = Value::String("#01020304".into());
        }
        let marked: ThemeColor =
            serde_json::from_value(Value::Object(base)).expect("slots read back");
        for (theme, t) in tokens::THEMES {
            let from_marked = slots(&colours(t, marked));
            let from_default = slots(&colours(t, ThemeColor::default()));
            let unset: Vec<&String> = from_marked
                .keys()
                .filter(|k| from_marked[*k] != from_default[*k])
                .collect();
            assert!(
                unset.is_empty(),
                "{theme}: slots left at the library's own colour: {unset:?}"
            );
            assert!(
                from_marked.len() > 100,
                "only {} slots read",
                from_marked.len()
            );
        }
    }

    #[test]
    fn no_slot_breaks_the_ban_list_in_any_theme() {
        let mut found: Vec<String> = Vec::new();
        for (theme, t) in tokens::THEMES {
            for (slot, v) in slots(&colours(t, ThemeColor::default())) {
                let Some(hex) = v.as_str() else { continue };
                if let Some(b) = banned(rgba_of(hex)) {
                    found.push(format!("{theme}: {slot} {hex} is {}", b.name));
                }
            }
        }
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    #[test]
    fn slots_take_the_roles_on_the_colours_page() {
        for (theme, t) in tokens::THEMES {
            let s = slots(&colours(t, ThemeColor::default()));
            let hex = |slot: &str| rgba_of(s[slot].as_str().expect("a colour"));
            let roles = [
                ("background", t.tm_surface_page, "the page"),
                ("sidebar", t.tm_surface_panel, "side columns"),
                ("popover", t.tm_surface_raised, "menus and popovers"),
                ("primary", t.bp_intent_primary_rest, "the accent as a fill"),
                ("ring", t.bp_emphasis_focus_color, "the focus ring"),
                ("table_active", t.tm_selected_fill, "the selected row"),
                ("table_row_border", t.tm_edge_quiet, "rules between rows"),
            ];
            for (slot, token, role) in roles {
                assert_eq!(hex(slot), token, "{theme}: {role}");
            }
        }
    }

    #[test]
    fn motion_takes_the_design_systems_durations_and_curves() {
        for (theme, t) in tokens::THEMES {
            let m = motion(t, MotionTokens::default());
            assert_eq!(m.duration_fast, t.tm_d_small, "{theme}");
            assert_eq!(m.duration_normal, t.tm_d_panel, "{theme}");
            let Easing::CubicBezier { x1, y1, x2, y2 } = m.easing_enter else {
                panic!("{theme}: enter is not a curve: {:?}", m.easing_enter);
            };
            assert_eq!(Curve(x1, y1, x2, y2), t.tm_ease_out, "{theme}");
        }
    }

    #[test]
    fn the_light_theme_has_a_light_ground_and_dark_text() {
        let (_, l) = tokens::THEMES
            .iter()
            .find(|(n, _)| *n == "light")
            .expect("a light theme");
        let (.., ground) = l.tm_surface_page.hsl();
        let (.., ink) = l.bp_typography_color_default_rest.hsl();
        assert!(ground > 0.8 && ink < 0.3, "ground {ground}, text {ink}");
    }

    /// Every `look::get(cx, "Part", "variant", State::X)` in the app's source, as written.
    fn looks_asked() -> Vec<(String, String, String, String)> {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut out = Vec::new();
        let mut todo = vec![src];
        while let Some(dir) = todo.pop() {
            for e in std::fs::read_dir(&dir).expect("src reads").flatten() {
                let path = e.path();
                if path.is_dir() {
                    todo.push(path);
                    continue;
                }
                // Code only, with a call that rustfmt split over lines joined up again.
                let text: String = std::fs::read_to_string(&path)
                    .unwrap_or_default()
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .collect::<Vec<_>>()
                    .join(" ");
                for call in text.split("look::get(").skip(1) {
                    let args: Vec<&str> = call
                        .split(')')
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(str::trim)
                        .collect();
                    if let [_, part, variant, state, ..] = args[..]
                        && part.starts_with('"')
                        && variant.starts_with('"')
                    {
                        out.push((
                            path.display().to_string(),
                            part.trim_matches('"').to_string(),
                            variant.trim_matches('"').to_string(),
                            state.trim_start_matches("State::").to_string(),
                        ));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn every_look_the_app_asks_for_is_in_every_theme() {
        let mut missing = Vec::new();
        for (file, part, variant, _state) in looks_asked() {
            for (theme, t) in tokens::THEMES {
                if t.look(&part, &variant, look::State::Rest).is_none() {
                    missing.push(format!(
                        "{file}: {theme} has no look for {part} `{variant}`"
                    ));
                }
            }
        }
        assert!(missing.is_empty(), "{}", missing.join("\n"));
    }

    #[test]
    fn a_look_reads_the_measured_values() {
        let l = tokens::DARK
            .look("Tag", "minimal", look::State::Rest)
            .expect("Tag minimal");
        assert!(l.radius.0 <= 2.0 && l.font_size.0 > 0.0, "{l:?}");
    }

    #[test]
    fn a_font_stack_takes_its_first_installed_family() {
        let fonts = ["DejaVu Sans".to_string(), "Menlo".to_string()];
        assert_eq!(
            installed(&["ui-monospace", "Menlo", "DejaVu Sans"], &fonts).as_deref(),
            Some("Menlo")
        );
        assert_eq!(installed(&["sans-serif"], &fonts), None);
    }
}
