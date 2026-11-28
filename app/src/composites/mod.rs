//! Composites: working pieces of a screen, arranged from components. A composite reads only
//! the theme's `space` and `size` values; its components set every colour, type and shape
//! (`docs/ui.md`, Composites).

pub mod check_list;
pub mod decision_row;
pub mod group_list;
pub mod issue_actions;
pub mod issue_card;
pub mod issue_pane;
pub mod issue_table;
pub mod meter_list;
pub mod milestone_panel;
pub mod needs_you;
pub mod page_header;
pub mod problem_list;
pub mod refine_dialog;
pub mod review_rounds;
pub mod side_nav;
pub mod stage_board;
pub mod stage_counts;
