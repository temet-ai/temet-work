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

use gpui::{App, Div, ParentElement, Pixels, SharedString, Styled, Window, div, font, px};

use crate::theme::look::{self, Look, State};
use crate::theme::{colour, size, space, text};

/// The design system's intents, as the components' `intent` props take them.
pub use crate::theme::Intent;

pub mod alert;
pub mod app_shell;
pub mod attention_button;
pub mod attention_text;
pub mod button;
pub mod callout;
pub mod cut_panel;
pub mod heading;
pub mod html_table;
pub mod icon;
pub mod list_row;
pub mod meter;
pub mod plus_minus_tree;
pub mod progress_bar;
pub mod property_list;
pub mod section;
pub mod segmented_control;
pub mod stage_tag;
pub mod status_tag;
pub mod table_row;
pub mod tabs;
pub mod tag;
pub mod toast;

/// How far below the top of its line box the baseline of `size` text sits, in the text face in
/// force, as GPUI places a line: half the leading, then the ascent. GPUI rounds a line height to
/// whole pixels (`TextStyle::line_height_in_pixels`), so the line is rounded first.
pub(crate) fn baseline(cx: &App, size: Pixels, line: Pixels) -> Pixels {
    let line = line.round();
    let system = cx.text_system();
    let id = system.resolve_font(&font(text::family(cx)));
    let ascent = system.ascent(id, size);
    // GPUI gives the descent as a negative offset from the baseline, as fonts store it.
    let descent = system.descent(id, size).abs();
    (line - ascent - descent) / 2. + ascent
}

/// The height of a line box, no taller than `line`, that GPUI centres in a box of `line` and
/// draws `size` text in on the pixel row a browser draws it in a line of `line`. A browser
/// rounds the font's ascent and descent to whole pixels and gives the ascent side the floor of
/// the half leading; GPUI centres the unrounded ascent and descent and rounds the result, which
/// can put the text a pixel lower. The design system's stories are drawn by a browser.
pub(crate) fn browser_line(cx: &App, size: Pixels, line: Pixels) -> Pixels {
    let system = cx.text_system();
    let id = system.resolve_font(&font(text::family(cx)));
    let ascent = system.ascent(id, size);
    let descent = system.descent(id, size).abs();
    let browser = ascent.round() + ((line - ascent.round() - descent.round()) / 2.).floor();
    // GPUI snaps a centred box's top edge to whole pixels, halves toward zero.
    let top = |height: Pixels| px((f32::from((line - height) / 2.) - 0.5).ceil());
    // GPUI rounds line heights to whole pixels: try each, tallest first.
    let tallest = f32::from(line.round()) as i32;
    let shortest = f32::from((ascent + descent).ceil()) as i32;
    (shortest..=tallest)
        .rev()
        .map(|whole| px(whole as f32))
        .find(|&height| top(height) + baseline(cx, size, height).round() == browser)
        .unwrap_or(line)
}

/// How far a line box reaches below its baseline, in the text style in force: the font's
/// descent plus half the leading. GPUI gives the descent as a negative offset from the
/// baseline, as fonts store it.
pub(crate) fn below_baseline(window: &Window) -> Pixels {
    let style = window.text_style();
    let rem = window.rem_size();
    let font_size = style.font_size.to_pixels(rem);
    let line = style.line_height_in_pixels(rem);
    let text = window.text_system();
    let font = text.resolve_font(&style.font());
    let ascent = text.ascent(font, font_size);
    let descent = text.descent(font, font_size).abs();
    (line - (ascent + descent)) / 2. + descent
}

/// How far to move text in `low`'s look down so its baseline meets that of text in `high`'s
/// look beside it: the design system aligns such text on its baseline (`align-items:
/// baseline`). A look's height is its one line box.
pub(crate) fn to_baseline(cx: &App, high: &Look, low: &Look) -> Pixels {
    baseline(cx, high.font_size, high.height) - baseline(cx, low.font_size, low.height)
}

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
    /// `.tm-mono`: the code face at the code size, in the colour of the text around it.
    Mono,
    /// `.tm-issue-key`: a key cell, cut short with an ellipsis.
    IssueKey,
    /// `.tm-issue-title`: a title of up to two lines, then an ellipsis.
    IssueTitle,
    /// `.tm-issue-quiet`: an issue's agent and last change, and any other cell of an `Issues`
    /// table.
    IssueQuiet,
    /// `.tm-issue-quiet` on the padded SectionCard an empty IssueTable draws in place of a
    /// table: "No issues", muted.
    NoIssues,
    /// `.tm-issues` on an HTMLTable: its rows and cells take IssueTable's looks, and a tag in
    /// a cell is raised so its word lines up with the text beside it.
    Issues,
    /// `.tm-issues .tm-status` on the StageTag of a group row: raised by `--tm-space-1` above
    /// and below, as a tag in a body row is, its word in the head's weight, which the tag
    /// takes from its `th`.
    IssuesStatus,
    /// `.tm-issue-group`: a group row's stage or name, then its count.
    IssueGroup,
    /// `.tm-issue-count`: a group row's count of issues.
    IssueCount,
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
    /// `.tm-pagehead-muted`: a fact's label, and the label before the page's controls.
    PageheadMuted,
    /// `.tm-pagehead-icon`: a fact's icon, in the same colour as its label.
    PageheadIcon,
    /// `.tm-needs-head`: the padding around NeedsYou's heading.
    NeedsHead,
    /// NeedsYou's Heading, inside `.tm-needs-head`: its line is tall enough for the bell icon.
    NeedsHeading,
    /// `.tm-needs-title` on AttentionText: NeedsYou's icon and title.
    NeedsTitle,
    /// `.tm-decision-issue`: the issue's key at the start of a decision's meta line.
    DecisionIssue,
    /// `.tm-decision-done`: the note that replaces an overturned decision's action.
    DecisionDone,
    /// `.tm-round-head` on a compact ListRow: a review round's heading line, which starts where
    /// its points' text does.
    RoundHead,
}

/// Text in runs, some with a class, flowing and wrapping as one paragraph, as the design
/// system's composites write `Proven by <span className="tm-mono">...</span>`. Each word is
/// its own element, so a word in the code face is measured in that face when the line wraps:
/// GPUI wraps a run of mixed faces by the widths of its first face.
pub fn spans(runs: Vec<(SharedString, Option<Class>)>, cx: &App) -> Div {
    let words = runs.into_iter().flat_map(|(text, class)| {
        text.split_inclusive(' ')
            .map(|word| {
                let el = div().child(SharedString::from(word.to_string()));
                match class {
                    Some(class) => apply(el, class, cx),
                    None => el,
                }
            })
            .collect::<Vec<_>>()
    });
    // A line holding code is the measured line of a meta with code in it, taller than one of
    // text alone, as the code face's ascent and descent widen the line box. GPUI rounds a line
    // height to whole pixels, where the story keeps the fraction and a wrapped line lands the
    // next whole pixel down, so the line is rounded up.
    let line = look::get(cx, "IssuePane", "tm-row-main tm-row-meta", State::Rest).height;
    div()
        .flex()
        .flex_wrap()
        .line_height(line.ceil())
        .children(words)
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
        // `.tm-mono` sets the face and size only; the colour is the text's around it.
        Class::Mono => el
            .text_size(look::get(cx, "AppShell", "tm-mono tm-props-item", rest).font_size)
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
        // The export has no look for it yet: the muted role, and a compact section card's
        // padding (`.bp6-section.bp6-compact .bp6-section-card.bp6-padded`).
        Class::NoIssues => el
            .text_color(colour::muted(cx))
            .p(size::section_pad(cx, true)),
        Class::IssueGroup => el.flex().items_center().gap(space::s4(cx)),
        Class::IssuesStatus => el
            .mt(-space::s1(cx))
            .mb(-space::s1(cx))
            .font_weight(look::get(cx, "HTMLTable", "th: compact interactive", rest).font_weight),
        // Measured nowhere: `font-weight: default; color: muted`.
        Class::IssueCount => el
            .font_weight(text::normal_weight(cx))
            .text_color(colour::muted(cx)),
        Class::GroupsHead => {
            let head = look::get(cx, "GroupList", "tm-groups-group tm-groups-head", rest);
            box_of(type_of(el, &head), &head)
                .flex()
                .justify_between()
                .items_start()
                .gap(space::s4(cx))
        }
        Class::GroupsName => {
            let name = look::get(cx, "GroupList", "tm-groups-head tm-groups-name", rest);
            type_of(el, &name)
                .line_height(name.height)
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
        }
        Class::GroupsNote => {
            let name = look::get(cx, "GroupList", "tm-groups-head tm-groups-name", rest);
            let note = look::get(cx, "GroupList", "tm-groups-head tm-groups-note", rest);
            type_of(el, &note)
                .flex_none()
                .line_height(note.height)
                .mt(to_baseline(cx, &name, &note))
        }
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
        Class::NeedsHead => padded(
            el,
            &look::get(cx, "NeedsYou", "tm-cut-body tm-needs-head", rest),
        ),
        Class::NeedsTitle => {
            // An inline-flex box one line of its own tall, at the top of the heading's taller
            // line, with the bell centred beside the words.
            let title = look::get(
                cx,
                "NeedsYou",
                "tm-attention-text tm-heading-text tm-needs-title",
                rest,
            );
            // The words take the line a browser would draw them on; the box keeps the title's
            // line, so the bell stays centred in it.
            type_of(el, &title)
                .flex()
                .items_center()
                .gap(space::s3(cx))
                .h(title.height)
                .line_height(browser_line(cx, title.font_size, title.height))
        }
        Class::DecisionIssue => type_of(
            el,
            &look::get(cx, "DecisionRow", "tm-decision-issue tm-row-meta", rest),
        ),
        Class::DecisionDone => type_of(
            el,
            &look::get(cx, "DecisionRow", "tm-decision-done tm-row-end", rest),
        ),
        Class::RoundHead => padded(
            el,
            &look::get(
                cx,
                "ReviewRounds",
                "tm-round-head tm-rounds tm-row tm-row-compact",
                rest,
            ),
        ),
        Class::Issues | Class::GroupsEmpty | Class::GroupsGroup | Class::NeedsHeading => el,
    }
}
