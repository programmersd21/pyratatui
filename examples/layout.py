"""
layout.py — Multi-panel layout with nested splits.

Demonstrates: Layout, Constraint, Direction, nested splits.
Press q to quit.
"""

from pyratatui import (
    Block,
    Color,
    Constraint,
    Direction,
    Frame,
    Layout,
    Paragraph,
    Style,
    Terminal,
)

with Terminal() as term:
    while True:

        def ui(frame: Frame) -> None:
            area = frame.area

            # Outer: header | body | footer
            outer = (
                Layout()
                .direction(Direction.Vertical)
                .constraints([Constraint.length(3), Constraint.fill(1), Constraint.length(1)])
                .split(area)
            )

            # Body: sidebar | main
            body = (
                Layout()
                .direction(Direction.Horizontal)
                .constraints([Constraint.percentage(30), Constraint.fill(1)])
                .split(outer[1])
            )

            frame.render_widget(
                Block().bordered().title(" Header ").style(Style().fg(Color.cyan())),
                outer[0],
            )
            frame.render_widget(
                Paragraph.from_string("Sidebar\n\nNav items here").block(
                    Block().bordered().title("Nav")
                ),
                body[0],
            )
            frame.render_widget(
                Paragraph.from_string("Main content\n\nResize the terminal to see layout adapt.")
                .block(Block().bordered().title("Content"))
                .wrap(True),
                body[1],
            )
            frame.render_widget(
                Paragraph.from_string(" q: Quit").style(Style().fg(Color.dark_gray())),
                outer[2],
            )

        term.draw(ui)
        ev = term.poll_event(timeout_ms=100)
        if ev and ev.code == "q":
            break
