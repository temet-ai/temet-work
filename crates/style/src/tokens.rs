//! The design system's tokens, from @temet-ai/design 0.2.0, commit db51caf (looks measured at fedd337).
//! Written by `scripts/sync-tokens` from the design system's dist/tokens.json; do not edit, run
//! the script again.
//!
//! A theme is one [`Tokens`] value with a field per token, named after it: `--tm-edge-quiet` is
//! `tm_edge_quiet`. Blueprint's ramp names hold other hues in the temet theme: blue holds the
//! accent (`--tm-accent-1` to `-6`), green holds blue (success) and orange holds yellow (warning).
//! Each value is the theme's default; the user's settings win over it, for the tokens in
//! [`SETTINGS`].
//!
//! A theme's `looks` are its parts as the design system's stories draw them, measured per
//! component, variant and state. A variant is named by the classes of the element and of its
//! part's root or nearest tm-* ancestor, sorted (`intent-primary minimal tm-row-end tm-status`),
//! with the sub-element first where a part has several (`th: default`).

use std::time::Duration;

use crate::{Curve, FontFile, Intent, Look, Px, Rgba, Shadow, StageView, State, StatusView};

/// One theme: a value for every token, and its parts' measured looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    /// `--bp-emphasis-ease-bounce`
    pub bp_emphasis_ease_bounce: Curve,
    /// `--bp-emphasis-ease-default`
    pub bp_emphasis_ease_default: Curve,
    /// `--bp-emphasis-focus-color`
    pub bp_emphasis_focus_color: Rgba,
    /// `--bp-emphasis-focus-offset`
    pub bp_emphasis_focus_offset: Px,
    /// `--bp-emphasis-focus-width`
    pub bp_emphasis_focus_width: Px,
    /// `--bp-emphasis-motion-reduced`
    pub bp_emphasis_motion_reduced: f32,
    /// `--bp-emphasis-transition-duration`
    pub bp_emphasis_transition_duration: Duration,
    /// `--bp-intent-danger-active`
    pub bp_intent_danger_active: Rgba,
    /// `--bp-intent-danger-disabled`
    pub bp_intent_danger_disabled: Rgba,
    /// `--bp-intent-danger-foreground`
    pub bp_intent_danger_foreground: Rgba,
    /// `--bp-intent-danger-hover`
    pub bp_intent_danger_hover: Rgba,
    /// `--bp-intent-danger-rest`
    pub bp_intent_danger_rest: Rgba,
    /// `--bp-intent-default-active`
    pub bp_intent_default_active: Rgba,
    /// `--bp-intent-default-disabled`
    pub bp_intent_default_disabled: Rgba,
    /// `--bp-intent-default-foreground`
    pub bp_intent_default_foreground: Rgba,
    /// `--bp-intent-default-hover`
    pub bp_intent_default_hover: Rgba,
    /// `--bp-intent-default-rest`
    pub bp_intent_default_rest: Rgba,
    /// `--bp-intent-primary-active`
    pub bp_intent_primary_active: Rgba,
    /// `--bp-intent-primary-disabled`
    pub bp_intent_primary_disabled: Rgba,
    /// `--bp-intent-primary-foreground`
    pub bp_intent_primary_foreground: Rgba,
    /// `--bp-intent-primary-hover`
    pub bp_intent_primary_hover: Rgba,
    /// `--bp-intent-primary-rest`
    pub bp_intent_primary_rest: Rgba,
    /// `--bp-intent-success-active`
    pub bp_intent_success_active: Rgba,
    /// `--bp-intent-success-disabled`
    pub bp_intent_success_disabled: Rgba,
    /// `--bp-intent-success-foreground`
    pub bp_intent_success_foreground: Rgba,
    /// `--bp-intent-success-hover`
    pub bp_intent_success_hover: Rgba,
    /// `--bp-intent-success-rest`
    pub bp_intent_success_rest: Rgba,
    /// `--bp-intent-warning-active`
    pub bp_intent_warning_active: Rgba,
    /// `--bp-intent-warning-disabled`
    pub bp_intent_warning_disabled: Rgba,
    /// `--bp-intent-warning-foreground`
    pub bp_intent_warning_foreground: Rgba,
    /// `--bp-intent-warning-hover`
    pub bp_intent_warning_hover: Rgba,
    /// `--bp-intent-warning-rest`
    pub bp_intent_warning_rest: Rgba,
    /// `--bp-palette-black`
    pub bp_palette_black: Rgba,
    /// `--bp-palette-blue-1`
    pub bp_palette_blue_1: Rgba,
    /// `--bp-palette-blue-2`
    pub bp_palette_blue_2: Rgba,
    /// `--bp-palette-blue-3`
    pub bp_palette_blue_3: Rgba,
    /// `--bp-palette-blue-4`
    pub bp_palette_blue_4: Rgba,
    /// `--bp-palette-blue-5`
    pub bp_palette_blue_5: Rgba,
    /// `--bp-palette-blue-6`
    pub bp_palette_blue_6: Rgba,
    /// `--bp-palette-dark-gray-1`
    pub bp_palette_dark_gray_1: Rgba,
    /// `--bp-palette-dark-gray-2`
    pub bp_palette_dark_gray_2: Rgba,
    /// `--bp-palette-dark-gray-3`
    pub bp_palette_dark_gray_3: Rgba,
    /// `--bp-palette-dark-gray-4`
    pub bp_palette_dark_gray_4: Rgba,
    /// `--bp-palette-dark-gray-5`
    pub bp_palette_dark_gray_5: Rgba,
    /// `--bp-palette-gray-1`
    pub bp_palette_gray_1: Rgba,
    /// `--bp-palette-gray-2`
    pub bp_palette_gray_2: Rgba,
    /// `--bp-palette-gray-3`
    pub bp_palette_gray_3: Rgba,
    /// `--bp-palette-gray-4`
    pub bp_palette_gray_4: Rgba,
    /// `--bp-palette-gray-5`
    pub bp_palette_gray_5: Rgba,
    /// `--bp-palette-green-1`
    pub bp_palette_green_1: Rgba,
    /// `--bp-palette-green-2`
    pub bp_palette_green_2: Rgba,
    /// `--bp-palette-green-3`
    pub bp_palette_green_3: Rgba,
    /// `--bp-palette-green-4`
    pub bp_palette_green_4: Rgba,
    /// `--bp-palette-green-5`
    pub bp_palette_green_5: Rgba,
    /// `--bp-palette-green-6`
    pub bp_palette_green_6: Rgba,
    /// `--bp-palette-light-gray-1`
    pub bp_palette_light_gray_1: Rgba,
    /// `--bp-palette-light-gray-2`
    pub bp_palette_light_gray_2: Rgba,
    /// `--bp-palette-light-gray-3`
    pub bp_palette_light_gray_3: Rgba,
    /// `--bp-palette-light-gray-4`
    pub bp_palette_light_gray_4: Rgba,
    /// `--bp-palette-light-gray-5`
    pub bp_palette_light_gray_5: Rgba,
    /// `--bp-palette-orange-1`
    pub bp_palette_orange_1: Rgba,
    /// `--bp-palette-orange-2`
    pub bp_palette_orange_2: Rgba,
    /// `--bp-palette-orange-3`
    pub bp_palette_orange_3: Rgba,
    /// `--bp-palette-orange-4`
    pub bp_palette_orange_4: Rgba,
    /// `--bp-palette-orange-5`
    pub bp_palette_orange_5: Rgba,
    /// `--bp-palette-orange-6`
    pub bp_palette_orange_6: Rgba,
    /// `--bp-palette-red-1`
    pub bp_palette_red_1: Rgba,
    /// `--bp-palette-red-2`
    pub bp_palette_red_2: Rgba,
    /// `--bp-palette-red-3`
    pub bp_palette_red_3: Rgba,
    /// `--bp-palette-red-4`
    pub bp_palette_red_4: Rgba,
    /// `--bp-palette-red-5`
    pub bp_palette_red_5: Rgba,
    /// `--bp-palette-red-6`
    pub bp_palette_red_6: Rgba,
    /// `--bp-palette-white`
    pub bp_palette_white: Rgba,
    /// `--bp-surface-background-color-danger-active`
    pub bp_surface_background_color_danger_active: Rgba,
    /// `--bp-surface-background-color-danger-disabled`
    pub bp_surface_background_color_danger_disabled: Rgba,
    /// `--bp-surface-background-color-danger-hover`
    pub bp_surface_background_color_danger_hover: Rgba,
    /// `--bp-surface-background-color-danger-rest`
    pub bp_surface_background_color_danger_rest: Rgba,
    /// `--bp-surface-background-color-default-active`
    pub bp_surface_background_color_default_active: Rgba,
    /// `--bp-surface-background-color-default-disabled`
    pub bp_surface_background_color_default_disabled: Rgba,
    /// `--bp-surface-background-color-default-hover`
    pub bp_surface_background_color_default_hover: Rgba,
    /// `--bp-surface-background-color-default-rest`
    pub bp_surface_background_color_default_rest: Rgba,
    /// `--bp-surface-background-color-primary-active`
    pub bp_surface_background_color_primary_active: Rgba,
    /// `--bp-surface-background-color-primary-disabled`
    pub bp_surface_background_color_primary_disabled: Rgba,
    /// `--bp-surface-background-color-primary-hover`
    pub bp_surface_background_color_primary_hover: Rgba,
    /// `--bp-surface-background-color-primary-rest`
    pub bp_surface_background_color_primary_rest: Rgba,
    /// `--bp-surface-background-color-success-active`
    pub bp_surface_background_color_success_active: Rgba,
    /// `--bp-surface-background-color-success-disabled`
    pub bp_surface_background_color_success_disabled: Rgba,
    /// `--bp-surface-background-color-success-hover`
    pub bp_surface_background_color_success_hover: Rgba,
    /// `--bp-surface-background-color-success-rest`
    pub bp_surface_background_color_success_rest: Rgba,
    /// `--bp-surface-background-color-warning-active`
    pub bp_surface_background_color_warning_active: Rgba,
    /// `--bp-surface-background-color-warning-disabled`
    pub bp_surface_background_color_warning_disabled: Rgba,
    /// `--bp-surface-background-color-warning-hover`
    pub bp_surface_background_color_warning_hover: Rgba,
    /// `--bp-surface-background-color-warning-rest`
    pub bp_surface_background_color_warning_rest: Rgba,
    /// `--bp-surface-border-color-default`
    pub bp_surface_border_color_default: Rgba,
    /// `--bp-surface-border-color-strong`
    pub bp_surface_border_color_strong: Rgba,
    /// `--bp-surface-border-radius`
    pub bp_surface_border_radius: Px,
    /// `--bp-surface-border-width`
    pub bp_surface_border_width: Px,
    /// `--bp-surface-color-code`
    pub bp_surface_color_code: Rgba,
    /// `--bp-surface-layer-color-danger`
    pub bp_surface_layer_color_danger: Rgba,
    /// `--bp-surface-layer-color-default`
    pub bp_surface_layer_color_default: Rgba,
    /// `--bp-surface-layer-color-primary`
    pub bp_surface_layer_color_primary: Rgba,
    /// `--bp-surface-layer-color-success`
    pub bp_surface_layer_color_success: Rgba,
    /// `--bp-surface-layer-color-warning`
    pub bp_surface_layer_color_warning: Rgba,
    /// `--bp-surface-layer-danger`
    pub bp_surface_layer_danger: Rgba,
    /// `--bp-surface-layer-default`
    pub bp_surface_layer_default: Rgba,
    /// `--bp-surface-layer-opacity`
    pub bp_surface_layer_opacity: f32,
    /// `--bp-surface-layer-primary`
    pub bp_surface_layer_primary: Rgba,
    /// `--bp-surface-layer-success`
    pub bp_surface_layer_success: Rgba,
    /// `--bp-surface-layer-warning`
    pub bp_surface_layer_warning: Rgba,
    /// `--bp-surface-shadow-0`
    pub bp_surface_shadow_0: &'static [Shadow],
    /// `--bp-surface-shadow-1`
    pub bp_surface_shadow_1: &'static [Shadow],
    /// `--bp-surface-shadow-2`
    pub bp_surface_shadow_2: &'static [Shadow],
    /// `--bp-surface-shadow-3`
    pub bp_surface_shadow_3: &'static [Shadow],
    /// `--bp-surface-shadow-4`
    pub bp_surface_shadow_4: &'static [Shadow],
    /// `--bp-surface-spacing`
    pub bp_surface_spacing: Px,
    /// `--bp-surface-z-index-0`
    pub bp_surface_z_index_0: f32,
    /// `--bp-surface-z-index-1`
    pub bp_surface_z_index_1: f32,
    /// `--bp-surface-z-index-2`
    pub bp_surface_z_index_2: f32,
    /// `--bp-surface-z-index-3`
    pub bp_surface_z_index_3: f32,
    /// `--bp-surface-z-index-4`
    pub bp_surface_z_index_4: f32,
    /// `--bp-typography-color-danger-active`
    pub bp_typography_color_danger_active: Rgba,
    /// `--bp-typography-color-danger-disabled`
    pub bp_typography_color_danger_disabled: Rgba,
    /// `--bp-typography-color-danger-hover`
    pub bp_typography_color_danger_hover: Rgba,
    /// `--bp-typography-color-danger-rest`
    pub bp_typography_color_danger_rest: Rgba,
    /// `--bp-typography-color-default-active`
    pub bp_typography_color_default_active: Rgba,
    /// `--bp-typography-color-default-disabled`
    pub bp_typography_color_default_disabled: Rgba,
    /// `--bp-typography-color-default-hover`
    pub bp_typography_color_default_hover: Rgba,
    /// `--bp-typography-color-default-rest`
    pub bp_typography_color_default_rest: Rgba,
    /// `--bp-typography-color-muted`
    pub bp_typography_color_muted: Rgba,
    /// `--bp-typography-color-primary-active`
    pub bp_typography_color_primary_active: Rgba,
    /// `--bp-typography-color-primary-disabled`
    pub bp_typography_color_primary_disabled: Rgba,
    /// `--bp-typography-color-primary-hover`
    pub bp_typography_color_primary_hover: Rgba,
    /// `--bp-typography-color-primary-rest`
    pub bp_typography_color_primary_rest: Rgba,
    /// `--bp-typography-color-success-active`
    pub bp_typography_color_success_active: Rgba,
    /// `--bp-typography-color-success-disabled`
    pub bp_typography_color_success_disabled: Rgba,
    /// `--bp-typography-color-success-hover`
    pub bp_typography_color_success_hover: Rgba,
    /// `--bp-typography-color-success-rest`
    pub bp_typography_color_success_rest: Rgba,
    /// `--bp-typography-color-warning-active`
    pub bp_typography_color_warning_active: Rgba,
    /// `--bp-typography-color-warning-disabled`
    pub bp_typography_color_warning_disabled: Rgba,
    /// `--bp-typography-color-warning-hover`
    pub bp_typography_color_warning_hover: Rgba,
    /// `--bp-typography-color-warning-rest`
    pub bp_typography_color_warning_rest: Rgba,
    /// `--bp-typography-family-default`
    pub bp_typography_family_default: &'static [&'static str],
    /// `--bp-typography-family-mono`
    pub bp_typography_family_mono: &'static [&'static str],
    /// `--bp-typography-line-height-default`
    pub bp_typography_line_height_default: f32,
    /// `--bp-typography-line-height-large`
    pub bp_typography_line_height_large: f32,
    /// `--bp-typography-size-body-large`
    pub bp_typography_size_body_large: Px,
    /// `--bp-typography-size-body-medium`
    pub bp_typography_size_body_medium: Px,
    /// `--bp-typography-size-body-small`
    pub bp_typography_size_body_small: Px,
    /// `--bp-typography-size-body-x-small`
    pub bp_typography_size_body_x_small: Px,
    /// `--bp-typography-size-code-large`
    pub bp_typography_size_code_large: Px,
    /// `--bp-typography-size-code-medium`
    pub bp_typography_size_code_medium: Px,
    /// `--bp-typography-size-code-small`
    pub bp_typography_size_code_small: Px,
    /// `--bp-typography-size-heading-display`
    pub bp_typography_size_heading_display: Px,
    /// `--bp-typography-size-heading-large`
    pub bp_typography_size_heading_large: Px,
    /// `--bp-typography-size-heading-medium`
    pub bp_typography_size_heading_medium: Px,
    /// `--bp-typography-size-heading-small`
    pub bp_typography_size_heading_small: Px,
    /// `--bp-typography-size-heading-x-large`
    pub bp_typography_size_heading_x_large: Px,
    /// `--bp-typography-weight-bold`
    pub bp_typography_weight_bold: f32,
    /// `--bp-typography-weight-default`
    pub bp_typography_weight_default: f32,
    /// `--tm-accent-1`
    pub tm_accent_1: Rgba,
    /// `--tm-accent-2`
    pub tm_accent_2: Rgba,
    /// `--tm-accent-3`
    pub tm_accent_3: Rgba,
    /// `--tm-accent-4`
    pub tm_accent_4: Rgba,
    /// `--tm-accent-5`
    pub tm_accent_5: Rgba,
    /// `--tm-accent-6`
    pub tm_accent_6: Rgba,
    /// `--tm-attention`
    pub tm_attention: Rgba,
    /// `--tm-backdrop`
    pub tm_backdrop: Rgba,
    /// `--tm-board-col-empty`
    pub tm_board_col_empty: Px,
    /// `--tm-board-col-min`
    pub tm_board_col_min: Px,
    /// `--tm-col-left-medium`
    pub tm_col_left_medium: Px,
    /// `--tm-col-left-narrow`
    pub tm_col_left_narrow: Px,
    /// `--tm-col-left-wide`
    pub tm_col_left_wide: Px,
    /// `--tm-col-right-medium`
    pub tm_col_right_medium: Px,
    /// `--tm-col-right-narrow`
    pub tm_col_right_narrow: Px,
    /// `--tm-col-right-wide`
    pub tm_col_right_wide: Px,
    /// `--tm-cut`
    pub tm_cut: Px,
    /// `--tm-d-drift`
    pub tm_d_drift: Duration,
    /// `--tm-d-loop`
    pub tm_d_loop: Duration,
    /// `--tm-d-panel`
    pub tm_d_panel: Duration,
    /// `--tm-d-small`
    pub tm_d_small: Duration,
    /// `--tm-d-stage`
    pub tm_d_stage: Duration,
    /// `--tm-d-stagger`
    pub tm_d_stagger: Duration,
    /// `--tm-d-state`
    pub tm_d_state: Duration,
    /// `--tm-d-toast`
    pub tm_d_toast: Duration,
    /// `--tm-dialog-wide`
    pub tm_dialog_wide: Px,
    /// `--tm-ease-in`
    pub tm_ease_in: Curve,
    /// `--tm-ease-linear`
    pub tm_ease_linear: Curve,
    /// `--tm-ease-out`
    pub tm_ease_out: Curve,
    /// `--tm-ease-spatial`
    pub tm_ease_spatial: Curve,
    /// `--tm-ease-state`
    pub tm_ease_state: Curve,
    /// `--tm-edge`
    pub tm_edge: Rgba,
    /// `--tm-edge-quiet`
    pub tm_edge_quiet: Rgba,
    /// `--tm-font-code`
    pub tm_font_code: &'static [&'static str],
    /// `--tm-font-figure`
    pub tm_font_figure: &'static [&'static str],
    /// `--tm-font-text`
    pub tm_font_text: &'static [&'static str],
    /// `--tm-here-1`
    pub tm_here_1: Rgba,
    /// `--tm-here-2`
    pub tm_here_2: Rgba,
    /// `--tm-here-3`
    pub tm_here_3: Rgba,
    /// `--tm-here-4`
    pub tm_here_4: Rgba,
    /// `--tm-here-5`
    pub tm_here_5: Rgba,
    /// `--tm-here-6`
    pub tm_here_6: Rgba,
    /// `--tm-here-width`
    pub tm_here_width: Px,
    /// `--tm-issue-col-agent`
    pub tm_issue_col_agent: Px,
    /// `--tm-issue-col-key`
    pub tm_issue_col_key: Px,
    /// `--tm-issue-col-milestone`
    pub tm_issue_col_milestone: Px,
    /// `--tm-issue-col-stage`
    pub tm_issue_col_stage: Px,
    /// `--tm-issue-col-title-min`
    pub tm_issue_col_title_min: Px,
    /// `--tm-issue-col-title-narrow`
    pub tm_issue_col_title_narrow: Px,
    /// `--tm-issue-col-updated`
    pub tm_issue_col_updated: Px,
    /// `--tm-label-width`
    pub tm_label_width: Px,
    /// `--tm-label-width-narrow`
    pub tm_label_width_narrow: Px,
    /// `--tm-leading-page`
    pub tm_leading_page: f32,
    /// `--tm-leading-pane`
    pub tm_leading_pane: f32,
    /// `--tm-meter-inline-width`
    pub tm_meter_inline_width: Px,
    /// `--tm-needs-col-cost`
    pub tm_needs_col_cost: Px,
    /// `--tm-needs-col-count`
    pub tm_needs_col_count: Px,
    /// `--tm-needs-col-deadline`
    pub tm_needs_col_deadline: Px,
    /// `--tm-rise`
    pub tm_rise: Px,
    /// `--tm-row-side`
    pub tm_row_side: Px,
    /// `--tm-screen-medium`
    pub tm_screen_medium: Px,
    /// `--tm-screen-wide`
    pub tm_screen_wide: Px,
    /// `--tm-selected-fill`
    pub tm_selected_fill: Rgba,
    /// `--tm-selected-line`
    pub tm_selected_line: Rgba,
    /// `--tm-shadow-overlay`
    pub tm_shadow_overlay: &'static [Shadow],
    /// `--tm-slide`
    pub tm_slide: Px,
    /// `--tm-space-1`
    pub tm_space_1: Px,
    /// `--tm-space-2`
    pub tm_space_2: Px,
    /// `--tm-space-3`
    pub tm_space_3: Px,
    /// `--tm-space-4`
    pub tm_space_4: Px,
    /// `--tm-space-5`
    pub tm_space_5: Px,
    /// `--tm-space-6`
    pub tm_space_6: Px,
    /// `--tm-surface-hover`
    pub tm_surface_hover: Rgba,
    /// `--tm-surface-page`
    pub tm_surface_page: Rgba,
    /// `--tm-surface-panel`
    pub tm_surface_panel: Rgba,
    /// `--tm-surface-raised`
    pub tm_surface_raised: Rgba,
    /// `--tm-surface-toast`
    pub tm_surface_toast: Rgba,
    /// `--tm-surface-toast-hover`
    pub tm_surface_toast_hover: Rgba,
    /// `--tm-surface-tooltip`
    pub tm_surface_tooltip: Rgba,
    /// `--tm-text-body`
    pub tm_text_body: Px,
    /// `--tm-text-code`
    pub tm_text_code: Px,
    /// `--tm-text-page`
    pub tm_text_page: Px,
    /// `--tm-text-pane`
    pub tm_text_pane: Px,
    /// `--tm-text-section`
    pub tm_text_section: Px,
    /// `--tm-text-small`
    pub tm_text_small: Px,
    /// `--tm-weight-heading`
    pub tm_weight_heading: f32,
    /// The parts' looks, by component, variant and state.
    pub looks: &'static [Look],
}

/// The dark theme.
pub const DARK: Tokens = Tokens {
    bp_emphasis_ease_bounce: Curve(0.54, 1.12, 0.38, 1.11), // cubic-bezier(0.54, 1.12, 0.38, 1.11)
    bp_emphasis_ease_default: Curve(0.4, 1.0, 0.75, 0.9),   // cubic-bezier(0.4, 1, 0.75, 0.9)
    bp_emphasis_focus_color: Rgba(0xa66bffff),              // #a66bff
    bp_emphasis_focus_offset: Px(2.0),                      // 2px
    bp_emphasis_focus_width: Px(2.0),                       // 2px
    bp_emphasis_motion_reduced: 0.0,                        // 0
    bp_emphasis_transition_duration: Duration::from_millis(100), // 100ms
    bp_intent_danger_active: Rgba(0x8e292cff),              // #8e292c
    bp_intent_danger_disabled: Rgba(0xe76a6eff),            // #e76a6e
    bp_intent_danger_foreground: Rgba(0xffffffff),          // #ffffff
    bp_intent_danger_hover: Rgba(0xac2f33ff),               // #ac2f33
    bp_intent_danger_rest: Rgba(0xcd4246ff),                // #cd4246
    bp_intent_default_active: Rgba(0x3a3146ff),             // #3a3146
    bp_intent_default_disabled: Rgba(0x8d849bff),           // #8d849b
    bp_intent_default_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_default_hover: Rgba(0x473d55ff),              // #473d55
    bp_intent_default_rest: Rgba(0x685e78ff),               // #685e78
    bp_intent_primary_active: Rgba(0x3b1578ff),             // #3b1578
    bp_intent_primary_disabled: Rgba(0x7c3aedff),           // #7c3aed
    bp_intent_primary_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_primary_hover: Rgba(0x4c1d95ff),              // #4c1d95
    bp_intent_primary_rest: Rgba(0x5b21b6ff),               // #5b21b6
    bp_intent_success_active: Rgba(0x184a90ff),             // #184a90
    bp_intent_success_disabled: Rgba(0x4c90f0ff),           // #4c90f0
    bp_intent_success_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_success_hover: Rgba(0x215db0ff),              // #215db0
    bp_intent_success_rest: Rgba(0x2d72d2ff),               // #2d72d2
    bp_intent_warning_active: Rgba(0xc4a51fff),             // #c4a51f
    bp_intent_warning_disabled: Rgba(0xe6c935ff),           // #e6c935
    bp_intent_warning_foreground: Rgba(0x14101aff),         // #14101a
    bp_intent_warning_hover: Rgba(0xe6c935ff),              // #e6c935
    bp_intent_warning_rest: Rgba(0xdcbb2aff),               // #dcbb2a
    bp_palette_black: Rgba(0x14101aff),                     // #14101a
    bp_palette_blue_1: Rgba(0x3b1578ff),                    // #3b1578
    bp_palette_blue_2: Rgba(0x4c1d95ff),                    // #4c1d95
    bp_palette_blue_3: Rgba(0x5b21b6ff),                    // #5b21b6
    bp_palette_blue_4: Rgba(0x7c3aedff),                    // #7c3aed
    bp_palette_blue_5: Rgba(0xa66bffff),                    // #a66bff
    bp_palette_blue_6: Rgba(0xc29bffff),                    // #c29bff
    bp_palette_dark_gray_1: Rgba(0x1b1622ff),               // #1b1622
    bp_palette_dark_gray_2: Rgba(0x241d2dff),               // #241d2d
    bp_palette_dark_gray_3: Rgba(0x2e2638ff),               // #2e2638
    bp_palette_dark_gray_4: Rgba(0x3a3146ff),               // #3a3146
    bp_palette_dark_gray_5: Rgba(0x473d55ff),               // #473d55
    bp_palette_gray_1: Rgba(0x685e78ff),                    // #685e78
    bp_palette_gray_2: Rgba(0x7a7089ff),                    // #7a7089
    bp_palette_gray_3: Rgba(0x8d849bff),                    // #8d849b
    bp_palette_gray_4: Rgba(0xa8a0b4ff),                    // #a8a0b4
    bp_palette_gray_5: Rgba(0xbdb6c7ff),                    // #bdb6c7
    bp_palette_green_1: Rgba(0x184a90ff),                   // #184a90
    bp_palette_green_2: Rgba(0x215db0ff),                   // #215db0
    bp_palette_green_3: Rgba(0x2d72d2ff),                   // #2d72d2
    bp_palette_green_4: Rgba(0x4c90f0ff),                   // #4c90f0
    bp_palette_green_5: Rgba(0x8abbffff),                   // #8abbff
    bp_palette_green_6: Rgba(0x99c4ffff),                   // #99c4ff
    bp_palette_light_gray_1: Rgba(0xd6d1dcff),              // #d6d1dc
    bp_palette_light_gray_2: Rgba(0xdfdbe4ff),              // #dfdbe4
    bp_palette_light_gray_3: Rgba(0xe8e5ecff),              // #e8e5ec
    bp_palette_light_gray_4: Rgba(0xefedf2ff),              // #efedf2
    bp_palette_light_gray_5: Rgba(0xf7f6f9ff),              // #f7f6f9
    bp_palette_orange_1: Rgba(0xa88d1aff),                  // #a88d1a
    bp_palette_orange_2: Rgba(0xc4a51fff),                  // #c4a51f
    bp_palette_orange_3: Rgba(0xdcbb2aff),                  // #dcbb2a
    bp_palette_orange_4: Rgba(0xe6c935ff),                  // #e6c935
    bp_palette_orange_5: Rgba(0xf0d455ff),                  // #f0d455
    bp_palette_orange_6: Rgba(0xf6e28cff),                  // #f6e28c
    bp_palette_red_1: Rgba(0x8e292cff),                     // #8e292c
    bp_palette_red_2: Rgba(0xac2f33ff),                     // #ac2f33
    bp_palette_red_3: Rgba(0xcd4246ff),                     // #cd4246
    bp_palette_red_4: Rgba(0xe76a6eff),                     // #e76a6e
    bp_palette_red_5: Rgba(0xfa999cff),                     // #fa999c
    bp_palette_red_6: Rgba(0xffa1a4ff),                     // #ffa1a4
    bp_palette_white: Rgba(0xffffffff),                     // #ffffff
    bp_surface_background_color_danger_active: Rgba(0x8e292cff), // #8e292c
    bp_surface_background_color_danger_disabled: Rgba(0xe76a6eff), // #e76a6e
    bp_surface_background_color_danger_hover: Rgba(0xac2f33ff), // #ac2f33
    bp_surface_background_color_danger_rest: Rgba(0xcd4246ff), // #cd4246
    bp_surface_background_color_default_active: Rgba(0x2e2638ff), // #2e2638
    bp_surface_background_color_default_disabled: Rgba(0x1b1622ff), // #1b1622
    bp_surface_background_color_default_hover: Rgba(0x241d2dff), // #241d2d
    bp_surface_background_color_default_rest: Rgba(0x1b1622ff), // #1b1622
    bp_surface_background_color_primary_active: Rgba(0x3b1578ff), // #3b1578
    bp_surface_background_color_primary_disabled: Rgba(0x7c3aedff), // #7c3aed
    bp_surface_background_color_primary_hover: Rgba(0x4c1d95ff), // #4c1d95
    bp_surface_background_color_primary_rest: Rgba(0x5b21b6ff), // #5b21b6
    bp_surface_background_color_success_active: Rgba(0x184a90ff), // #184a90
    bp_surface_background_color_success_disabled: Rgba(0x4c90f0ff), // #4c90f0
    bp_surface_background_color_success_hover: Rgba(0x215db0ff), // #215db0
    bp_surface_background_color_success_rest: Rgba(0x2d72d2ff), // #2d72d2
    bp_surface_background_color_warning_active: Rgba(0xc4a51fff), // #c4a51f
    bp_surface_background_color_warning_disabled: Rgba(0xe6c935ff), // #e6c935
    bp_surface_background_color_warning_hover: Rgba(0xe6c935ff), // #e6c935
    bp_surface_background_color_warning_rest: Rgba(0xdcbb2aff), // #dcbb2a
    bp_surface_border_color_default: Rgba(0xffffff24),      // #ffffff at 14%
    bp_surface_border_color_strong: Rgba(0xffffff3d),       // #ffffff at 24%
    bp_surface_border_radius: Px(2.0),                      // 2px
    bp_surface_border_width: Px(1.0),                       // 1px
    bp_surface_color_code: Rgba(0x14101a4d),                // #14101a at 30%
    bp_surface_layer_color_danger: Rgba(0xcd42460d),        // #cd4246 at 5%
    bp_surface_layer_color_default: Rgba(0x685e780d),       // #685e78 at 5%
    bp_surface_layer_color_primary: Rgba(0x5b21b60d),       // #5b21b6 at 5%
    bp_surface_layer_color_success: Rgba(0x2d72d20d),       // #2d72d2 at 5%
    bp_surface_layer_color_warning: Rgba(0xdcbb2a0d),       // #dcbb2a at 5%
    bp_surface_layer_danger: Rgba(0xcd42460d),              // #cd4246 at 5%
    bp_surface_layer_default: Rgba(0x685e780d),             // #685e78 at 5%
    bp_surface_layer_opacity: 0.05,                         // 0.05
    bp_surface_layer_primary: Rgba(0x5b21b60d),             // #5b21b6 at 5%
    bp_surface_layer_success: Rgba(0x2d72d20d),             // #2d72d2 at 5%
    bp_surface_layer_warning: Rgba(0xdcbb2a0d),             // #dcbb2a at 5%
    bp_surface_shadow_0: &[
        Shadow {
            inset: true,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0xffffff33),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(10.0),
            spread: Px(0.0),
            colour: Rgba(0x00000033),
        },
    ], // 2 layers
    bp_surface_shadow_1: &[
        Shadow {
            inset: true,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0xffffff33),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(1.0),
            blur: Px(10.0),
            spread: Px(0.0),
            colour: Rgba(0x00000033),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(1.0),
            blur: Px(10.0),
            spread: Px(-1.0),
            colour: Rgba(0x00000033),
        },
    ], // 3 layers
    bp_surface_shadow_2: &[
        Shadow {
            inset: true,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0xffffff33),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(4.0),
            blur: Px(6.0),
            spread: Px(-4.0),
            colour: Rgba(0x00000080),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(10.0),
            blur: Px(30.0),
            spread: Px(-5.0),
            colour: Rgba(0x00000080),
        },
    ], // 3 layers
    bp_surface_shadow_3: &[
        Shadow {
            inset: true,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0xffffff33),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(20.0),
            blur: Px(25.0),
            spread: Px(-5.0),
            colour: Rgba(0x0000004d),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(10.0),
            blur: Px(30.0),
            spread: Px(-5.0),
            colour: Rgba(0x0000004d),
        },
    ], // 3 layers
    bp_surface_shadow_4: &[
        Shadow {
            inset: true,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0xffffff33),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(25.0),
            blur: Px(60.0),
            spread: Px(-12.0),
            colour: Rgba(0x000000d9),
        },
    ], // 2 layers
    bp_surface_spacing: Px(4.0),                            // 4px
    bp_surface_z_index_0: 0.0,                              // 0
    bp_surface_z_index_1: 10.0,                             // 10
    bp_surface_z_index_2: 20.0,                             // 20
    bp_surface_z_index_3: 30.0,                             // 30
    bp_surface_z_index_4: 40.0,                             // 40
    bp_typography_color_danger_active: Rgba(0xffa1a4ff),    // #ffa1a4
    bp_typography_color_danger_disabled: Rgba(0xe76a6eff),  // #e76a6e
    bp_typography_color_danger_hover: Rgba(0xfa999cff),     // #fa999c
    bp_typography_color_danger_rest: Rgba(0xfa999cff),      // #fa999c
    bp_typography_color_default_active: Rgba(0xefedf2ff),   // #efedf2
    bp_typography_color_default_disabled: Rgba(0xa8a0b499), // #a8a0b4 at 60%
    bp_typography_color_default_hover: Rgba(0xffffffff),    // #ffffff
    bp_typography_color_default_rest: Rgba(0xf7f6f9ff),     // #f7f6f9
    bp_typography_color_muted: Rgba(0xa8a0b4ff),            // #a8a0b4
    bp_typography_color_primary_active: Rgba(0xc29bffff),   // #c29bff
    bp_typography_color_primary_disabled: Rgba(0x7c3aedff), // #7c3aed
    bp_typography_color_primary_hover: Rgba(0xa66bffff),    // #a66bff
    bp_typography_color_primary_rest: Rgba(0xa66bffff),     // #a66bff
    bp_typography_color_success_active: Rgba(0x99c4ffff),   // #99c4ff
    bp_typography_color_success_disabled: Rgba(0x4c90f0ff), // #4c90f0
    bp_typography_color_success_hover: Rgba(0x8abbffff),    // #8abbff
    bp_typography_color_success_rest: Rgba(0x8abbffff),     // #8abbff
    bp_typography_color_warning_active: Rgba(0xf6e28cff),   // #f6e28c
    bp_typography_color_warning_disabled: Rgba(0xe6c935ff), // #e6c935
    bp_typography_color_warning_hover: Rgba(0xf0d455ff),    // #f0d455
    bp_typography_color_warning_rest: Rgba(0xf0d455ff),     // #f0d455
    bp_typography_family_default: &["Archivo Variable", "Archivo", "system-ui", "sans-serif"], // Archivo Variable, Archivo, system-ui, sans-serif
    bp_typography_family_mono: &["IBM Plex Mono", "ui-monospace", "monospace"], // IBM Plex Mono, ui-monospace, monospace
    bp_typography_line_height_default: 1.28581,                                 // 1.28581
    bp_typography_line_height_large: 1.5,                                       // 1.5
    bp_typography_size_body_large: Px(16.0),                                    // 16px
    bp_typography_size_body_medium: Px(13.0),                                   // 13px
    bp_typography_size_body_small: Px(12.0),                                    // 12px
    bp_typography_size_body_x_small: Px(10.0),                                  // 10px
    bp_typography_size_code_large: Px(14.0),                                    // 14px
    bp_typography_size_code_medium: Px(13.0),                                   // 13px
    bp_typography_size_code_small: Px(12.0),                                    // 12px
    bp_typography_size_heading_display: Px(46.0),                               // 46px
    bp_typography_size_heading_large: Px(24.0),                                 // 24px
    bp_typography_size_heading_medium: Px(20.0),                                // 20px
    bp_typography_size_heading_small: Px(16.0),                                 // 16px
    bp_typography_size_heading_x_large: Px(28.0),                               // 28px
    bp_typography_weight_bold: 600.0,                                           // 600
    bp_typography_weight_default: 400.0,                                        // 400
    tm_accent_1: Rgba(0x3b1578ff),                                              // #3b1578
    tm_accent_2: Rgba(0x4c1d95ff),                                              // #4c1d95
    tm_accent_3: Rgba(0x5b21b6ff),                                              // #5b21b6
    tm_accent_4: Rgba(0x7c3aedff),                                              // #7c3aed
    tm_accent_5: Rgba(0xa66bffff),                                              // #a66bff
    tm_accent_6: Rgba(0xc29bffff),                                              // #c29bff
    tm_attention: Rgba(0xff9e4aff),                                             // #ff9e4a
    tm_backdrop: Rgba(0x14101ab3),                                              // #14101a at 70%
    tm_board_col_empty: Px(32.0),                                               // 32px
    tm_board_col_min: Px(146.0),                                                // 146px
    tm_col_left_medium: Px(260.0),                                              // 260px
    tm_col_left_narrow: Px(256.0),                                              // 256px
    tm_col_left_wide: Px(300.0),                                                // 300px
    tm_col_right_medium: Px(440.0),                                             // 440px
    tm_col_right_narrow: Px(360.0),                                             // 360px
    tm_col_right_wide: Px(520.0),                                               // 520px
    tm_cut: Px(8.0),                                                            // 8px
    tm_d_drift: Duration::from_millis(16000),                                   // 16000ms
    tm_d_loop: Duration::from_millis(1200),                                     // 1200ms
    tm_d_panel: Duration::from_millis(250),                                     // 250ms
    tm_d_small: Duration::from_millis(200),                                     // 200ms
    tm_d_stage: Duration::from_millis(350),                                     // 350ms
    tm_d_stagger: Duration::from_millis(60),                                    // 60ms
    tm_d_state: Duration::from_millis(100),                                     // 100ms
    tm_d_toast: Duration::from_millis(5000),                                    // 5000ms
    tm_dialog_wide: Px(720.0),                                                  // 720px
    tm_ease_in: Curve(0.3, 0.0, 1.0, 1.0), // cubic-bezier(0.3, 0, 1, 1)
    tm_ease_linear: Curve(0.0, 0.0, 1.0, 1.0), // cubic-bezier(0, 0, 1, 1)
    tm_ease_out: Curve(0.0, 0.0, 0.0, 1.0), // cubic-bezier(0, 0, 0, 1)
    tm_ease_spatial: Curve(0.27, 1.06, 0.18, 1.0), // cubic-bezier(0.27, 1.06, 0.18, 1)
    tm_ease_state: Curve(0.2, 0.0, 0.0, 1.0), // cubic-bezier(0.2, 0, 0, 1)
    tm_edge: Rgba(0xffffff2e),             // #ffffff at 18%
    tm_edge_quiet: Rgba(0xffffff14),       // #ffffff at 8%
    tm_font_code: &["IBM Plex Mono", "ui-monospace", "monospace"], // IBM Plex Mono, ui-monospace, monospace
    tm_font_figure: &["Big Shoulders Display", "Archivo Variable", "sans-serif"], // Big Shoulders Display, Archivo Variable, sans-serif
    tm_font_text: &["Archivo Variable", "Archivo", "system-ui", "sans-serif"], // Archivo Variable, Archivo, system-ui, sans-serif
    tm_here_1: Rgba(0xec5151ff),                                               // #ec5151
    tm_here_2: Rgba(0xeec551ff),                                               // #eec551
    tm_here_3: Rgba(0x4feb74ff),                                               // #4feb74
    tm_here_4: Rgba(0x4f74ebff),                                               // #4f74eb
    tm_here_5: Rgba(0xa04beeff),                                               // #a04bee
    tm_here_6: Rgba(0xee51c5ff),                                               // #ee51c5
    tm_here_width: Px(1.5),                                                    // 1.5px
    tm_issue_col_agent: Px(124.0),                                             // 124px
    tm_issue_col_key: Px(96.0),                                                // 96px
    tm_issue_col_milestone: Px(88.0),                                          // 88px
    tm_issue_col_stage: Px(120.0),                                             // 120px
    tm_issue_col_title_min: Px(272.0),                                         // 272px
    tm_issue_col_title_narrow: Px(180.0),                                      // 180px
    tm_issue_col_updated: Px(88.0),                                            // 88px
    tm_label_width: Px(88.0),                                                  // 88px
    tm_label_width_narrow: Px(60.0),                                           // 60px
    tm_leading_page: 1.2,                                                      // 1.2
    tm_leading_pane: 1.3,                                                      // 1.3
    tm_meter_inline_width: Px(140.0),                                          // 140px
    tm_needs_col_cost: Px(168.0),                                              // 168px
    tm_needs_col_count: Px(96.0),                                              // 96px
    tm_needs_col_deadline: Px(128.0),                                          // 128px
    tm_rise: Px(4.0),                                                          // 4px
    tm_row_side: Px(26.0),                                                     // 26px
    tm_screen_medium: Px(1300.0),                                              // 1300px
    tm_screen_wide: Px(1600.0),                                                // 1600px
    tm_selected_fill: Rgba(0x5b21b633),                                        // #5b21b6 at 20%
    tm_selected_line: Rgba(0x5b21b6ff),                                        // #5b21b6
    tm_shadow_overlay: &[Shadow {
        inset: false,
        x: Px(0.0),
        y: Px(0.0),
        blur: Px(0.0),
        spread: Px(1.0),
        colour: Rgba(0xffffff2e),
    }], // 1 layer
    tm_slide: Px(24.0),                                                        // 24px
    tm_space_1: Px(2.0),                                                       // 2px
    tm_space_2: Px(4.0),                                                       // 4px
    tm_space_3: Px(6.0),                                                       // 6px
    tm_space_4: Px(8.0),                                                       // 8px
    tm_space_5: Px(12.0),                                                      // 12px
    tm_space_6: Px(16.0),                                                      // 16px
    tm_surface_hover: Rgba(0x241d2dff),                                        // #241d2d
    tm_surface_page: Rgba(0x14101aff),                                         // #14101a
    tm_surface_panel: Rgba(0x1b1622ff),                                        // #1b1622
    tm_surface_raised: Rgba(0x2e2638ff),                                       // #2e2638
    tm_surface_toast: Rgba(0x3a3146ff),                                        // #3a3146
    tm_surface_toast_hover: Rgba(0x473d55ff),                                  // #473d55
    tm_surface_tooltip: Rgba(0x473d55ff),                                      // #473d55
    tm_text_body: Px(13.0),                                                    // 13px
    tm_text_code: Px(12.0),                                                    // 12px
    tm_text_page: Px(20.0),                                                    // 20px
    tm_text_pane: Px(16.0),                                                    // 16px
    tm_text_section: Px(14.0),                                                 // 14px
    tm_text_small: Px(12.0),                                                   // 12px
    tm_weight_heading: 600.0,                                                  // 600
    looks: &DARK_LOOKS,
};

/// The light theme.
pub const LIGHT: Tokens = Tokens {
    bp_emphasis_ease_bounce: Curve(0.54, 1.12, 0.38, 1.11), // cubic-bezier(0.54, 1.12, 0.38, 1.11)
    bp_emphasis_ease_default: Curve(0.4, 1.0, 0.75, 0.9),   // cubic-bezier(0.4, 1, 0.75, 0.9)
    bp_emphasis_focus_color: Rgba(0x5b21b6ff),              // #5b21b6
    bp_emphasis_focus_offset: Px(2.0),                      // 2px
    bp_emphasis_focus_width: Px(2.0),                       // 2px
    bp_emphasis_motion_reduced: 0.0,                        // 0
    bp_emphasis_transition_duration: Duration::from_millis(100), // 100ms
    bp_intent_danger_active: Rgba(0x8e292cff),              // #8e292c
    bp_intent_danger_disabled: Rgba(0xe76a6eff),            // #e76a6e
    bp_intent_danger_foreground: Rgba(0xffffffff),          // #ffffff
    bp_intent_danger_hover: Rgba(0xac2f33ff),               // #ac2f33
    bp_intent_danger_rest: Rgba(0xcd4246ff),                // #cd4246
    bp_intent_default_active: Rgba(0x3a3146ff),             // #3a3146
    bp_intent_default_disabled: Rgba(0x8d849bff),           // #8d849b
    bp_intent_default_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_default_hover: Rgba(0x473d55ff),              // #473d55
    bp_intent_default_rest: Rgba(0x685e78ff),               // #685e78
    bp_intent_primary_active: Rgba(0x3b1578ff),             // #3b1578
    bp_intent_primary_disabled: Rgba(0x7c3aedff),           // #7c3aed
    bp_intent_primary_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_primary_hover: Rgba(0x4c1d95ff),              // #4c1d95
    bp_intent_primary_rest: Rgba(0x5b21b6ff),               // #5b21b6
    bp_intent_success_active: Rgba(0x184a90ff),             // #184a90
    bp_intent_success_disabled: Rgba(0x4c90f0ff),           // #4c90f0
    bp_intent_success_foreground: Rgba(0xffffffff),         // #ffffff
    bp_intent_success_hover: Rgba(0x215db0ff),              // #215db0
    bp_intent_success_rest: Rgba(0x2d72d2ff),               // #2d72d2
    bp_intent_warning_active: Rgba(0x7a5e00ff),             // #7a5e00
    bp_intent_warning_disabled: Rgba(0xe6c935ff),           // #e6c935
    bp_intent_warning_foreground: Rgba(0x14101aff),         // #14101a
    bp_intent_warning_hover: Rgba(0xe6c935ff),              // #e6c935
    bp_intent_warning_rest: Rgba(0xdcbb2aff),               // #dcbb2a
    bp_palette_black: Rgba(0x14101aff),                     // #14101a
    bp_palette_blue_1: Rgba(0x3b1578ff),                    // #3b1578
    bp_palette_blue_2: Rgba(0x4c1d95ff),                    // #4c1d95
    bp_palette_blue_3: Rgba(0x5b21b6ff),                    // #5b21b6
    bp_palette_blue_4: Rgba(0x7c3aedff),                    // #7c3aed
    bp_palette_blue_5: Rgba(0xa66bffff),                    // #a66bff
    bp_palette_blue_6: Rgba(0xc29bffff),                    // #c29bff
    bp_palette_dark_gray_1: Rgba(0x1b1622ff),               // #1b1622
    bp_palette_dark_gray_2: Rgba(0x241d2dff),               // #241d2d
    bp_palette_dark_gray_3: Rgba(0x2e2638ff),               // #2e2638
    bp_palette_dark_gray_4: Rgba(0x3a3146ff),               // #3a3146
    bp_palette_dark_gray_5: Rgba(0x473d55ff),               // #473d55
    bp_palette_gray_1: Rgba(0x685e78ff),                    // #685e78
    bp_palette_gray_2: Rgba(0x7a7089ff),                    // #7a7089
    bp_palette_gray_3: Rgba(0x8d849bff),                    // #8d849b
    bp_palette_gray_4: Rgba(0xa8a0b4ff),                    // #a8a0b4
    bp_palette_gray_5: Rgba(0xbdb6c7ff),                    // #bdb6c7
    bp_palette_green_1: Rgba(0x184a90ff),                   // #184a90
    bp_palette_green_2: Rgba(0x215db0ff),                   // #215db0
    bp_palette_green_3: Rgba(0x2d72d2ff),                   // #2d72d2
    bp_palette_green_4: Rgba(0x4c90f0ff),                   // #4c90f0
    bp_palette_green_5: Rgba(0x8abbffff),                   // #8abbff
    bp_palette_green_6: Rgba(0x99c4ffff),                   // #99c4ff
    bp_palette_light_gray_1: Rgba(0xd6d1dcff),              // #d6d1dc
    bp_palette_light_gray_2: Rgba(0xdfdbe4ff),              // #dfdbe4
    bp_palette_light_gray_3: Rgba(0xe8e5ecff),              // #e8e5ec
    bp_palette_light_gray_4: Rgba(0xefedf2ff),              // #efedf2
    bp_palette_light_gray_5: Rgba(0xf7f6f9ff),              // #f7f6f9
    bp_palette_orange_1: Rgba(0x6b5200ff),                  // #6b5200
    bp_palette_orange_2: Rgba(0x7a5e00ff),                  // #7a5e00
    bp_palette_orange_3: Rgba(0xdcbb2aff),                  // #dcbb2a
    bp_palette_orange_4: Rgba(0xe6c935ff),                  // #e6c935
    bp_palette_orange_5: Rgba(0xf0d455ff),                  // #f0d455
    bp_palette_orange_6: Rgba(0xf6e28cff),                  // #f6e28c
    bp_palette_red_1: Rgba(0x8e292cff),                     // #8e292c
    bp_palette_red_2: Rgba(0xac2f33ff),                     // #ac2f33
    bp_palette_red_3: Rgba(0xcd4246ff),                     // #cd4246
    bp_palette_red_4: Rgba(0xe76a6eff),                     // #e76a6e
    bp_palette_red_5: Rgba(0xfa999cff),                     // #fa999c
    bp_palette_red_6: Rgba(0xffa1a4ff),                     // #ffa1a4
    bp_palette_white: Rgba(0xffffffff),                     // #ffffff
    bp_surface_background_color_danger_active: Rgba(0x8e292cff), // #8e292c
    bp_surface_background_color_danger_disabled: Rgba(0xe76a6eff), // #e76a6e
    bp_surface_background_color_danger_hover: Rgba(0xac2f33ff), // #ac2f33
    bp_surface_background_color_danger_rest: Rgba(0xcd4246ff), // #cd4246
    bp_surface_background_color_default_active: Rgba(0xe8e5ecff), // #e8e5ec
    bp_surface_background_color_default_disabled: Rgba(0xf7f6f9ff), // #f7f6f9
    bp_surface_background_color_default_hover: Rgba(0xf7f6f9ff), // #f7f6f9
    bp_surface_background_color_default_rest: Rgba(0xffffffff), // #ffffff
    bp_surface_background_color_primary_active: Rgba(0x3b1578ff), // #3b1578
    bp_surface_background_color_primary_disabled: Rgba(0x7c3aedff), // #7c3aed
    bp_surface_background_color_primary_hover: Rgba(0x4c1d95ff), // #4c1d95
    bp_surface_background_color_primary_rest: Rgba(0x5b21b6ff), // #5b21b6
    bp_surface_background_color_success_active: Rgba(0x184a90ff), // #184a90
    bp_surface_background_color_success_disabled: Rgba(0x4c90f0ff), // #4c90f0
    bp_surface_background_color_success_hover: Rgba(0x215db0ff), // #215db0
    bp_surface_background_color_success_rest: Rgba(0x2d72d2ff), // #2d72d2
    bp_surface_background_color_warning_active: Rgba(0x7a5e00ff), // #7a5e00
    bp_surface_background_color_warning_disabled: Rgba(0xe6c935ff), // #e6c935
    bp_surface_background_color_warning_hover: Rgba(0xe6c935ff), // #e6c935
    bp_surface_background_color_warning_rest: Rgba(0xdcbb2aff), // #dcbb2a
    bp_surface_border_color_default: Rgba(0x14101a24),      // #14101a at 14%
    bp_surface_border_color_strong: Rgba(0x14101a3d),       // #14101a at 24%
    bp_surface_border_radius: Px(2.0),                      // 2px
    bp_surface_border_width: Px(1.0),                       // 1px
    bp_surface_color_code: Rgba(0xffffffb3),                // #ffffff at 70%
    bp_surface_layer_color_danger: Rgba(0xcd42460d),        // #cd4246 at 5%
    bp_surface_layer_color_default: Rgba(0x685e780d),       // #685e78 at 5%
    bp_surface_layer_color_primary: Rgba(0x5b21b60d),       // #5b21b6 at 5%
    bp_surface_layer_color_success: Rgba(0x2d72d20d),       // #2d72d2 at 5%
    bp_surface_layer_color_warning: Rgba(0xdcbb2a0d),       // #dcbb2a at 5%
    bp_surface_layer_danger: Rgba(0xcd42460d),              // #cd4246 at 5%
    bp_surface_layer_default: Rgba(0x685e780d),             // #685e78 at 5%
    bp_surface_layer_opacity: 0.05,                         // 0.05
    bp_surface_layer_primary: Rgba(0x5b21b60d),             // #5b21b6 at 5%
    bp_surface_layer_success: Rgba(0x2d72d20d),             // #2d72d2 at 5%
    bp_surface_layer_warning: Rgba(0xdcbb2a0d),             // #dcbb2a at 5%
    bp_surface_shadow_0: &[
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0x00000026),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(5.0),
            spread: Px(0.0),
            colour: Rgba(0x00000005),
        },
    ], // 2 layers
    bp_surface_shadow_1: &[
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(1.0),
            blur: Px(3.0),
            spread: Px(0.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(1.0),
            blur: Px(2.0),
            spread: Px(-1.0),
            colour: Rgba(0x0000001a),
        },
    ], // 3 layers
    bp_surface_shadow_2: &[
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(4.0),
            blur: Px(6.0),
            spread: Px(-4.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(10.0),
            blur: Px(15.0),
            spread: Px(-3.0),
            colour: Rgba(0x0000001a),
        },
    ], // 3 layers
    bp_surface_shadow_3: &[
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(20.0),
            blur: Px(25.0),
            spread: Px(-5.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(10.0),
            blur: Px(15.0),
            spread: Px(-3.0),
            colour: Rgba(0x0000001a),
        },
    ], // 3 layers
    bp_surface_shadow_4: &[
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(0.0),
            blur: Px(0.0),
            spread: Px(1.0),
            colour: Rgba(0x0000001a),
        },
        Shadow {
            inset: false,
            x: Px(0.0),
            y: Px(25.0),
            blur: Px(50.0),
            spread: Px(-12.0),
            colour: Rgba(0x0000004d),
        },
    ], // 2 layers
    bp_surface_spacing: Px(4.0),                            // 4px
    bp_surface_z_index_0: 0.0,                              // 0
    bp_surface_z_index_1: 10.0,                             // 10
    bp_surface_z_index_2: 20.0,                             // 20
    bp_surface_z_index_3: 30.0,                             // 30
    bp_surface_z_index_4: 40.0,                             // 40
    bp_typography_color_danger_active: Rgba(0x8e292cff),    // #8e292c
    bp_typography_color_danger_disabled: Rgba(0xe76a6eff),  // #e76a6e
    bp_typography_color_danger_hover: Rgba(0xac2f33ff),     // #ac2f33
    bp_typography_color_danger_rest: Rgba(0xac2f33ff),      // #ac2f33
    bp_typography_color_default_active: Rgba(0x241d2dff),   // #241d2d
    bp_typography_color_default_disabled: Rgba(0x685e7899), // #685e78 at 60%
    bp_typography_color_default_hover: Rgba(0x14101aff),    // #14101a
    bp_typography_color_default_rest: Rgba(0x1b1622ff),     // #1b1622
    bp_typography_color_muted: Rgba(0x685e78ff),            // #685e78
    bp_typography_color_primary_active: Rgba(0x3b1578ff),   // #3b1578
    bp_typography_color_primary_disabled: Rgba(0x7c3aedff), // #7c3aed
    bp_typography_color_primary_hover: Rgba(0x4c1d95ff),    // #4c1d95
    bp_typography_color_primary_rest: Rgba(0x4c1d95ff),     // #4c1d95
    bp_typography_color_success_active: Rgba(0x184a90ff),   // #184a90
    bp_typography_color_success_disabled: Rgba(0x4c90f0ff), // #4c90f0
    bp_typography_color_success_hover: Rgba(0x215db0ff),    // #215db0
    bp_typography_color_success_rest: Rgba(0x215db0ff),     // #215db0
    bp_typography_color_warning_active: Rgba(0x6b5200ff),   // #6b5200
    bp_typography_color_warning_disabled: Rgba(0xe6c935ff), // #e6c935
    bp_typography_color_warning_hover: Rgba(0x7a5e00ff),    // #7a5e00
    bp_typography_color_warning_rest: Rgba(0x7a5e00ff),     // #7a5e00
    bp_typography_family_default: &["Archivo Variable", "Archivo", "system-ui", "sans-serif"], // Archivo Variable, Archivo, system-ui, sans-serif
    bp_typography_family_mono: &["IBM Plex Mono", "ui-monospace", "monospace"], // IBM Plex Mono, ui-monospace, monospace
    bp_typography_line_height_default: 1.28581,                                 // 1.28581
    bp_typography_line_height_large: 1.5,                                       // 1.5
    bp_typography_size_body_large: Px(16.0),                                    // 16px
    bp_typography_size_body_medium: Px(13.0),                                   // 13px
    bp_typography_size_body_small: Px(12.0),                                    // 12px
    bp_typography_size_body_x_small: Px(10.0),                                  // 10px
    bp_typography_size_code_large: Px(14.0),                                    // 14px
    bp_typography_size_code_medium: Px(13.0),                                   // 13px
    bp_typography_size_code_small: Px(12.0),                                    // 12px
    bp_typography_size_heading_display: Px(46.0),                               // 46px
    bp_typography_size_heading_large: Px(24.0),                                 // 24px
    bp_typography_size_heading_medium: Px(20.0),                                // 20px
    bp_typography_size_heading_small: Px(16.0),                                 // 16px
    bp_typography_size_heading_x_large: Px(28.0),                               // 28px
    bp_typography_weight_bold: 600.0,                                           // 600
    bp_typography_weight_default: 400.0,                                        // 400
    tm_accent_1: Rgba(0x3b1578ff),                                              // #3b1578
    tm_accent_2: Rgba(0x4c1d95ff),                                              // #4c1d95
    tm_accent_3: Rgba(0x5b21b6ff),                                              // #5b21b6
    tm_accent_4: Rgba(0x7c3aedff),                                              // #7c3aed
    tm_accent_5: Rgba(0xa66bffff),                                              // #a66bff
    tm_accent_6: Rgba(0xc29bffff),                                              // #c29bff
    tm_attention: Rgba(0xbf330cff),                                             // #bf330c
    tm_backdrop: Rgba(0x1b16223d),                                              // #1b1622 at 24%
    tm_board_col_empty: Px(32.0),                                               // 32px
    tm_board_col_min: Px(146.0),                                                // 146px
    tm_col_left_medium: Px(260.0),                                              // 260px
    tm_col_left_narrow: Px(256.0),                                              // 256px
    tm_col_left_wide: Px(300.0),                                                // 300px
    tm_col_right_medium: Px(440.0),                                             // 440px
    tm_col_right_narrow: Px(360.0),                                             // 360px
    tm_col_right_wide: Px(520.0),                                               // 520px
    tm_cut: Px(8.0),                                                            // 8px
    tm_d_drift: Duration::from_millis(16000),                                   // 16000ms
    tm_d_loop: Duration::from_millis(1200),                                     // 1200ms
    tm_d_panel: Duration::from_millis(250),                                     // 250ms
    tm_d_small: Duration::from_millis(200),                                     // 200ms
    tm_d_stage: Duration::from_millis(350),                                     // 350ms
    tm_d_stagger: Duration::from_millis(60),                                    // 60ms
    tm_d_state: Duration::from_millis(100),                                     // 100ms
    tm_d_toast: Duration::from_millis(5000),                                    // 5000ms
    tm_dialog_wide: Px(720.0),                                                  // 720px
    tm_ease_in: Curve(0.3, 0.0, 1.0, 1.0), // cubic-bezier(0.3, 0, 1, 1)
    tm_ease_linear: Curve(0.0, 0.0, 1.0, 1.0), // cubic-bezier(0, 0, 1, 1)
    tm_ease_out: Curve(0.0, 0.0, 0.0, 1.0), // cubic-bezier(0, 0, 0, 1)
    tm_ease_spatial: Curve(0.27, 1.06, 0.18, 1.0), // cubic-bezier(0.27, 1.06, 0.18, 1)
    tm_ease_state: Curve(0.2, 0.0, 0.0, 1.0), // cubic-bezier(0.2, 0, 0, 1)
    tm_edge: Rgba(0x14101a2e),             // #14101a at 18%
    tm_edge_quiet: Rgba(0x14101a14),       // #14101a at 8%
    tm_font_code: &["IBM Plex Mono", "ui-monospace", "monospace"], // IBM Plex Mono, ui-monospace, monospace
    tm_font_figure: &["Big Shoulders Display", "Archivo Variable", "sans-serif"], // Big Shoulders Display, Archivo Variable, sans-serif
    tm_font_text: &["Archivo Variable", "Archivo", "system-ui", "sans-serif"], // Archivo Variable, Archivo, system-ui, sans-serif
    tm_here_1: Rgba(0xd93636ff),                                               // #d93636
    tm_here_2: Rgba(0xe8590cff),                                               // #e8590c
    tm_here_3: Rgba(0x1f9348ff),                                               // #1f9348
    tm_here_4: Rgba(0x2f5fe0ff),                                               // #2f5fe0
    tm_here_5: Rgba(0x7b3fe0ff),                                               // #7b3fe0
    tm_here_6: Rgba(0xc42d9aff),                                               // #c42d9a
    tm_here_width: Px(1.5),                                                    // 1.5px
    tm_issue_col_agent: Px(124.0),                                             // 124px
    tm_issue_col_key: Px(96.0),                                                // 96px
    tm_issue_col_milestone: Px(88.0),                                          // 88px
    tm_issue_col_stage: Px(120.0),                                             // 120px
    tm_issue_col_title_min: Px(272.0),                                         // 272px
    tm_issue_col_title_narrow: Px(180.0),                                      // 180px
    tm_issue_col_updated: Px(88.0),                                            // 88px
    tm_label_width: Px(88.0),                                                  // 88px
    tm_label_width_narrow: Px(60.0),                                           // 60px
    tm_leading_page: 1.2,                                                      // 1.2
    tm_leading_pane: 1.3,                                                      // 1.3
    tm_meter_inline_width: Px(140.0),                                          // 140px
    tm_needs_col_cost: Px(168.0),                                              // 168px
    tm_needs_col_count: Px(96.0),                                              // 96px
    tm_needs_col_deadline: Px(128.0),                                          // 128px
    tm_rise: Px(4.0),                                                          // 4px
    tm_row_side: Px(26.0),                                                     // 26px
    tm_screen_medium: Px(1300.0),                                              // 1300px
    tm_screen_wide: Px(1600.0),                                                // 1600px
    tm_selected_fill: Rgba(0x5b21b61f),                                        // #5b21b6 at 12%
    tm_selected_line: Rgba(0x5b21b6ff),                                        // #5b21b6
    tm_shadow_overlay: &[Shadow {
        inset: false,
        x: Px(0.0),
        y: Px(0.0),
        blur: Px(0.0),
        spread: Px(1.0),
        colour: Rgba(0x14101a2e),
    }], // 1 layer
    tm_slide: Px(24.0),                                                        // 24px
    tm_space_1: Px(2.0),                                                       // 2px
    tm_space_2: Px(4.0),                                                       // 4px
    tm_space_3: Px(6.0),                                                       // 6px
    tm_space_4: Px(8.0),                                                       // 8px
    tm_space_5: Px(12.0),                                                      // 12px
    tm_space_6: Px(16.0),                                                      // 16px
    tm_surface_hover: Rgba(0xf7f6f9ff),                                        // #f7f6f9
    tm_surface_page: Rgba(0xefedf2ff),                                         // #efedf2
    tm_surface_panel: Rgba(0xffffffff),                                        // #ffffff
    tm_surface_raised: Rgba(0xffffffff),                                       // #ffffff
    tm_surface_toast: Rgba(0xffffffff),                                        // #ffffff
    tm_surface_toast_hover: Rgba(0xefedf2ff),                                  // #efedf2
    tm_surface_tooltip: Rgba(0x473d55ff),                                      // #473d55
    tm_text_body: Px(13.0),                                                    // 13px
    tm_text_code: Px(12.0),                                                    // 12px
    tm_text_page: Px(20.0),                                                    // 20px
    tm_text_pane: Px(16.0),                                                    // 16px
    tm_text_section: Px(14.0),                                                 // 14px
    tm_text_small: Px(12.0),                                                   // 12px
    tm_weight_heading: 600.0,                                                  // 600
    looks: &LIGHT_LOOKS,
};

/// The dark theme's parts, as its stories draw them.
#[rustfmt::skip]
static DARK_LOOKS: [Look; 834] = [
    Look { component: "Button", variant: "default", state: State::Rest, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "default", state: State::Hover, background: Rgba(0x292333ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "default", state: State::Active, background: Rgba(0x1f1a27ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "default", state: State::Focus, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "intent-primary", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Active, background: Rgba(0x3b1578ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Focus, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xffffff4d), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Rest, background: Rgba(0x2d72d2ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Hover, background: Rgba(0x215db0ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Active, background: Rgba(0x184a90ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Focus, background: Rgba(0x2d72d2ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Rest, background: Rgba(0xdcbb2aff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Hover, background: Rgba(0xdebe2dff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Active, background: Rgba(0xcfaf24ff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Focus, background: Rgba(0xdcbb2aff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Rest, background: Rgba(0xcd4246ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Hover, background: Rgba(0xac2f33ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Active, background: Rgba(0x8e292cff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Focus, background: Rgba(0xcd4246ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Active, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa495dfff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xa495df99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9c8fcdff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xa495df99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Active, background: Rgba(0x5b21b64d), text: Rgba(0x9288bcff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xa495df99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xa495dfff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xa495df99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x8fb4eaff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x8fb4ea99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x87a7d7ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x8fb4ea99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Active, background: Rgba(0x2d72d24d), text: Rgba(0x809cc5ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x8fb4ea99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x8fb4eaff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x8fb4ea99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xecdc9eff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xecdc9e99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Hover, background: Rgba(0xdcbb2a33), text: Rgba(0xf1e3a3ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xecdc9e99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Active, background: Rgba(0xdcbb2a4d), text: Rgba(0xdfd097ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xecdc9e99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xecdc9eff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xecdc9e99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xec9f9bff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xec9f9b99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Hover, background: Rgba(0xcd424633), text: Rgba(0xda948fff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xec9f9b99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Active, background: Rgba(0xcd42464d), text: Rgba(0xc88e8aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xec9f9b99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xec9f9bff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xec9f9b99)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Active, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa495dfff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9c8fcdff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Active, background: Rgba(0x5b21b64d), text: Rgba(0x9288bcff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xa495dfff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x8fb4eaff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x87a7d7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Active, background: Rgba(0x2d72d24d), text: Rgba(0x809cc5ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x8fb4eaff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xecdc9eff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Hover, background: Rgba(0xdcbb2a33), text: Rgba(0xf1e3a3ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Active, background: Rgba(0xdcbb2a4d), text: Rgba(0xdfd097ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xecdc9eff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xec9f9bff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Hover, background: Rgba(0xcd424633), text: Rgba(0xda948fff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Active, background: Rgba(0xcd42464d), text: Rgba(0xc88e8aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xec9f9bff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "small", state: State::Rest, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "small", state: State::Hover, background: Rgba(0x292333ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "small", state: State::Active, background: Rgba(0x1f1a27ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "small", state: State::Focus, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "large", state: State::Rest, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "large", state: State::Hover, background: Rgba(0x292333ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "large", state: State::Active, background: Rgba(0x1f1a27ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "large", state: State::Focus, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Button", variant: "active outlined", state: State::Rest, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Hover, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Active, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Focus, background: Rgba(0x685e787d), text: Rgba(0xffffffff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xffffff3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Hover, background: Rgba(0xff9e4a1f), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Active, background: Rgba(0xff9e4a33), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "default", state: State::Rest, background: Rgba(0x473d55ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "SegmentedControl", variant: "default", state: State::Hover, background: Rgba(0x473d55ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "SegmentedControl", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal small", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal small", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "small", state: State::Rest, background: Rgba(0x473d55ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "SegmentedControl", variant: "small", state: State::Hover, background: Rgba(0x473d55ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Tag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfff789ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xfff78999) }] },
    Look { component: "Tag", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "StatusTag", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfff789ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xfff78999) }] },
    Look { component: "StatusTag", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "StageTag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "InputGroup", variant: "default", state: State::Rest, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "default", state: State::Hover, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "default", state: State::Focus, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }] },
    Look { component: "InputGroup", variant: "default", state: State::Disabled, background: Rgba(0x473d5580), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "InputGroup", variant: "small", state: State::Rest, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "small", state: State::Hover, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "small", state: State::Focus, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Rest, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xe76b69ff) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Hover, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xe76b69ff) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Focus, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xe76b69c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(2.0), colour: Rgba(0xe76b69c0) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "TextArea", variant: "fill", state: State::Rest, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(66.11), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
    Look { component: "TextArea", variant: "fill", state: State::Focus, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(66.11), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x967be8c0) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Rest, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Hover, background: Rgba(0x292333ff), text: Rgba(0xf8f7f8ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Focus, background: Rgba(0x362f40ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "checked", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff1a) }] },
    Look { component: "Checkbox", variant: "checked", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xffffff99), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "checked", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff1a) }] },
    Look { component: "Checkbox", variant: "default", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff1a) }] },
    Look { component: "Checkbox", variant: "default", state: State::Disabled, background: Rgba(0x8d849b26), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "default", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff1a) }] },
    Look { component: "Switch", variant: "checked", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "checked", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "checked", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Rest, background: Rgba(0x8d849b4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Disabled, background: Rgba(0x8d849b26), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Hover, background: Rgba(0x8d849b66), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Hover, background: Rgba(0x8d849b26), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "active", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "active", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "selected", state: State::Hover, background: Rgba(0x8d849b26), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "intent-danger", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "intent-danger", state: State::Hover, background: Rgba(0xcd424633), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Card", variant: "elevation-0", state: State::Rest, background: Rgba(0x28242fff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Card", variant: "elevation-0", state: State::Hover, background: Rgba(0x28242fff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Card", variant: "elevation-0 interactive", state: State::Rest, background: Rgba(0x28242fff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Card", variant: "elevation-0 interactive", state: State::Hover, background: Rgba(0x28242fff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Callout", variant: "default", state: State::Rest, background: Rgba(0x8d849b33), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Callout", variant: "intent-primary", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa66bffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "Callout", variant: "intent-success", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x8abbffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x2d72d2ff) }] },
    Look { component: "Callout", variant: "intent-warning", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf0d455ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xdcbb2aff) }] },
    Look { component: "Callout", variant: "intent-danger", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "HTMLTable", variant: "th: default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(38.7), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "td: first-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(42.0), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "HTMLTable", variant: "td: default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(42.0), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "th: compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "td: compact first-row interactive selected", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "HTMLTable", variant: "td: compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TableRow", variant: "compact first-row interactive selected", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "TableRow", variant: "compact first-row interactive selected", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "TableRow", variant: "compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TableRow", variant: "compact interactive", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Navbar", variant: "default", state: State::Rest, background: Rgba(0x241d2dff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.0), padding: [Px(0.0), Px(16.0), Px(0.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "ProgressBar", variant: "progress-bar: default", state: State::Rest, background: Rgba(0xffffff14), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProgressBar", variant: "progress-meter: default", state: State::Rest, background: Rgba(0x8d849bff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tooltip", variant: "default", state: State::Rest, background: Rgba(0x473d55ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(32.7), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Popover", variant: "default", state: State::Rest, background: Rgba(0x2e2638ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(128.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Toast", variant: "default", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Toast", variant: "intent-primary", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "Toast", variant: "intent-success", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x2d72d2ff) }] },
    Look { component: "Toast", variant: "intent-warning", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xdcbb2aff) }] },
    Look { component: "Toast", variant: "intent-danger", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "Dialog", variant: "dialog: default", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(183.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "Dialog", variant: "dialog-header: default", state: State::Rest, background: Rgba(0x2e2638ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(38.0), padding: [Px(4.0), Px(4.0), Px(4.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }] },
    Look { component: "Dialog", variant: "dialog-footer: default", state: State::Rest, background: Rgba(0x3a3146ff), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff33)), radius: Px(0.0), height: Px(47.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-attention-text tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-attention-text tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-body tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-body tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-narrow tm-row-body", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-narrow tm-row-body", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-item tm-props-narrow", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-item tm-props-narrow", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props-item tm-props-set", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props-item tm-props-set", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(73.13), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(73.13), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(71.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(71.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-cut-body tm-row tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(71.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-cut-body tm-row tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(71.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-inset tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(54.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-inset tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(54.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-issue tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-issue tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-deadline tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-deadline tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-row-end", state: State::Hover, background: Rgba(0xff9e4a1f), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(107.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(107.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-here", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(60.41), padding: [Px(1.5), Px(1.5), Px(1.5), Px(1.5)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-here", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(60.41), padding: [Px(1.5), Px(1.5), Px(1.5), Px(1.5)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-here tm-padded", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-here tm-padded", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(139.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(139.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut-body tm-section-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut-body tm-section-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-side", state: State::Rest, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-side", state: State::Hover, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-side", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-side", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-side", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-side", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-shell-col tm-shell-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-shell-col tm-shell-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-main", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(148.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-main", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(148.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(146.25), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(146.25), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-heading tm-heading-section tm-padded", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-heading tm-heading-section tm-padded", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-decision-issue tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-decision-issue tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-pane tm-shell-side", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(1.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-pane tm-shell-side", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(1.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-shell-col tm-shell-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-shell-col tm-shell-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-shell-col tm-shell-pane tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-shell-col tm-shell-pane tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-pane", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.11), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-pane", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.11), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(62.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(62.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-mono tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-mono tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-top", state: State::Rest, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-top", state: State::Hover, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-bar tm-shell-top", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(37.0), padding: [Px(6.0), Px(12.0), Px(6.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-bar tm-shell-top", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(37.0), padding: [Px(6.0), Px(12.0), Px(6.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell-bar tm-topbar", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell-bar tm-topbar", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-topbar tm-topbar-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-topbar tm-topbar-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-top", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(863.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-top", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(863.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-top", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(863.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-top", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(863.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav tm-tree", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav tm-tree", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-tree tm-tree-group", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(150.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-tree tm-tree-group", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(150.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-facts tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-facts tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-facts", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-facts", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-muted", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-muted", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(124.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(124.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-groups-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-groups-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-name", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-name", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-note", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-note", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "GroupList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(237.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xff9e4aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(237.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(235.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(235.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-needs-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-needs-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention-text tm-heading-text tm-needs-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention-text tm-heading-text tm-needs-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-col-key tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-col-key tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-count tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-cost tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-cost tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-title tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-title tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issues tm-needs-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issues tm-needs-count tm-needs-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "NeedsYou", variant: "tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-title tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-title tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-count tm-needs-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-more", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-more", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(53.7), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(53.7), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-issue tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-issue tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-deadline tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-deadline tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-done tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-done tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-body tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-body tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-row-body", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-row-body", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-fields tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-fields tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn-fields tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn-fields tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-actions", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-actions", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-attention tm-overturn-actions", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-attention tm-overturn-actions", state: State::Hover, background: Rgba(0xff9e4a1f), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(182.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(182.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(180.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(180.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-section-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-section-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-milestone-bar", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-milestone-bar", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-key tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-key tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-stage tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-stage tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-agent tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-agent tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-updated tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-updated tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issue-quiet", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issue-quiet", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(194.22), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(194.22), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-key tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-key tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-stage tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-stage tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-agent tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-agent tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-updated tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-updated tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x685e784d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0xffffff33) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issue-quiet", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issue-quiet", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(414.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(414.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(88.59), padding: [Px(12.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(88.59), padding: [Px(12.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-issue-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-issue-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-issue-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-issue-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-issue-facts-line tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-issue-facts-line tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-head-buttons", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-head-buttons", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane tm-issue-scroll tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(326.05), padding: [Px(8.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane tm-issue-scroll tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(326.05), padding: [Px(8.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-scroll tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(256.05), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-scroll tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(256.05), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-stack tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(39.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-stack tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(39.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-mono tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-mono tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(204.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(204.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-pane-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-pane-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(66.55), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(66.55), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-word tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-word tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-check-toggle-none tm-row-end", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(171.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(171.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-pane-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-pane-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(83.25), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(83.25), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(66.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(66.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-word tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-word tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "minimal tm-check-toggle-none tm-row-end", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0xa8a0b499), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "CheckList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "ReviewRounds", variant: "tm-rounds", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(228.08), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(228.08), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-rounds tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(0.0), Px(2.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-rounds tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(0.0), Px(2.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-warning minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfff789ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xfff78999) }] },
    Look { component: "ReviewRounds", variant: "intent-warning minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xfff789ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xfff78999) }] },
    Look { component: "ReviewRounds", variant: "tm-rounds tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(65.83), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(65.83), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.83), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.83), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Disclosure", variant: "minimal tm-disclosure-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Disclosure", variant: "minimal tm-disclosure-head", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Disclosure", variant: "intent-success minimal tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(33.41), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Disclosure", variant: "intent-success minimal tm-status", state: State::Hover, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(33.41), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(27.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(27.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count tm-stage-none", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count tm-stage-none", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "IssueCard", variant: "tm-card", state: State::Hover, background: Rgba(0x241d2dff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-head tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-head tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-agents tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-agents tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(751.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(751.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head", state: State::Rest, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(33.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head", state: State::Hover, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(33.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-head tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-head tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-count tm-board-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-count tm-board-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head tm-board-head-empty", state: State::Rest, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(33.7), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head tm-board-head-empty", state: State::Hover, background: Rgba(0x14101aff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff2e)), radius: Px(0.0), height: Px(33.7), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-head tm-board-head-empty tm-board-icon", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-head tm-board-head-empty tm-board-icon", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-lane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(33.0), padding: [Px(4.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-lane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(33.0), padding: [Px(4.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-lane tm-disclosure-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-lane tm-disclosure-head", state: State::Hover, background: Rgba(0x685e783d), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-lane tm-board-lane-note", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-lane tm-board-lane-note", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(106.83), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(106.83), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(94.83), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card", state: State::Hover, background: Rgba(0x241d2dff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(94.83), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-head tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-head tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-agents tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-agents tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell tm-board-cell-empty", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(26.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell tm-board-cell-empty", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(26.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card tm-card-selected", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(116.11), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card tm-card-selected", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(116.11), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-selected tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-selected tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-board-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-board-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut", state: State::Rest, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(136.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut", state: State::Hover, background: Rgba(0xffffff2e), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(136.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(134.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(134.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-problems-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-problems-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-problems-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-problems-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading-text tm-problems-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading-text tm-problems-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-row tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-row tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-inset tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-inset tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-mono tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-mono tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-end tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-end tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(29.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(29.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-primary minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-primary minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0x5b21b633), text: Rgba(0x9773f7ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(41.47), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0xffffff14)), radius: Px(0.0), height: Px(41.47), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.47), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.47), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention-text tm-hold-held tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention-text tm-hold-held tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention tm-row-end", state: State::Hover, background: Rgba(0xff9e4a1f), text: Rgba(0xff9e4aff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xff9e4aff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-danger tm-hold-error", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(77.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "HoldControl", variant: "intent-danger tm-hold-error", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(77.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "HoldControl", variant: "intent-danger minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "HoldControl", variant: "intent-danger minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xff8b88ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xff8b8899) }] },
    Look { component: "SendToSession", variant: "tm-send: tm-send", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(398.48), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props tm-send", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(281.48), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-send-preview", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-props-set", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(161.78), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-group tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-option tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-mono tm-send-option", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-detail tm-send-option", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-new tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(24.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-status", state: State::Rest, background: Rgba(0x2d72d233), text: Rgba(0x89c3ffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send tm-send-error", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xfa999cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(57.5), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "RefineDialog", variant: "tm-refine: tm-refine", state: State::Rest, background: Rgba(0x1b1622ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(398.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff2e) }] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine tm-refine-checks", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(214.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine-checks tm-refine-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xa8a0b4ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(19.42), padding: [Px(0.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine-checks tm-refine-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-mono tm-refine-row", state: State::Rest, background: Rgba(0x14101a4d), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(47.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x967be800) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xffffff33) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0xffffff4d) }] },
];

/// The light theme's parts, as its stories draw them.
#[rustfmt::skip]
static LIGHT_LOOKS: [Look; 834] = [
    Look { component: "Button", variant: "default", state: State::Rest, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "default", state: State::Hover, background: Rgba(0xeeeef0ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "default", state: State::Active, background: Rgba(0xdfdee1ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "default", state: State::Focus, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "intent-primary", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Active, background: Rgba(0x3b1578ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Focus, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xffffff99), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Rest, background: Rgba(0x2d72d2ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Hover, background: Rgba(0x215db0ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Active, background: Rgba(0x184a90ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success", state: State::Focus, background: Rgba(0x2d72d2ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Rest, background: Rgba(0xdcbb2aff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Hover, background: Rgba(0xdebe2dff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Active, background: Rgba(0xa78712ff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning", state: State::Focus, background: Rgba(0xdcbb2aff), text: Rgba(0x14101aff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Rest, background: Rgba(0xcd4246ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Hover, background: Rgba(0xac2f33ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Active, background: Rgba(0x8e292cff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger", state: State::Focus, background: Rgba(0xcd4246ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Active, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x4c1d9599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Hover, background: Rgba(0x5b21b626), text: Rgba(0x4c1d95ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x4c1d9599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Active, background: Rgba(0x5b21b64d), text: Rgba(0x3b1578ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x4c1d9599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x4c1d9599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x215db0ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x215db099)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Hover, background: Rgba(0x2d72d226), text: Rgba(0x215db0ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x215db099)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Active, background: Rgba(0x2d72d24d), text: Rgba(0x184a90ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x215db099)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x215db0ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x215db099)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x7a5e00ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xe6c93599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Hover, background: Rgba(0xdcbb2a26), text: Rgba(0x7a5e00ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xe6c93599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Active, background: Rgba(0xdcbb2a4d), text: Rgba(0x7a5e00ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xe6c93599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x7a5e00ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xe6c93599)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xac2f3399)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Hover, background: Rgba(0xcd424626), text: Rgba(0xac2f33ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xac2f3399)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Active, background: Rgba(0xcd42464d), text: Rgba(0x8e292cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xac2f3399)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger outlined", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xac2f3399)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Active, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Hover, background: Rgba(0x5b21b626), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Active, background: Rgba(0x5b21b64d), text: Rgba(0x3b1578ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-primary minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Hover, background: Rgba(0x2d72d226), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Active, background: Rgba(0x2d72d24d), text: Rgba(0x184a90ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-success minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x7a5e00ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Hover, background: Rgba(0xdcbb2a26), text: Rgba(0x7a5e00ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Active, background: Rgba(0xdcbb2a4d), text: Rgba(0x7a5e00ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-warning minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x7a5e00ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Hover, background: Rgba(0xcd424626), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Active, background: Rgba(0xcd42464d), text: Rgba(0x8e292cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "intent-danger minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "small", state: State::Rest, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "small", state: State::Hover, background: Rgba(0xeeeef0ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "small", state: State::Active, background: Rgba(0xdfdee1ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "small", state: State::Focus, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "large", state: State::Rest, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "large", state: State::Hover, background: Rgba(0xeeeef0ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "large", state: State::Active, background: Rgba(0xdfdee1ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "large", state: State::Focus, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(4.0), Px(16.0), Px(4.0), Px(16.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Button", variant: "active outlined", state: State::Rest, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Hover, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Active, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Button", variant: "active outlined", state: State::Focus, background: Rgba(0x3a314629), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0x14101a3d)), radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Hover, background: Rgba(0xbf330c1f), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Active, background: Rgba(0xbf330c33), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AttentionButton", variant: "outlined small", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "SegmentedControl", variant: "default", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "SegmentedControl", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal small", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "minimal small", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SegmentedControl", variant: "small", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "SegmentedControl", variant: "small", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Tag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tag", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-success minimal", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-primary minimal", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-warning minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StatusTag", variant: "intent-danger minimal", state: State::Rest, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageTag", variant: "minimal", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "InputGroup", variant: "default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "default", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "default", state: State::Focus, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a33) }] },
    Look { component: "InputGroup", variant: "default", state: State::Disabled, background: Rgba(0xc9c4d280), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "InputGroup", variant: "small", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "small", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "small", state: State::Focus, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(24.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a33) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x00000000) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "InputGroup", variant: "intent-danger", state: State::Focus, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(2.0), colour: Rgba(0xcd4246c0) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a33) }] },
    Look { component: "TextArea", variant: "fill", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(66.11), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
    Look { component: "TextArea", variant: "fill", state: State::Focus, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(66.11), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x470895c0) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a33) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Rest, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Hover, background: Rgba(0xeeeef0ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "HTMLSelect", variant: "default", state: State::Focus, background: Rgba(0xf7f7f8ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLSelect", variant: "minimal", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(0.0), Px(24.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "checked", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }] },
    Look { component: "Checkbox", variant: "checked", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xffffff99), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "checked", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }] },
    Look { component: "Checkbox", variant: "default", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }] },
    Look { component: "Checkbox", variant: "default", state: State::Disabled, background: Rgba(0x8d849b26), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Checkbox", variant: "default", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }] },
    Look { component: "Switch", variant: "checked", state: State::Rest, background: Rgba(0x5b21b6ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "checked", state: State::Disabled, background: Rgba(0x5b21b680), text: Rgba(0xffffff99), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "checked", state: State::Hover, background: Rgba(0x4c1d95ff), text: Rgba(0xffffffff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Rest, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Disabled, background: Rgba(0x8d849b26), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Switch", variant: "default", state: State::Hover, background: Rgba(0x8d849b66), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Hover, background: Rgba(0x8d849b26), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "default", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "active", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "active", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "selected", state: State::Hover, background: Rgba(0x8d849b26), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "intent-danger", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MenuItem", variant: "intent-danger", state: State::Hover, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "selected", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Focus, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tab", variant: "default", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Card", variant: "elevation-0", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Card", variant: "elevation-0", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Card", variant: "elevation-0 interactive", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Card", variant: "elevation-0 interactive", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(90.11), padding: [Px(20.0), Px(20.0), Px(20.0), Px(20.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Callout", variant: "default", state: State::Rest, background: Rgba(0x8d849b26), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Callout", variant: "intent-primary", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x4c1d95ff) }] },
    Look { component: "Callout", variant: "intent-success", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x215db0ff) }] },
    Look { component: "Callout", variant: "intent-warning", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x7a5e00ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x7a5e00ff) }] },
    Look { component: "Callout", variant: "intent-danger", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(73.5), padding: [Px(16.0), Px(16.0), Px(16.0), Px(40.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xac2f33ff) }] },
    Look { component: "HTMLTable", variant: "th: default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(38.7), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "td: first-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(42.0), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "HTMLTable", variant: "td: default", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(42.0), padding: [Px(11.0), Px(11.0), Px(11.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "th: compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HTMLTable", variant: "td: compact first-row interactive selected", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "HTMLTable", variant: "td: compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TableRow", variant: "compact first-row interactive selected", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "TableRow", variant: "compact first-row interactive selected", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "TableRow", variant: "compact interactive", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TableRow", variant: "compact interactive", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(28.7), padding: [Px(6.0), Px(11.0), Px(6.0), Px(11.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Navbar", variant: "default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.0), padding: [Px(0.0), Px(16.0), Px(0.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "ProgressBar", variant: "progress-bar: default", state: State::Rest, background: Rgba(0x685e7833), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProgressBar", variant: "progress-meter: default", state: State::Rest, background: Rgba(0x685e78cc), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Tooltip", variant: "default", state: State::Rest, background: Rgba(0x473d55ff), text: Rgba(0xf7f6f9ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(32.7), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Popover", variant: "default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(128.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Toast", variant: "default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Toast", variant: "intent-primary", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "Toast", variant: "intent-success", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x2d72d2ff) }] },
    Look { component: "Toast", variant: "intent-warning", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xdcbb2aff) }] },
    Look { component: "Toast", variant: "intent-danger", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(40.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xcd4246ff) }] },
    Look { component: "Dialog", variant: "dialog: default", state: State::Rest, background: Rgba(0xf7f6f9ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(183.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "Dialog", variant: "dialog-header: default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(38.0), padding: [Px(4.0), Px(4.0), Px(4.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "Dialog", variant: "dialog-footer: default", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a26)), radius: Px(0.0), height: Px(47.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-attention-text tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Heading", variant: "tm-attention-text tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-body tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-body tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-narrow tm-row-body", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-narrow tm-row-body", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-item tm-props-narrow", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props tm-props-item tm-props-narrow", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props-item tm-props-set", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-props-item tm-props-set", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(73.13), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(73.13), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(71.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(71.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-cut-body tm-row tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(71.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-cut-body tm-row tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(71.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-inset tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(54.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-inset tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(54.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-issue tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-issue tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-deadline tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-decision-deadline tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-row tm-row-end tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ListRow", variant: "tm-attention tm-row-end", state: State::Hover, background: Rgba(0xbf330c1f), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(107.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(107.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PropertyList", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Meter", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(59.41), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-attention tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-here", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(60.41), padding: [Px(1.5), Px(1.5), Px(1.5), Px(1.5)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-here", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(60.41), padding: [Px(1.5), Px(1.5), Px(1.5), Px(1.5)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-here tm-padded", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body tm-here tm-padded", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(57.41), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(139.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(139.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut-body tm-section-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "CutPanel", variant: "tm-cut-body tm-section-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-side", state: State::Rest, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-side", state: State::Hover, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-side", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-side", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-shell-col tm-shell-nav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-shell-col tm-shell-nav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-side", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-side", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-shell-col tm-shell-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-shell-col tm-shell-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-main", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(148.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-main", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(148.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-cut-body tm-padded", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(146.25), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-cut-body tm-padded", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(146.25), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-heading tm-heading-section tm-padded", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-heading tm-heading-section tm-padded", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-decision-issue tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-decision-issue tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-pane tm-shell-side", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(1.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-shell tm-shell-col tm-shell-pane tm-shell-side", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(1.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(900.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-shell-col tm-shell-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-shell-col tm-shell-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.8), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-shell-col tm-shell-pane tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "minimal tm-shell-col tm-shell-pane tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-pane", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.11), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut tm-shell-col tm-shell-pane", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.11), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(62.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-cut-body tm-padded tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(62.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-mono tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "AppShell", variant: "tm-mono tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-top", state: State::Rest, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-top", state: State::Hover, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(900.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-bar tm-shell-top", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(37.0), padding: [Px(6.0), Px(12.0), Px(6.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-bar tm-shell-top", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(37.0), padding: [Px(6.0), Px(12.0), Px(6.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell-bar tm-topbar", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell-bar tm-topbar", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-topbar tm-topbar-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-topbar tm-topbar-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-top", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(863.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-nav tm-shell-top", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(1.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(863.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-top", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(863.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "TopBar", variant: "tm-shell tm-shell-col tm-shell-main tm-shell-top", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(863.0), padding: [Px(12.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav tm-tree", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-sidenav tm-tree", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(270.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-tree tm-tree-group", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(150.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SideNav", variant: "tm-tree tm-tree-group", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(150.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-heading tm-heading-page tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(20.0), font_weight: 600.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-facts tm-pagehead-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-facts tm-pagehead-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-facts", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-facts", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-muted", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "PageHeader", variant: "tm-pagehead-fact tm-pagehead-muted", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(124.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(124.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-groups-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-groups-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-name", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-name", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-note", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-head tm-groups-note", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-groups-group tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "GroupList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter tm-meter-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter tm-meter-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-label", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-label", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-reading", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MeterList", variant: "tm-meter-head tm-meter-reading", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut", state: State::Rest, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(237.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut", state: State::Hover, background: Rgba(0xbf330cff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(237.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(235.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(235.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-needs-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-needs-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention-text tm-heading-text tm-needs-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-attention-text tm-heading-text tm-needs-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-col-key tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-col-key tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-count tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-cost tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-col-cost tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(167.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-title tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issue-title tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "first-row tm-issues tm-needs-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "NeedsYou", variant: "first-row tm-issues tm-needs-count tm-needs-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "NeedsYou", variant: "tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-key tm-issues tm-mono tm-needs-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-title tm-issues tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issue-title tm-issues tm-needs-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-count tm-needs-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-issues tm-needs-count tm-needs-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(34.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-more", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "NeedsYou", variant: "tm-cut-body tm-needs-more", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.0), padding: [Px(2.0), Px(4.0), Px(6.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(53.7), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(53.7), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-issue tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-issue tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-deadline tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-deadline tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-done tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-decision-done tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-body tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-row-body tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-row-body", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-row-body", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(120.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-fields tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-fields tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(88.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn-fields tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn-fields tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-actions", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-overturn tm-overturn-actions", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-attention tm-overturn-actions", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "DecisionRow", variant: "tm-attention tm-overturn-actions", state: State::Hover, background: Rgba(0xbf330c1f), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(182.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(182.52), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(180.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(180.52), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-section-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-section-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-milestone-bar", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-milestone-bar", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(8.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-key tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-key tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-stage tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-stage tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-agent tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-agent tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-updated tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-col-updated tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(126.81), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "MilestonePanel", variant: "tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issue-quiet", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "MilestonePanel", variant: "tm-cut-body tm-issue-quiet", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(194.22), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(194.22), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-key tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-key tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-stage tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-stage tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-agent tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-agent tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-updated tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-col-updated tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(192.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "first-row selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row minimal tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "first-row tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "IssueTable", variant: "first-row tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x14101a26) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-key tm-issues tm-mono", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-key tm-issues tm-mono", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(2.0), Px(6.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-title tm-issues", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "selected tm-issue-title tm-issues", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal selected tm-issues tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "minimal selected tm-issues tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "selected tm-issue-quiet tm-issues", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "selected tm-issue-quiet tm-issues", state: State::Hover, background: Rgba(0x8d849b4d), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.7), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issue-quiet", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueTable", variant: "tm-cut-body tm-issue-quiet", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.7), padding: [Px(16.0), Px(16.0), Px(16.0), Px(16.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(414.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(414.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-pane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(88.59), padding: [Px(12.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-pane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(88.59), padding: [Px(12.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-issue-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-issue-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-heading tm-heading-pane tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(41.59), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(16.0), font_weight: 600.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-issue-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-issue-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-facts-line tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-issue-facts-line tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-issue-facts-line tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-head-buttons", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-head tm-issue-head-buttons", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane tm-issue-scroll tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(326.05), padding: [Px(8.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-pane tm-issue-scroll tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(326.05), padding: [Px(8.0), Px(12.0), Px(12.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-scroll tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(256.05), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-scroll tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(256.05), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-stack tm-props", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(39.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-issue-stack tm-props", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(39.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-props tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-mono tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-mono tm-props-item", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-issue-stack", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(204.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-issue-stack", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(204.64), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-pane-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-pane-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(66.55), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-checks tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(66.55), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-clamp tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-word tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-check-word tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-check-toggle-none tm-row-end", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssuePane", variant: "minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(171.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(171.22), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-pane-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-pane-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(21.42), padding: [Px(0.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(83.25), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-checks tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(83.25), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(66.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(66.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.84), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-clamp tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(31.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-word tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-check-word tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "minimal tm-check-toggle-none tm-row-end", state: State::Disabled, background: Rgba(0x00000000), text: Rgba(0x685e7899), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "CheckList", variant: "intent-danger minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(228.08), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(228.08), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-rounds tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(0.0), Px(2.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-rounds tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(26.0), padding: [Px(2.0), Px(0.0), Px(2.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-strong tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-round-head tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-warning minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-warning minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(65.83), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-rounds tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(65.83), padding: [Px(8.0), Px(0.0), Px(8.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.83), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(48.83), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-success minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-success minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-primary minimal tm-row-end tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ReviewRounds", variant: "intent-primary minimal tm-row-end tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Disclosure", variant: "minimal tm-disclosure-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Disclosure", variant: "minimal tm-disclosure-head", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(30.0), padding: [Px(4.0), Px(8.0), Px(4.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "Disclosure", variant: "intent-success minimal tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(33.41), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "Disclosure", variant: "intent-success minimal tm-status", state: State::Hover, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(33.41), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(27.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(27.7), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count tm-stage-none", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageCounts", variant: "tm-row-end tm-stage-count tm-stage-none", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "IssueCard", variant: "tm-card", state: State::Hover, background: Rgba(0xf7f6f9ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-head tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-head tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-agents tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-agents tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected", state: State::Hover, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(159.53), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-head tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-selected tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "IssueCard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(63.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(751.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(751.55), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head", state: State::Rest, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(33.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head", state: State::Hover, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(33.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-head tm-status", state: State::Rest, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-head tm-status", state: State::Hover, background: Rgba(0x978da926), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-count tm-board-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-count tm-board-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head tm-board-head-empty", state: State::Rest, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(33.7), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-head tm-board-head-empty", state: State::Hover, background: Rgba(0xefedf2ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a2e)), radius: Px(0.0), height: Px(33.7), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-head tm-board-head-empty tm-board-icon", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-head tm-board-head-empty tm-board-icon", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-lane", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(33.0), padding: [Px(4.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-lane", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(1.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(33.0), padding: [Px(4.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-lane tm-disclosure-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageBoard", variant: "minimal tm-board-lane tm-disclosure-head", state: State::Hover, background: Rgba(0x473d5514), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 600.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-lane tm-board-lane-note", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-lane tm-board-lane-note", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(106.83), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(106.83), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(94.83), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card", state: State::Hover, background: Rgba(0xf7f6f9ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(94.83), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-head tm-mono", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-head tm-mono", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(33.41), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-agents tm-card-foot", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-agents tm-card-foot", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card-foot tm-card-muted tm-card-updated", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell tm-board-cell-empty", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(26.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board tm-board-cell tm-board-cell-empty", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(26.0), padding: [Px(6.0), Px(0.0), Px(6.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-attention-text tm-card-head tm-card-needs", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "intent-primary minimal tm-card-agents tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card tm-card-selected", state: State::Rest, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(116.11), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-card tm-card-selected", state: State::Hover, background: Rgba(0x5b21b61f), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(116.11), padding: [Px(6.0), Px(8.0), Px(6.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }, Shadow { inset: true, x: Px(0.0), y: Px(-1.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x5b21b6ff) }] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-head tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(26.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-selected tm-card-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-selected tm-card-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(50.11), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-card tm-card-foot tm-card-selected", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(20.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-board-count", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "StageBoard", variant: "tm-board-cell tm-board-count", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut", state: State::Rest, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(136.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut", state: State::Hover, background: Rgba(0x14101a2e), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(136.25), padding: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut tm-cut-body", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(134.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut tm-cut-body", state: State::Hover, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(134.25), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-problems-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-problems-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(30.0), padding: [Px(8.0), Px(12.0), Px(0.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-problems-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-problems-head", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-section tm-heading-text", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading-text tm-problems-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading-text tm-problems-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-heading tm-heading-note tm-heading-section", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-row tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-cut-body tm-row tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(49.13), padding: [Px(8.0), Px(12.0), Px(8.0), Px(12.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-inset tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-inset tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(32.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-mono tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-mono tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-end tm-row-inset", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "ProblemList", variant: "tm-row tm-row-end tm-row-inset", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(4.0), Px(0.0), Px(4.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-heading tm-heading-section tm-heading-text tm-list-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(18.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(14.0), font_weight: 600.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(29.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(29.0), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(22.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-primary minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-primary minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0x5b21b61a), text: Rgba(0x4c1d95ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-compact tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(41.47), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(1.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: Some(Rgba(0x14101a14)), radius: Px(0.0), height: Px(41.47), padding: [Px(2.0), Px(4.0), Px(2.0), Px(4.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-main", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.47), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-main", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(36.47), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention-text tm-hold-held tm-row-title", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention-text tm-hold-held tm-row-title", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-meta", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-row-main tm-row-meta", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-hold-row tm-row tm-row-end", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(24.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention tm-row-end", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "tm-attention tm-row-end", state: State::Hover, background: Rgba(0xbf330c1f), text: Rgba(0xbf330cff), border: [Px(1.0), Px(1.0), Px(1.0), Px(1.0)], border_colour: Some(Rgba(0xbf330cff)), radius: Px(2.0), height: Px(24.0), padding: [Px(0.0), Px(8.0), Px(0.0), Px(8.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-danger tm-hold-error", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(77.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xac2f33ff) }] },
    Look { component: "HoldControl", variant: "intent-danger tm-hold-error", state: State::Hover, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(77.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xac2f33ff) }] },
    Look { component: "HoldControl", variant: "intent-danger minimal tm-row-title tm-status", state: State::Rest, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "HoldControl", variant: "intent-danger minimal tm-row-title tm-status", state: State::Hover, background: Rgba(0xcd42461a), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: "tm-send: tm-send", state: State::Rest, background: Rgba(0xf7f6f9ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(398.48), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props tm-send", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(281.48), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props tm-props-item", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(0.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-send-preview", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-props-set", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(44.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(161.78), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-group tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-option tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-mono tm-send-option", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-detail tm-send-option", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(15.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send-new tm-send-to", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(16.7), padding: [Px(0.0), Px(0.0), Px(0.0), Px(24.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-props-item tm-status", state: State::Rest, background: Rgba(0x2d72d21a), text: Rgba(0x215db0ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(20.0), padding: [Px(2.0), Px(6.0), Px(2.0), Px(6.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "SendToSession", variant: ".tm-send [class*='tm-']: tm-send tm-send-error", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0xac2f33ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(57.5), padding: [Px(8.0), Px(8.0), Px(8.0), Px(32.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0xac2f33ff) }] },
    Look { component: "RefineDialog", variant: "tm-refine: tm-refine", state: State::Rest, background: Rgba(0xf7f6f9ff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(398.13), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a2e) }] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine tm-refine-checks", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(214.42), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine-checks tm-refine-head", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x685e78ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(19.42), padding: [Px(0.0), Px(0.0), Px(4.0), Px(0.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-refine-checks tm-refine-row", state: State::Rest, background: Rgba(0x00000000), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(0.0), height: Px(49.0), padding: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], font_size: Px(13.0), font_weight: 400.0, shadow: &[] },
    Look { component: "RefineDialog", variant: ".tm-refine [class*='tm-']: tm-mono tm-refine-row", state: State::Rest, background: Rgba(0xffffffff), text: Rgba(0x1b1622ff), border: [Px(0.0), Px(0.0), Px(0.0), Px(0.0)], border_colour: None, radius: Px(2.0), height: Px(47.0), padding: [Px(8.0), Px(8.0), Px(8.0), Px(8.0)], font_size: Px(12.0), font_weight: 400.0, shadow: &[Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: false, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(0.0), colour: Rgba(0x47089500) }, Shadow { inset: true, x: Px(0.0), y: Px(0.0), blur: Px(0.0), spread: Px(1.0), colour: Rgba(0x14101a33) }, Shadow { inset: true, x: Px(0.0), y: Px(1.0), blur: Px(1.0), spread: Px(0.0), colour: Rgba(0x14101a4d) }] },
];

/// Every theme, by name.
pub const THEMES: [(&str, &Tokens); 2] = [("dark", &DARK), ("light", &LIGHT)];

impl Tokens {
    /// Every colour token, by name.
    pub fn colours(&self) -> [(&'static str, Rgba); 146] {
        [
            ("--bp-emphasis-focus-color", self.bp_emphasis_focus_color),
            ("--bp-intent-danger-active", self.bp_intent_danger_active),
            (
                "--bp-intent-danger-disabled",
                self.bp_intent_danger_disabled,
            ),
            (
                "--bp-intent-danger-foreground",
                self.bp_intent_danger_foreground,
            ),
            ("--bp-intent-danger-hover", self.bp_intent_danger_hover),
            ("--bp-intent-danger-rest", self.bp_intent_danger_rest),
            ("--bp-intent-default-active", self.bp_intent_default_active),
            (
                "--bp-intent-default-disabled",
                self.bp_intent_default_disabled,
            ),
            (
                "--bp-intent-default-foreground",
                self.bp_intent_default_foreground,
            ),
            ("--bp-intent-default-hover", self.bp_intent_default_hover),
            ("--bp-intent-default-rest", self.bp_intent_default_rest),
            ("--bp-intent-primary-active", self.bp_intent_primary_active),
            (
                "--bp-intent-primary-disabled",
                self.bp_intent_primary_disabled,
            ),
            (
                "--bp-intent-primary-foreground",
                self.bp_intent_primary_foreground,
            ),
            ("--bp-intent-primary-hover", self.bp_intent_primary_hover),
            ("--bp-intent-primary-rest", self.bp_intent_primary_rest),
            ("--bp-intent-success-active", self.bp_intent_success_active),
            (
                "--bp-intent-success-disabled",
                self.bp_intent_success_disabled,
            ),
            (
                "--bp-intent-success-foreground",
                self.bp_intent_success_foreground,
            ),
            ("--bp-intent-success-hover", self.bp_intent_success_hover),
            ("--bp-intent-success-rest", self.bp_intent_success_rest),
            ("--bp-intent-warning-active", self.bp_intent_warning_active),
            (
                "--bp-intent-warning-disabled",
                self.bp_intent_warning_disabled,
            ),
            (
                "--bp-intent-warning-foreground",
                self.bp_intent_warning_foreground,
            ),
            ("--bp-intent-warning-hover", self.bp_intent_warning_hover),
            ("--bp-intent-warning-rest", self.bp_intent_warning_rest),
            ("--bp-palette-black", self.bp_palette_black),
            ("--bp-palette-blue-1", self.bp_palette_blue_1),
            ("--bp-palette-blue-2", self.bp_palette_blue_2),
            ("--bp-palette-blue-3", self.bp_palette_blue_3),
            ("--bp-palette-blue-4", self.bp_palette_blue_4),
            ("--bp-palette-blue-5", self.bp_palette_blue_5),
            ("--bp-palette-blue-6", self.bp_palette_blue_6),
            ("--bp-palette-dark-gray-1", self.bp_palette_dark_gray_1),
            ("--bp-palette-dark-gray-2", self.bp_palette_dark_gray_2),
            ("--bp-palette-dark-gray-3", self.bp_palette_dark_gray_3),
            ("--bp-palette-dark-gray-4", self.bp_palette_dark_gray_4),
            ("--bp-palette-dark-gray-5", self.bp_palette_dark_gray_5),
            ("--bp-palette-gray-1", self.bp_palette_gray_1),
            ("--bp-palette-gray-2", self.bp_palette_gray_2),
            ("--bp-palette-gray-3", self.bp_palette_gray_3),
            ("--bp-palette-gray-4", self.bp_palette_gray_4),
            ("--bp-palette-gray-5", self.bp_palette_gray_5),
            ("--bp-palette-green-1", self.bp_palette_green_1),
            ("--bp-palette-green-2", self.bp_palette_green_2),
            ("--bp-palette-green-3", self.bp_palette_green_3),
            ("--bp-palette-green-4", self.bp_palette_green_4),
            ("--bp-palette-green-5", self.bp_palette_green_5),
            ("--bp-palette-green-6", self.bp_palette_green_6),
            ("--bp-palette-light-gray-1", self.bp_palette_light_gray_1),
            ("--bp-palette-light-gray-2", self.bp_palette_light_gray_2),
            ("--bp-palette-light-gray-3", self.bp_palette_light_gray_3),
            ("--bp-palette-light-gray-4", self.bp_palette_light_gray_4),
            ("--bp-palette-light-gray-5", self.bp_palette_light_gray_5),
            ("--bp-palette-orange-1", self.bp_palette_orange_1),
            ("--bp-palette-orange-2", self.bp_palette_orange_2),
            ("--bp-palette-orange-3", self.bp_palette_orange_3),
            ("--bp-palette-orange-4", self.bp_palette_orange_4),
            ("--bp-palette-orange-5", self.bp_palette_orange_5),
            ("--bp-palette-orange-6", self.bp_palette_orange_6),
            ("--bp-palette-red-1", self.bp_palette_red_1),
            ("--bp-palette-red-2", self.bp_palette_red_2),
            ("--bp-palette-red-3", self.bp_palette_red_3),
            ("--bp-palette-red-4", self.bp_palette_red_4),
            ("--bp-palette-red-5", self.bp_palette_red_5),
            ("--bp-palette-red-6", self.bp_palette_red_6),
            ("--bp-palette-white", self.bp_palette_white),
            (
                "--bp-surface-background-color-danger-active",
                self.bp_surface_background_color_danger_active,
            ),
            (
                "--bp-surface-background-color-danger-disabled",
                self.bp_surface_background_color_danger_disabled,
            ),
            (
                "--bp-surface-background-color-danger-hover",
                self.bp_surface_background_color_danger_hover,
            ),
            (
                "--bp-surface-background-color-danger-rest",
                self.bp_surface_background_color_danger_rest,
            ),
            (
                "--bp-surface-background-color-default-active",
                self.bp_surface_background_color_default_active,
            ),
            (
                "--bp-surface-background-color-default-disabled",
                self.bp_surface_background_color_default_disabled,
            ),
            (
                "--bp-surface-background-color-default-hover",
                self.bp_surface_background_color_default_hover,
            ),
            (
                "--bp-surface-background-color-default-rest",
                self.bp_surface_background_color_default_rest,
            ),
            (
                "--bp-surface-background-color-primary-active",
                self.bp_surface_background_color_primary_active,
            ),
            (
                "--bp-surface-background-color-primary-disabled",
                self.bp_surface_background_color_primary_disabled,
            ),
            (
                "--bp-surface-background-color-primary-hover",
                self.bp_surface_background_color_primary_hover,
            ),
            (
                "--bp-surface-background-color-primary-rest",
                self.bp_surface_background_color_primary_rest,
            ),
            (
                "--bp-surface-background-color-success-active",
                self.bp_surface_background_color_success_active,
            ),
            (
                "--bp-surface-background-color-success-disabled",
                self.bp_surface_background_color_success_disabled,
            ),
            (
                "--bp-surface-background-color-success-hover",
                self.bp_surface_background_color_success_hover,
            ),
            (
                "--bp-surface-background-color-success-rest",
                self.bp_surface_background_color_success_rest,
            ),
            (
                "--bp-surface-background-color-warning-active",
                self.bp_surface_background_color_warning_active,
            ),
            (
                "--bp-surface-background-color-warning-disabled",
                self.bp_surface_background_color_warning_disabled,
            ),
            (
                "--bp-surface-background-color-warning-hover",
                self.bp_surface_background_color_warning_hover,
            ),
            (
                "--bp-surface-background-color-warning-rest",
                self.bp_surface_background_color_warning_rest,
            ),
            (
                "--bp-surface-border-color-default",
                self.bp_surface_border_color_default,
            ),
            (
                "--bp-surface-border-color-strong",
                self.bp_surface_border_color_strong,
            ),
            ("--bp-surface-color-code", self.bp_surface_color_code),
            (
                "--bp-surface-layer-color-danger",
                self.bp_surface_layer_color_danger,
            ),
            (
                "--bp-surface-layer-color-default",
                self.bp_surface_layer_color_default,
            ),
            (
                "--bp-surface-layer-color-primary",
                self.bp_surface_layer_color_primary,
            ),
            (
                "--bp-surface-layer-color-success",
                self.bp_surface_layer_color_success,
            ),
            (
                "--bp-surface-layer-color-warning",
                self.bp_surface_layer_color_warning,
            ),
            ("--bp-surface-layer-danger", self.bp_surface_layer_danger),
            ("--bp-surface-layer-default", self.bp_surface_layer_default),
            ("--bp-surface-layer-primary", self.bp_surface_layer_primary),
            ("--bp-surface-layer-success", self.bp_surface_layer_success),
            ("--bp-surface-layer-warning", self.bp_surface_layer_warning),
            (
                "--bp-typography-color-danger-active",
                self.bp_typography_color_danger_active,
            ),
            (
                "--bp-typography-color-danger-disabled",
                self.bp_typography_color_danger_disabled,
            ),
            (
                "--bp-typography-color-danger-hover",
                self.bp_typography_color_danger_hover,
            ),
            (
                "--bp-typography-color-danger-rest",
                self.bp_typography_color_danger_rest,
            ),
            (
                "--bp-typography-color-default-active",
                self.bp_typography_color_default_active,
            ),
            (
                "--bp-typography-color-default-disabled",
                self.bp_typography_color_default_disabled,
            ),
            (
                "--bp-typography-color-default-hover",
                self.bp_typography_color_default_hover,
            ),
            (
                "--bp-typography-color-default-rest",
                self.bp_typography_color_default_rest,
            ),
            (
                "--bp-typography-color-muted",
                self.bp_typography_color_muted,
            ),
            (
                "--bp-typography-color-primary-active",
                self.bp_typography_color_primary_active,
            ),
            (
                "--bp-typography-color-primary-disabled",
                self.bp_typography_color_primary_disabled,
            ),
            (
                "--bp-typography-color-primary-hover",
                self.bp_typography_color_primary_hover,
            ),
            (
                "--bp-typography-color-primary-rest",
                self.bp_typography_color_primary_rest,
            ),
            (
                "--bp-typography-color-success-active",
                self.bp_typography_color_success_active,
            ),
            (
                "--bp-typography-color-success-disabled",
                self.bp_typography_color_success_disabled,
            ),
            (
                "--bp-typography-color-success-hover",
                self.bp_typography_color_success_hover,
            ),
            (
                "--bp-typography-color-success-rest",
                self.bp_typography_color_success_rest,
            ),
            (
                "--bp-typography-color-warning-active",
                self.bp_typography_color_warning_active,
            ),
            (
                "--bp-typography-color-warning-disabled",
                self.bp_typography_color_warning_disabled,
            ),
            (
                "--bp-typography-color-warning-hover",
                self.bp_typography_color_warning_hover,
            ),
            (
                "--bp-typography-color-warning-rest",
                self.bp_typography_color_warning_rest,
            ),
            ("--tm-accent-1", self.tm_accent_1),
            ("--tm-accent-2", self.tm_accent_2),
            ("--tm-accent-3", self.tm_accent_3),
            ("--tm-accent-4", self.tm_accent_4),
            ("--tm-accent-5", self.tm_accent_5),
            ("--tm-accent-6", self.tm_accent_6),
            ("--tm-attention", self.tm_attention),
            ("--tm-backdrop", self.tm_backdrop),
            ("--tm-edge", self.tm_edge),
            ("--tm-edge-quiet", self.tm_edge_quiet),
            ("--tm-here-1", self.tm_here_1),
            ("--tm-here-2", self.tm_here_2),
            ("--tm-here-3", self.tm_here_3),
            ("--tm-here-4", self.tm_here_4),
            ("--tm-here-5", self.tm_here_5),
            ("--tm-here-6", self.tm_here_6),
            ("--tm-selected-fill", self.tm_selected_fill),
            ("--tm-selected-line", self.tm_selected_line),
            ("--tm-surface-hover", self.tm_surface_hover),
            ("--tm-surface-page", self.tm_surface_page),
            ("--tm-surface-panel", self.tm_surface_panel),
            ("--tm-surface-raised", self.tm_surface_raised),
            ("--tm-surface-toast", self.tm_surface_toast),
            ("--tm-surface-toast-hover", self.tm_surface_toast_hover),
            ("--tm-surface-tooltip", self.tm_surface_tooltip),
        ]
    }
    /// Every size token, by name.
    pub fn sizes(&self) -> [(&'static str, Px); 58] {
        [
            ("--bp-emphasis-focus-offset", self.bp_emphasis_focus_offset),
            ("--bp-emphasis-focus-width", self.bp_emphasis_focus_width),
            ("--bp-surface-border-radius", self.bp_surface_border_radius),
            ("--bp-surface-border-width", self.bp_surface_border_width),
            ("--bp-surface-spacing", self.bp_surface_spacing),
            (
                "--bp-typography-size-body-large",
                self.bp_typography_size_body_large,
            ),
            (
                "--bp-typography-size-body-medium",
                self.bp_typography_size_body_medium,
            ),
            (
                "--bp-typography-size-body-small",
                self.bp_typography_size_body_small,
            ),
            (
                "--bp-typography-size-body-x-small",
                self.bp_typography_size_body_x_small,
            ),
            (
                "--bp-typography-size-code-large",
                self.bp_typography_size_code_large,
            ),
            (
                "--bp-typography-size-code-medium",
                self.bp_typography_size_code_medium,
            ),
            (
                "--bp-typography-size-code-small",
                self.bp_typography_size_code_small,
            ),
            (
                "--bp-typography-size-heading-display",
                self.bp_typography_size_heading_display,
            ),
            (
                "--bp-typography-size-heading-large",
                self.bp_typography_size_heading_large,
            ),
            (
                "--bp-typography-size-heading-medium",
                self.bp_typography_size_heading_medium,
            ),
            (
                "--bp-typography-size-heading-small",
                self.bp_typography_size_heading_small,
            ),
            (
                "--bp-typography-size-heading-x-large",
                self.bp_typography_size_heading_x_large,
            ),
            ("--tm-board-col-empty", self.tm_board_col_empty),
            ("--tm-board-col-min", self.tm_board_col_min),
            ("--tm-col-left-medium", self.tm_col_left_medium),
            ("--tm-col-left-narrow", self.tm_col_left_narrow),
            ("--tm-col-left-wide", self.tm_col_left_wide),
            ("--tm-col-right-medium", self.tm_col_right_medium),
            ("--tm-col-right-narrow", self.tm_col_right_narrow),
            ("--tm-col-right-wide", self.tm_col_right_wide),
            ("--tm-cut", self.tm_cut),
            ("--tm-dialog-wide", self.tm_dialog_wide),
            ("--tm-here-width", self.tm_here_width),
            ("--tm-issue-col-agent", self.tm_issue_col_agent),
            ("--tm-issue-col-key", self.tm_issue_col_key),
            ("--tm-issue-col-milestone", self.tm_issue_col_milestone),
            ("--tm-issue-col-stage", self.tm_issue_col_stage),
            ("--tm-issue-col-title-min", self.tm_issue_col_title_min),
            (
                "--tm-issue-col-title-narrow",
                self.tm_issue_col_title_narrow,
            ),
            ("--tm-issue-col-updated", self.tm_issue_col_updated),
            ("--tm-label-width", self.tm_label_width),
            ("--tm-label-width-narrow", self.tm_label_width_narrow),
            ("--tm-meter-inline-width", self.tm_meter_inline_width),
            ("--tm-needs-col-cost", self.tm_needs_col_cost),
            ("--tm-needs-col-count", self.tm_needs_col_count),
            ("--tm-needs-col-deadline", self.tm_needs_col_deadline),
            ("--tm-rise", self.tm_rise),
            ("--tm-row-side", self.tm_row_side),
            ("--tm-screen-medium", self.tm_screen_medium),
            ("--tm-screen-wide", self.tm_screen_wide),
            ("--tm-slide", self.tm_slide),
            ("--tm-space-1", self.tm_space_1),
            ("--tm-space-2", self.tm_space_2),
            ("--tm-space-3", self.tm_space_3),
            ("--tm-space-4", self.tm_space_4),
            ("--tm-space-5", self.tm_space_5),
            ("--tm-space-6", self.tm_space_6),
            ("--tm-text-body", self.tm_text_body),
            ("--tm-text-code", self.tm_text_code),
            ("--tm-text-page", self.tm_text_page),
            ("--tm-text-pane", self.tm_text_pane),
            ("--tm-text-section", self.tm_text_section),
            ("--tm-text-small", self.tm_text_small),
        ]
    }
    /// Every duration token, by name.
    pub fn durations(&self) -> [(&'static str, Duration); 9] {
        [
            (
                "--bp-emphasis-transition-duration",
                self.bp_emphasis_transition_duration,
            ),
            ("--tm-d-drift", self.tm_d_drift),
            ("--tm-d-loop", self.tm_d_loop),
            ("--tm-d-panel", self.tm_d_panel),
            ("--tm-d-small", self.tm_d_small),
            ("--tm-d-stage", self.tm_d_stage),
            ("--tm-d-stagger", self.tm_d_stagger),
            ("--tm-d-state", self.tm_d_state),
            ("--tm-d-toast", self.tm_d_toast),
        ]
    }
    /// Every curve token, by name.
    pub fn curves(&self) -> [(&'static str, Curve); 7] {
        [
            ("--bp-emphasis-ease-bounce", self.bp_emphasis_ease_bounce),
            ("--bp-emphasis-ease-default", self.bp_emphasis_ease_default),
            ("--tm-ease-in", self.tm_ease_in),
            ("--tm-ease-linear", self.tm_ease_linear),
            ("--tm-ease-out", self.tm_ease_out),
            ("--tm-ease-spatial", self.tm_ease_spatial),
            ("--tm-ease-state", self.tm_ease_state),
        ]
    }
    /// Every number token, by name.
    pub fn numbers(&self) -> [(&'static str, f32); 14] {
        [
            (
                "--bp-emphasis-motion-reduced",
                self.bp_emphasis_motion_reduced,
            ),
            ("--bp-surface-layer-opacity", self.bp_surface_layer_opacity),
            ("--bp-surface-z-index-0", self.bp_surface_z_index_0),
            ("--bp-surface-z-index-1", self.bp_surface_z_index_1),
            ("--bp-surface-z-index-2", self.bp_surface_z_index_2),
            ("--bp-surface-z-index-3", self.bp_surface_z_index_3),
            ("--bp-surface-z-index-4", self.bp_surface_z_index_4),
            (
                "--bp-typography-line-height-default",
                self.bp_typography_line_height_default,
            ),
            (
                "--bp-typography-line-height-large",
                self.bp_typography_line_height_large,
            ),
            (
                "--bp-typography-weight-bold",
                self.bp_typography_weight_bold,
            ),
            (
                "--bp-typography-weight-default",
                self.bp_typography_weight_default,
            ),
            ("--tm-leading-page", self.tm_leading_page),
            ("--tm-leading-pane", self.tm_leading_pane),
            ("--tm-weight-heading", self.tm_weight_heading),
        ]
    }
    /// Every font token, by name.
    pub fn fonts(&self) -> [(&'static str, &'static [&'static str]); 5] {
        [
            (
                "--bp-typography-family-default",
                self.bp_typography_family_default,
            ),
            (
                "--bp-typography-family-mono",
                self.bp_typography_family_mono,
            ),
            ("--tm-font-code", self.tm_font_code),
            ("--tm-font-figure", self.tm_font_figure),
            ("--tm-font-text", self.tm_font_text),
        ]
    }
    /// Every shadow token, by name.
    pub fn shadows(&self) -> [(&'static str, &'static [Shadow]); 6] {
        [
            ("--bp-surface-shadow-0", self.bp_surface_shadow_0),
            ("--bp-surface-shadow-1", self.bp_surface_shadow_1),
            ("--bp-surface-shadow-2", self.bp_surface_shadow_2),
            ("--bp-surface-shadow-3", self.bp_surface_shadow_3),
            ("--bp-surface-shadow-4", self.bp_surface_shadow_4),
            ("--tm-shadow-overlay", self.tm_shadow_overlay),
        ]
    }
    /// A part's look in one variant and state, as the design system names them.
    pub fn look(&self, component: &str, variant: &str, state: State) -> Option<&'static Look> {
        self.looks
            .iter()
            .find(|l| l.component == component && l.variant == variant && l.state == state)
    }
}

/// StatusTag's statuses, in the design system's order.
pub const STATUS: [StatusView; 9] = [
    StatusView {
        name: "passed",
        word: "Passed",
        icon: "tick-circle",
        intent: Intent::Success,
    },
    StatusView {
        name: "resolved",
        word: "Resolved",
        icon: "tick-circle",
        intent: Intent::Success,
    },
    StatusView {
        name: "running",
        word: "Running",
        icon: "refresh",
        intent: Intent::Primary,
    },
    StatusView {
        name: "inProgress",
        word: "In progress",
        icon: "refresh",
        intent: Intent::Primary,
    },
    StatusView {
        name: "waiting",
        word: "Waiting",
        icon: "time",
        intent: Intent::None,
    },
    StatusView {
        name: "idle",
        word: "Idle",
        icon: "pause",
        intent: Intent::None,
    },
    StatusView {
        name: "open",
        word: "Open",
        icon: "issue",
        intent: Intent::Warning,
    },
    StatusView {
        name: "blocked",
        word: "Blocked",
        icon: "ban-circle",
        intent: Intent::Warning,
    },
    StatusView {
        name: "failed",
        word: "Failed",
        icon: "cross-circle",
        intent: Intent::Danger,
    },
];

/// StageTag's stages, in the order an issue goes through them.
pub const STAGES: [StageView; 11] = [
    StageView {
        name: "new",
        icon: "inbox",
    },
    StageView {
        name: "refining",
        icon: "edit",
    },
    StageView {
        name: "ready",
        icon: "clipboard",
    },
    StageView {
        name: "working",
        icon: "code",
    },
    StageView {
        name: "in review",
        icon: "eye-open",
    },
    StageView {
        name: "fixing",
        icon: "wrench",
    },
    StageView {
        name: "with arbiter",
        icon: "take-action",
    },
    StageView {
        name: "merging",
        icon: "git-merge",
    },
    StageView {
        name: "deploying",
        icon: "rocket",
    },
    StageView {
        name: "verifying",
        icon: "lab-test",
    },
    StageView {
        name: "done",
        icon: "tick",
    },
];

/// The design system's parts and docs entries, as (id, name): the ids its docs tools take.
pub const PARTS: [(&str, &str); 101] = [
    ("temet-appshell", "AppShell"),
    ("temet-attentionbutton", "AttentionButton"),
    ("temet-checklist", "CheckList"),
    ("temet-cutpanel", "CutPanel"),
    ("temet-decisionrow", "DecisionRow"),
    ("temet-disclosure", "Disclosure"),
    ("temet-grouplist", "GroupList"),
    ("temet-heading", "Heading"),
    ("temet-holdcontrol", "HoldControl"),
    ("temet-inputgroup", "InputGroup"),
    ("temet-issueactions", "IssueActions"),
    ("temet-issuecard", "IssueCard"),
    ("temet-issuepane", "IssuePane"),
    ("temet-issuetable", "IssueTable"),
    ("temet-listrow", "ListRow"),
    ("temet-meter", "Meter"),
    ("temet-meterlist", "MeterList"),
    ("temet-milestonepanel", "MilestonePanel"),
    ("temet-needsyou", "NeedsYou"),
    ("temet-pageheader", "PageHeader"),
    ("temet-plusminustree", "PlusMinusTree"),
    ("temet-propertylist", "PropertyList"),
    ("temet-refinedialog", "RefineDialog"),
    ("temet-reviewrounds", "ReviewRounds"),
    ("temet-section", "Section"),
    ("temet-sidenav", "SideNav"),
    ("temet-stageboard", "StageBoard"),
    ("temet-stagecounts", "StageCounts"),
    ("temet-stagetag", "StageTag"),
    ("temet-statustag", "StatusTag"),
    ("temet-tablerow", "TableRow"),
    ("temet-tag", "Tag"),
    ("temet-topbar", "TopBar"),
    ("temet-sendtosession", "SendToSession"),
    ("temet-sendbutton", "SendButton"),
    ("temet-problemlist", "ProblemList"),
    ("pages-temet-work-radar", "RadarScreen"),
    ("blueprint-alert", "Alert"),
    ("blueprint-anchorbutton", "AnchorButton"),
    ("blueprint-breadcrumbs", "Breadcrumbs"),
    ("blueprint-button", "Button"),
    ("blueprint-buttongroup", "ButtonGroup"),
    ("blueprint-callout", "Callout"),
    ("blueprint-card", "Card"),
    ("blueprint-cardlist", "CardList"),
    ("blueprint-checkbox", "Checkbox"),
    ("blueprint-collapse", "Collapse"),
    ("blueprint-compoundtag", "CompoundTag"),
    ("blueprint-contextmenu", "ContextMenu"),
    ("blueprint-controlgroup", "ControlGroup"),
    ("blueprint-dateinput", "DateInput"),
    ("blueprint-datepicker", "DatePicker"),
    ("blueprint-daterangeinput", "DateRangeInput"),
    ("blueprint-dialog", "Dialog"),
    ("blueprint-divider", "Divider"),
    ("blueprint-drawer", "Drawer"),
    ("blueprint-editabletext", "EditableText"),
    ("blueprint-entitytitle", "EntityTitle"),
    ("blueprint-fileinput", "FileInput"),
    ("blueprint-formgroup", "FormGroup"),
    ("blueprint-htmlselect", "HTMLSelect"),
    ("blueprint-htmltable", "HTMLTable"),
    ("blueprint-hotkeys", "HotkeysProvider"),
    ("blueprint-icon", "Icon"),
    ("blueprint-keycombotag", "KeyComboTag"),
    ("blueprint-menu", "Menu"),
    ("blueprint-menuitem", "MenuItem"),
    ("blueprint-multiselect", "MultiSelect"),
    ("blueprint-navbar", "Navbar"),
    ("blueprint-navbargroup", "NavbarGroup"),
    ("blueprint-nonidealstate", "NonIdealState"),
    ("blueprint-numericinput", "NumericInput"),
    ("blueprint-omnibar", "Omnibar"),
    ("blueprint-overflowlist", "OverflowList"),
    ("blueprint-overlaytoaster", "OverlayToaster"),
    ("blueprint-panelstack", "PanelStack"),
    ("blueprint-popover", "Popover"),
    ("blueprint-progressbar", "ProgressBar"),
    ("blueprint-radiogroup", "RadioGroup"),
    ("blueprint-segmentedcontrol", "SegmentedControl"),
    ("blueprint-select", "Select"),
    ("blueprint-slider", "Slider"),
    ("blueprint-spinner", "Spinner"),
    ("blueprint-suggest", "Suggest"),
    ("blueprint-switch", "Switch"),
    ("blueprint-table", "Table"),
    ("blueprint-tabs", "Tabs"),
    ("blueprint-taginput", "TagInput"),
    ("blueprint-textarea", "TextArea"),
    ("blueprint-timepicker", "TimePicker"),
    ("blueprint-toast", "Toast"),
    ("blueprint-tooltip", "Tooltip"),
    ("blueprint-typography", "H4"),
    ("herdr-window", "HerdrWindow"),
    ("herdr-raster", "Raster"),
    ("start-using-the-package--docs", "Start/Using the package"),
    ("start-rules--docs", "Start/Rules"),
    ("theme-colours--docs", "Theme/Colours"),
    ("theme-type--docs", "Theme/Type"),
    ("theme-motion--docs", "Theme/Motion"),
    ("herdr-colours--docs", "Herdr/Colours"),
];

/// The tokens a person may override in temet-work's settings, the same in every theme.
pub const SETTINGS: [&str; 7] = [
    "--tm-attention",
    "--tm-accent-1",
    "--tm-accent-2",
    "--tm-accent-3",
    "--tm-accent-4",
    "--tm-accent-5",
    "--tm-accent-6",
];

/// The faces' font files the app embeds, under `app/assets/fonts/`, each with its licence.
pub const FONTS: [FontFile; 9] = [
    FontFile {
        family: "Archivo",
        weight: 400.0,
        file: "archivo/Archivo-Regular.ttf",
        licence: "archivo/OFL.txt",
    },
    FontFile {
        family: "Archivo",
        weight: 500.0,
        file: "archivo/Archivo-Medium.ttf",
        licence: "archivo/OFL.txt",
    },
    FontFile {
        family: "Archivo",
        weight: 600.0,
        file: "archivo/Archivo-SemiBold.ttf",
        licence: "archivo/OFL.txt",
    },
    FontFile {
        family: "Archivo",
        weight: 700.0,
        file: "archivo/Archivo-Bold.ttf",
        licence: "archivo/OFL.txt",
    },
    FontFile {
        family: "Big Shoulders Display",
        weight: 600.0,
        file: "big-shoulders-display/BigShouldersDisplay-SemiBold.ttf",
        licence: "big-shoulders-display/OFL.txt",
    },
    FontFile {
        family: "Big Shoulders Display",
        weight: 700.0,
        file: "big-shoulders-display/BigShouldersDisplay-Bold.ttf",
        licence: "big-shoulders-display/OFL.txt",
    },
    FontFile {
        family: "IBM Plex Mono",
        weight: 400.0,
        file: "ibm-plex-mono/IBMPlexMono-Regular.ttf",
        licence: "ibm-plex-mono/LICENSE.txt",
    },
    FontFile {
        family: "IBM Plex Mono",
        weight: 500.0,
        file: "ibm-plex-mono/IBMPlexMono-Medium.ttf",
        licence: "ibm-plex-mono/LICENSE.txt",
    },
    FontFile {
        family: "IBM Plex Mono",
        weight: 600.0,
        file: "ibm-plex-mono/IBMPlexMono-SemiBold.ttf",
        licence: "ibm-plex-mono/LICENSE.txt",
    },
];
