"""
gauge.py — Live sync progress bar.

Demonstrates: Gauge, LineGauge, live value updates, time-based animation.
"""

import time

from pyratatui import (
    Block,
    Color,
    Constraint,
    Direction,
    Frame,
    Gauge,
    Layout,
    Line,
    LineGauge,
    Paragraph,
    Span,
    Style,
    Terminal,
    Text,
)

STEPS = 80
STEP_DELAY = 0.05

with Terminal() as term:
    term.hide_cursor()
    for step in range(STEPS + 1):
        pct = int(step / STEPS * 100)

        def ui(frame: Frame, p: int = pct, s: int = step) -> None:
            outer = (
                Layout()
                .direction(Direction.Vertical)
                .constraints(
                    [
                        Constraint.fill(1),
                        Constraint.length(3),
                        Constraint.length(3),
                        Constraint.length(1),
                    ]
                )
                .split(frame.area)
            )

            # Info pane
            color = Color.green() if p < 40 else Color.yellow() if p < 80 else Color.red()
            info = Text(
                [
                    Line([Span("Processing files…", Style().fg(Color.white()).bold())]),
                    Line([]),
                    Line(
                        [
                            Span(f"Step {s}/{STEPS}  —  ", Style().fg(Color.gray())),
                            Span(f"{p}% complete", Style().fg(color).bold()),
                        ]
                    ),
                ]
            )
            frame.render_widget(
                Paragraph(info).block(Block().bordered().title("Task Progress")),
                outer[0],
            )

            # Block gauge
            frame.render_widget(
                Gauge()
                .percent(p)
                .label(f"{p}%")
                .style(Style().fg(color))
                .gauge_style(Style().fg(Color.dark_gray()))
                .block(Block().bordered()),
                outer[1],
            )

            # Line gauge
            frame.render_widget(
                LineGauge()
                .percent(p)
                .style(Style().fg(Color.blue()))
                .gauge_style(Style().fg(Color.dark_gray()))
                .line_set("thick"),
                outer[2],
            )

            frame.render_widget(
                Paragraph.from_string(f" {p}% — {'█' * (p // 5)}{'░' * (20 - p // 5)}").style(
                    Style().fg(color)
                ),
                outer[3],
            )

        term.draw(ui)
        time.sleep(STEP_DELAY)

    # Show "done" for 1 second
    time.sleep(1.0)
    term.show_cursor()
