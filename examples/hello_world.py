"""
hello_world.py — Minimal hello world.

Demonstrates: Terminal context manager, Paragraph, Block, Style, Color.
Press q to quit.
"""

from pyratatui import Block, Color, Frame, Paragraph, Style, Terminal

with Terminal() as term:
    while True:

        def ui(frame: Frame) -> None:
            frame.render_widget(
                Paragraph.from_string("Hello, pyratatui! 🐀  Press q to quit.")
                .block(Block().bordered().title("Hello World"))
                .style(Style().fg(Color.cyan())),
                frame.area,
            )

        term.draw(ui)
        ev = term.poll_event(timeout_ms=100)
        if ev and ev.code == "q":
            break
