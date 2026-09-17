"""
examples/40_inline_viewport.py — Progress in a few lines of the normal buffer.

Demonstrates: Terminal(inline_height=...), LineGauge, Layout.
The shell prompt above stays where it is, and the summary is printed below the
block once the app is done. Press q to stop early.
"""

import time

from pyratatui import (
    Color,
    Constraint,
    Direction,
    Layout,
    LineGauge,
    Paragraph,
    Style,
    Terminal,
)

STEPS = [
    "Resolving dependencies",
    "Compiling sources",
    "Linking",
    "Running tests",
    "Packaging",
]

print("$ build --release")

with Terminal(inline_height=3) as term:
    started = time.monotonic()
    step = 0

    while step < len(STEPS):
        ratio = (step + 1) / len(STEPS)

        def ui(frame, _step=step, _ratio=ratio):
            title, gauge = (
                Layout()
                .direction(Direction.Vertical)
                .constraints([Constraint.length(1), Constraint.length(1)])
                .split(frame.area)
            )
            frame.render_widget(
                Paragraph.from_string(
                    f"{STEPS[_step]}… {int(_ratio * 100)}%"
                ).style(Style().fg(Color.cyan())),
                title,
            )
            frame.render_widget(
                LineGauge()
                .ratio(_ratio)
                .label("")
                .gauge_style(Style().fg(Color.green())),
                gauge,
            )

        term.draw(ui)

        ev = term.poll_event(timeout_ms=400)
        if ev and ev.code == "q":
            break
        step += 1

print(f"built in {time.monotonic() - started:.1f}s")
