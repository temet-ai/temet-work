//! temet-work's desktop app. It draws the daemon's view data and sends commands; it owns no
//! state. Every part it draws is a part of the temet design system, in the layers
//! `docs/ui.md` sets out: `theme.rs` turns the design system's tokens into the UI library's
//! theme, and the parts live in `components/`, `composites/` and `pages/`. `sample.rs` holds
//! the data the window shows until the daemon sends its view, `assets.rs` the icons and faces,
//! and `launch.rs` the text rendering the window starts with.

pub mod assets;
pub mod components;
pub mod composites;
pub mod launch;
pub mod pages;
pub mod sample;
pub mod theme;
