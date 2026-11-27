//! The check used when reviewing UI work. It reads the front-end crate's source and the layer
//! map, `docs/ui.md`, and lists every place where a part breaks the map's rules.
//!
//! Files are named by their path under the front-end crate's `src/`. `theme.rs` is the theme
//! module, the one file that reads this crate. `components/`, `composites/` and `pages/` hold one
//! part per file, named after the part. Every other file is plumbing: opening the window,
//! wiring in the daemon.

/// One place where the UI breaks a rule of the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub file: String,
    /// From 1; 0 when the finding is about the whole file.
    pub line: usize,
    pub rule: Rule,
    pub what: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {:?}: {}",
            self.file, self.line, self.rule, self.what
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// A colour, size, shape, weight or duration written as a literal, not read from a token.
    InlineValue,
    /// A part uses a layer above its own, or a kind of value its layer may not set.
    Layer,
    /// A part file with no row in the map, a row the design system does not have yet, or no
    /// type with the part's name.
    Map,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    Theme,
    Component,
    Composite,
    Page,
    Plumbing,
}

fn layer_of(path: &str) -> Layer {
    match path.split('/').next() {
        _ if path == "theme.rs" => Layer::Theme,
        Some("components") => Layer::Component,
        Some("composites") => Layer::Composite,
        Some("pages") => Layer::Page,
        _ => Layer::Plumbing,
    }
}

/// One row of the map: a part's names, and its design-system id, or `None` while it is a gap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapRow {
    pub names: Vec<String>,
    pub id: Option<String>,
}

/// The parts the map lists, by layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    pub components: Vec<MapRow>,
    pub composites: Vec<MapRow>,
    pub pages: Vec<MapRow>,
}

/// Reads the Components, Composites and Pages tables of the map. A part's names are the first
/// column; its id is the first code span in the column headed "Design system", or none when that
/// cell starts with "gap".
pub fn parse_map(markdown: &str) -> Map {
    let mut map = Map::default();
    let mut section = "";
    let mut id_col: Option<usize> = None;
    for line in markdown.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            section = h.trim();
            id_col = None;
            continue;
        }
        let rows = match section {
            "Components" => &mut map.components,
            "Composites" => &mut map.composites,
            "Pages" => &mut map.pages,
            _ => continue,
        };
        let Some(inner) = line.trim().strip_prefix('|') else {
            continue;
        };
        let cells: Vec<&str> = inner
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let Some(col) = id_col else {
            id_col = cells.iter().position(|c| c.starts_with("Design system"));
            continue;
        };
        if cells.first().is_some_and(|c| c.starts_with("---")) {
            continue;
        }
        let names = cells[0]
            .split(',')
            .map(|n| n.trim().trim_matches('`').to_string())
            .filter(|n| !n.is_empty())
            .collect();
        let cell = cells.get(col).copied().unwrap_or("");
        let id = if cell.starts_with("gap") {
            None
        } else {
            cell.split('`').nth(1).map(str::to_string)
        };
        rows.push(MapRow { names, id });
    }
    map
}

/// A part's name as a file stem: `StatusTag` is `status_tag`, `HTMLTable` is `html_table`.
pub fn snake(name: &str) -> String {
    let c: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, &ch) in c.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let after_lower = i > 0 && (c[i - 1].is_ascii_lowercase() || c[i - 1].is_ascii_digit());
            let ends_acronym = i > 0
                && c[i - 1].is_ascii_uppercase()
                && c.get(i + 1).is_some_and(char::is_ascii_lowercase);
            if after_lower || ends_acronym {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Reviews the front-end crate's files, each given as its path under `src/` and its text,
/// against the map.
pub fn review(files: &[(String, String)], map_markdown: &str) -> Vec<Finding> {
    let map = parse_map(map_markdown);
    let mut found = Vec::new();
    for (path, text) in files {
        let layer = layer_of(path);
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split("//").next().unwrap_or("");
            let mut add = |rule, what: String| {
                let f = Finding {
                    file: path.clone(),
                    line: n + 1,
                    rule,
                    what,
                };
                if !found.contains(&f) {
                    found.push(f);
                }
            };
            for what in inline_values(line) {
                add(Rule::InlineValue, what);
            }
            for what in layer_breaks(layer, line) {
                add(Rule::Layer, what);
            }
        }
        found.extend(map_breaks(&map, layer, path, text));
    }
    found
}

// ---------- Reading a line ----------

const fn ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every call in a line: whether it is a method call, the name called, and the text after `(`.
fn calls(line: &str) -> Vec<(bool, &str, &str)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if (c.is_ascii_alphabetic() || c == '_') && (i == 0 || !ident_char(bytes[i - 1] as char)) {
            let start = i;
            while i < bytes.len() && ident_char(bytes[i] as char) {
                i += 1;
            }
            let name = &line[start..i];
            let mut j = i;
            if line[j..].starts_with("::<") {
                // A turbofish: skip to its end.
                j = line[j..].find('>').map_or(j, |k| j + k + 1);
            }
            if line[j..].starts_with('(') {
                let method = start > 0 && bytes[start - 1] == b'.';
                out.push((method, name, &line[j + 1..]));
            }
            continue;
        }
        i += 1;
    }
    out
}

/// Whether an argument list starts with a number other than zero.
fn literal(args: &str) -> bool {
    let a = args.trim_start();
    let a = a.strip_prefix('-').unwrap_or(a);
    if !a.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
        return false;
    }
    let num: String = a
        .chars()
        .take_while(|c| ident_char(*c) || *c == '.')
        .collect();
    let digits = num.trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '_');
    !digits
        .trim_end_matches('.')
        .chars()
        .all(|c| c == '0' || c == '.')
}

fn no_args(args: &str) -> bool {
    args.trim_start().starts_with(')')
}

/// Whether a line names `word` as a path segment: not as the end of a longer name.
fn names(line: &str, word: &str) -> bool {
    line.match_indices(word)
        .any(|(at, _)| !line[..at].ends_with(ident_char))
}

/// The UI library's size scale helpers, such as `p_2`, `gap_x_4` and `w_64`.
fn scale_helper(name: &str) -> bool {
    const SIDES: [&str; 30] = [
        "px",
        "py",
        "pt",
        "pr",
        "pb",
        "pl",
        "p",
        "mx",
        "my",
        "mt",
        "mr",
        "mb",
        "ml",
        "m",
        "gap_x",
        "gap_y",
        "gap",
        "min_w",
        "min_h",
        "max_w",
        "max_h",
        "w",
        "h",
        "size",
        "top",
        "right",
        "bottom",
        "left",
        "inset",
        "flex_basis",
    ];
    SIDES.iter().any(|s| {
        name.strip_prefix(s)
            .and_then(|r| r.strip_prefix('_'))
            .is_some_and(|step| {
                let step = step.strip_prefix("neg_").unwrap_or(step);
                step == "px"
                    || (step.starts_with(|c: char| c.is_ascii_digit())
                        && step != "0"
                        && step
                            .chars()
                            .all(|c| c.is_ascii_digit() || c == 'p' || c == '_'))
            })
    })
}

fn inline_values(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (method, name, args) in calls(line) {
        let what = match name {
            "rgb" | "rgba" | "hsl" | "hsla" | "Rgba" if literal(args) => "a colour literal",
            "black" | "white" | "red" | "green" | "blue" | "yellow" | "transparent_black"
            | "transparent_white"
                if !method && no_args(args) =>
            {
                "a named colour"
            }
            "px" | "rems" | "rem" | "relative" if literal(args) => "a size literal",
            "from_millis" | "from_micros" | "from_secs" | "from_secs_f32" | "from_secs_f64"
                if literal(args) =>
            {
                "a duration literal"
            }
            "opacity" if method && literal(args) => "an opacity literal",
            "corner_radii" => "a corner set by hand",
            n if method && (n == "rounded" || n.starts_with("rounded_")) => "a corner set by hand",
            n if method && no_args(args) && scale_helper(n) => "a size from the library's scale",
            "text_xs" | "text_sm" | "text_base" | "text_lg" | "text_xl" | "text_2xl"
            | "text_3xl" | "text_4xl"
                if method =>
            {
                "a text size from the library's scale"
            }
            "font_thin" | "font_extralight" | "font_light" | "font_normal" | "font_medium"
            | "font_semibold" | "font_bold" | "font_extrabold" | "font_black"
                if method =>
            {
                "a font weight from the library's scale"
            }
            n if method
                && n.starts_with("border")
                && n.rsplit('_')
                    .next()
                    .is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()))
                && n.rsplit('_').next() != Some("0")
                && n.contains('_') =>
            {
                "a border width from the library's scale"
            }
            n if method && n.starts_with("shadow_") => "a shadow from the library's scale",
            _ => continue,
        };
        out.push(format!("{what}: `{name}(`"));
    }
    for (at, _) in line.match_indices("0x") {
        let hex: String = line[at + 2..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect();
        let next = line[at + 2 + hex.len()..].chars().next();
        if (hex.len() == 6 || hex.len() == 8) && !next.is_some_and(ident_char) {
            out.push(format!("a colour literal: `0x{hex}`"));
        }
    }
    for word in ["FontWeight::", "BoxShadow"] {
        if names(line, word) {
            out.push(format!("a literal `{}`", word.trim_end_matches(':')));
        }
    }
    out
}

/// The kinds of token a component reads and a composite may not.
const LOOK: [&str; 4] = [
    "theme::colour",
    "theme::text",
    "theme::shape",
    "theme::motion",
];

/// Calls that set how something looks.
const LOOK_CALLS: [&str; 9] = [
    "bg",
    "text_color",
    "border_color",
    "font_family",
    "text_size",
    "font_weight",
    "line_height",
    "opacity",
    "shadow",
];

/// Calls that set space or size.
const SPACE_CALLS: [&str; 24] = [
    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mx", "my", "mt", "mr", "mb", "ml", "gap",
    "gap_x", "gap_y", "w", "h", "size", "min_w", "min_h", "max_w", "max_h",
];

fn layer_breaks(layer: Layer, line: &str) -> Vec<String> {
    let mut out = Vec::new();
    if layer != Layer::Theme && names(line, "temet_work_style") {
        out.push("reads temet-work-style; only the theme module does".into());
    }
    match layer {
        Layer::Theme => {}
        Layer::Component => {
            for above in ["composites::", "pages::"] {
                if names(line, above) {
                    out.push(format!("a component uses `{above}`, a layer above it"));
                }
            }
        }
        Layer::Composite => {
            if names(line, "pages::") {
                out.push("a composite uses `pages::`, a layer above it".into());
            }
            if names(line, "gpui_component") {
                out.push("a composite uses the UI library; it is built from components".into());
            }
            for kind in LOOK {
                if names(line, kind) {
                    out.push(format!(
                        "a composite reads `{kind}`; it reads only space and size"
                    ));
                }
            }
            for (method, name, _) in calls(line) {
                if method && LOOK_CALLS.contains(&name) {
                    out.push(format!(
                        "a composite sets `{name}`; its components set the look"
                    ));
                }
            }
        }
        Layer::Page | Layer::Plumbing => {
            let who = if layer == Layer::Page {
                "a page"
            } else {
                "plumbing"
            };
            if layer == Layer::Page && names(line, "gpui_component") {
                out.push(
                    "a page uses the UI library; it is built from composites and components".into(),
                );
            }
            if layer == Layer::Page && names(line, "theme::") {
                out.push("a page reads the theme; it sets no value".into());
            }
            for (method, name, _) in calls(line) {
                if !method && ["div", "h_flex", "v_flex"].contains(&name) {
                    out.push(format!("{who} draws its own `{name}()`; parts draw"));
                }
                if method && (LOOK_CALLS.contains(&name) || SPACE_CALLS.contains(&name)) {
                    out.push(format!("{who} sets `{name}`; it sets no value"));
                }
            }
        }
    }
    out
}

fn map_breaks(map: &Map, layer: Layer, path: &str, text: &str) -> Vec<Finding> {
    let (rows, table) = match layer {
        Layer::Component => (&map.components, "Components"),
        Layer::Composite => (&map.composites, "Composites"),
        Layer::Page => (&map.pages, "Pages"),
        _ => return Vec::new(),
    };
    let stem = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".rs");
    if stem == "mod" {
        return Vec::new();
    }
    let finding = |what: String| {
        vec![Finding {
            file: path.into(),
            line: 0,
            rule: Rule::Map,
            what,
        }]
    };
    let Some((row, name)) = rows
        .iter()
        .find_map(|r| r.names.iter().find(|n| snake(n) == stem).map(|n| (r, n)))
    else {
        return finding(format!(
            "`{stem}` has no row in the {table} table of docs/ui.md"
        ));
    };
    if row.id.is_none() {
        return finding(format!(
            "{name} is a gap: the design system does not have it yet"
        ));
    }
    if !text.contains(&format!("pub struct {name}")) {
        return finding(format!(
            "declares no `pub struct {name}`, the design system's name"
        ));
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP: &str = "\
## Components

| Part | Design system id | Variants |
|---|---|---|
| Tag | `temet-tag` | `minimal` |
| StatusTag | `temet-statustag` | `status` |
| HTMLTable | `blueprint-htmltable` | `compact` |
| Tabs, Tab | `blueprint-tabs` | |
| Heading | gap | page title |

## Composites

| Part | What it is | Built from | Design system |
|---|---|---|---|
| SideNav | areas and projects | PlusMinusTree | gap; drawn by the radar page |
| IssueTable | issues as rows | HTMLTable | `temet-issuetable` |

## Pages

| Page | What it shows | Built from | Design system |
|---|---|---|---|
| Project | one project | AppShell | `pages-temet-work-radar` (RadarScreen) |
";

    fn one(path: &str, text: &str) -> Vec<Finding> {
        review(&[(path.into(), text.into())], MAP)
    }

    fn whats(path: &str, text: &str) -> Vec<String> {
        one(path, text).into_iter().map(|f| f.what).collect()
    }

    #[test]
    fn reads_the_maps_tables() {
        let m = parse_map(MAP);
        assert_eq!(m.components.len(), 5);
        assert_eq!(m.components[3].names, ["Tabs", "Tab"]);
        assert_eq!(m.components[1].id.as_deref(), Some("temet-statustag"));
        assert_eq!(m.components[4].id, None);
        assert_eq!(m.composites[0].id, None);
        assert_eq!(m.composites[1].id.as_deref(), Some("temet-issuetable"));
        assert_eq!(m.pages[0].id.as_deref(), Some("pages-temet-work-radar"));
    }

    #[test]
    fn snake_names_follow_the_part() {
        assert_eq!(snake("StatusTag"), "status_tag");
        assert_eq!(snake("HTMLTable"), "html_table");
        assert_eq!(snake("KeyComboTag"), "key_combo_tag");
        assert_eq!(snake("SideNav"), "side_nav");
        assert_eq!(snake("Tag"), "tag");
    }

    #[test]
    fn flags_every_kind_of_inline_value() {
        let lines = [
            ("div().bg(rgb(0x14101a))", "a colour literal"),
            ("let c = rgba(0xd41b6f33);", "a colour literal"),
            (
                "const EDGE: u32 = 0xffffff2e;",
                "a colour literal: `0xffffff2e`",
            ),
            (".text_color(white())", "a named colour"),
            (".w(px(260.))", "a size literal"),
            (".h(rems(1.5))", "a size literal"),
            (".p_2().gap_x_4()", "a size from the library's scale"),
            (".text_sm()", "a text size from the library's scale"),
            (".font_bold()", "a font weight from the library's scale"),
            (".font_weight(FontWeight::BOLD)", "a literal `FontWeight`"),
            (".rounded_md()", "a corner set by hand"),
            (".rounded(px(4.))", "a corner set by hand"),
            (".border_1()", "a border width from the library's scale"),
            (".shadow_lg()", "a shadow from the library's scale"),
            (".opacity(0.45)", "an opacity literal"),
            ("Duration::from_millis(200)", "a duration literal"),
        ];
        for (code, want) in lines {
            let got = whats("components/tag.rs", code);
            assert!(got.iter().any(|w| w.starts_with(want)), "{code}: {got:?}");
        }
    }

    #[test]
    fn lets_values_read_from_tokens_through() {
        let code = "\
pub struct Tag;
// .rounded_md() in a comment
fn tag(t: &Theme) -> Div {
    div().w_full().h_full().size_full().flex_1().p(t.space.small).bg(rgba(t.colour.edge.0))
        .w(px(0.)).border_0().top_0().opacity(t.motion.dim)
}";
        assert!(
            one("components/tag.rs", code).is_empty(),
            "{:?}",
            one("components/tag.rs", code)
        );
    }

    #[test]
    fn the_theme_module_holds_no_literal_either() {
        assert!(
            whats("theme.rs", "rgba(0xd41b6fff)")
                .iter()
                .any(|w| w.starts_with("a colour literal"))
        );
        assert!(whats("theme.rs", "use temet_work_style::tokens::*;").is_empty());
    }

    #[test]
    fn only_the_theme_module_reads_the_style_crate() {
        let got = whats(
            "components/tag.rs",
            "pub struct Tag; use temet_work_style::tokens;",
        );
        assert!(
            got.iter().any(|w| w.contains("only the theme module")),
            "{got:?}"
        );
    }

    #[test]
    fn a_component_uses_no_layer_above_it() {
        let got = whats(
            "components/tag.rs",
            "pub struct Tag; use crate::composites::side_nav;",
        );
        assert!(got.iter().any(|w| w.contains("`composites::`")), "{got:?}");
    }

    #[test]
    fn a_composite_reads_only_space_and_size() {
        let code = "pub struct IssueTable;\nuse gpui_component::table;\nlet c = theme::colour::EDGE;\nrow.bg(c)";
        let got = whats("composites/issue_table.rs", code);
        assert!(got.iter().any(|w| w.contains("UI library")), "{got:?}");
        assert!(got.iter().any(|w| w.contains("`theme::colour`")), "{got:?}");
        assert!(got.iter().any(|w| w.contains("sets `bg`")), "{got:?}");
        let ok = "pub struct IssueTable;\nlet g = theme::space::ROW;\nrows.gap(g)";
        assert!(one("composites/issue_table.rs", ok).is_empty());
    }

    #[test]
    fn a_page_sets_no_value() {
        let code = "pub struct Project;\nuse gpui_component::Root;\nv_flex().gap(theme::space::PAGE).child(nav)";
        let got = whats("pages/project.rs", code);
        assert!(got.iter().any(|w| w.contains("UI library")), "{got:?}");
        assert!(got.iter().any(|w| w.contains("reads the theme")), "{got:?}");
        assert!(got.iter().any(|w| w.contains("`v_flex()`")), "{got:?}");
        assert!(got.iter().any(|w| w.contains("sets `gap`")), "{got:?}");
    }

    #[test]
    fn plumbing_draws_nothing_but_may_start_the_library() {
        assert!(one("main.rs", "gpui_component::init(cx); theme::apply(cx);").is_empty());
        let got = whats("main.rs", "div().bg(theme::colour::PAGE)");
        assert!(got.iter().any(|w| w.contains("`div()`")), "{got:?}");
    }

    #[test]
    fn every_part_file_has_a_row_with_a_design_system_id() {
        assert!(one("components/status_tag.rs", "pub struct StatusTag;").is_empty());
        assert!(one("components/html_table.rs", "pub struct HTMLTable;").is_empty());
        assert!(one("components/tab.rs", "pub struct Tab;").is_empty());
        assert!(one("composites/issue_table.rs", "pub struct IssueTable;").is_empty());
        assert!(one("pages/project.rs", "pub struct Project;").is_empty());
        assert!(one("components/mod.rs", "pub mod tag;").is_empty());

        let got = whats("components/glow_button.rs", "pub struct GlowButton;");
        assert!(
            got[0].contains("has no row in the Components table"),
            "{got:?}"
        );
        let got = whats("components/heading.rs", "pub struct Heading;");
        assert!(got[0].contains("Heading is a gap"), "{got:?}");
        let got = whats("composites/side_nav.rs", "pub struct SideNav;");
        assert!(got[0].contains("SideNav is a gap"), "{got:?}");
        let got = whats("components/tag.rs", "pub struct Chip;");
        assert!(got[0].contains("no `pub struct Tag`"), "{got:?}");
        let got = whats("composites/tag.rs", "pub struct Tag;");
        assert!(
            got[0].contains("Composites table"),
            "a component file in the wrong layer: {got:?}"
        );
    }

    #[test]
    fn findings_name_file_and_line() {
        let f = one("components/tag.rs", "pub struct Tag;\n.p_2()");
        assert_eq!(
            f[0].to_string(),
            "components/tag.rs:2: InlineValue: a size from the library's scale: `p_2(`"
        );
    }
}
