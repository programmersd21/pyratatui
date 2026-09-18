![pyratatui banner](https://raw.githubusercontent.com/pyratatui/pyratatui/main/assets/banner.svg)

# pyratatui

Python bindings for [Ratatui](https://ratatui.rs), implemented in Rust with
[PyO3](https://pyo3.rs).

Version 0.3.0 targets Ratatui 0.30.2 and supports Python 3.10+.

## Installation

```bash
pip install pyratatui
```

Building from source requires Rust 1.88 or newer and maturin:

```bash
pip install maturin
maturin develop --release
```

## Example

```python
from pyratatui import Block, Color, Paragraph, Style, run_app


def ui(frame):
    frame.render_widget(
        Paragraph.from_string("Hello! Press q to quit.")
        .block(Block().bordered().title("Hello"))
        .style(Style().fg(Color.cyan())),
        frame.area,
    )


run_app(ui, on_key=lambda ev: ev.code == "q")
```

More examples live in `examples/`. Each one is a small runnable program:

```bash
python examples/hello_world.py
python examples/dashboard.py
```

## Patterns

`term.draw` takes a callback that receives the frame. Snapshot loop state
into default arguments so each frame renders its own tick:

```python
with Terminal() as term:
    while True:
        count = state["count"]
        term.draw(lambda frame, _count=count: render(frame, _count))
        ev = term.poll_event(timeout_ms=100)
        if ev and ev.code == "q":
            break
```

Quitting is always explicit — `run_app` stops when `on_key` returns `True`,
and manual loops break on whatever key they choose.

## API

The public API is exported from the top-level package:

```python
from pyratatui import (
    Terminal,  # terminal driver (also AsyncTerminal, run_app, run_app_async)
    Frame,  # render surface handed to the draw callback
    KeyEvent,  # keyboard events from poll_event
    Layout,
    Constraint,
    Direction,
    Alignment,
    Rect,
    Style,
    Color,
    Modifier,
    Span,
    Line,
    Text,
    Block,
    Paragraph,
    List,
    Table,
    Gauge,
    BarChart,
    Sparkline,
    Scrollbar,
    Tabs,
    Clear,
    Chart,
    Canvas,
    Map,
    Monthly,
    TextPrompt,
    TextState,
    prompt_text,  # input helpers
    Buffer,  # off-screen surface (testing)
)
```

Widgets are Ratatui's built-in set plus thin `Chart`, `Canvas`, `Map`, and
calendar (`Monthly`) bindings. Types carry docstrings, and
`python/pyratatui/_pyratatui.pyi` ships full type annotations.

## Development

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
maturin develop

pytest            # Python tests (includes pty-based terminal tests)
cargo test        # Rust tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
ruff check . && ruff format --check .
```

See [CONTRIBUTING.md](https://github.com/pyratatui/pyratatui/blob/main/CONTRIBUTING.md) for details.

## Acknowledgements
* Special thanks to the [Ratatui](https://github.com/ratatui/ratatui) project and designer Pavel Fomchenkov for the Ratatui logo.
* Also, thanks to everyone who has contributed to `pyratatui`!

## License

MIT — see [LICENSE](https://github.com/pyratatui/pyratatui/blob/main/LICENSE).
