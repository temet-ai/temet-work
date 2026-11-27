//! The design system's tokens, from @temet-ai/design 0.2.0, commit 21a1ca0.
//! Written by `scripts/sync-tokens`; do not edit, run the script again.
//!
//! Each constant has its token's name and holds its value in the dark theme. Blueprint's ramp
//! names hold other hues in the temet theme: blue holds magenta (the accent), green holds blue
//! (success) and orange holds yellow (warning). `TM_ATTENTION` is the theme's default.
//!
//! Not a single value, so left out:
//! - `--bp-surface-layer-danger`
//! - `--bp-surface-layer-default`
//! - `--bp-surface-layer-primary`
//! - `--bp-surface-layer-success`
//! - `--bp-surface-layer-warning`
//! - `--bp-surface-shadow-0`
//! - `--bp-surface-shadow-1`
//! - `--bp-surface-shadow-2`
//! - `--bp-surface-shadow-3`
//! - `--bp-surface-shadow-4`
//! - `--tm-cut-border`
//! - `--tm-cut-in`

use std::time::Duration;

use crate::{Curve, Intent, Px, Rgba, StageView, StatusView};

/// `--bp-emphasis-ease-bounce`: cubic-bezier(0.54, 1.12, 0.38, 1.11)
pub const BP_EMPHASIS_EASE_BOUNCE: Curve = Curve(0.54, 1.12, 0.38, 1.11);

/// `--bp-emphasis-ease-default`: cubic-bezier(0.4, 1, 0.75, 0.9)
pub const BP_EMPHASIS_EASE_DEFAULT: Curve = Curve(0.40, 1.00, 0.75, 0.90);

/// `--bp-emphasis-focus-color`: #f77cb3
pub const BP_EMPHASIS_FOCUS_COLOR: Rgba = Rgba(0xf77cb3ff);

/// `--bp-emphasis-focus-offset`: 2px
pub const BP_EMPHASIS_FOCUS_OFFSET: Px = Px(2.0);

/// `--bp-emphasis-focus-width`: 2px
pub const BP_EMPHASIS_FOCUS_WIDTH: Px = Px(2.0);

/// `--bp-emphasis-motion-reduced`: 0
pub const BP_EMPHASIS_MOTION_REDUCED: f32 = 0.0;

/// `--bp-emphasis-transition-duration`: 100ms
pub const BP_EMPHASIS_TRANSITION_DURATION: Duration = Duration::from_millis(100);

/// `--bp-intent-danger-active`: #8e292c
pub const BP_INTENT_DANGER_ACTIVE: Rgba = Rgba(0x8e292cff);

/// `--bp-intent-danger-disabled`: #e76a6e
pub const BP_INTENT_DANGER_DISABLED: Rgba = Rgba(0xe76a6eff);

/// `--bp-intent-danger-foreground`: #ffffff
pub const BP_INTENT_DANGER_FOREGROUND: Rgba = Rgba(0xffffffff);

/// `--bp-intent-danger-hover`: #ac2f33
pub const BP_INTENT_DANGER_HOVER: Rgba = Rgba(0xac2f33ff);

/// `--bp-intent-danger-rest`: #cd4246
pub const BP_INTENT_DANGER_REST: Rgba = Rgba(0xcd4246ff);

/// `--bp-intent-default-active`: #3a3146
pub const BP_INTENT_DEFAULT_ACTIVE: Rgba = Rgba(0x3a3146ff);

/// `--bp-intent-default-disabled`: #8d849b
pub const BP_INTENT_DEFAULT_DISABLED: Rgba = Rgba(0x8d849bff);

/// `--bp-intent-default-foreground`: #ffffff
pub const BP_INTENT_DEFAULT_FOREGROUND: Rgba = Rgba(0xffffffff);

/// `--bp-intent-default-hover`: #473d55
pub const BP_INTENT_DEFAULT_HOVER: Rgba = Rgba(0x473d55ff);

/// `--bp-intent-default-rest`: #685e78
pub const BP_INTENT_DEFAULT_REST: Rgba = Rgba(0x685e78ff);

/// `--bp-intent-primary-active`: #8f1049
pub const BP_INTENT_PRIMARY_ACTIVE: Rgba = Rgba(0x8f1049ff);

/// `--bp-intent-primary-disabled`: #e24a8e
pub const BP_INTENT_PRIMARY_DISABLED: Rgba = Rgba(0xe24a8eff);

/// `--bp-intent-primary-foreground`: #ffffff
pub const BP_INTENT_PRIMARY_FOREGROUND: Rgba = Rgba(0xffffffff);

/// `--bp-intent-primary-hover`: #b5155c
pub const BP_INTENT_PRIMARY_HOVER: Rgba = Rgba(0xb5155cff);

/// `--bp-intent-primary-rest`: #d41b6f
pub const BP_INTENT_PRIMARY_REST: Rgba = Rgba(0xd41b6fff);

/// `--bp-intent-success-active`: #184a90
pub const BP_INTENT_SUCCESS_ACTIVE: Rgba = Rgba(0x184a90ff);

/// `--bp-intent-success-disabled`: #4c90f0
pub const BP_INTENT_SUCCESS_DISABLED: Rgba = Rgba(0x4c90f0ff);

/// `--bp-intent-success-foreground`: #ffffff
pub const BP_INTENT_SUCCESS_FOREGROUND: Rgba = Rgba(0xffffffff);

/// `--bp-intent-success-hover`: #215db0
pub const BP_INTENT_SUCCESS_HOVER: Rgba = Rgba(0x215db0ff);

/// `--bp-intent-success-rest`: #2d72d2
pub const BP_INTENT_SUCCESS_REST: Rgba = Rgba(0x2d72d2ff);

/// `--bp-intent-warning-active`: #c4a51f
pub const BP_INTENT_WARNING_ACTIVE: Rgba = Rgba(0xc4a51fff);

/// `--bp-intent-warning-disabled`: #e6c935
pub const BP_INTENT_WARNING_DISABLED: Rgba = Rgba(0xe6c935ff);

/// `--bp-intent-warning-foreground`: #14101a
pub const BP_INTENT_WARNING_FOREGROUND: Rgba = Rgba(0x14101aff);

/// `--bp-intent-warning-hover`: #e6c935
pub const BP_INTENT_WARNING_HOVER: Rgba = Rgba(0xe6c935ff);

/// `--bp-intent-warning-rest`: #dcbb2a
pub const BP_INTENT_WARNING_REST: Rgba = Rgba(0xdcbb2aff);

/// `--bp-palette-black`: #14101a
pub const BP_PALETTE_BLACK: Rgba = Rgba(0x14101aff);

/// `--bp-palette-blue-1`: #8f1049
pub const BP_PALETTE_BLUE_1: Rgba = Rgba(0x8f1049ff);

/// `--bp-palette-blue-2`: #b5155c
pub const BP_PALETTE_BLUE_2: Rgba = Rgba(0xb5155cff);

/// `--bp-palette-blue-3`: #d41b6f
pub const BP_PALETTE_BLUE_3: Rgba = Rgba(0xd41b6fff);

/// `--bp-palette-blue-4`: #e24a8e
pub const BP_PALETTE_BLUE_4: Rgba = Rgba(0xe24a8eff);

/// `--bp-palette-blue-5`: #f77cb3
pub const BP_PALETTE_BLUE_5: Rgba = Rgba(0xf77cb3ff);

/// `--bp-palette-blue-6`: #f99bc5
pub const BP_PALETTE_BLUE_6: Rgba = Rgba(0xf99bc5ff);

/// `--bp-palette-dark-gray-1`: #1b1622
pub const BP_PALETTE_DARK_GRAY_1: Rgba = Rgba(0x1b1622ff);

/// `--bp-palette-dark-gray-2`: #241d2d
pub const BP_PALETTE_DARK_GRAY_2: Rgba = Rgba(0x241d2dff);

/// `--bp-palette-dark-gray-3`: #2e2638
pub const BP_PALETTE_DARK_GRAY_3: Rgba = Rgba(0x2e2638ff);

/// `--bp-palette-dark-gray-4`: #3a3146
pub const BP_PALETTE_DARK_GRAY_4: Rgba = Rgba(0x3a3146ff);

/// `--bp-palette-dark-gray-5`: #473d55
pub const BP_PALETTE_DARK_GRAY_5: Rgba = Rgba(0x473d55ff);

/// `--bp-palette-gray-1`: #685e78
pub const BP_PALETTE_GRAY_1: Rgba = Rgba(0x685e78ff);

/// `--bp-palette-gray-2`: #7a7089
pub const BP_PALETTE_GRAY_2: Rgba = Rgba(0x7a7089ff);

/// `--bp-palette-gray-3`: #8d849b
pub const BP_PALETTE_GRAY_3: Rgba = Rgba(0x8d849bff);

/// `--bp-palette-gray-4`: #a8a0b4
pub const BP_PALETTE_GRAY_4: Rgba = Rgba(0xa8a0b4ff);

/// `--bp-palette-gray-5`: #bdb6c7
pub const BP_PALETTE_GRAY_5: Rgba = Rgba(0xbdb6c7ff);

/// `--bp-palette-green-1`: #184a90
pub const BP_PALETTE_GREEN_1: Rgba = Rgba(0x184a90ff);

/// `--bp-palette-green-2`: #215db0
pub const BP_PALETTE_GREEN_2: Rgba = Rgba(0x215db0ff);

/// `--bp-palette-green-3`: #2d72d2
pub const BP_PALETTE_GREEN_3: Rgba = Rgba(0x2d72d2ff);

/// `--bp-palette-green-4`: #4c90f0
pub const BP_PALETTE_GREEN_4: Rgba = Rgba(0x4c90f0ff);

/// `--bp-palette-green-5`: #8abbff
pub const BP_PALETTE_GREEN_5: Rgba = Rgba(0x8abbffff);

/// `--bp-palette-green-6`: #99c4ff
pub const BP_PALETTE_GREEN_6: Rgba = Rgba(0x99c4ffff);

/// `--bp-palette-light-gray-1`: #d6d1dc
pub const BP_PALETTE_LIGHT_GRAY_1: Rgba = Rgba(0xd6d1dcff);

/// `--bp-palette-light-gray-2`: #dfdbe4
pub const BP_PALETTE_LIGHT_GRAY_2: Rgba = Rgba(0xdfdbe4ff);

/// `--bp-palette-light-gray-3`: #e8e5ec
pub const BP_PALETTE_LIGHT_GRAY_3: Rgba = Rgba(0xe8e5ecff);

/// `--bp-palette-light-gray-4`: #efedf2
pub const BP_PALETTE_LIGHT_GRAY_4: Rgba = Rgba(0xefedf2ff);

/// `--bp-palette-light-gray-5`: #f7f6f9
pub const BP_PALETTE_LIGHT_GRAY_5: Rgba = Rgba(0xf7f6f9ff);

/// `--bp-palette-orange-1`: #a88d1a
pub const BP_PALETTE_ORANGE_1: Rgba = Rgba(0xa88d1aff);

/// `--bp-palette-orange-2`: #c4a51f
pub const BP_PALETTE_ORANGE_2: Rgba = Rgba(0xc4a51fff);

/// `--bp-palette-orange-3`: #dcbb2a
pub const BP_PALETTE_ORANGE_3: Rgba = Rgba(0xdcbb2aff);

/// `--bp-palette-orange-4`: #e6c935
pub const BP_PALETTE_ORANGE_4: Rgba = Rgba(0xe6c935ff);

/// `--bp-palette-orange-5`: #f0d455
pub const BP_PALETTE_ORANGE_5: Rgba = Rgba(0xf0d455ff);

/// `--bp-palette-orange-6`: #f6e28c
pub const BP_PALETTE_ORANGE_6: Rgba = Rgba(0xf6e28cff);

/// `--bp-palette-red-1`: #8e292c
pub const BP_PALETTE_RED_1: Rgba = Rgba(0x8e292cff);

/// `--bp-palette-red-2`: #ac2f33
pub const BP_PALETTE_RED_2: Rgba = Rgba(0xac2f33ff);

/// `--bp-palette-red-3`: #cd4246
pub const BP_PALETTE_RED_3: Rgba = Rgba(0xcd4246ff);

/// `--bp-palette-red-4`: #e76a6e
pub const BP_PALETTE_RED_4: Rgba = Rgba(0xe76a6eff);

/// `--bp-palette-red-5`: #fa999c
pub const BP_PALETTE_RED_5: Rgba = Rgba(0xfa999cff);

/// `--bp-palette-red-6`: #ffa1a4
pub const BP_PALETTE_RED_6: Rgba = Rgba(0xffa1a4ff);

/// `--bp-palette-white`: #ffffff
pub const BP_PALETTE_WHITE: Rgba = Rgba(0xffffffff);

/// `--bp-surface-background-color-danger-active`: #8e292c
pub const BP_SURFACE_BACKGROUND_COLOR_DANGER_ACTIVE: Rgba = Rgba(0x8e292cff);

/// `--bp-surface-background-color-danger-disabled`: #e76a6e
pub const BP_SURFACE_BACKGROUND_COLOR_DANGER_DISABLED: Rgba = Rgba(0xe76a6eff);

/// `--bp-surface-background-color-danger-hover`: #ac2f33
pub const BP_SURFACE_BACKGROUND_COLOR_DANGER_HOVER: Rgba = Rgba(0xac2f33ff);

/// `--bp-surface-background-color-danger-rest`: #cd4246
pub const BP_SURFACE_BACKGROUND_COLOR_DANGER_REST: Rgba = Rgba(0xcd4246ff);

/// `--bp-surface-background-color-default-active`: #2e2638
pub const BP_SURFACE_BACKGROUND_COLOR_DEFAULT_ACTIVE: Rgba = Rgba(0x2e2638ff);

/// `--bp-surface-background-color-default-disabled`: #1b1622
pub const BP_SURFACE_BACKGROUND_COLOR_DEFAULT_DISABLED: Rgba = Rgba(0x1b1622ff);

/// `--bp-surface-background-color-default-hover`: #241d2d
pub const BP_SURFACE_BACKGROUND_COLOR_DEFAULT_HOVER: Rgba = Rgba(0x241d2dff);

/// `--bp-surface-background-color-default-rest`: #1b1622
pub const BP_SURFACE_BACKGROUND_COLOR_DEFAULT_REST: Rgba = Rgba(0x1b1622ff);

/// `--bp-surface-background-color-primary-active`: #8f1049
pub const BP_SURFACE_BACKGROUND_COLOR_PRIMARY_ACTIVE: Rgba = Rgba(0x8f1049ff);

/// `--bp-surface-background-color-primary-disabled`: #e24a8e
pub const BP_SURFACE_BACKGROUND_COLOR_PRIMARY_DISABLED: Rgba = Rgba(0xe24a8eff);

/// `--bp-surface-background-color-primary-hover`: #b5155c
pub const BP_SURFACE_BACKGROUND_COLOR_PRIMARY_HOVER: Rgba = Rgba(0xb5155cff);

/// `--bp-surface-background-color-primary-rest`: #d41b6f
pub const BP_SURFACE_BACKGROUND_COLOR_PRIMARY_REST: Rgba = Rgba(0xd41b6fff);

/// `--bp-surface-background-color-success-active`: #184a90
pub const BP_SURFACE_BACKGROUND_COLOR_SUCCESS_ACTIVE: Rgba = Rgba(0x184a90ff);

/// `--bp-surface-background-color-success-disabled`: #4c90f0
pub const BP_SURFACE_BACKGROUND_COLOR_SUCCESS_DISABLED: Rgba = Rgba(0x4c90f0ff);

/// `--bp-surface-background-color-success-hover`: #215db0
pub const BP_SURFACE_BACKGROUND_COLOR_SUCCESS_HOVER: Rgba = Rgba(0x215db0ff);

/// `--bp-surface-background-color-success-rest`: #2d72d2
pub const BP_SURFACE_BACKGROUND_COLOR_SUCCESS_REST: Rgba = Rgba(0x2d72d2ff);

/// `--bp-surface-background-color-warning-active`: #c4a51f
pub const BP_SURFACE_BACKGROUND_COLOR_WARNING_ACTIVE: Rgba = Rgba(0xc4a51fff);

/// `--bp-surface-background-color-warning-disabled`: #e6c935
pub const BP_SURFACE_BACKGROUND_COLOR_WARNING_DISABLED: Rgba = Rgba(0xe6c935ff);

/// `--bp-surface-background-color-warning-hover`: #e6c935
pub const BP_SURFACE_BACKGROUND_COLOR_WARNING_HOVER: Rgba = Rgba(0xe6c935ff);

/// `--bp-surface-background-color-warning-rest`: #dcbb2a
pub const BP_SURFACE_BACKGROUND_COLOR_WARNING_REST: Rgba = Rgba(0xdcbb2aff);

/// `--bp-surface-border-color-default`: #ffffff at 14%
pub const BP_SURFACE_BORDER_COLOR_DEFAULT: Rgba = Rgba(0xffffff24);

/// `--bp-surface-border-color-strong`: #ffffff at 24%
pub const BP_SURFACE_BORDER_COLOR_STRONG: Rgba = Rgba(0xffffff3d);

/// `--bp-surface-border-radius`: 2px
pub const BP_SURFACE_BORDER_RADIUS: Px = Px(2.0);

/// `--bp-surface-border-width`: 1px
pub const BP_SURFACE_BORDER_WIDTH: Px = Px(1.0);

/// `--bp-surface-color-code`: #14101a at 30%
pub const BP_SURFACE_COLOR_CODE: Rgba = Rgba(0x14101a4d);

/// `--bp-surface-layer-color-danger`: #cd4246 at 5%
pub const BP_SURFACE_LAYER_COLOR_DANGER: Rgba = Rgba(0xcd42460d);

/// `--bp-surface-layer-color-default`: #685e78 at 5%
pub const BP_SURFACE_LAYER_COLOR_DEFAULT: Rgba = Rgba(0x685e780d);

/// `--bp-surface-layer-color-primary`: #d41b6f at 5%
pub const BP_SURFACE_LAYER_COLOR_PRIMARY: Rgba = Rgba(0xd41b6f0d);

/// `--bp-surface-layer-color-success`: #2d72d2 at 5%
pub const BP_SURFACE_LAYER_COLOR_SUCCESS: Rgba = Rgba(0x2d72d20d);

/// `--bp-surface-layer-color-warning`: #dcbb2a at 5%
pub const BP_SURFACE_LAYER_COLOR_WARNING: Rgba = Rgba(0xdcbb2a0d);

/// `--bp-surface-layer-opacity`: 0.05
pub const BP_SURFACE_LAYER_OPACITY: f32 = 0.05;

/// `--bp-surface-spacing`: 4px
pub const BP_SURFACE_SPACING: Px = Px(4.0);

/// `--bp-surface-z-index-0`: 0
pub const BP_SURFACE_Z_INDEX_0: f32 = 0.0;

/// `--bp-surface-z-index-1`: 10
pub const BP_SURFACE_Z_INDEX_1: f32 = 10.0;

/// `--bp-surface-z-index-2`: 20
pub const BP_SURFACE_Z_INDEX_2: f32 = 20.0;

/// `--bp-surface-z-index-3`: 30
pub const BP_SURFACE_Z_INDEX_3: f32 = 30.0;

/// `--bp-surface-z-index-4`: 40
pub const BP_SURFACE_Z_INDEX_4: f32 = 40.0;

/// `--bp-typography-color-danger-active`: #ffa1a4
pub const BP_TYPOGRAPHY_COLOR_DANGER_ACTIVE: Rgba = Rgba(0xffa1a4ff);

/// `--bp-typography-color-danger-disabled`: #e76a6e
pub const BP_TYPOGRAPHY_COLOR_DANGER_DISABLED: Rgba = Rgba(0xe76a6eff);

/// `--bp-typography-color-danger-hover`: #fa999c
pub const BP_TYPOGRAPHY_COLOR_DANGER_HOVER: Rgba = Rgba(0xfa999cff);

/// `--bp-typography-color-danger-rest`: #fa999c
pub const BP_TYPOGRAPHY_COLOR_DANGER_REST: Rgba = Rgba(0xfa999cff);

/// `--bp-typography-color-default-active`: #efedf2
pub const BP_TYPOGRAPHY_COLOR_DEFAULT_ACTIVE: Rgba = Rgba(0xefedf2ff);

/// `--bp-typography-color-default-disabled`: #a8a0b4 at 60%
pub const BP_TYPOGRAPHY_COLOR_DEFAULT_DISABLED: Rgba = Rgba(0xa8a0b499);

/// `--bp-typography-color-default-hover`: #ffffff
pub const BP_TYPOGRAPHY_COLOR_DEFAULT_HOVER: Rgba = Rgba(0xffffffff);

/// `--bp-typography-color-default-rest`: #f7f6f9
pub const BP_TYPOGRAPHY_COLOR_DEFAULT_REST: Rgba = Rgba(0xf7f6f9ff);

/// `--bp-typography-color-muted`: #a8a0b4
pub const BP_TYPOGRAPHY_COLOR_MUTED: Rgba = Rgba(0xa8a0b4ff);

/// `--bp-typography-color-primary-active`: #f99bc5
pub const BP_TYPOGRAPHY_COLOR_PRIMARY_ACTIVE: Rgba = Rgba(0xf99bc5ff);

/// `--bp-typography-color-primary-disabled`: #e24a8e
pub const BP_TYPOGRAPHY_COLOR_PRIMARY_DISABLED: Rgba = Rgba(0xe24a8eff);

/// `--bp-typography-color-primary-hover`: #f77cb3
pub const BP_TYPOGRAPHY_COLOR_PRIMARY_HOVER: Rgba = Rgba(0xf77cb3ff);

/// `--bp-typography-color-primary-rest`: #f77cb3
pub const BP_TYPOGRAPHY_COLOR_PRIMARY_REST: Rgba = Rgba(0xf77cb3ff);

/// `--bp-typography-color-success-active`: #99c4ff
pub const BP_TYPOGRAPHY_COLOR_SUCCESS_ACTIVE: Rgba = Rgba(0x99c4ffff);

/// `--bp-typography-color-success-disabled`: #4c90f0
pub const BP_TYPOGRAPHY_COLOR_SUCCESS_DISABLED: Rgba = Rgba(0x4c90f0ff);

/// `--bp-typography-color-success-hover`: #8abbff
pub const BP_TYPOGRAPHY_COLOR_SUCCESS_HOVER: Rgba = Rgba(0x8abbffff);

/// `--bp-typography-color-success-rest`: #8abbff
pub const BP_TYPOGRAPHY_COLOR_SUCCESS_REST: Rgba = Rgba(0x8abbffff);

/// `--bp-typography-color-warning-active`: #f6e28c
pub const BP_TYPOGRAPHY_COLOR_WARNING_ACTIVE: Rgba = Rgba(0xf6e28cff);

/// `--bp-typography-color-warning-disabled`: #e6c935
pub const BP_TYPOGRAPHY_COLOR_WARNING_DISABLED: Rgba = Rgba(0xe6c935ff);

/// `--bp-typography-color-warning-hover`: #f0d455
pub const BP_TYPOGRAPHY_COLOR_WARNING_HOVER: Rgba = Rgba(0xf0d455ff);

/// `--bp-typography-color-warning-rest`: #f0d455
pub const BP_TYPOGRAPHY_COLOR_WARNING_REST: Rgba = Rgba(0xf0d455ff);

/// `--bp-typography-family-default`: "Archivo Variable", Archivo, system-ui, sans-serif
pub const BP_TYPOGRAPHY_FAMILY_DEFAULT: &[&str] =
    &["Archivo Variable", "Archivo", "system-ui", "sans-serif"];

/// `--bp-typography-family-mono`: "IBM Plex Mono", ui-monospace, monospace
pub const BP_TYPOGRAPHY_FAMILY_MONO: &[&str] = &["IBM Plex Mono", "ui-monospace", "monospace"];

/// `--bp-typography-line-height-default`: 1.28581
pub const BP_TYPOGRAPHY_LINE_HEIGHT_DEFAULT: f32 = 1.28581;

/// `--bp-typography-line-height-large`: 1.5
pub const BP_TYPOGRAPHY_LINE_HEIGHT_LARGE: f32 = 1.5;

/// `--bp-typography-size-body-large`: 16px
pub const BP_TYPOGRAPHY_SIZE_BODY_LARGE: Px = Px(16.0);

/// `--bp-typography-size-body-medium`: 13px
pub const BP_TYPOGRAPHY_SIZE_BODY_MEDIUM: Px = Px(13.0);

/// `--bp-typography-size-body-small`: 12px
pub const BP_TYPOGRAPHY_SIZE_BODY_SMALL: Px = Px(12.0);

/// `--bp-typography-size-body-x-small`: 10px
pub const BP_TYPOGRAPHY_SIZE_BODY_X_SMALL: Px = Px(10.0);

/// `--bp-typography-size-code-large`: 14px
pub const BP_TYPOGRAPHY_SIZE_CODE_LARGE: Px = Px(14.0);

/// `--bp-typography-size-code-medium`: 13px
pub const BP_TYPOGRAPHY_SIZE_CODE_MEDIUM: Px = Px(13.0);

/// `--bp-typography-size-code-small`: 12px
pub const BP_TYPOGRAPHY_SIZE_CODE_SMALL: Px = Px(12.0);

/// `--bp-typography-size-heading-display`: 46px
pub const BP_TYPOGRAPHY_SIZE_HEADING_DISPLAY: Px = Px(46.0);

/// `--bp-typography-size-heading-large`: 24px
pub const BP_TYPOGRAPHY_SIZE_HEADING_LARGE: Px = Px(24.0);

/// `--bp-typography-size-heading-medium`: 20px
pub const BP_TYPOGRAPHY_SIZE_HEADING_MEDIUM: Px = Px(20.0);

/// `--bp-typography-size-heading-small`: 16px
pub const BP_TYPOGRAPHY_SIZE_HEADING_SMALL: Px = Px(16.0);

/// `--bp-typography-size-heading-x-large`: 28px
pub const BP_TYPOGRAPHY_SIZE_HEADING_X_LARGE: Px = Px(28.0);

/// `--bp-typography-weight-bold`: 600
pub const BP_TYPOGRAPHY_WEIGHT_BOLD: f32 = 600.0;

/// `--bp-typography-weight-default`: 400
pub const BP_TYPOGRAPHY_WEIGHT_DEFAULT: f32 = 400.0;

/// `--tm-attention`: #ff9e4a
pub const TM_ATTENTION: Rgba = Rgba(0xff9e4aff);

/// `--tm-cut`: 8px
pub const TM_CUT: Px = Px(8.0);

/// `--tm-d-loop`: 1200ms
pub const TM_D_LOOP: Duration = Duration::from_millis(1200);

/// `--tm-d-panel`: 250ms
pub const TM_D_PANEL: Duration = Duration::from_millis(250);

/// `--tm-d-small`: 200ms
pub const TM_D_SMALL: Duration = Duration::from_millis(200);

/// `--tm-d-stage`: 350ms
pub const TM_D_STAGE: Duration = Duration::from_millis(350);

/// `--tm-d-stagger`: 60ms
pub const TM_D_STAGGER: Duration = Duration::from_millis(60);

/// `--tm-d-state`: 100ms
pub const TM_D_STATE: Duration = Duration::from_millis(100);

/// `--tm-ease-in`: cubic-bezier(0.3, 0, 1, 1)
pub const TM_EASE_IN: Curve = Curve(0.30, 0.00, 1.00, 1.00);

/// `--tm-ease-out`: cubic-bezier(0, 0, 0, 1)
pub const TM_EASE_OUT: Curve = Curve(0.00, 0.00, 0.00, 1.00);

/// `--tm-ease-spatial`: cubic-bezier(0.27, 1.06, 0.18, 1)
pub const TM_EASE_SPATIAL: Curve = Curve(0.27, 1.06, 0.18, 1.00);

/// `--tm-ease-state`: cubic-bezier(0.2, 0, 0, 1)
pub const TM_EASE_STATE: Curve = Curve(0.20, 0.00, 0.00, 1.00);

/// `--tm-edge`: #ffffff at 18%
pub const TM_EDGE: Rgba = Rgba(0xffffff2e);

/// `--tm-edge-quiet`: #ffffff at 8%
pub const TM_EDGE_QUIET: Rgba = Rgba(0xffffff14);

/// `--tm-font-code`: "IBM Plex Mono", ui-monospace, monospace
pub const TM_FONT_CODE: &[&str] = &["IBM Plex Mono", "ui-monospace", "monospace"];

/// `--tm-font-figure`: "Big Shoulders Display", "Archivo Variable", sans-serif
pub const TM_FONT_FIGURE: &[&str] = &["Big Shoulders Display", "Archivo Variable", "sans-serif"];

/// `--tm-font-text`: "Archivo Variable", Archivo, system-ui, sans-serif
pub const TM_FONT_TEXT: &[&str] = &["Archivo Variable", "Archivo", "system-ui", "sans-serif"];

/// `--tm-rise`: 4px
pub const TM_RISE: Px = Px(4.0);

/// `--tm-selected-fill`: #d41b6f at 20%
pub const TM_SELECTED_FILL: Rgba = Rgba(0xd41b6f33);

/// `--tm-selected-line`: #d41b6f
pub const TM_SELECTED_LINE: Rgba = Rgba(0xd41b6fff);

/// `--tm-slide`: 24px
pub const TM_SLIDE: Px = Px(24.0);

/// `--tm-text-body`: 13px
pub const TM_TEXT_BODY: Px = Px(13.0);

/// `--tm-text-code`: 12px
pub const TM_TEXT_CODE: Px = Px(12.0);

/// `--tm-text-page`: 20px
pub const TM_TEXT_PAGE: Px = Px(20.0);

/// `--tm-text-pane`: 16px
pub const TM_TEXT_PANE: Px = Px(16.0);

/// `--tm-text-section`: 14px
pub const TM_TEXT_SECTION: Px = Px(14.0);

/// `--tm-text-small`: 12px
pub const TM_TEXT_SMALL: Px = Px(12.0);

/// `--tm-weight-heading`: 600
pub const TM_WEIGHT_HEADING: f32 = 600.0;

/// Every colour token, by name.
pub const COLOURS: &[(&str, Rgba)] = &[
    ("--bp-emphasis-focus-color", BP_EMPHASIS_FOCUS_COLOR),
    ("--bp-intent-danger-active", BP_INTENT_DANGER_ACTIVE),
    ("--bp-intent-danger-disabled", BP_INTENT_DANGER_DISABLED),
    ("--bp-intent-danger-foreground", BP_INTENT_DANGER_FOREGROUND),
    ("--bp-intent-danger-hover", BP_INTENT_DANGER_HOVER),
    ("--bp-intent-danger-rest", BP_INTENT_DANGER_REST),
    ("--bp-intent-default-active", BP_INTENT_DEFAULT_ACTIVE),
    ("--bp-intent-default-disabled", BP_INTENT_DEFAULT_DISABLED),
    (
        "--bp-intent-default-foreground",
        BP_INTENT_DEFAULT_FOREGROUND,
    ),
    ("--bp-intent-default-hover", BP_INTENT_DEFAULT_HOVER),
    ("--bp-intent-default-rest", BP_INTENT_DEFAULT_REST),
    ("--bp-intent-primary-active", BP_INTENT_PRIMARY_ACTIVE),
    ("--bp-intent-primary-disabled", BP_INTENT_PRIMARY_DISABLED),
    (
        "--bp-intent-primary-foreground",
        BP_INTENT_PRIMARY_FOREGROUND,
    ),
    ("--bp-intent-primary-hover", BP_INTENT_PRIMARY_HOVER),
    ("--bp-intent-primary-rest", BP_INTENT_PRIMARY_REST),
    ("--bp-intent-success-active", BP_INTENT_SUCCESS_ACTIVE),
    ("--bp-intent-success-disabled", BP_INTENT_SUCCESS_DISABLED),
    (
        "--bp-intent-success-foreground",
        BP_INTENT_SUCCESS_FOREGROUND,
    ),
    ("--bp-intent-success-hover", BP_INTENT_SUCCESS_HOVER),
    ("--bp-intent-success-rest", BP_INTENT_SUCCESS_REST),
    ("--bp-intent-warning-active", BP_INTENT_WARNING_ACTIVE),
    ("--bp-intent-warning-disabled", BP_INTENT_WARNING_DISABLED),
    (
        "--bp-intent-warning-foreground",
        BP_INTENT_WARNING_FOREGROUND,
    ),
    ("--bp-intent-warning-hover", BP_INTENT_WARNING_HOVER),
    ("--bp-intent-warning-rest", BP_INTENT_WARNING_REST),
    ("--bp-palette-black", BP_PALETTE_BLACK),
    ("--bp-palette-blue-1", BP_PALETTE_BLUE_1),
    ("--bp-palette-blue-2", BP_PALETTE_BLUE_2),
    ("--bp-palette-blue-3", BP_PALETTE_BLUE_3),
    ("--bp-palette-blue-4", BP_PALETTE_BLUE_4),
    ("--bp-palette-blue-5", BP_PALETTE_BLUE_5),
    ("--bp-palette-blue-6", BP_PALETTE_BLUE_6),
    ("--bp-palette-dark-gray-1", BP_PALETTE_DARK_GRAY_1),
    ("--bp-palette-dark-gray-2", BP_PALETTE_DARK_GRAY_2),
    ("--bp-palette-dark-gray-3", BP_PALETTE_DARK_GRAY_3),
    ("--bp-palette-dark-gray-4", BP_PALETTE_DARK_GRAY_4),
    ("--bp-palette-dark-gray-5", BP_PALETTE_DARK_GRAY_5),
    ("--bp-palette-gray-1", BP_PALETTE_GRAY_1),
    ("--bp-palette-gray-2", BP_PALETTE_GRAY_2),
    ("--bp-palette-gray-3", BP_PALETTE_GRAY_3),
    ("--bp-palette-gray-4", BP_PALETTE_GRAY_4),
    ("--bp-palette-gray-5", BP_PALETTE_GRAY_5),
    ("--bp-palette-green-1", BP_PALETTE_GREEN_1),
    ("--bp-palette-green-2", BP_PALETTE_GREEN_2),
    ("--bp-palette-green-3", BP_PALETTE_GREEN_3),
    ("--bp-palette-green-4", BP_PALETTE_GREEN_4),
    ("--bp-palette-green-5", BP_PALETTE_GREEN_5),
    ("--bp-palette-green-6", BP_PALETTE_GREEN_6),
    ("--bp-palette-light-gray-1", BP_PALETTE_LIGHT_GRAY_1),
    ("--bp-palette-light-gray-2", BP_PALETTE_LIGHT_GRAY_2),
    ("--bp-palette-light-gray-3", BP_PALETTE_LIGHT_GRAY_3),
    ("--bp-palette-light-gray-4", BP_PALETTE_LIGHT_GRAY_4),
    ("--bp-palette-light-gray-5", BP_PALETTE_LIGHT_GRAY_5),
    ("--bp-palette-orange-1", BP_PALETTE_ORANGE_1),
    ("--bp-palette-orange-2", BP_PALETTE_ORANGE_2),
    ("--bp-palette-orange-3", BP_PALETTE_ORANGE_3),
    ("--bp-palette-orange-4", BP_PALETTE_ORANGE_4),
    ("--bp-palette-orange-5", BP_PALETTE_ORANGE_5),
    ("--bp-palette-orange-6", BP_PALETTE_ORANGE_6),
    ("--bp-palette-red-1", BP_PALETTE_RED_1),
    ("--bp-palette-red-2", BP_PALETTE_RED_2),
    ("--bp-palette-red-3", BP_PALETTE_RED_3),
    ("--bp-palette-red-4", BP_PALETTE_RED_4),
    ("--bp-palette-red-5", BP_PALETTE_RED_5),
    ("--bp-palette-red-6", BP_PALETTE_RED_6),
    ("--bp-palette-white", BP_PALETTE_WHITE),
    (
        "--bp-surface-background-color-danger-active",
        BP_SURFACE_BACKGROUND_COLOR_DANGER_ACTIVE,
    ),
    (
        "--bp-surface-background-color-danger-disabled",
        BP_SURFACE_BACKGROUND_COLOR_DANGER_DISABLED,
    ),
    (
        "--bp-surface-background-color-danger-hover",
        BP_SURFACE_BACKGROUND_COLOR_DANGER_HOVER,
    ),
    (
        "--bp-surface-background-color-danger-rest",
        BP_SURFACE_BACKGROUND_COLOR_DANGER_REST,
    ),
    (
        "--bp-surface-background-color-default-active",
        BP_SURFACE_BACKGROUND_COLOR_DEFAULT_ACTIVE,
    ),
    (
        "--bp-surface-background-color-default-disabled",
        BP_SURFACE_BACKGROUND_COLOR_DEFAULT_DISABLED,
    ),
    (
        "--bp-surface-background-color-default-hover",
        BP_SURFACE_BACKGROUND_COLOR_DEFAULT_HOVER,
    ),
    (
        "--bp-surface-background-color-default-rest",
        BP_SURFACE_BACKGROUND_COLOR_DEFAULT_REST,
    ),
    (
        "--bp-surface-background-color-primary-active",
        BP_SURFACE_BACKGROUND_COLOR_PRIMARY_ACTIVE,
    ),
    (
        "--bp-surface-background-color-primary-disabled",
        BP_SURFACE_BACKGROUND_COLOR_PRIMARY_DISABLED,
    ),
    (
        "--bp-surface-background-color-primary-hover",
        BP_SURFACE_BACKGROUND_COLOR_PRIMARY_HOVER,
    ),
    (
        "--bp-surface-background-color-primary-rest",
        BP_SURFACE_BACKGROUND_COLOR_PRIMARY_REST,
    ),
    (
        "--bp-surface-background-color-success-active",
        BP_SURFACE_BACKGROUND_COLOR_SUCCESS_ACTIVE,
    ),
    (
        "--bp-surface-background-color-success-disabled",
        BP_SURFACE_BACKGROUND_COLOR_SUCCESS_DISABLED,
    ),
    (
        "--bp-surface-background-color-success-hover",
        BP_SURFACE_BACKGROUND_COLOR_SUCCESS_HOVER,
    ),
    (
        "--bp-surface-background-color-success-rest",
        BP_SURFACE_BACKGROUND_COLOR_SUCCESS_REST,
    ),
    (
        "--bp-surface-background-color-warning-active",
        BP_SURFACE_BACKGROUND_COLOR_WARNING_ACTIVE,
    ),
    (
        "--bp-surface-background-color-warning-disabled",
        BP_SURFACE_BACKGROUND_COLOR_WARNING_DISABLED,
    ),
    (
        "--bp-surface-background-color-warning-hover",
        BP_SURFACE_BACKGROUND_COLOR_WARNING_HOVER,
    ),
    (
        "--bp-surface-background-color-warning-rest",
        BP_SURFACE_BACKGROUND_COLOR_WARNING_REST,
    ),
    (
        "--bp-surface-border-color-default",
        BP_SURFACE_BORDER_COLOR_DEFAULT,
    ),
    (
        "--bp-surface-border-color-strong",
        BP_SURFACE_BORDER_COLOR_STRONG,
    ),
    ("--bp-surface-color-code", BP_SURFACE_COLOR_CODE),
    (
        "--bp-surface-layer-color-danger",
        BP_SURFACE_LAYER_COLOR_DANGER,
    ),
    (
        "--bp-surface-layer-color-default",
        BP_SURFACE_LAYER_COLOR_DEFAULT,
    ),
    (
        "--bp-surface-layer-color-primary",
        BP_SURFACE_LAYER_COLOR_PRIMARY,
    ),
    (
        "--bp-surface-layer-color-success",
        BP_SURFACE_LAYER_COLOR_SUCCESS,
    ),
    (
        "--bp-surface-layer-color-warning",
        BP_SURFACE_LAYER_COLOR_WARNING,
    ),
    (
        "--bp-typography-color-danger-active",
        BP_TYPOGRAPHY_COLOR_DANGER_ACTIVE,
    ),
    (
        "--bp-typography-color-danger-disabled",
        BP_TYPOGRAPHY_COLOR_DANGER_DISABLED,
    ),
    (
        "--bp-typography-color-danger-hover",
        BP_TYPOGRAPHY_COLOR_DANGER_HOVER,
    ),
    (
        "--bp-typography-color-danger-rest",
        BP_TYPOGRAPHY_COLOR_DANGER_REST,
    ),
    (
        "--bp-typography-color-default-active",
        BP_TYPOGRAPHY_COLOR_DEFAULT_ACTIVE,
    ),
    (
        "--bp-typography-color-default-disabled",
        BP_TYPOGRAPHY_COLOR_DEFAULT_DISABLED,
    ),
    (
        "--bp-typography-color-default-hover",
        BP_TYPOGRAPHY_COLOR_DEFAULT_HOVER,
    ),
    (
        "--bp-typography-color-default-rest",
        BP_TYPOGRAPHY_COLOR_DEFAULT_REST,
    ),
    ("--bp-typography-color-muted", BP_TYPOGRAPHY_COLOR_MUTED),
    (
        "--bp-typography-color-primary-active",
        BP_TYPOGRAPHY_COLOR_PRIMARY_ACTIVE,
    ),
    (
        "--bp-typography-color-primary-disabled",
        BP_TYPOGRAPHY_COLOR_PRIMARY_DISABLED,
    ),
    (
        "--bp-typography-color-primary-hover",
        BP_TYPOGRAPHY_COLOR_PRIMARY_HOVER,
    ),
    (
        "--bp-typography-color-primary-rest",
        BP_TYPOGRAPHY_COLOR_PRIMARY_REST,
    ),
    (
        "--bp-typography-color-success-active",
        BP_TYPOGRAPHY_COLOR_SUCCESS_ACTIVE,
    ),
    (
        "--bp-typography-color-success-disabled",
        BP_TYPOGRAPHY_COLOR_SUCCESS_DISABLED,
    ),
    (
        "--bp-typography-color-success-hover",
        BP_TYPOGRAPHY_COLOR_SUCCESS_HOVER,
    ),
    (
        "--bp-typography-color-success-rest",
        BP_TYPOGRAPHY_COLOR_SUCCESS_REST,
    ),
    (
        "--bp-typography-color-warning-active",
        BP_TYPOGRAPHY_COLOR_WARNING_ACTIVE,
    ),
    (
        "--bp-typography-color-warning-disabled",
        BP_TYPOGRAPHY_COLOR_WARNING_DISABLED,
    ),
    (
        "--bp-typography-color-warning-hover",
        BP_TYPOGRAPHY_COLOR_WARNING_HOVER,
    ),
    (
        "--bp-typography-color-warning-rest",
        BP_TYPOGRAPHY_COLOR_WARNING_REST,
    ),
    ("--tm-attention", TM_ATTENTION),
    ("--tm-edge", TM_EDGE),
    ("--tm-edge-quiet", TM_EDGE_QUIET),
    ("--tm-selected-fill", TM_SELECTED_FILL),
    ("--tm-selected-line", TM_SELECTED_LINE),
];

/// Every size token, by name.
pub const SIZES: &[(&str, Px)] = &[
    ("--bp-emphasis-focus-offset", BP_EMPHASIS_FOCUS_OFFSET),
    ("--bp-emphasis-focus-width", BP_EMPHASIS_FOCUS_WIDTH),
    ("--bp-surface-border-radius", BP_SURFACE_BORDER_RADIUS),
    ("--bp-surface-border-width", BP_SURFACE_BORDER_WIDTH),
    ("--bp-surface-spacing", BP_SURFACE_SPACING),
    (
        "--bp-typography-size-body-large",
        BP_TYPOGRAPHY_SIZE_BODY_LARGE,
    ),
    (
        "--bp-typography-size-body-medium",
        BP_TYPOGRAPHY_SIZE_BODY_MEDIUM,
    ),
    (
        "--bp-typography-size-body-small",
        BP_TYPOGRAPHY_SIZE_BODY_SMALL,
    ),
    (
        "--bp-typography-size-body-x-small",
        BP_TYPOGRAPHY_SIZE_BODY_X_SMALL,
    ),
    (
        "--bp-typography-size-code-large",
        BP_TYPOGRAPHY_SIZE_CODE_LARGE,
    ),
    (
        "--bp-typography-size-code-medium",
        BP_TYPOGRAPHY_SIZE_CODE_MEDIUM,
    ),
    (
        "--bp-typography-size-code-small",
        BP_TYPOGRAPHY_SIZE_CODE_SMALL,
    ),
    (
        "--bp-typography-size-heading-display",
        BP_TYPOGRAPHY_SIZE_HEADING_DISPLAY,
    ),
    (
        "--bp-typography-size-heading-large",
        BP_TYPOGRAPHY_SIZE_HEADING_LARGE,
    ),
    (
        "--bp-typography-size-heading-medium",
        BP_TYPOGRAPHY_SIZE_HEADING_MEDIUM,
    ),
    (
        "--bp-typography-size-heading-small",
        BP_TYPOGRAPHY_SIZE_HEADING_SMALL,
    ),
    (
        "--bp-typography-size-heading-x-large",
        BP_TYPOGRAPHY_SIZE_HEADING_X_LARGE,
    ),
    ("--tm-cut", TM_CUT),
    ("--tm-rise", TM_RISE),
    ("--tm-slide", TM_SLIDE),
    ("--tm-text-body", TM_TEXT_BODY),
    ("--tm-text-code", TM_TEXT_CODE),
    ("--tm-text-page", TM_TEXT_PAGE),
    ("--tm-text-pane", TM_TEXT_PANE),
    ("--tm-text-section", TM_TEXT_SECTION),
    ("--tm-text-small", TM_TEXT_SMALL),
];

/// Every duration token, by name.
pub const DURATIONS: &[(&str, Duration)] = &[
    (
        "--bp-emphasis-transition-duration",
        BP_EMPHASIS_TRANSITION_DURATION,
    ),
    ("--tm-d-loop", TM_D_LOOP),
    ("--tm-d-panel", TM_D_PANEL),
    ("--tm-d-small", TM_D_SMALL),
    ("--tm-d-stage", TM_D_STAGE),
    ("--tm-d-stagger", TM_D_STAGGER),
    ("--tm-d-state", TM_D_STATE),
];

/// Every curve token, by name.
pub const CURVES: &[(&str, Curve)] = &[
    ("--bp-emphasis-ease-bounce", BP_EMPHASIS_EASE_BOUNCE),
    ("--bp-emphasis-ease-default", BP_EMPHASIS_EASE_DEFAULT),
    ("--tm-ease-in", TM_EASE_IN),
    ("--tm-ease-out", TM_EASE_OUT),
    ("--tm-ease-spatial", TM_EASE_SPATIAL),
    ("--tm-ease-state", TM_EASE_STATE),
];

/// Every number token, by name.
pub const NUMBERS: &[(&str, f32)] = &[
    ("--bp-emphasis-motion-reduced", BP_EMPHASIS_MOTION_REDUCED),
    ("--bp-surface-layer-opacity", BP_SURFACE_LAYER_OPACITY),
    ("--bp-surface-z-index-0", BP_SURFACE_Z_INDEX_0),
    ("--bp-surface-z-index-1", BP_SURFACE_Z_INDEX_1),
    ("--bp-surface-z-index-2", BP_SURFACE_Z_INDEX_2),
    ("--bp-surface-z-index-3", BP_SURFACE_Z_INDEX_3),
    ("--bp-surface-z-index-4", BP_SURFACE_Z_INDEX_4),
    (
        "--bp-typography-line-height-default",
        BP_TYPOGRAPHY_LINE_HEIGHT_DEFAULT,
    ),
    (
        "--bp-typography-line-height-large",
        BP_TYPOGRAPHY_LINE_HEIGHT_LARGE,
    ),
    ("--bp-typography-weight-bold", BP_TYPOGRAPHY_WEIGHT_BOLD),
    (
        "--bp-typography-weight-default",
        BP_TYPOGRAPHY_WEIGHT_DEFAULT,
    ),
    ("--tm-weight-heading", TM_WEIGHT_HEADING),
];

/// Every font token, by name.
pub const FONTS: &[(&str, &[&str])] = &[
    (
        "--bp-typography-family-default",
        BP_TYPOGRAPHY_FAMILY_DEFAULT,
    ),
    ("--bp-typography-family-mono", BP_TYPOGRAPHY_FAMILY_MONO),
    ("--tm-font-code", TM_FONT_CODE),
    ("--tm-font-figure", TM_FONT_FIGURE),
    ("--tm-font-text", TM_FONT_TEXT),
];

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
pub const STAGES: [StageView; 7] = [
    StageView {
        name: "refining",
        icon: "edit",
    },
    StageView {
        name: "working",
        icon: "cog",
    },
    StageView {
        name: "in review",
        icon: "eye-open",
    },
    StageView {
        name: "with arbiter",
        icon: "comparison",
    },
    StageView {
        name: "checks",
        icon: "properties",
    },
    StageView {
        name: "merging",
        icon: "git-merge",
    },
    StageView {
        name: "deployed",
        icon: "rocket",
    },
];
