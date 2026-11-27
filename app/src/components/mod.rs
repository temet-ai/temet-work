//! Components: one control or container each, drawn from the theme module's values
//! (`docs/ui.md`, Components). Each file is one part of the design system, with its name.
//!
//! This file also holds [`Class`]: the design system's own CSS classes that its composites put
//! on their markup, such as GroupList's group head or IssueTable's key cell. A composite here
//! reads no colour or type, so it names the class and the component layer draws it.

use gpui::{App, Div, Styled, div};

use crate::theme;

pub mod app_shell;
pub mod cut_panel;
pub mod heading;
pub mod html_table;
pub mod icon;
pub mod list_row;
pub mod plus_minus_tree;
pub mod progress_bar;
pub mod section;
pub mod stage_tag;
pub mod status_tag;
pub mod table_row;
pub mod tag;

/// A class from the design system's `temet.css` or `blueprint-overrides.css`, by its name less
/// the `tm-` prefix. Each draws what its rule there says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `.tm-mono`: the code face at the code size.
    Mono,
    /// `.tm-issue-key`: a key cut short with an ellipsis; in an `Issues` table its cell's end
    /// padding is `--tm-space-1`.
    IssueKey,
    /// `.tm-issue-title`: a title of up to two lines, then an ellipsis.
    IssueTitle,
    /// `.tm-issues` on an HTMLTable: its cells' side padding is `--tm-space-4`, and a tag in a
    /// cell is raised by `--tm-space-1`.
    Issues,
    /// `.tm-groups-head`: a group's name and note on one line, muted, at body size in the
    /// heading weight.
    GroupsHead,
    /// `.tm-groups-name`: a group's name, cut short with an ellipsis.
    GroupsName,
    /// `.tm-groups-note`: small muted text at the end of a group's head.
    GroupsNote,
    /// `.tm-groups-empty` on a ListRow: the row has no title, so its meta starts the line.
    GroupsEmpty,
    /// `.tm-groups-group` around a compact ListRow: its meta moves under the title when the
    /// two do not fit on one line, and is cut with an ellipsis only when longer than the line.
    GroupsGroup,
    /// `.tm-list-title` on a Heading: a list's title starts on the same line as its rows' text.
    ListTitle,
    /// `.tm-pagehead-muted`: a fact's label, muted.
    PageheadMuted,
    /// `.tm-pagehead-icon`: a fact's icon, muted.
    PageheadIcon,
}

/// A plain element with a class, as the design system's composites write `<span
/// className="...">`. Its children are added with `child`.
pub fn classed(class: Class, cx: &App) -> Div {
    apply(div(), class, cx)
}

/// The rules of `class` on any element.
pub(crate) fn apply<E: Styled>(el: E, class: Class, cx: &App) -> E {
    use theme::{colour, space, text};
    match class {
        Class::Mono => el
            .font_family(text::code_family(cx))
            .text_size(text::code(cx)),
        Class::IssueKey => el.overflow_hidden().whitespace_nowrap().text_ellipsis(),
        Class::IssueTitle => el.line_clamp(2).text_ellipsis(),
        Class::GroupsHead => el
            .flex()
            .justify_between()
            .items_baseline()
            .gap(space::s4(cx))
            .py(space::s1(cx))
            .px(space::s2(cx))
            .text_color(colour::muted(cx))
            .text_size(text::body(cx))
            .font_weight(text::heading_weight(cx)),
        Class::GroupsName => el
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis(),
        Class::GroupsNote => el
            .flex_none()
            .text_size(text::small(cx))
            .font_weight(text::normal_weight(cx))
            .text_color(colour::muted(cx)),
        Class::ListTitle => el.px(space::s2(cx)),
        Class::PageheadMuted => el.text_color(colour::muted(cx)),
        Class::PageheadIcon => el.flex().text_color(colour::muted(cx)),
        Class::Issues | Class::GroupsEmpty | Class::GroupsGroup => el,
    }
}
