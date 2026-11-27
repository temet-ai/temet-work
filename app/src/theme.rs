//! The theme module: the one file in the app that reads `temet-work-style`. It turns the design
//! system's tokens into gpui-component's `Theme`, so that the library's own widgets draw in the
//! temet theme. Every value here is a token; the roles follow the design system's Theme/Colours
//! page.
//!
//! Today the tokens are the dark theme's, as `scripts/sync-tokens` wrote them. When the design
//! system's token file brings its light theme and the settings a user may change, such as the
//! attention colour, `apply` takes the theme in force instead of the constants.
//!
//! Some slots are a component's own look (a switch track, a slider thumb, a tab). The design
//! system's token file will carry each component's look per variant and state; until then those
//! slots take the colour of the nearest role on the Colours page.

use gpui::{App, Hsla, SharedString, px, rems, rgba};
use gpui_base::{Easing, Spring};
use gpui_component::{MotionTokens, Theme, ThemeColor, ThemeMode};
use temet_work_style::{Curve, Rgba, tokens::*};

/// A token's colour as the library's colour.
fn c(token: Rgba) -> Hsla {
    rgba(token.0).into()
}

/// gpui-component's colour slots, each set from the token for its role. `base` matters only to
/// the tests: every slot is overwritten.
pub fn colours(base: ThemeColor) -> ThemeColor {
    let mut s = base;

    // Surfaces, edges and text (Theme/Colours: Surfaces, Text).
    let page = c(BP_PALETTE_BLACK);
    let panel = c(BP_PALETTE_DARK_GRAY_1);
    let hover = c(BP_PALETTE_DARK_GRAY_2);
    let raised = c(BP_PALETTE_DARK_GRAY_3);
    let edge = c(TM_EDGE);
    let quiet = c(TM_EDGE_QUIET);
    let strong = c(BP_SURFACE_BORDER_COLOR_STRONG);
    let text = c(BP_TYPOGRAPHY_COLOR_DEFAULT_REST);
    let muted = c(BP_TYPOGRAPHY_COLOR_MUTED);
    let clear = Hsla::transparent_black();

    // The accent, the selected row and the focus ring (Theme/Colours: Accent and attention).
    let accent_text = c(BP_TYPOGRAPHY_COLOR_PRIMARY_REST);
    let selected = c(TM_SELECTED_FILL);
    let selected_line = c(TM_SELECTED_LINE);
    let focus = c(BP_EMPHASIS_FOCUS_COLOR);

    s.background = page;
    s.foreground = text;
    s.border = edge;
    s.input = strong;
    s.muted = raised;
    s.muted_foreground = muted;
    s.accent = hover;
    s.accent_foreground = text;
    s.overlay = c(BP_SURFACE_COLOR_CODE);
    s.window_border = edge;
    s.ring = focus;
    s.caret = accent_text;
    s.selection = selected;
    s.drag_border = focus;
    s.drop_target = selected;

    // Intents. The design system has no info intent; info takes the primary one.
    s.primary = c(BP_INTENT_PRIMARY_REST);
    s.primary_hover = c(BP_INTENT_PRIMARY_HOVER);
    s.primary_active = c(BP_INTENT_PRIMARY_ACTIVE);
    s.primary_foreground = c(BP_INTENT_PRIMARY_FOREGROUND);
    s.info = s.primary;
    s.info_hover = s.primary_hover;
    s.info_active = s.primary_active;
    s.info_foreground = s.primary_foreground;
    s.success = c(BP_INTENT_SUCCESS_REST);
    s.success_hover = c(BP_INTENT_SUCCESS_HOVER);
    s.success_active = c(BP_INTENT_SUCCESS_ACTIVE);
    s.success_foreground = c(BP_INTENT_SUCCESS_FOREGROUND);
    s.warning = c(BP_INTENT_WARNING_REST);
    s.warning_hover = c(BP_INTENT_WARNING_HOVER);
    s.warning_active = c(BP_INTENT_WARNING_ACTIVE);
    s.warning_foreground = c(BP_INTENT_WARNING_FOREGROUND);
    s.danger = c(BP_INTENT_DANGER_REST);
    s.danger_hover = c(BP_INTENT_DANGER_HOVER);
    s.danger_active = c(BP_INTENT_DANGER_ACTIVE);
    s.danger_foreground = c(BP_INTENT_DANGER_FOREGROUND);
    s.secondary = panel;
    s.secondary_hover = hover;
    s.secondary_active = raised;
    s.secondary_foreground = text;

    // Buttons: Button's solid variant in each intent.
    s.button = c(BP_INTENT_DEFAULT_REST);
    s.button_hover = c(BP_INTENT_DEFAULT_HOVER);
    s.button_active = c(BP_INTENT_DEFAULT_ACTIVE);
    s.button_foreground = c(BP_INTENT_DEFAULT_FOREGROUND);
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
    s.link_hover = c(BP_TYPOGRAPHY_COLOR_PRIMARY_HOVER);
    s.link_active = c(BP_TYPOGRAPHY_COLOR_PRIMARY_ACTIVE);

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
    s.red = c(BP_PALETTE_RED_3);
    s.red_light = c(BP_PALETTE_RED_5);
    s.yellow = c(BP_PALETTE_ORANGE_3);
    s.yellow_light = c(BP_PALETTE_ORANGE_5);
    s.magenta = c(BP_PALETTE_BLUE_3);
    s.magenta_light = c(BP_PALETTE_BLUE_5);
    s.blue = c(BP_PALETTE_GREEN_3);
    s.blue_light = c(BP_PALETTE_GREEN_5);
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
pub fn motion(base: MotionTokens) -> MotionTokens {
    let curve =
        |t: Curve, fallback: Easing| Easing::cubic_bezier(t.0, t.1, t.2, t.3).unwrap_or(fallback);
    let mut m = base;
    m.duration_instant = TM_D_STATE;
    m.duration_fast = TM_D_SMALL;
    m.duration_normal = TM_D_PANEL;
    m.duration_slow = TM_D_STAGE;
    m.easing_enter = curve(TM_EASE_OUT, m.easing_enter);
    m.easing_exit = curve(TM_EASE_IN, m.easing_exit);
    m.easing_move = curve(TM_EASE_STATE, m.easing_move);
    m.spring_control = Spring::new(TM_D_SMALL);
    m.spring_move = Spring::new(TM_D_PANEL);
    let body = TM_TEXT_BODY.0;
    m.distance_short = rems(TM_RISE.0 / body);
    m.distance_medium = rems(TM_SLIDE.0 / body);
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

/// Sets the library's theme from the tokens. Call it once, after `gpui_component::init`.
pub fn apply(cx: &mut App) {
    let fonts = cx.text_system().all_font_names();
    Theme::update(cx, |t| {
        t.mode = ThemeMode::Dark;
        t.colors = colours(t.colors);
        t.motion = motion(t.motion.clone());
        t.radius = px(BP_SURFACE_BORDER_RADIUS.0);
        t.radius_lg = px(BP_SURFACE_BORDER_RADIUS.0);
        t.font_size = px(TM_TEXT_BODY.0);
        t.mono_font_size = px(TM_TEXT_CODE.0);
        if let Some(family) = installed(TM_FONT_TEXT, &fonts) {
            t.font_family = family;
        }
        if let Some(family) = installed(TM_FONT_CODE, &fonts) {
            t.mono_font_family = family;
        }
    });
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
    fn every_slot_comes_from_a_token() {
        // Fill every slot with one colour no token has, then set them: a slot left unset keeps it.
        let mut base = slots(&ThemeColor::default());
        for v in base.values_mut() {
            *v = Value::String("#01020304".into());
        }
        let marked: ThemeColor =
            serde_json::from_value(Value::Object(base)).expect("slots read back");
        let from_marked = slots(&colours(marked));
        let from_default = slots(&colours(ThemeColor::default()));
        let unset: Vec<&String> = from_marked
            .keys()
            .filter(|k| from_marked[*k] != from_default[*k])
            .collect();
        assert!(
            unset.is_empty(),
            "slots left at the library's own colour: {unset:?}"
        );
        assert!(
            from_marked.len() > 100,
            "only {} slots read",
            from_marked.len()
        );
    }

    #[test]
    fn no_slot_breaks_the_ban_list() {
        let found: Vec<String> = slots(&colours(ThemeColor::default()))
            .into_iter()
            .filter_map(|(slot, v)| {
                let hex = v.as_str()?.to_string();
                banned(rgba_of(&hex)).map(|b| format!("{slot} {hex} is {}", b.name))
            })
            .collect();
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    #[test]
    fn slots_take_the_roles_on_the_colours_page() {
        let s = slots(&colours(ThemeColor::default()));
        let hex = |slot: &str| rgba_of(s[slot].as_str().expect("a colour"));
        assert_eq!(hex("background"), BP_PALETTE_BLACK, "the page");
        assert_eq!(hex("sidebar"), BP_PALETTE_DARK_GRAY_1, "side columns");
        assert_eq!(hex("popover"), BP_PALETTE_DARK_GRAY_3, "menus and popovers");
        assert_eq!(
            hex("primary"),
            BP_INTENT_PRIMARY_REST,
            "the accent as a fill"
        );
        assert_eq!(hex("ring"), BP_EMPHASIS_FOCUS_COLOR, "the focus ring");
        assert_eq!(hex("table_active"), TM_SELECTED_FILL, "the selected row");
        assert_eq!(hex("table_row_border"), TM_EDGE_QUIET, "rules between rows");
    }

    #[test]
    fn motion_takes_the_design_systems_durations_and_curves() {
        let m = motion(MotionTokens::default());
        assert_eq!(m.duration_fast, TM_D_SMALL);
        assert_eq!(m.duration_normal, TM_D_PANEL);
        let Easing::CubicBezier { x1, y1, x2, y2 } = m.easing_enter else {
            panic!("enter is not a curve: {:?}", m.easing_enter);
        };
        assert_eq!(Curve(x1, y1, x2, y2), TM_EASE_OUT);
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
