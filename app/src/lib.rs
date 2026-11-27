//! temet-work's desktop app. It draws the daemon's view data and sends commands; it owns no
//! state. Every part it draws is a part of the temet design system, in the layers
//! `docs/ui.md` sets out: `theme.rs` turns the design system's tokens into the UI library's
//! theme, and parts will live in `components/`, `composites/` and `pages/`.

pub mod theme;
