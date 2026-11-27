---
session_id:
  - 7d0707d6-9b74-4137-9184-72c6591eeaed
---

# temet-work's UI parts

This document lists every part temet-work's desktop app is drawn from, in layers, and maps each
one to its part in the design system. It also gives the rule for adding a part. Read it before
you change anything the app draws.

The design system is `@temet-ai/design`: Blueprint 6 in the temet theme, plus temet's own parts,
documented in its Storybook. Look a part up there (the `temet-design` MCP tools `docs-list`,
`docs-show` and `docs-show-story`) before you use it, and use only the props it documents.
Its rules are in its Storybook under Start/Rules.

## The layers

| Layer | Holds | Built from | In temet-work |
|---|---|---|---|
| Base items | colour, type, spacing, sizes, shape, icons, motion | the design system's tokens | `temet-work-style`, generated |
| Components | one control or container each: Button, Tag, StatusTag, CutPanel, AppShell | base items and smaller components | `app/src/components/` |
| Composites | a working piece of a screen: SideNav, PageHeader, IssueTable, IssuePane | components and smaller composites | `app/src/composites/` |
| Pages | a screen | composites and components | `app/src/pages/` |

The front-end crate is `app/`, a Cargo workspace of its own like `spikes/gpui-shell`, the
spike it grows from, so that building the main workspace does not compile its UI library. It
may depend on `temet-work-core` and `temet-work-style` and on no other crate here, and it is the
only crate that may use a UI library (`tests/parts.rs`). Each part is one file named after it:
StatusTag is `app/src/components/status_tag.rs`, declaring `pub struct StatusTag`. The theme
module, `app/src/theme.rs`, is the one file that reads `temet-work-style`.

These rules hold the layers in place:

- A part uses only its own layer and the layers below it. Within a layer a part may hold a
  smaller one (StatusTag holds a Tag, NeedsYou holds DecisionRows), never in a cycle.
- Components are the only parts that read colour, type, shape and motion: the theme module's
  `colour`, `text`, `shape` and `motion` values. A composite arranges components and reads only
  its `space` and `size` values. A page sets no value at all and draws nothing of its own.
- No colour, size or shape is written inline. Every value is a token from `temet-work-style`,
  and the front-end crate's theme module is the one place that turns tokens into GPUI values.
- A part has the same name and the same variants as its part in the design system. In Rust the
  name is the same type name, each prop is a method with the same name in snake case, and each
  value is the same word: `StatusTag::new(Status::InProgress).size(Size::Large)`,
  `CutPanel::new().attention(true).build_up(true)`.
- A part that is not in the design system does not go in the app. A part marked gap below
  waits until the design system has it.
- Use only the props the design system documents or its stories show. Where the props
  listing is missing (NavbarGroup, MenuItem, OverlayToaster), the stories' props are the
  whole set.

## Adding a part

1. Look for the part in this map and in the design system. Use the part that does the job, or
   compose it from parts that exist.
2. Only when nothing composes to it, define it: its name, what it is for, its props and their
   values, the parts it is built from (in lower layers, or smaller parts of its own) and a story with
   temet-work's objects in it (issues, agents, machines, milestones).
3. Send the definition to the design system's maintainers, who work in `ui/` of
   `temet-ai/temet-design`. It is agreed with them and lands there with its story first.
4. Build it in temet-work with the same name and variants, from the parts the definition names.
5. Add it to this map in the same commit.

A new variant of an existing part goes the same way.

## Base items

`scripts/sync-tokens` reads the design system's tokens from its running Storybook and writes
`crates/style/src/tokens.rs`; nobody edits that file by hand. It refuses a design system with
uncommitted changes or a commit that is not pushed, and the file names the commit it came from. Each constant there has its token's
name: `--tm-edge-quiet` is `TM_EDGE_QUIET`.

Every value belongs to a theme, and each constant in `tokens.rs` is a theme's default. The user
picks the theme and can change the values the design system makes theme settings, such as the
attention colour. So the theme module reads the theme in force when the app runs, with the
user's settings, and turns it into gpui-component's `Theme` and the values temet's own parts
read. Nothing in the app bakes a token's value in.

| Item | Design system source | Values |
|---|---|---|
| Surfaces | `--bp-palette-black` (the page), `--bp-palette-dark-gray-1` (panels, side columns), `-2` (hover), `-3` (menus, popovers, pressed), `-4` (toasts) | plum-tinted greys |
| Edges | `--tm-edge` (panel borders, column rules), `--tm-edge-quiet` (rules between rows), `--bp-surface-border-color-default`, `-strong` | white at 8 to 24% |
| Text | `--bp-typography-color-default-rest`, `-muted`, `-default-disabled` | |
| Accent | `--bp-intent-primary-rest` (`#D41B6F`, fills), `--bp-typography-color-primary-rest` (text), `--bp-emphasis-focus-color` (`#F77CB3`, focus ring) | magenta |
| Intents | `--bp-intent-{success,warning,danger}-{rest,hover,active}` and `--bp-typography-color-{success,warning,danger}-rest` | success blue, warning yellow, danger red |
| Attention | `--tm-attention` | a theme setting the user can change |
| Selected row | `--tm-selected-fill`, `--tm-selected-line` | accent wash, accent line |
| Type | `--tm-font-text`, `--tm-font-figure`, `--tm-font-code`; `--tm-text-page`, `-pane`, `-section`, `-body`, `-small`, `-code`; `--tm-weight-heading`; Blueprint's `--bp-typography-*`, which read them | Archivo for text, Big Shoulders Display for figures, IBM Plex Mono for code; controls and page text at 13 px |
| Spacing and sizes | `--bp-surface-spacing` | |
| Shape | `--bp-surface-border-radius` (2 px), `--tm-cut` (8 px) | |
| Icons | `@blueprintjs/icons`, 16 and 20 px | SVG files embedded in the app |
| Motion | `--tm-d-*` durations, `--tm-ease-*` curves, `--tm-rise`, `--tm-slide`; the movements in `motion`: arrive, grow, unfold, fold, show, hide, pane, edge, collapse, buildUp, change, swap | |
| Status words | StatusTag's map: each status's word, icon and intent | 9 statuses |
| Stages | StageTag's map: each stage's icon | 7 stages |

## Components

Each row is a design-system part temet-work uses or will need. The GPUI basis is the
gpui-component 0.11 module a part is drawn with, where one fits; the look still comes from the
tokens.

| Part | Design system id | Variants temet-work uses | GPUI basis |
|---|---|---|---|
| AppShell | gap | regions: nav bar, main area, side pane, optional top bar | |
| Icon | `blueprint-icon` | 16, 20 px | `icon` |
| Button | `blueprint-button` | `variant` solid, minimal, outlined; `intent` none, primary; `size` small, medium; `icon`; `loading`; `disabled` | `button` |
| AttentionButton | `temet-attentionbutton` | `size` small | `button` |
| AttentionText | `temet-attentionbutton` (subcomponent) | | `label` |
| Tag | `temet-tag` | `minimal`; `intent` none, primary, success, warning, danger; `icon`; `interactive` | `tag` |
| StatusTag | `temet-statustag` | `status` (9); `size` medium, large; own word | Tag and Icon |
| StageTag | `temet-stagetag` | `stage` (7) | Tag and Icon |
| KeyComboTag | `blueprint-keycombotag` | | `kbd` |
| Tooltip | `blueprint-tooltip` | `compact`; `placement` | `tooltip` |
| ProgressBar | `blueprint-progressbar` | `intent` primary; `stripes` off; `animate` off | `progress` |
| Spinner | `blueprint-spinner` | | `spinner` |
| Divider | `blueprint-divider` | | `separator` |
| InputGroup | `blueprint-inputgroup` | `leftIcon` (search); `size` small, medium | `input` |
| TextArea | `blueprint-textarea` | | `input` |
| HTMLSelect | `blueprint-htmlselect` | | `select` |
| Switch | `blueprint-switch` | | `switch` |
| Checkbox | `blueprint-checkbox` | | `checkbox` |
| SegmentedControl | `blueprint-segmentedcontrol` | `intent` primary; `size` small | `button` (ToggleGroup) |
| Tabs, Tab | `blueprint-tabs` | `renderActiveTabPanelOnly` | `tab` |
| Menu, MenuItem, MenuDivider | `blueprint-menu` | | `menu` |
| Navbar, NavbarGroup, NavbarHeading, NavbarDivider | `blueprint-navbar` | NavbarGroup `align` right | `title_bar` |
| Popover | `blueprint-popover` | | `popover` |
| ContextMenu | `blueprint-contextmenu` | | `menu` |
| Alert | `blueprint-alert` | `intent` primary; `icon` | `dialog` |
| Dialog | `blueprint-dialog` | | `dialog` |
| Toast, OverlayToaster | `blueprint-toast` | | `notification` |
| Callout | `blueprint-callout` | `intent`; `compact` | `alert` |
| NonIdealState | `blueprint-nonidealstate` | | `empty` |
| Omnibar | `blueprint-omnibar` | | `command` |
| HTMLTable | `blueprint-htmltable` | `compact`; `interactive` | `table`, `virtual_list` |
| TableRow | `temet-tablerow` | `selected` | `table` |
| Section, SectionCard | `temet-section` | `compact`; `elevation` 0; `headingLevel`; `rightElement`; `padded` off | |
| CutPanel | `temet-cutpanel` | `attention`; `padded`; `buildUp` | |
| Disclosure | `temet-disclosure` | `defaultOpen`; `rightElement` | `collapsible` |
| PlusMinusTree | `temet-plusminustree` | `defaultExpanded`; node `secondaryLabel` | `tree` |
| Heading | gap | page title 20 px, pane title 16 px, section title 14 px with a muted note at the right | |
| PropertyList | gap | label and value pairs, labels muted in a fixed column | `description_list` |
| Meter | gap | name and value on one line, ProgressBar under them | |
| ListRow | gap | a row outside a table with a quiet rule above: title, muted meta line, muted detail line, an end element | |
| TerminalView | gap | an agent's terminal screen, read only, in the mono face | |

## Composites

| Part | What it is | Built from | Design system |
|---|---|---|---|
| SideNav | the guide's nav bar, the left column's areas and projects: an area is a group label that opens and closes, a project an item with its icon, name and a count of what waits on a person; two levels at most; past about five areas, an area selector at the top (`docs/areas.md`) | PlusMinusTree, Tag, Tooltip, Icon, HTMLSelect | gap; the Storybook page `pages-temet-work-radar` draws it in its own code |
| TopBar | across the window: the app's sections and search; needed once the app has a second section, such as settings (`docs/settings.md`) | Navbar, NavbarGroup, NavbarHeading, Button, InputGroup, Omnibar | gap |
| GroupList | the left column's groups: each group's name and milestone, and its agents with role, machine and model, and status | Heading, ListRow, StatusTag | gap |
| MeterList | the left column's machines (build slots in use) and limits (use per window, when it resets) | Heading, Meter | gap |
| PageHeader | the top of the main area: the project or area name, where its issues and code are, and the controls for the whole page | Heading, Icon, SegmentedControl, Button | gap |
| NeedsYou | the decisions that wait on a person, costliest to undo first | CutPanel (`attention`, `buildUp`), AttentionText, DecisionRow | gap |
| DecisionRow | one agent decision: who, when, what it took, the cost to undo, what it weighed and why, the deadline and Overturn | ListRow, Tag, PropertyList, AttentionText, AttentionButton | gap; the AttentionButton story "Decision Row" shows one |
| MilestonePanel | one milestone: name, done of total, due date, progress, and its issues | CutPanel, Section, ProgressBar, IssueTable | gap |
| IssueTable | issues as rows: key, title, stage, agent, updated; optional group rows; one set of column widths in every panel | HTMLTable, TableRow, StageTag | gap |
| IssuePane | the side pane for an issue: title, key, stage, and tabs for its plan, pull request, review rounds and decisions | Heading, StageTag, Tabs, PropertyList, CheckList, ReviewRounds, DecisionRow, StatusTag | gap |
| CheckList | a plan's checks: what is done when, its status, and what proves it | ListRow, StatusTag | gap |
| ReviewRounds | the review rounds of a pull request: each round's reviewer and verdict, its points, the author's reply and each point's status | Disclosure, ListRow, StatusTag | gap; the Disclosure story "Open" shows one round |
| AgentPane | the side pane for an agent: its role, state, terminal and decisions | Heading, StatusTag, TerminalView, DecisionRow | gap |

## Pages

| Page | What it shows | Built from | Design system |
|---|---|---|---|
| Project | one project: the nav bar, groups, machines and limits on the left; the project's milestones and issues in the middle; the selected issue or agent on the right (`docs/objects.md`) | AppShell, SideNav, GroupList, MeterList, PageHeader, NeedsYou, MilestonePanel, IssueTable, IssuePane, AgentPane, Alert, Toast | `pages-temet-work-radar` (RadarScreen) |
| Area | one area: its projects' milestones as a list, its tracker and code-host groups in the page header (`docs/areas.md`) | AppShell, SideNav, PageHeader, MilestonePanel | gap |
| Settings | the settings chain, global down to one task (`docs/settings.md`) | not defined yet | gap |
| Rules | rules with where they came from, their state and what measures them (`docs/objects.md`) | not defined yet | gap |

## Gaps in the design system

Each of these goes to the design system as a definition. When the part lands with its story,
its row above takes the part's id in place of "gap".

- A token export: every token in the Base items table, resolved for each theme, as JSON from
  the package's build, with the status and stage maps, each component's look per variant and
  state, and the movements as data. `scripts/sync-tokens` then reads
  that file in place of the running Storybook.
- A light theme beside the dark one, with the same tokens.
- Which tokens a user may change, such as `--tm-attention`, marked in the token export.
- The spacing steps and the column widths the radar page sets in its own CSS, as tokens. The
  text sizes and faces are tokens now (`--tm-text-*`, `--tm-font-*`).
- The faces' font files as TTF or OTF, which the app embeds. The design system ships them as
  woff2, and GPUI's Linux text system (cosmic-text, reading fonts through fontdb and
  ttf-parser) parses TrueType and OpenType only. Until then the app falls back to the system's
  fonts.
- The design system's part list in the token export, so the review check can confirm every id
  in this map.
- AppShell, Heading, PropertyList, Meter and ListRow as components, and every composite above
  but AgentPane.
- TerminalView and AgentPane, once temet-work sends their definitions.
- InputGroup without `round`, as Tag has: a round input is a pill.
- Props the Storybook does not document: NavbarGroup, MenuItem and OverlayToaster.

## Checks

`temet-work-style`'s `review` module holds the UI review check, and its test
`crates/style/tests/review.rs` runs it over `app/src` and this map in every workspace test run.
The app's own tests run with `cargo test` in `app/`. CI does not build the app yet, since that
compiles its UI library: run them after changing `app/` or the tokens.

| Check | Where | State |
|---|---|---|
| The synced tokens keep the design system's rules: the accent, corners of 2 px at most, success in blue, no teal, cyan, brown or green | `temet-work-style` tests and a build-time assertion | built |
| Tokens come from a pushed commit of the design system | `scripts/sync-tokens` refuses any other | built |
| Every one of gpui-component's colour slots takes a token, and none breaks the ban list | `app/src/theme.rs` tests | built |
| The app is a workspace of its own and depends on `temet-work-core` and `temet-work-style` only | `tests/parts.rs` | built |
| No colour, size, corner, border, shadow, weight, opacity or duration written as a literal anywhere in `app/src`, the theme module included | `review` | built |
| Only the theme module reads `temet-work-style` | `review` | built |
| Each file in `app/src/components/`, `composites/` and `pages/` has a row in this map with a design-system id, in its layer's table, and declares a type with the part's name | `review` | built |
| A part uses no layer above its own; a composite sets no colour, type, shape or motion and uses the UI library only through components; a page and the plumbing draw nothing of their own | `review` | built |
| Every design-system id in this map is in the design system's part list | `review`, from the list `scripts/sync-tokens` will read | to build, with the next sync |
