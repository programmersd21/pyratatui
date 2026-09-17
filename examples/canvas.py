#!/usr/bin/env python3
"""Canvas drawing demo."""

from pyratatui import Canvas, Terminal


def main() -> None:
    canvas = Canvas(width=60, height=20)
    canvas.draw_line(0, 0, 20, 10)
    canvas.draw_point(5, 5)
    canvas.draw_rect(10, 5, 20, 6)

    with Terminal() as term:
        while True:
            term.draw(lambda frame: frame.render_widget(canvas, frame.area))
            event = term.poll_event(timeout_ms=100)
            if event and event.code in {"q", "Esc"}:
                break


if __name__ == "__main__":
    main()
