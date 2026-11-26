---
session_id:
  - f7d71e90-1e85-48b8-98e1-c72951b77c6d
---

# GPUI and gpui-component on omit

`spikes/gpui-shell` is one window with a projects sidebar and a virtual list of issue rows, built
and run on omit on 26 November 2026. The issues are made-up sample data in `src/main.rs`.

![The spike window: a sidebar with project names and counts, and a list of issue rows](gpui-shell.png)

The machine has 16 cores and an NVIDIA GPU (driver 615.78.08) and runs Hyprland 0.56.2 on Wayland, and the
toolchain was Rust 1.98.1.

## Versions

| Dependency name | crates.io package | Version |
|---|---|---|
| `gpui` | `gpui-pre` | `=0.3.14` |
| `gpui_platform` (feature `wayland`) | `gpui-pre-platform` | `=0.3.14` |
| `gpui-component` | `gpui-component` | `0.11.0` |
| `gpui-kit-assets` | `gpui-kit-assets` | `0.11.0` |

- `gpui-component` 0.11.0 requires `gpui-pre =0.3.14`, which its crates.io description calls a
  snapshot of Zed's source (`zed@f126950`). The `gpui` crate (0.2.2) is a different package. No git revision was
  needed.
- `gpui-pre` and `gpui-component` have one crates.io owner, `huacnlee` (Jason Lee). The `gpui`
  crate (0.2.2, 22 October 2025) is owned by `maxbrunsfeld`, `mikayla-maki`, `MrSubidubi` and the
  team `github:zed-industries:crates-io`.
- `gpui-pre` had a release every week or two from 0.3.0 (3 September) to 0.3.14 (23 November).
- `cargo tree -d --depth 0 | rg gpui` prints nothing, and `Cargo.lock` holds one `gpui-pre`.
- `Cargo.lock` holds 790 packages across all platforms. The `gpui*` crates compiled for Linux are
  Apache-2.0.
- No `gpui-component` feature is on, so no tree-sitter grammar is compiled.
- The application comes from `gpui_platform::application()`. The `gpui-pre` README says to enable
  at least one windowing backend on Linux, `wayland`, `x11` or both. The spike enables `wayland`
  only.
- `gpui = "0.2.2"` beside `gpui-component` 0.11.0 would put two `gpui` packages in the build with
  incompatible types (read from the dependency lists, not tried), so the spike depends on
  `gpui-pre` under the name `gpui`.

## Build times

Every command ran in `spikes/gpui-shell/` with `CARGO_TARGET_DIR="$PWD/target"` and
`nice -n 10`. Crates were already downloaded (`cargo fetch` took 18 s). Other sessions were
working on the machine; the 1-minute load average at the start and end of the clean builds was
between 6.5 and 10.

| Build | Command | Wall time |
|---|---|---|
| Clean, debug | `cargo clean`, then `cargo build -j 4` | 158 s |
| Clean, release (after the debug build) | `cargo build -j 4 --release` | 324 s |
| One line changed in `src/main.rs`, debug | `cargo build -j 4` | 11.7 s, 8.9 s, 11.1 s, 8.1 s |
| One line changed in `src/main.rs`, release | `cargo build -j 4 --release` | 3.7 s, 3.0 s, 1.9 s |
| Nothing changed, debug | `cargo build -j 4` | 0.3 s |

The changed line was `const ROW_HEIGHT`. The debug rebuild was slower than the release rebuild
every time; why was not measured. `spikes/gpui-shell/run.log` holds the clean-build lines, the last
of these rebuilds and a later launch of each build.

## Size and running cost

| | Debug | Release |
|---|---|---|
| Binary | 767 MB | 53 MB (39 MB after `strip`) |
| Window listed by `hyprctl` after launch | 1.03 s, 0.88 s | 0.65 s, 0.33 s |
| Resident memory | 226 MB, 227 MB | 211 MB, 178 MB |
| CPU while idle, over 10 s | 0.1%, 0.2% | 0.1%, 0.1% |
| GPU memory (`nvidia-smi`) | 50 MiB | 50 MiB |

`target/` was 4.4 GiB after the clean debug build and 6.2 GiB with the release build added. Each
build was launched twice. Launch time runs from the launch command to the window appearing in
`hyprctl clients`, CPU is measured over 10 s starting 6 s after that, and memory is `VmRSS`.

`hyprctl clients -j` showed class `temet-work-gpui-shell`, `xwayland: false` and the requested
size of 1100x700 (floating), on a monitor at scale 1.

## Problems

The first build had the only compile errors. No build script failed and the final build printed
no warning.

### The first build failed on the row function

All dependencies compiled, and after 146 s the build failed with three errors at one place in
`src/main.rs`. The first (line numbers as the file was then):

```text
error: lifetime may not live long enough
   --> src/main.rs:201:17
    |
195 |               |this, range, _, cx| {
    |                ----              - return type of closure `Vec<impl IntoElement>` contains a lifetime `'2`
    |                |
    |                has type `&'1 mut Shell`
...
201 | /                 range
202 | |                     .map(|row| this.issue_row(this.rows[row], cx))
203 | |                     .collect()
    | |______________________________^ returning this value requires that `'1` must outlive `'2`
```

The other two say the same of `cx` (`&mut gpui::Context<'_, Shell>`).

In edition 2024 a returned `impl IntoElement` captures the lifetimes of the arguments, here
`&self` and `cx`. Fix: declare the row function as `-> impl IntoElement + use<>`.

### The list calls the row closure for one row to measure it

No error. A log line in the `v_virtual_list` closure alternated between these two:

```text
built rows 0..1 of 36
built rows 0..23 of 36
```

The one-row call is the measuring pass, and code that reacts to the visible range (loading
more rows, for example) has to ignore it. The spike logs only ranges longer than one row.

## What gpui-component gave and what the spike wrote

`src/main.rs` is 265 lines, with the 36 issue rows as a fixed table in it.

| Part | From gpui-component 0.11.0 | Written in `src/main.rs` |
|---|---|---|
| Sidebar | `Sidebar`, `SidebarHeader`, `SidebarMenu`, `SidebarMenuItem` with active state and suffix slot | the filter a click applies, the issue counts |
| Issue list | `v_virtual_list`, `VirtualListScrollHandle`, `Scrollbar` | one height per row, columns shared by header and rows |
| Colours | `Theme::change` to dark, colour names | done rows in `muted_foreground` |
| Window | `Root`, `h_flex`, `v_flex` | opening it, `app_id`, quitting when it closes |

The crate also has `table` (with a `DataTable`) and `dock` modules. Neither was tried; the rows
here are plain flex rows.

## For temet-work

### The daemon

Build on:

- Keeping one warm app target directory per project, since the daemon is what places agents' work.
  A clean debug build took 158 s with `-j 4` and 4.4 GiB of disk; a one-line change in a warm
  directory took 2 to 12 s (Build times, Size and running cost).
- A daemon crate with no GPUI dependency, so that building the daemon alone does not compile GPUI.
  This is a recommendation from the same build costs; no daemon was built (Build times).

Avoid:

- Starting each item's app build in an empty target directory, which pays the clean build again
  (Build times).

Still unknown:

- Whether two work folders can build into one target directory at once. Only one folder built here
  (Build times).
- Whether a shared compiler cache shortens the clean build. None was set up for these builds
  (Build times).
- How the app receives daemon state. Feeding a socket stream into GPUI's executor and redrawing
  was not tried; the spike's rows are a fixed table in `main.rs` (What gpui-component gave and what the spike wrote).

### The app

Build on:

- `gpui-component` 0.11.0 with `gpui-pre` pinned to the version it requires; the two upgrade
  together. The pair ran on Wayland with NVIDIA from the first successful build (Versions, Size
  and running cost).
- `Sidebar` and `v_virtual_list` for the project list and the issue list. With 36 rows the list
  built the 23 in view. Longer lists, scrolling and sidebar clicks were not tried (Problems,
  What gpui-component gave and what the spike wrote).

Avoid:

- Depending on `gpui` 0.2.2 beside `gpui-component` (Versions).

Still unknown:

- Whether clicking a project filters the list in the running window. The code is there; no click
  was sent to the live desktop (What gpui-component gave and what the spike wrote).
- Scrolling, keyboard focus and navigation, text input and IME, tiled resizing, scale other than 1,
  and accessibility with a screen reader. The window was only drawn, floating, at scale 1 (Size and
  running cost).
- `DataTable`, `dock`, and text-heavy views such as diffs (What gpui-component gave and what the spike wrote).
- Whether the look from the design canvas can replace gpui-component's theme. Themes load from
  JSON (`ThemeRegistry::load_themes_from_str`, `watch_dir`). The spike used only the built-in dark
  theme, whose file names shadcn as author (What gpui-component gave and what the spike wrote).
- What the `tree-sitter-*` features add to the clean build; none was turned on (Versions).
- How long `gpui-pre` keeps being published, and how an upgrade goes. Its one owner also publishes
  `gpui-component`, its README says to expect breaking changes between versions, and only one pair
  of versions was built (Versions).
- macOS and X11; only the `wayland` backend was enabled (Versions).
- Whether smaller debug info shortens the debug rebuild, and shrinks the 767 MB debug binary
  (Build times, Size and running cost).
