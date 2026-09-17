"""
styled_text.py — Rich styled text with Span, Line, Text.

Demonstrates: Span, Line, Text, Style, Color, Modifier.
Press q to quit.
"""

from pyratatui import Block, Color, Frame, Line, Paragraph, Span, Style, Terminal, Text

with Terminal() as term:
    while True:

        def ui(frame: Frame) -> None:
            text = Text(
                [
                    Line(
                        [
                            Span("Status: ", Style().bold()),
                            Span("● Running", Style().fg(Color.green()).bold()),
                        ]
                    ),
                    Line([]),
                    Line(
                        [
                            Span("CPU:  ", Style().fg(Color.white())),
                            Span("▓▓▓▓▓▓░░░░ 60%", Style().fg(Color.yellow())),
                        ]
                    ),
                    Line(
                        [
                            Span("MEM:  ", Style().fg(Color.white())),
                            Span("▓▓▓▓░░░░░░ 40%", Style().fg(Color.cyan())),
                        ]
                    ),
                    Line([]),
                    Line(
                        [
                            Span("Press ", Style().fg(Color.dark_gray())),
                            Span("q", Style().fg(Color.red()).bold().underlined()),
                            Span(" to quit", Style().fg(Color.dark_gray())),
                        ]
                    ),
                ]
            )
            frame.render_widget(
                Paragraph(text).block(Block().bordered().title("Styled Text Demo")),
                frame.area,
            )

        term.draw(ui)
        ev = term.poll_event(timeout_ms=100)
        if ev and ev.code == "q":
            break
