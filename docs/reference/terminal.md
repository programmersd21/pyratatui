# Terminal & Frame Reference

The `Terminal` and `Frame` classes form the heart of every pyratatui application. `Terminal` owns the screen, drives the render loop, and bridges keyboard events. `Frame` is passed to your draw callback each tick and provides all rendering methods.

---

## Terminal

```python
from pyratatui import Terminal
```

`Terminal` wraps ratatui's `CrosstermBackend` and manages alternate-screen mode, raw input, and the draw cycle.

### Constructor

```python
Terminal(inline_height=None)
```

Creates a new `Terminal` instance. The screen is **not** initialized until `__enter__` is called (via `with Terminal() as t:`).

| Parameter | Type | Description |
|---|---|---|
| `inline_height` | `int \| None` | Height of an inline viewport, in lines. `None` (the default) takes over the whole screen. |

### Context Manager

```python
with Terminal() as term:
    ...
```

`__enter__`:
1. Enables raw terminal mode (`enable_raw_mode`)
2. Enters alternate screen (`EnterAlternateScreen`)
3. Initializes the crossterm backend

`__exit__`:
1. Disables raw mode
2. Leaves alternate screen
3. Suppresses `False` (does not swallow exceptions)

### Inline Viewport

```python
with Terminal(inline_height=8) as term:
    ...
```

With `inline_height` the app draws in a block of that many lines **inside the normal buffer**, at the point the cursor was on. The alternate screen is never entered, so the scrollback and whatever was printed before the app stay where they are, and anything printed afterwards continues below the block — the shape a build tool or a progress display usually wants.

```python
from pyratatui import Paragraph, Terminal

print("$ deploy --to production")          # stays on screen

with Terminal(inline_height=3) as term:    # three lines under it
    for step in ("uploading", "migrating", "restarting"):
        term.draw(
            lambda frame, step=step: frame.render_widget(
                Paragraph.from_string(step), frame.area
            )
        )

print("done")                              # continues below the block
```

`frame.area` is the block, not the terminal: its `height` is the `inline_height` that was asked for. On leaving, the block is cleared and the cursor is left under it.

!!! note "The terminal is asked where the cursor is"
    An inline viewport needs the current cursor row, which it gets by writing a cursor position report (`ESC [ 6n`) and reading the answer. A terminal that never answers — some CI logs, a pipe that is not a TTY — makes `__enter__` fail with `BackendError`. Fall back to the fullscreen `Terminal()`, or to plain printing, when there is no real terminal.

!!! warning "Always Use as Context Manager"
    Never call `Terminal()` without a `with` statement. If your code raises an exception, the context manager guarantees terminal restoration. Without it, your user's terminal may be left in raw mode with the alternate screen active.

### `draw(draw_fn)`

```python
term.draw(lambda frame: ...)
```

Renders one frame by calling `draw_fn(frame)`. The `frame` object is only valid inside the callback — do not store references to it.

| Parameter | Type | Description |
|---|---|---|
| `draw_fn` | `Callable[[Frame], None]` | Function that renders widgets onto the frame |

**Raises:** `BackendError` if the terminal is not initialized or the flush fails.

### `poll_event(timeout_ms=0)`

```python
ev = term.poll_event(timeout_ms=100)
```

Polls for a keyboard event.

| Parameter | Type | Default | Description |
|---|---|---|---|
| `timeout_ms` | `int` | `0` | Milliseconds to wait (0 = non-blocking) |

**Returns:** `KeyEvent` if a key was pressed during the timeout, otherwise `None`.

Only `KeyEventKind::Press` events are returned — key-repeat and release events are filtered out.

### `area()`

```python
rect = term.area()
```

**Returns:** `Rect` — the current terminal dimensions (updated on resize).

**Raises:** `RuntimeError` if the terminal is not initialized.

### `clear()`

Forces a full repaint on the next `draw()` call. Use after a terminal resize event or when switching between large UI states.

### `hide_cursor()` / `show_cursor()`

```python
term.hide_cursor()
# ... rendering loop ...
term.show_cursor()
```

Hides or shows the terminal cursor. Always call `show_cursor()` before exiting so the user's cursor is restored.

### `restore()`

```python
term.restore()
```

Manually restore the terminal (disable raw mode, leave alternate screen). Called automatically by `__exit__`, but available for emergency cleanup.

### Async Context Manager

```python
async with Terminal() as term:
    ...
```

`__aenter__` and `__aexit__` provide the same behavior as the sync versions but in an async context. For typical async apps, prefer `AsyncTerminal` which handles frame pacing automatically.

---

## AsyncTerminal

```python
from pyratatui import AsyncTerminal
```

An asyncio-compatible wrapper around `Terminal`. All `Terminal` calls happen on the asyncio event-loop thread — never via thread-pool executors.

### Constructor

```python
AsyncTerminal(inline_height=None)
```

| Parameter | Type | Description |
|---|---|---|
| `inline_height` | `int \| None` | Height of an inline viewport, in lines, passed straight to `Terminal`. `None` (the default) takes over the whole screen. |

### Async Context Manager

```python
async with AsyncTerminal(inline_height=6) as term:
    ...
```

Equivalent to entering and exiting `Terminal` synchronously on the main async thread.

### `events(fps=30.0, *, stop_on_quit=False)`

By default `events()` keeps yielding each frame until your code breaks the loop; pass `stop_on_quit=True` to opt into auto-stopping when `q` or `Ctrl+C` are pressed.

```python
async for ev in term.events(fps=30):
    term.draw(ui)
    if ev:
        handle(ev)
```

Async generator that drives the render loop:

1. Non-blocking `poll_event(0)` on the main thread
2. Yields `KeyEvent | None`
3. `await asyncio.sleep(remaining_frame_time)` — yields to background coroutines

| Parameter | Type | Default | Description |
|---|---|---|---|
| `fps` | `float` | `30.0` | Target frames per second |
| `stop_on_quit` | `bool` | `False` | Auto-stop on `q` or `Ctrl+C` when enabled |

### `draw(draw_fn)` / `poll_event()` / `area()` / `clear()` / `hide_cursor()` / `show_cursor()`

Same signatures as `Terminal` — these are thin wrappers that assert the terminal is active before delegating.

---

## Frame

```python
# Frame is never constructed directly — it arrives as the argument to draw_fn:
def ui(frame: Frame):
    ...
```

`Frame` is only valid **inside** the `draw_fn` callback. Storing a reference outside the callback and using it later is undefined behavior.

### `area` (property)

```python
area: Rect = frame.area
```

The full terminal area available for this frame. Equivalent to `frame.size`.

### `size` (property)

Alias for `area`.

### `render_widget(widget, area)`

```python
frame.render_widget(widget, area)
```

Render a stateless widget into `area`.

**Supported widget types:** `Block`, `Paragraph`, `Gauge`, `LineGauge`, `BarChart`, `Sparkline`, `Clear`, `Tabs`, `Monthly`, `BarGraph`, `Canvas`, `Map`, `Button`, `Throbber`, `PieChart`, `Checkbox`, `Chart`, `List` (stateless), `Table` (stateless).

| Parameter | Type | Description |
|---|---|---|
| `widget` | widget object | Any supported widget instance |
| `area` | `Rect` | Where to draw the widget |

**Raises:** `RenderError` if the widget type is not recognized.

### `render_stateful_list(widget, area, state)`

```python
frame.render_stateful_list(list_widget, area, list_state)
```

Render a `List` with mutable selection state. The state object is mutated in-place (scroll offset is updated to keep the selection visible).

| Parameter | Type | Description |
|---|---|---|
| `widget` | `List` | The list widget |
| `area` | `Rect` | Target area |
| `state` | `ListState` | Mutable selection state |

### `render_stateful_table(widget, area, state)`

```python
frame.render_stateful_table(table_widget, area, table_state)
```

Render a `Table` with mutable selection state.

| Parameter | Type | Description |
|---|---|---|
| `widget` | `Table` | The table widget |
| `area` | `Rect` | Target area |
| `state` | `TableState` | Mutable selection state |

### `render_stateful_scrollbar(widget, area, state)`

```python
frame.render_stateful_scrollbar(scrollbar, area, scrollbar_state)
```

Render a `Scrollbar` with its scroll position state.

### `render_stateful_menu(widget, area, state)`

```python
frame.render_stateful_menu(menu, area, menu_state)
```

Render a `Menu` with mutable `MenuState`.

### `apply_effect(effect, elapsed_ms, area)`

```python
frame.apply_effect(effect, elapsed_ms, area)
```

Apply a single TachyonFX `Effect` to the frame buffer. Call **after** all `render_widget` calls.

| Parameter | Type | Description |
|---|---|---|
| `effect` | `Effect` | The effect to advance and apply |
| `elapsed_ms` | `int` | Wall-clock milliseconds since last frame |
| `area` | `Rect` | Area of buffer cells to transform |

### `apply_effect_manager(manager, elapsed_ms, area)`

```python
frame.apply_effect_manager(mgr, elapsed_ms, area)
```

Advance all effects in `mgr` and remove completed ones. Call **after** all `render_widget` calls.

| Parameter | Type | Description |
|---|---|---|
| `manager` | `EffectManager` | The effect manager to advance |
| `elapsed_ms` | `int` | Wall-clock ms since last frame |
| `area` | `Rect` | Area to apply effects to |

---

## KeyEvent

```python
from pyratatui import KeyEvent
```

Returned by `Terminal.poll_event()` and `AsyncTerminal.poll_event()`.

### Properties

| Property | Type | Description |
|---|---|---|
| `code` | `str` | Key code string (see table below) |
| `ctrl` | `bool` | Whether Ctrl was held |
| `alt` | `bool` | Whether Alt was held |
| `shift` | `bool` | Whether Shift was held |

### Key Code Strings

| Physical key | `code` value |
|---|---|
| Letter `a`–`z` | `"a"`–`"z"` |
| Letter `A`–`Z` | `"A"`–`"Z"` |
| Digits `0`–`9` | `"0"`–`"9"` |
| Enter / Return | `"Enter"` |
| Escape | `"Esc"` |
| Backspace | `"Backspace"` |
| Delete | `"Delete"` |
| Tab | `"Tab"` |
| Shift+Tab | `"BackTab"` |
| Arrow Up | `"Up"` |
| Arrow Down | `"Down"` |
| Arrow Left | `"Left"` |
| Arrow Right | `"Right"` |
| Home | `"Home"` |
| End | `"End"` |
| Page Up | `"PageUp"` |
| Page Down | `"PageDown"` |
| Insert | `"Insert"` |
| F1–F12 | `"F1"`–`"F12"` |
| Space | `" "` |
| Unknown | `"Unknown"` |

### Example: Key Handling

```python
ev = term.poll_event(timeout_ms=100)
if ev:
    if ev.code == "q":
        running = False
    elif ev.code == "c" and ev.ctrl:
        running = False
    elif ev.code == "Up":
        state.select_previous()
    elif ev.code == "Down":
        state.select_next()
    elif ev.code == "Enter":
        confirm_selection()
    elif ev.code == "F5":
        refresh()
```

---

## Exception Hierarchy

All pyratatui exceptions derive from `PyratatuiError`:

```python
from pyratatui import (
    PyratatuiError,   # Base class
    BackendError,     # Terminal I/O failures
    LayoutError,      # Invalid layout constraints
    RenderError,      # Unknown widget type
    AsyncError,       # Async bridging errors
    StyleError,       # Invalid style values
)

try:
    with Terminal() as term:
        term.draw(ui)
except BackendError as e:
    print(f"Terminal error: {e}")
except PyratatuiError as e:
    print(f"pyratatui error: {e}")
```

---

## Helper Functions

### `run_app(draw_fn, fps=30)`

```python
from pyratatui import run_app

def my_ui(frame):
    frame.render_widget(Paragraph.from_string("Hello!"), frame.area)

run_app(my_ui, fps=30)
```

Synchronous convenience wrapper: creates a `Terminal`, runs the draw loop at `fps`, and depends on your `on_key` callback returning `True` to exit; the helper no longer interprets `q` or `Ctrl+C` for you.

### `run_app_async(draw_fn, fps=30)`

```python
from pyratatui import run_app_async
import asyncio

async def my_ui(frame):
    frame.render_widget(Paragraph.from_string("Async!"), frame.area)

asyncio.run(run_app_async(my_ui, fps=30))
```

Async version using `AsyncTerminal`. Provide an `on_key` callback to dictate when the helper should stop.
