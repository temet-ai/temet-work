//! temet-work's desktop app. It draws the daemon's view data and sends commands; it owns no
//! state. Every part it draws is a part of the temet design system, in the layers
//! `docs/ui.md` sets out: `theme.rs` turns the design system's tokens into the UI library's
//! theme, and the parts live in `components/`, `composites/` and `pages/`. The rest is
//! plumbing and draws nothing: `daemon.rs` reads the daemon's view once a second into an entity
//! the window observes, `view_map.rs` maps that view into the Project page's props, `sample.rs`
//! holds sample data in the same shape for `--sample`, the tests and the screenshot tool,
//! `assets.rs` the icons and faces, and `launch.rs` the text rendering the window starts with.

pub mod assets;
pub mod components;
pub mod composites;
pub mod daemon;
pub mod install;
pub mod launch;
pub mod pages;
pub mod sample;
pub mod theme;
pub mod view_map;
