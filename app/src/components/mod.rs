//! Components: one control or container each, drawn from the theme module's values
//! (`docs/ui.md`, Components). Each file is one part of the design system, with its name.
//!
//! A component takes its colours, type, padding, borders and corners from the look the design
//! system measured for it (`theme::look`), and the theme's role values only for what has no
//! measured look, such as a gap between two elements.
//!
//! This file also holds [`Class`]: the design system's own CSS classes that its composites put
//! on their markup, such as GroupList's group head or IssueTable's key cell. A composite here
//! reads no colour or type, so it names the class, and the class resolves to the look the
//! design system measured for an element with that class.

use gpui::{App, Div, Styled, div};

use crate::theme::look::{self, Look, State};
use crate::theme::{space, text};

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

/// The colour, size and weight of a look's text.
pub(crate) fn type_of<E: Styled>(el: E, look: &Look) -> E {
    el.text_color(look.text)
        .text_size(look.font_size)
        .font_weight(look.font_weight)
}

/// A look's padding.
pub(crate) fn padded<E: Styled>(el: E, look: &Look) -> E {
    let p = look.padding;
    el.pt(p.top).pr(p.right).pb(p.bottom).pl(p.left)
}

/// A look's box: its ground, padding, borders, corner and shadows.
pub(crate) fn box_of<E: Styled>(el: E, look: &Look) -> E {
    let p = look.padding;
    let b = look.border;
    let mut el = el
        .bg(look.background)
        .pt(p.top)
        .pr(p.right)
        .pb(p.bottom)
        .pl(p.left)
        .rounded(look.radius)
        .border_t(b.top)
        .border_r(b.right)
        .border_b(b.bottom)
        .border_l(b.left);
    if let Some(colour) = look.border_colour {
        el = el.border_color(colour);
    }
    if look.shadow.is_empty() {
        el
    } else {
        el.shadow(look.shadow.clone())
    }
}

/// A class from the design system's `temet.css` or `blueprint-overrides.css`, by its name less
/// the `tm-` prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `.tm-mono`: the code face at the code size.
    Mono,
    /// `.tm-issue-key`: a key cell, cut short with an ellipsis.
    IssueKey,
    /// `.tm-issue-title`: a title of up to two lines, then an ellipsis.
    IssueTitle,
    /// `.tm-issue-quiet`: an issue's agent and last change, and any other cell of an `Issues`
    /// table.
    IssueQuiet,
    /// `.tm-issues` on an HTMLTable: its rows and cells take IssueTable's looks, and a tag in
    /// a cell is raised by `--tm-space-1`.
    Issues,
    /// `.tm-groups-head`: a group's name and note on one line.
    GroupsHead,
    /// `.tm-groups-name`: a group's name, cut short with an ellipsis.
    GroupsName,
    /// `.tm-groups-note`: the note at the end of a group's head.
    GroupsNote,
    /// `.tm-groups-empty` on a ListRow: the row has no title, so its meta starts the line.
    GroupsEmpty,
    /// `.tm-groups-group` around a compact ListRow: its meta moves under the title when the
    /// two do not fit on one line, and is cut with an ellipsis only when longer than the line.
    GroupsGroup,
    /// `.tm-list-title` on a Heading: a list's title starts on the same line as its rows' text.
    ListTitle,
    /// `.tm-pagehead-muted`: a fact's label.
    PageheadMuted,
    /// `.tm-pagehead-icon`: a fact's icon, in the same colour as its label.
    PageheadIcon,
}

/// A plain element with a class, as the design system's composites write `<span
/// className="...">`. Its children are added with `child`.
pub fn classed(class: Class, cx: &App) -> Div {
    apply(div(), class, cx)
}

/// The class's look and rules on any element.
pub(crate) fn apply<E: Styled>(el: E, class: Class, cx: &App) -> E {
    let rest = State::Rest;
    match class {
        Class::Mono => type_of(
            el,
            &look::get(cx, "AppShell", "tm-mono tm-props-item", rest),
        )
        .font_family(text::code_family(cx)),
        Class::IssueKey => {
            let key = look::get(cx, "IssueTable", "tm-issue-key tm-issues tm-mono", rest);
            padded(type_of(el, &key), &key)
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
        }
        Class::IssueTitle => type_of(
            el,
            &look::get(cx, "IssueTable", "tm-issue-title tm-issues", rest),
        )
        .line_clamp(2)
        .text_ellipsis(),
        Class::IssueQuiet => {
            let quiet = look::get(cx, "IssueTable", "tm-issue-quiet tm-issues", rest);
            padded(type_of(el, &quiet), &quiet)
        }
        Class::GroupsHead => {
            let head = look::get(cx, "GroupList", "tm-groups-group tm-groups-head", rest);
            box_of(type_of(el, &head), &head)
                .flex()
                .justify_between()
                .items_baseline()
                .gap(space::s4(cx))
        }
        Class::GroupsName => type_of(
            el,
            &look::get(cx, "GroupList", "tm-groups-head tm-groups-name", rest),
        )
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis(),
        Class::GroupsNote => type_of(
            el,
            &look::get(cx, "GroupList", "tm-groups-head tm-groups-note", rest),
        )
        .flex_none(),
        Class::ListTitle => {
            let title = look::get(
                cx,
                "GroupList",
                "tm-heading tm-heading-section tm-list-title",
                rest,
            );
            el.pl(title.padding.left).pr(title.padding.right)
        }
        Class::PageheadMuted => type_of(
            el,
            &look::get(cx, "PageHeader", "tm-pagehead-fact tm-pagehead-muted", rest),
        ),
        Class::PageheadIcon => el.flex().text_color(
            look::get(cx, "PageHeader", "tm-pagehead-fact tm-pagehead-muted", rest).text,
        ),
        Class::Issues | Class::GroupsEmpty | Class::GroupsGroup => el,
    }
}
