//! Composites: working pieces of a screen, arranged from components. A composite reads only
//! the theme's `space` and `size` values; its components set every colour, type and shape
//! (`docs/ui.md`, Composites).

pub mod group_list;
pub mod issue_table;
pub mod milestone_panel;
pub mod page_header;
pub mod side_nav;
