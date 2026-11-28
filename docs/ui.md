---
session_id:
  - 7d0707d6-9b74-4137-9184-72c6591eeaed
  - fec79389-b50c-4e28-8f21-c56c628a5232
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
module, `app/src/theme.rs`, is the one file that reads `temet-work-style`. Every other file
outside the layer folders is plumbing and draws nothing: `app/src/daemon.rs` reads the
daemon's view once a second into an entity the window observes, and `app/src/view_map.rs` maps
that view into the Project page's props; the page's notice about the daemon (no answer, a stale
view) is a Callout.

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
- Use only the props the design system documents or its stories show. Where a part has no
  props listing, its stories' props are the whole set.

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

`scripts/sync-tokens` reads the design system's token export, `dist/tokens.json`, which its
`npm run build` writes, and writes `crates/style/src/tokens.rs` and the faces in
`app/assets/fonts/`; nobody edits those by hand. It refuses a design system with uncommitted
changes, a commit that is not pushed, or an export built from another commit, and `tokens.rs`
names the commit it came from. Each theme is one `Tokens` value, `DARK` or `LIGHT`, with a field
per token named after it: `--tm-edge-quiet` is `tm_edge_quiet`. Both themes set the same tokens,
or the sync fails. A theme also holds its parts' looks: each part, variant and state as the
design system's stories draw it, measured by its build. `tokens.rs` also holds the status and
stage maps, the design system's part list, the tokens a user may change (`SETTINGS`) and the
font files.

Each value in `tokens.rs` is a theme's default. The user picks the theme and can change the
tokens in `SETTINGS`, the attention colour and the accent (not built yet).
`theme::apply` puts a theme in force; the parts read its values through the theme module's
`colour`, `text`, `shape`, `motion`, `space` and `size` functions, each named for a role on the
design system's Theme pages. Nothing in the app bakes a token's value in.

| Item | Design system source | Values |
|---|---|---|
| Surfaces | `--tm-surface-page`, `-panel` (side columns, panels), `-hover`, `-raised` (menus, popovers, pressed), `-toast`, `-tooltip` | plum-tinted greys in the dark theme, light plum in the light one |
| Edges | `--tm-edge` (panel borders, column rules), `--tm-edge-quiet` (rules between rows), `--bp-surface-border-color-default`, `-strong` | white at 8 to 24% |
| Text | `--bp-typography-color-default-rest`, `-muted`, `-default-disabled` | |
| Accent | `--tm-accent-1` to `-6`, which Blueprint's blue ramp reads: `--bp-intent-primary-rest` (`--tm-accent-3`, fills), `--bp-typography-color-primary-rest` (text), `--bp-emphasis-focus-color` (focus ring) | a setting: deep violet by default (`#5B21B6` as a fill), magenta as the design system's other choice |
| Intents | `--bp-intent-{success,warning,danger}-{rest,hover,active}` and `--bp-typography-color-{success,warning,danger}-rest` | success blue, warning yellow, danger red |
| Attention | `--tm-attention` | a theme setting the user can change |
| Selected row | `--tm-selected-fill`, `--tm-selected-line` | accent wash, accent line |
| Hue edge | `--tm-here-1` to `-6`, `--tm-here-width` | the six colours and the width of the edge on the one panel the person is in (CutPanel's `here`) |
| Type | `--tm-font-text`, `--tm-font-figure`, `--tm-font-code`; `--tm-text-page`, `-pane`, `-section`, `-body`, `-small`, `-code`; `--tm-weight-heading`; Blueprint's `--bp-typography-*`, which read them | Archivo for text, Big Shoulders Display for figures, IBM Plex Mono for code, embedded in the app as TTF (`app/assets/fonts`) and registered at startup; controls and page text at 13 px, section titles at 14 px |
| Columns | `--tm-col-*`, `--tm-issue-col-*` (IssueTable, with `-milestone`), `--tm-needs-col-count`, `-cost`, `-deadline` (NeedsYou) | |
| Spacing and sizes | `--bp-surface-spacing` | |
| Shape | `--bp-surface-border-radius` (2 px), `--tm-cut` (8 px) | |
| Icons | `@blueprintjs/icons`, 16 and 20 px | SVG files embedded in the app |
| Motion | `--tm-d-*` durations, `--tm-ease-*` curves, `--tm-rise`, `--tm-slide`; the movements in `motion`: arrive, grow, unfold, fold, show, hide, pane, edge, collapse, buildUp, change, swap | the hue edge drifts one way over `--tm-d-drift` (16 s) at `--tm-ease-linear` and back, and stays still under reduced motion |
| Status words | StatusTag's map: each status's word, icon and intent | 9 statuses |
| Stages | StageTag's map: each stage's icon | the daemon's 11 stages, in its order |

## Components

Each row is a design-system part temet-work uses or will need. The GPUI basis is the
gpui-component 0.11 module a part is drawn with, where one fits; the look still comes from the
tokens.

Some composites in the design system style their own markup with a class of their own, such as
GroupList's group head (`tm-groups-head`) or IssueTable's key cell (`tm-mono tm-issue-key`). A
composite here reads no colour or type, so `app/src/components/mod.rs` holds these classes as
`Class`, and the component layer draws them: `classed(Class::GroupsHead, cx)` for plain markup,
or `class_name(Class::Issues)` on a component, as the design system passes `className`, or
`spans` for a paragraph with a classed run such as `Proven by <span className="tm-mono">`. Each
class resolves to the look the design system measured for an element with that class, such as
GroupList `tm-groups-head`; a class with no measured look follows its rule in `temet.css` or
`blueprint-overrides.css`.

Where the radar page and a part's own stories draw a part differently, the window follows the
radar page, because the page is the screen the window reproduces. Its Alert asks with the primary
intent and no icon, and its Toast has the Notifications icon and no intent. The page closes the
toast after 5000 ms; no motion token holds that duration yet, so the window's toast stays until it
is closed.

| Part | Design system id | Variants temet-work uses | GPUI basis |
|---|---|---|---|
| AppShell | `temet-appshell` | regions: nav bar, main area, side pane, optional top bar | |
| Icon | `blueprint-icon` | 16, 20 px | `icon` |
| Button | `blueprint-button` | `variant` solid, minimal, outlined; `intent` none, primary; `size` small, medium; `icon`; `loading`; `disabled` | `button` |
| AttentionButton | `temet-attentionbutton` | `size` small | `button` |
| AttentionText | `temet-attentionbutton` (subcomponent) | | `label` |
| Tag | `temet-tag` | `minimal`; `intent` none, primary, success, warning, danger; `icon`; `interactive`. Story: Intents. In the dark theme the warning and danger tags are outlined in their colour with no wash; in the light theme the warning tag is a neutral tag and the danger tag keeps its red wash, as the design system measured them | `tag` |
| StatusTag | `temet-statustag` | `status` (9); `size` medium, large; own word | Tag and Icon |
| StageTag | `temet-stagetag` | `stage` (11: new, refining, ready, working, in review, fixing, with arbiter, merging, deploying, verifying, done); `round`, shown after in review, fixing and with arbiter ("fixing, round 1"). Stories: All Stages, With Round. The window maps each of the view's stage words to the stage of the same name (`app/src/view_map.rs`) | Tag and Icon |
| KeyComboTag | `blueprint-keycombotag` | | `kbd` |
| Tooltip | `blueprint-tooltip` | `compact`; `placement` | `tooltip` |
| ProgressBar | `blueprint-progressbar` | `intent` primary; `stripes` off; `animate` off | `progress` |
| Spinner | `blueprint-spinner` | | `spinner` |
| Divider | `blueprint-divider` | | `separator` |
| InputGroup | `temet-inputgroup` | `leftIcon` (search); `size` small, medium | `input` |
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
| Alert | `blueprint-alert` | `intent` primary; `cancelButtonText`, `confirmButtonText` | `dialog` |
| Dialog | `blueprint-dialog` | | `dialog` |
| Toast, OverlayToaster | `blueprint-toast` | `icon`; `position` bottom-right; no `timeout` (no duration token) | `notification` |
| Callout | `blueprint-callout` | `intent`; `compact` | `alert` |
| NonIdealState | `blueprint-nonidealstate` | | `empty` |
| Omnibar | `blueprint-omnibar` | | `command` |
| HTMLTable | `blueprint-htmltable` | `compact`; `interactive` | `table`, `virtual_list` |
| TableRow | `temet-tablerow` | `selected` | `table` |
| Section, SectionCard | `temet-section` | `compact`; `elevation` 0; `headingLevel`; `rightElement`; `padded` off | |
| CutPanel | `temet-cutpanel` | `attention`; `here`, the moving hue edge on the one panel the person is in, which `attention` overrides; `padded`; `buildUp`. Stories: Padded, Attention, Here, Here With Attention, Machines With One Here | |
| Disclosure | `temet-disclosure` | `defaultOpen`; `rightElement` | `collapsible` |
| PlusMinusTree | `temet-plusminustree` | `defaultExpanded`; node `secondaryLabel` | `tree` |
| Heading | `temet-heading` | page title 20 px, pane title 16 px, section title 14 px with a muted note at the right | |
| PropertyList | `temet-propertylist` | label and value pairs, labels muted in a fixed column | `description_list` |
| Meter | `temet-meter` | name and value on one line, ProgressBar under them | |
| ListRow | `temet-listrow` | a row outside a table with a quiet rule above: title, muted meta line, muted detail line, an end element | |
| TerminalView | gap | an agent's terminal screen, read only, in the mono face | |

## Composites

| Part | What it is | Built from | Design system |
|---|---|---|---|
| SideNav | the guide's nav bar, the left column's areas and projects: an area is a group label that opens and closes, a project an item with its icon, name and a count of what waits on a person; two levels at most; past about five areas, an area selector at the top (`docs/areas.md`) | PlusMinusTree, Tag, Tooltip, Icon, HTMLSelect | `temet-sidenav` |
| TopBar | across the window: the app's sections and search; needed once the app has a second section, such as settings (`docs/settings.md`) | Navbar, NavbarGroup, NavbarHeading, Button, InputGroup, Omnibar | `temet-topbar` |
| GroupList | the left column's groups: each group's name and milestone, and its agents with role, machine and model, and status | Heading, ListRow, StatusTag | `temet-grouplist` |
| MeterList | the left column's machines (build slots in use) and limits (use per window, when it resets) | Heading, Meter | `temet-meterlist` |
| PageHeader | the top of the main area: the project or area name, where its issues and code are, and the controls for the whole page | Heading, Icon, SegmentedControl, Button | `temet-pageheader` |
| NeedsYou | the decisions that wait on a person, costliest to undo first | CutPanel (`attention`, `buildUp`), AttentionText, DecisionRow | `temet-needsyou` |
| DecisionRow | one agent decision: who, when, what it took, the cost to undo, what it weighed and why, the deadline and Overturn | ListRow, Tag, PropertyList, AttentionText, AttentionButton | `temet-decisionrow` |
| MilestonePanel | one milestone: name, done of total, due date, progress, and its issues | CutPanel, Section, ProgressBar, IssueTable | `temet-milestonepanel` |
| IssueTable | issues as rows: key, title, stage, agent, updated; optional group rows; one set of column widths in every panel | HTMLTable, TableRow, StageTag | `temet-issuetable` |
| IssuePane | the side pane for an issue: title, key, stage, and tabs for its plan, pull request, review rounds and decisions | Heading, StageTag, Tabs, PropertyList, CheckList, ReviewRounds, DecisionRow, StatusTag | `temet-issuepane` |
| CheckList | a plan's checks: a line counting them by status, then for each what is done when, what proves it and its status, if it has run. A done-when over three lines and a proven-by over two show their first lines, and plus opens the row whole, minus closes it. Stories: Default, Failed And Timed, No Checks, Not Run Yet, Long Checks | ListRow, StatusTag, Button | `temet-checklist` |
| ReviewRounds | the review rounds of a pull request: each round's reviewer and verdict, its points, the author's reply and each point's status | Disclosure, ListRow, StatusTag | `temet-reviewrounds` |
| AgentPane | the side pane for an agent: its role, state, terminal and decisions | Heading, StatusTag, TerminalView, DecisionRow | gap |

## Pages

| Page | What it shows | Built from | Design system |
|---|---|---|---|
| Project | one project: the nav bar, groups, machines and limits on the left (a list the view has no data for is left out); the project's milestones and issues in the middle; the selected issue or agent on the right (`docs/objects.md`) | AppShell, SideNav, GroupList, MeterList, PageHeader, NeedsYou, MilestonePanel, IssueTable, IssuePane, AgentPane, Callout, Alert, Toast | `pages-temet-work-radar` (RadarScreen) |
| Area | one area: its projects' milestones as a list, its tracker and code-host groups in the page header (`docs/areas.md`) | AppShell, SideNav, PageHeader, MilestonePanel | gap |
| Settings | the settings chain, global down to one task (`docs/settings.md`) | not defined yet | gap |
| Rules | rules with where they came from, their state and what measures them (`docs/objects.md`) | not defined yet | gap |

## Gaps in the design system

Each of these goes to the design system as a definition. When the part lands with its story,
its row above takes the part's id in place of "gap". `crates/style/src/tokens.rs` is synced from
the design system's main at db39ff8.

- TerminalView and AgentPane, once temet-work sends their definitions.
- NonIdealState has no measured look in the token export, so the window shows a problem with no
  view as a Callout alone. The page would use NonIdealState once its look is measured.

## Checks

`temet-work-style`'s `review` module holds the UI review check, and its test
`crates/style/tests/review.rs` runs it over `app/src` and this map in every workspace test run.
The CI job `app` builds `app/` and runs its `cargo fmt --check`, clippy and tests on every pull
request, beside the main workspace's job. Run the app's tests locally with
`cd app && cargo nextest run`.

| Check | Where | State |
|---|---|---|
| The synced tokens keep the design system's rules in every theme: the accent, corners of 2 px at most, success in blue, no teal, cyan, brown or green | `temet-work-style` tests and a build-time assertion | built |
| Every part's measured look, in every variant, state and theme, keeps those rules too, and every theme measures the same parts | `temet-work-style` tests | built |
| Tokens and faces come from a pushed commit of the design system, and the export from that commit | `scripts/sync-tokens` refuses any other | built |
| Every one of gpui-component's colour slots takes a token, and none breaks the ban list | `app/src/theme.rs` tests | built |
| The app is a workspace of its own and depends on `temet-work-core` and `temet-work-style` only | `tests/parts.rs` | built |
| No colour, size, corner, border, shadow, weight, opacity or duration written as a literal anywhere in `app/src`, the theme module included | `review` | built |
| Only the theme module reads `temet-work-style` | `review` | built |
| Each file in `app/src/components/`, `composites/` and `pages/` has a row in this map with a design-system id, in its layer's table, and declares a type with the part's name | `review` | built |
| A part uses no layer above its own; a composite sets no colour, type, shape or motion and uses the UI library only through components; a page and the plumbing draw nothing of their own | `review` | built |
| Every design-system id in this map is in the design system's part list, and each component and composite has that part's name | `review`, against `tokens::PARTS` | built |
| Every `look::get` the app makes names a look both themes measured | `app/src/theme.rs` tests | built |
| The window as the design system draws it: each theme at 1440, 1280 and 1700 px wide, each tab of the issue pane, the issues grouped by stage and by group, and nothing waiting on the person, drawn offscreen with GPUI's headless renderer, with no window or compositor, beside the design system's story shots | `cargo run --example shots --features shots -- DIR` in `app/` | built; compared by eye |
| StageTag, Tag, CheckList and CutPanel as their stories draw them, in each theme at 1440 px: the `part-` shots, named after each story's id (`part-temet-stagetag--all-stages-dark`) | the same command | built; compared by eye |
