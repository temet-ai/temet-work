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

use gpui::{
    App, Div, Font, ParentElement, Pixels, SharedString, Styled, StyledText, TextRun, Window, div,
    font, px,
};

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
pub mod dialog;
pub mod form_group;
pub mod heading;
pub mod html_table;
pub mod icon;
pub mod input_group;
pub mod list_row;
pub mod meter;
pub mod plus_minus_tree;
pub mod progress_bar;
pub mod property_list;
pub mod radio_group;
pub mod section;
pub mod segmented_control;
pub mod stage_tag;
pub mod status_tag;
pub mod table_row;
pub mod tabs;
pub mod tag;
pub mod text_area;
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
    /// `.tm-decision-deadline`: "Overturn before ...", muted, in a decision's meta line or
    /// beside its action.
    DecisionDeadline,
    /// `.tm-round-head` on a compact ListRow: a review round's heading line, which starts where
    /// its points' text does.
    RoundHead,
    /// `.tm-pane-count`: the muted line counting where a pane's checks or decisions stand,
    /// over the list.
    PaneCount,
    /// `.tm-check-clamp` on a check's done-when: its first [`CHECK_LINES`] lines, the last
    /// ending in an ellipsis when there is more.
    CheckClamp,
    /// `.tm-check-toggle-none` on a Button: hidden, it holds the place of the plus a long check
    /// has, so status tags line up whether a row opens or not.
    CheckToggleNone,
    /// `.tm-refine` on a Dialog: RefineDialog's wide dialog, `--tm-dialog-wide` at most.
    Refine,
    /// `.tm-refine-head`: the muted "Done when" and "Proven by" over RefineDialog's checks.
    RefineHead,
    /// `.tm-refine-none`: "No checks yet", muted, in place of RefineDialog's checks.
    RefineNone,
    /// `.tm-issue-head`: IssuePane's head, its padding and the quiet rule under it.
    IssueHead,
    /// `.tm-issue-scroll`: the padding of the area under IssuePane's head that scrolls.
    IssueScroll,
    /// Blueprint's `.bp6-text-muted` (`Classes.TEXT_MUTED`): muted text, such as the label
    /// before a second control in a page's head.
    TextMuted,
    /// `.tm-problems-head`: the padding around ProblemList's heading.
    ProblemsHead,
    /// `.tm-problems-title`: ProblemList's icon and title on one line.
    ProblemsTitle,
}

/// The lines a long done-when shows until its row is opened: `.tm-check-clamp {
/// -webkit-line-clamp: 3 }`.
pub const CHECK_LINES: usize = 3;
/// The lines a long proven-by shows: `.tm-row-meta .tm-check-clamp { -webkit-line-clamp: 2 }`.
pub const CHECK_META_LINES: usize = 2;

/// One word of [`spans`], with its trailing space, and its class.
type Word = (SharedString, Option<Class>);

/// The words of `runs`, each with its trailing space.
fn words_of(runs: Vec<(SharedString, Option<Class>)>) -> Vec<Word> {
    runs.into_iter()
        .flat_map(|(text, class)| {
            text.split_inclusive(' ')
                .map(|word| (SharedString::from(word.to_string()), class))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A word as its own element, in its class.
fn word_element((word, class): Word, cx: &App) -> Div {
    let el = div().child(word);
    match class {
        Some(class) => apply(el, class, cx),
        None => el,
    }
}

/// The face and size a meta's word draws in: the code face for `.tm-mono`, else the text face.
fn word_run(class: Option<Class>, len: usize, cx: &App) -> (TextRun, Pixels) {
    let meta = look::get(cx, "CheckList", "tm-check-clamp tm-row-meta", State::Rest);
    let (family, size) = match class {
        Some(Class::Mono) => (
            text::code_family(cx),
            look::get(cx, "AppShell", "tm-mono tm-props-item", State::Rest).font_size,
        ),
        _ => (text::family(cx), meta.font_size),
    };
    let run = TextRun {
        len,
        font: Font {
            weight: meta.font_weight,
            ..font(family)
        },
        color: meta.text,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    (run, size)
}

/// One laid-out line of a meta as one run of text in each face, so its words keep the advances
/// the faces give them and share one baseline.
fn line_element(words: Vec<Word>, line: Pixels, cx: &App) -> Div {
    let mut text = String::new();
    let mut runs: Vec<TextRun> = Vec::new();
    for (word, class) in words {
        let (run, _) = word_run(class, word.len(), cx);
        text.push_str(&word);
        match runs.last_mut() {
            Some(last) if last.font == run.font => last.len += run.len,
            _ => runs.push(run),
        }
    }
    div()
        .h(line)
        .line_height(line)
        .whitespace_nowrap()
        .child(StyledText::new(text).with_runs(runs))
}

/// One line of a meta that holds code. The code face's ascent and descent make it taller than
/// a line of text alone: the measured meta of a check, which may run over several lines, divided
/// by its lines of text.
fn code_line(cx: &App) -> Pixels {
    let rest = State::Rest;
    let meta = look::get(cx, "CheckList", "tm-row-main tm-row-meta", rest).height;
    let text = look::get(cx, "ListRow", "tm-row-main tm-row-meta", rest).height;
    meta / (meta / text).round().max(1.)
}

/// How wide a word draws in a meta line, in its face.
fn word_width(word: &str, class: Option<Class>, window: &Window, cx: &App) -> Pixels {
    let (run, size) = word_run(class, word.len(), cx);
    window
        .text_system()
        .shape_line(SharedString::from(word.to_string()), size, &[run], None)
        .width
}

/// Text in runs, some with a class, flowing and wrapping as one paragraph, as the design
/// system's composites write `Proven by <span className="tm-mono">...</span>`. Each word is its
/// own element, measured in its own face: GPUI wraps a run of mixed faces by the widths of its
/// first face.
///
/// With the paragraph's `width` known, its words are laid out in lines as a browser lays them
/// out, a space at the end of a line taking no room, and with `lines` it shows at most that
/// many, the last ending in an ellipsis when words are left out, as `.tm-row-meta
/// .tm-check-clamp` shows a long proven-by. Also says whether words were left out. Until the
/// width is known the words wrap as a flex row.
pub fn spans(
    runs: Vec<(SharedString, Option<Class>)>,
    width: Option<Pixels>,
    lines: Option<usize>,
    window: &Window,
    cx: &App,
) -> (Div, bool) {
    let words = words_of(runs);
    let line = code_line(cx);
    let Some(width) = width else {
        let flow = div()
            .flex()
            .flex_wrap()
            .line_height(line)
            .children(words.into_iter().map(|w| word_element(w, cx)));
        return (flow, false);
    };
    // Each line's words, and how far its last word's text reaches.
    let mut laid: Vec<(Vec<Word>, Pixels)> = Vec::new();
    let mut used = Pixels::ZERO;
    for word in words {
        let text = word_width(word.0.trim_end(), word.1, window, cx);
        let whole = word_width(&word.0, word.1, window, cx);
        match laid.last_mut() {
            Some((these, end)) if used + text <= width => {
                *end = used + text;
                these.push(word);
            }
            _ => {
                laid.push((vec![word], text));
                used = Pixels::ZERO;
            }
        }
        used += whole;
    }
    let cut = lines.is_some_and(|n| laid.len() > n);
    if let Some(n) = lines.filter(|_| cut) {
        laid.truncate(n);
        let dots: SharedString = "\u{2026}".into();
        let room = width - word_width(&dots, None, window, cx);
        if let Some((these, end)) = laid.last_mut() {
            // Words come off the last line until the ellipsis fits after the last one.
            while these.len() > 1 && *end > room {
                these.pop();
                let mut reach = Pixels::ZERO;
                for (i, w) in these.iter().enumerate() {
                    let last = i + 1 == these.len();
                    let shown = if last { w.0.trim_end() } else { &w.0 };
                    reach += word_width(shown, w.1, window, cx);
                }
                *end = reach;
            }
            if let Some(last) = these.last_mut() {
                last.0 = last.0.trim_end().to_string().into();
            }
            these.push((dots, None));
        }
    }
    let rows = laid.into_iter().map(|(mut these, _)| {
        // A space at the end of a line takes no room.
        if let Some(last) = these.last_mut() {
            last.0 = last.0.trim_end().to_string().into();
        }
        line_element(these, line, cx)
    });
    (div().children(rows), cut)
}

/// How many lines `text` takes at `width` as a check's done-when (`.tm-check-clamp`) wraps it.
pub fn check_lines(text: &SharedString, width: Pixels, window: &Window, cx: &App) -> usize {
    let title = look::get(cx, "CheckList", "tm-check-clamp tm-row-title", State::Rest);
    let run = TextRun {
        len: text.len(),
        font: Font {
            weight: title.font_weight,
            ..font(text::family(cx))
        },
        color: title.text,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_text(text.clone(), title.font_size, &[run], Some(width), None)
        .map_or(1, |lines| {
            lines.iter().map(|l| l.wrap_boundaries().len() + 1).sum()
        })
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
        // `.tm-decision-deadline { color: muted }`; in the meta line it takes the meta's size.
        Class::DecisionDeadline => el.text_color(colour::muted(cx)),
        Class::RoundHead => padded(
            el,
            &look::get(
                cx,
                "ReviewRounds",
                "tm-round-head tm-rounds tm-row tm-row-compact",
                rest,
            ),
        ),
        Class::PaneCount => {
            let count = look::get(cx, "CheckList", "tm-checks tm-pane-count", rest);
            padded(type_of(el, &count), &count)
        }
        Class::CheckClamp => el.line_clamp(CHECK_LINES).text_ellipsis(),
        Class::IssueHead => {
            let head = look::get(cx, "IssuePane", "tm-issue-head tm-issue-pane", rest);
            box_of(el, &head)
        }
        Class::IssueScroll => padded(
            el,
            &look::get(
                cx,
                "IssuePane",
                "tm-issue-pane tm-issue-scroll tm-issue-stack",
                rest,
            ),
        ),
        Class::TextMuted => el.text_color(colour::muted(cx)),
        Class::ProblemsHead => padded(
            el,
            &look::get(cx, "ProblemList", "tm-cut-body tm-problems-head", rest),
        ),
        Class::ProblemsTitle => {
            let title = look::get(cx, "ProblemList", "tm-heading-text tm-problems-title", rest);
            type_of(el, &title)
                .flex()
                .items_center()
                .gap(space::s3(cx))
                .h(title.height)
                .line_height(browser_line(cx, title.font_size, title.height))
        }
        Class::CheckToggleNone => el.invisible(),
        Class::RefineHead => {
            let head = look::get(
                cx,
                "RefineDialog",
                ".tm-refine [class*='tm-']: tm-refine-checks tm-refine-head",
                rest,
            );
            padded(type_of(el, &head), &head)
        }
        // Measured nowhere: `.tm-refine-none { padding: var(--tm-space-3) 0; color: muted }`.
        Class::RefineNone => el
            .text_color(colour::muted(cx))
            .py(space::s3(cx)),
        // Dialog sizes itself for `Refine`, from the window's width.
        Class::Issues
        | Class::GroupsEmpty
        | Class::GroupsGroup
        | Class::NeedsHeading
        | Class::Refine => el,
    }
}
