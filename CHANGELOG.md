# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

* `Terminal(inline_height=...)` and `AsyncTerminal(inline_height=...)` draw in a block of the
  normal buffer instead of taking over the screen, leaving the scrollback and whatever was
  printed before and after the app in place
* Example `examples/40_inline_viewport.py` and reference documentation for the inline viewport

---

## [0.2.9]

### Fixed

* Fixed silent swallowing of Python exceptions raised inside non-`Result` drawing callbacks
* Added propagation of `PyErr` from `FnOnce` draw delegates out of `Terminal::draw`
* Ensured draw pipeline no longer hides runtime errors originating in Python execution context
* Improved error visibility for debugging and prototyping in UI rendering flow

### Summary

This release fixes a critical visibility issue in the draw system where Python exceptions raised inside draw callbacks were previously discarded due to non-`Result` execution paths. Errors are now correctly surfaced outside the draw boundary instead of being silently ignored.

### Impact

* Python exceptions in draw logic are now immediately visible at call site
* Eliminates silent failure mode in rendering pipeline
* Improves debugging reliability for UI development and experimentation

### Testing

Manual verification:

* Triggered Python exceptions inside `Terminal::draw`
* Confirmed propagation of traceback outside draw context
* Verified absence of silent failure behavior

## [0.2.8]

### Added

- `RatatuiMascot` widget binding (with `MascotEyeColor`) + docs and example.

## [0.2.7] — 2026-03-14

### Summary

`0.2.7` is a major internal reorganisation and tooling release for `pyratatui`.

The separate `pyratatui_manager` package has been fully integrated into the
main project as the `pyratatui.manager` subpackage, providing a built-in app
manager CLI directly from the `pyratatui` command.

The Python source tree has also been reorganised into focused subpackages
(`widgets`, `layout`, `styling`, `core`) to support more granular imports
while preserving full backward compatibility.

This release also fixes a crash affecting `pyratatui run` when launching
flat-layout apps (`main.py` projects) under Python 3.12+.

No public API changes were introduced.

---

# Added

## 📦 `pyratatui.manager` — integrated app manager

The app manager now lives at:

```
pyratatui/manager/
```

and is available both programmatically and through the unified CLI.

### New CLI commands

All commands are now available through the single `pyratatui` entry point.

```
pyratatui install user/repo          Clone & install a TUI app from GitHub
pyratatui install user/repo --branch dev
pyratatui install user/repo --force  Re-install, overwriting existing copy

pyratatui uninstall app              Remove an installed app
pyratatui uninstall app --yes        Skip confirmation

pyratatui list                       Pretty table of installed apps
pyratatui list --json                Machine-readable JSON output

pyratatui run app                    Launch an installed app
pyratatui run app -- --app-flag      Forward extra args to the app

pyratatui info app                   Show full metadata
```

### `install` behaviour

1. Validates the `user/repo` slug
2. Shallow-clones from GitHub
3. Installs into `~/.pyratatui/apps/user/repo/`
4. Runs `pip install -r requirements.txt` if present
5. Auto-detects the entry point (`main.py`, `<repo>/__main__.py`, or `<repo>/main.py`)
6. Creates an executable wrapper in `~/.pyratatui/bin/<repo>`
7. Stores metadata in `~/.pyratatui/meta/user__repo.json`

### Programmatic API

```python
from pyratatui.manager import install_app, uninstall_app, list_apps, run_app

install_app("pyratatui/test_app")
install_app("pyratatui/test_app", branch="dev", force=True)

list_apps()
list_apps(as_json=True)

run_app("test_app", extra_args=["--debug"])

uninstall_app("test_app", yes=True)
```

### `pyratatui.manager` module layout

| Module                 | Responsibility                                              |
| ---------------------- | ----------------------------------------------------------- |
| `manager/install.py`   | Clone repo, install dependencies, create wrapper            |
| `manager/uninstall.py` | Remove app source, wrapper, and metadata                    |
| `manager/list.py`      | List installed apps                                         |
| `manager/run.py`       | Launch installed apps                                       |
| `manager/utils.py`     | Metadata handling, entrypoint detection, wrapper generation |

---

## 🗂 New focused subpackages

Four new subpackages provide explicit import paths without breaking the
existing flat import style.

```python
# Still valid
from pyratatui import Layout, Constraint, Color, Style, Chart

# New focused imports
from pyratatui.widgets import Chart, Menu, PieChart
from pyratatui.layout import Layout, Constraint, Direction
from pyratatui.styling import Color, Style, Modifier
from pyratatui.core import AsyncTerminal, run_app
```

| Subpackage          | Contents                        |
| ------------------- | ------------------------------- |
| `pyratatui.widgets` | Widget classes                  |
| `pyratatui.layout`  | Layout system                   |
| `pyratatui.styling` | Colors, styles, text primitives |
| `pyratatui.core`    | Runtime and terminal management |

---

## 📄 `_pyratatui.pyi` type stub

Added `python/pyratatui/_pyratatui.pyi` as the canonical type stub for the
Rust extension module (`pyratatui._pyratatui`), matching the
`module-name` defined in `maturin`.

---

# Fixed

## `pyratatui run` crash on flat-layout apps

Running apps created with `pyratatui init` could crash with:

```
ValueError: __main__.__spec__ is None
```

This occurs in **Python 3.12+** when `python -m __main__` is executed without
an enclosing package.

The runner now detects flat-layout apps and executes them using
direct script mode:

```
python main.py
```

instead of `python -m __main__`.

---

## Windows `.bat` wrapper crash

The wrapper generated by `pyratatui install` previously used:

```
runpy.run_module('__main__')
```

which triggers the same Python restriction.

Wrappers now execute flat-layout projects directly as:

```
python /path/to/main.py
```

---

# Changed

### Entrypoint detection system

`manager/utils.py` now returns a **prefixed entrypoint string**
that encodes both the launch method and the target.

| Entrypoint string     | Launch command         |
| --------------------- | ---------------------- |
| `"script:main.py"`    | `python main.py`       |
| `"module:myapp"`      | `python -m myapp`      |
| `"module:myapp.main"` | `python -m myapp.main` |

New helper:

```
parse_entrypoint(ep) -> (kind, target)
```

Legacy entrypoints stored by earlier versions are automatically mapped:

| Legacy value | Interpreted as          |
| ------------ | ----------------------- |
| `"__main__"` | `("script", "main.py")` |
| `"myapp"`    | `("module", "myapp")`   |

---

### Wrapper generation

`_wrapper_script` and `_wrapper_bat` now:

* run scripts directly for `script:` entrypoints
* use `python -m` for `module:` entrypoints
* continue prepending `PYTHONPATH` so local imports resolve correctly

---

### CLI rewrite

`cli.py` has been fully rewritten using `typer` and now supports:

```
install
uninstall
list
run
info
init
version
```

---

### Package restructuring

Widget wrapper modules were moved from the package root to:

```
pyratatui/widgets/
```

Affected files:

```
chart.py
checkbox.py
menu.py
piechart.py
throbber.py
```

Imports were updated from:

```
._pyratatui
```

to

```
.._pyratatui
```

---

### Version bump

```
0.2.4 → 0.2.7
```

Updated in both:

* `pyproject.toml`
* `Cargo.toml`

---

# Removed

* The **separate `pyratatui_manager` package** — its functionality now lives
  inside `pyratatui.manager`.
* Top-level widget wrapper files (`chart.py`, `checkbox.py`, `menu.py`,
  `piechart.py`, `throbber.py`) which were replaced by the
  `pyratatui.widgets` subpackage.

---

# Migration

No code changes are required.

All existing imports from `pyratatui` remain valid.

Upgrade `pyratatui` now!

```bash
pip install --upgrade pyratatui
```

The `pyratatui` CLI now includes the full app manager.

---

## [0.2.4] — 2026-03-09

### Summary

`0.2.4` adds five new widget integrations and expands the Python API surface:
`Throbber`, `Menu`, `PieChart`, `Checkbox`, and built-in `Chart`.

### Added

- **Throbber widget** via `throbber-widgets-tui`:
  - `Throbber.start()`, `Throbber.stop()`, `Throbber.set_speed()`, `Throbber.set_style()`
  - Render support through `frame.render_widget(throbber, area)`
- **Menu widget** via `tui-menu`:
  - `Menu`, `MenuItem`, `MenuState`, `MenuEvent`
  - Nested menu trees with keyboard handling and event draining
  - New frame method: `frame.render_stateful_menu(menu, area, state)`
- **PieChart widget** via `tui-piechart`:
  - `PieChart`, `PieData`, `PieStyle`
  - Legend layout/alignment/position and resolution controls
- **Checkbox widget** via `tui-checkbox`:
  - `Checkbox` with checked/unchecked states and toggle helpers
  - Style controls for symbol/label and layout alignment
- **Built-in Chart widget binding**:
  - `Chart`, `Axis`, `Dataset`, `GraphType`, `Marker`, `LegendPosition`
  - Multi-dataset support for scatter/line/bar in one chart

### Changed

- Crate and Python package versions: `0.2.3` → `0.2.4`
- Added new dependencies:
  - `throbber-widgets-tui = 0.11.0`
  - `tui-menu = 0.3.1`
  - `tui-piechart = 0.3.1`
  - `tui-checkbox = 0.4.3`
- Python exports and stubs updated for all new widgets and frame methods.
- Added numbered examples:
  - `34_throbber.py`
  - `35_menu_widget.py`
  - `36_piechart.py`
  - `37_checkbox_widget.py`
  - `38_chart_widget.py`
- Added Python integration tests for:
  - throbber controls
  - menu interactions
  - pie chart setup
  - checkbox toggling
  - multi-dataset chart construction
- The async/event helpers no longer treat ``'q'`` (or ``Ctrl+C``) as a built-in exit key: ``AsyncTerminal.events()`` now defaults to ``stop_on_quit=False``, and ``run_app``/``run_app_async`` only exit when ``on_key`` returns ``True`` so developers explicitly handle quitting.
- Format scripts also now install the wheel.

### Fixed

- Previous missing imports/API exposure for new widget modules in Python package exports.
- Version expectations in tests and packaging metadata.

## [0.2.3] — 2026-03-08

Minor documentation updates. Added musllinux support. No significant changes from the previous release.

## [0.2.2] — 2026-03-08

### Summary

`0.2.2` adds three new widgets and a new CLI project scaffold command:
`Canvas`, `Map`, `Button`, and `pyratatui init`.

---

### Added

#### 🎨 Canvas Widget

New drawing widget backed by `ratatui::widgets::canvas::Canvas`.

```python
from pyratatui import Canvas, Terminal

canvas = Canvas(width=60, height=20)
canvas.draw_line(0, 0, 20, 10)
canvas.draw_point(5, 5)
canvas.draw_rect(10, 5, 20, 6)

with Terminal() as term:
    term.draw(lambda frame: frame.render_widget(canvas, frame.area))
```

Features:
- `Canvas.draw_line(x1, y1, x2, y2)` — Draw line primitive
- `Canvas.draw_point(x, y)` — Draw point primitive
- `Canvas.draw_rect(x, y, w, h)` — Draw rectangles
- `Canvas.clear()` — Remove all shapes

#### 🗺️ Map Widget

World map widget backed by `ratatui::widgets::canvas::Map`.

```python
from pyratatui import Map, MapResolution, Terminal

map_widget = Map(resolution=MapResolution.High)

with Terminal() as term:
    term.draw(lambda frame: frame.render_widget(map_widget, frame.area))
```

Features:
- `MapResolution.Low`
- `MapResolution.High`
- `Map(resolution=...)`

#### 🔘 Button Widget

Interactive button widget with focus-aware rendering.

```python
from pyratatui import Button, Terminal

button = Button("Save")

with Terminal() as term:
    term.draw(lambda frame: frame.render_widget(button, frame.area))
```

Features:
- `Button.focus()` / `Button.unfocus()` — Manage focus state
- `Button.is_focused()` — Query focus state
- `Button.set_label(new_label)` — Update button text
- `Button.handle_key(code, ctrl=False, alt=False, shift=False)` — keyboard interaction helper

#### 🚀 CLI Tool

New `pyratatui` command-line tool for project initialization.

```bash
pyratatui init my_app
cd my_app
pip install -r requirements.txt
python main.py
```

Creates a structured project with:
- `main.py` — Immediately runnable starter UI
- `app.py` — App module scaffold
- `requirements.txt` — Dependencies
- `README.md` — Project documentation

Usage: `pyratatui init <project_name>`

#### 📝 Examples

New examples for all widgets:
- `31_canvas_drawing.py` — Drawing primitives and shapes
- `32_map_widget.py` — World map rendering
- `33_button_widget.py` — Focus + keyboard button demo

### Documentation

- Updated `README.md` with 0.2.2 feature summary and screenshots
- Updated docs pages to include the 0.2.2 additions

---

## [0.2.1] — 2026-03-07

### Summary

`0.2.1` is a major widget integration release for terminal-only use.
Five new third-party widget crates have been integrated:
**tui-bar-graph** (gradient bar graphs), **tui-tree-widget** (interactive tree views),
**tui-markdown** (Markdown → Text conversion), **tui-logger** (real-time log viewer),
and **ratatui-image** (terminal image display). PyRatatui is now a pure terminal
TUI library. All 30 examples
are numbered sequentially.

---

### Added

#### 📊 BarGraph Widget (`tui-bar-graph`)

Colorful gradient bar graph widget using the `colorgrad` crate for gradients.

```python
from pyratatui import BarGraph, BarGraphStyle, BarColorMode

graph = (
    BarGraph([0.1, 0.4, 0.9, 0.6, 0.3])
    .bar_style(BarGraphStyle.Braille)
    .color_mode(BarColorMode.VerticalGradient)
    .gradient("viridis")
)
frame.render_widget(graph, area)
```

#### 🌲 Tree Widget (`tui-tree-widget`)

Interactive hierarchical tree view with keyboard navigation.

```python
from pyratatui import Tree, TreeItem, TreeState

items = [
    TreeItem("Documents", [TreeItem("report.txt"), TreeItem("notes.md")]),
    TreeItem("Downloads"),
]
tree  = Tree(items).block(Block().bordered().title(" Files "))
state = TreeState()
state.select([0])
frame.render_stateful_tree(tree, area, state)
```

#### 📝 Markdown Renderer (`tui-markdown`)

Convert Markdown text to a styled `Text` object for terminal rendering.

```python
from pyratatui import markdown_to_text, Paragraph

text = markdown_to_text("# Hello\n\n**bold** and *italic*")
para = Paragraph(text).wrap(True)
frame.render_widget(para, area)
```

#### 📜 Logger Widget (`tui-logger`)

Real-time scrollable log viewer with level filtering.

```python
from pyratatui import TuiLoggerWidget, TuiWidgetState, init_logger, log_message

init_logger("debug")
log_message("info", "Application started")

widget = TuiLoggerWidget().block(Block().bordered())
state  = TuiWidgetState()
frame.render_stateful_logger(widget, area, state)
```

#### 🖼 Image Widget (`ratatui-image`)

Display PNG/JPEG images using unicode halfblocks or native graphics protocols.

```python
from pyratatui import ImagePicker, ImageWidget

picker = ImagePicker.halfblocks()
state  = picker.load("./photo.png")
widget = ImageWidget()
frame.render_stateful_image(widget, area, state)
```

#### 📅 Calendar Widget

Three new Python types for the built-in ratatui calendar widget:
`CalendarDate`, `CalendarEventStore`, `Monthly`.

---

### Changed

- 30 examples numbered `01_hello_world.py` through `30_image_view.py`
- `tests/python/test_pyratatui.py` expanded with tests for all new widgets
- `docs/` updated to reflect terminal-only operation and all new widgets

---

### Added

#### 📅 Calendar Widget

Three new Python types expose ratatui's built-in calendar widget:

**`CalendarDate`** — wraps `time::Date`. Represents a calendar day.

```python
from pyratatui import CalendarDate

today = CalendarDate.today()                    # UTC today
d     = CalendarDate.from_ymd(2024, 3, 15)     # March 15, 2024
print(d.year, d.month, d.day)                   # 2024 3 15
```

- `from_ymd(year, month, day)` raises `ValueError` for invalid dates.
- Implements `__hash__` and `__eq__` for use as dict/set keys.

**`CalendarEventStore`** — maps `CalendarDate` keys to `Style` values.
Implements ratatui's `DateStyler` trait.

```python
from pyratatui import CalendarEventStore, Style, Color

store = CalendarEventStore()
store.add(CalendarDate.from_ymd(2024, 12, 25), Style().fg(Color.red()).bold())
store.add_today(Style().fg(Color.green()).bold())

# Factory with today pre-highlighted
store = CalendarEventStore.today_highlighted(Style().fg(Color.cyan()).bold())
```

**`Monthly`** — the calendar widget. Fully composable builder.

```python
from pyratatui import Monthly, Block, Style, Color

cal = (
    Monthly(CalendarDate.today(), store)
    .block(Block().bordered().title(" My Calendar "))
    .show_month_header(Style().bold().fg(Color.cyan()))
    .show_weekdays_header(Style().italic().fg(Color.white()))
    .show_surrounding(Style().dim())
    .default_style(Style().fg(Color.white()))
)
frame.render_widget(cal, area)
```

Builder methods: `.block()`, `.default_style()`, `.show_surrounding()`,
`.show_month_header()`, `.show_weekdays_header()`. All return a new instance.

**New files:**
- `src/widgets/calendar.rs` — Rust bindings
- `docs/reference/calendar.md` — reference documentation
- `examples/25_calendar.py` — interactive demo (month/year nav, event styling)

**Modified files:**
- `Cargo.toml` — added `time = { version = "0.3", features = ["local-offset"] }`
- `src/widgets/mod.rs` — `calendar` module declared; types re-exported
- `src/terminal/mod.rs` — `Monthly` added to `render_widget()` dispatch
- `python/pyratatui/__init__.py` — `CalendarDate`, `CalendarEventStore`, `Monthly` exported
- `python/pyratatui/__init__.pyi` — full typed stubs for all three classes

---

### Changed

- `Cargo.toml` — version `0.2.0` → `0.2.1`
- `pyproject.toml` — version `0.2.0` → `0.2.1`; description updated
- `src/widgets/mod.rs` — `pub mod calendar` declared; `CalendarDate`,
  `CalendarEventStore`, `Monthly` added to `pub use` re-exports
- `src/terminal/mod.rs` — `Monthly` imported; `try_widget!(Monthly, to_ratatui)`
  inserted in `render_widget()` dispatch
- `mkdocs.yml` — Calendar entry added under *API Reference*;
  `site_description` updated to reference ratatui 0.30
- `README.md` — fully rewritten: ratatui 0.30, all widgets, calendar,
  updated examples table (01–26), corrected code snippets
- `docs/index.md` — version badges corrected (0.29 → 0.30 / 0.2.0 → 0.2.1);
  feature table updated; architecture diagram updated
- `docs/installation.md` — verify snippet updated to expect `"0.2.1"` / `"0.30"`
- `docs/reference/api_surface.md` — Calendar rows added to
  Core Types, Widgets, and Terminal & Frame tables
- `docs/examples/minimal_examples.md` — Calendar examples added;
  stale `Layout.default()` calls fixed to `Layout()`
- `docs/examples/advanced_examples.md` — Calendar advanced section added;
  outdated `Table(rows, widths)` constructor corrected
- `docs/faq.md` — ratatui 0.29 references updated to 0.30

---

### Fixed

- `docs/index.md` — stale ratatui 0.29 badge and version strings
- `docs/installation.md` — `__ratatui_version__` verify showed `"0.29"`, now `"0.30"`
- `docs/reference/api_surface.md` — missing `Popup`, `TextArea`, `ScrollView`,
  `QrCodeWidget` rows in widget table; all now present
- `docs/examples/minimal_examples.md` — `Layout.default()` (non-existent)
  replaced with `Layout()` in inline snippets
- `docs/examples/advanced_examples.md` — `Table(rows, widths)` constructor
  replaced with `Table(rows).column_widths(widths)` throughout
- `README.md` — `Constraint.Length` / `Constraint.Min` (uppercase 0.1.x API)
  replaced with lowercase `Constraint.length` / `Constraint.min`

---

## [0.2.0] — 2026-03-06

### Summary

`0.2.0` upgrades the native backend to **ratatui 0.30** and resolves all
compile-time errors and deprecation warnings introduced by the upgrade.
Adds `Block.inner(area)` and fixes all example API mismatches.

### Changed (breaking — Rust internals only)

- **ratatui 0.30** is now the native backend (`ratatui-core 0.1`, `ratatui-widgets 0.3`).
- `Terminal::area()` returns `Rect` converted from ratatui 0.30's `Size` type.
- `Frame::apply_effect` no longer requires `tachyonfx::Shader` in scope.
- `LineGauge::line_set()` uses `filled_symbol` / `unfilled_symbol` internally.
- `Table::row_highlight_style()` replaces deprecated `highlight_style()`.
- `Table::to_ratatui()` returns `RTable<'_>` (lifetime-aware) instead of `'static`.

### Added

- **`Block.inner(area: Rect) -> Rect`** — compute inner area after borders/padding.
- `ratatui_compat` internal bridge for tui-textarea 0.7 (ratatui 0.29) types.
- Examples renumbered sequentially (01–24) with no gaps.

### Fixed

- Example 06, 10 — `Table(rows, widths, header=…)` → `Table(rows).column_widths(…).header(…)`
- Example 15 — `Layout.default()` → `Layout()`; `Constraint.Min` → `Constraint.min`
- Examples 16, 17 — `Block.inner(area)` now available; `Layout.default()` fixed
- 18 compile errors from ratatui 0.29 → 0.30 resolved
- 54 deprecation warnings resolved (PyO3 0.28 API, tui-scrollview 0.6.2, etc.)

---

## [0.1.3] — 2026-03-06

### Summary

`0.1.3` adds four major third-party widget integrations:

| New module | Backing crate | What it adds |
|---|---|---|
| `Popup` family | `tui-popup` 0.7 | Centered floating dialogs with drag/scroll |
| `TextArea` family | `tui-textarea` 0.7 | Full multi-line text editor |
| `ScrollView` family | `tui-scrollview` 0.6 | Scrollable viewport |
| `QrCodeWidget` family | `tui-qrcode` / native | Terminal QR code renderer |

### Added

- `Popup`, `PopupState`, `KnownSizeWrapper` — floating dialog widgets
- `TextArea`, `CursorMove`, `Scrolling` — rich multi-line editor with Emacs bindings
- `ScrollView`, `ScrollViewState` — scrollable content viewport
- `QrCodeWidget`, `QrColors` — native Unicode half-block QR renderer
- `Frame` methods: `render_popup`, `render_stateful_popup`, `render_textarea`,
  `render_stateful_scrollview`, `render_qrcode`
- Examples 11–17 (popups, textarea, scrollview, qrcode)
- Reference docs: `popups.md`, `textarea.md`, `scrollview.md`, `qrcode.md`

---

## [0.1.2] — Earlier

_(Changelog entries for 0.1.2 and earlier are tracked in git history.)_
