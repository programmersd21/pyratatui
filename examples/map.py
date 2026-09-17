#!/usr/bin/env python3
"""Map widget demo."""

from pyratatui import Map, MapResolution, Terminal


def main() -> None:
    map_widget = Map(resolution=MapResolution.High)

    with Terminal() as term:
        while True:
            term.draw(lambda frame: frame.render_widget(map_widget, frame.area))
            event = term.poll_event(timeout_ms=100)
            if event and event.code in {"q", "Esc"}:
                break


if __name__ == "__main__":
    main()
