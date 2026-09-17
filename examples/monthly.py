"""
monthly.py — Interactive calendar widget demo.

Demonstrates:
  - Monthly calendar widget
  - CalendarEventStore with custom date styles
  - CalendarDate (today, from_ymd)
  - Keyboard navigation (prev/next month, prev/next year, jump-to-today)

Controls:
  Left / Right   Previous / next month
  Up / Down      Previous / next year
  t              Jump to today
  q              Quit (returns currently-viewed date as datetime.date)
"""

from __future__ import annotations

import calendar as _cal
from contextlib import suppress
from datetime import date as _pydate

from pyratatui import (
    Block,
    CalendarDate,
    CalendarEventStore,
    Color,
    Constraint,
    Direction,
    Frame,
    Layout,
    Line,
    Monthly,
    Paragraph,
    Span,
    Style,
    Terminal,
    Text,
)

# ── Module-level state (updated by keyboard events) ───────────────────────────

_today: _pydate = _pydate.today()
_year: int = _today.year
_month: int = _today.month

_MONTH_NAMES: list[str] = [
    "",
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
]


# ── Event store builder ───────────────────────────────────────────────────────


def make_store(year: int, month: int) -> CalendarEventStore:
    """
    Build a ``CalendarEventStore`` for the given year/month.

    Highlights:
    - Today          → green bold
    - Weekends       → yellow dim
    - 1st of month   → cyan bold
    - 15th of month  → magenta bold
    """
    store = CalendarEventStore()

    if year == _today.year and month == _today.month:
        store.add_today(Style().fg(Color.green()).bold())

    _, days_in_month = _cal.monthrange(year, month)

    for day in range(1, days_in_month + 1):
        if _cal.weekday(year, month, day) >= 5:  # Saturday or Sunday
            with suppress(ValueError):
                store.add(
                    CalendarDate.from_ymd(year, month, day),
                    Style().fg(Color.yellow()).dim(),
                )

    with suppress(ValueError):
        store.add(
            CalendarDate.from_ymd(year, month, 1),
            Style().fg(Color.cyan()).bold(),
        )

    if days_in_month >= 15:
        with suppress(ValueError):
            store.add(
                CalendarDate.from_ymd(year, month, 15),
                Style().fg(Color.magenta()).bold(),
            )

    return store


# ── UI ────────────────────────────────────────────────────────────────────────


def ui(frame: Frame) -> None:
    area = frame.area

    outer = (
        Layout()
        .direction(Direction.Vertical)
        .constraints(
            [
                Constraint.length(3),
                Constraint.min(1),
                Constraint.length(3),
            ]
        )
        .split(area)
    )

    # Title bar
    frame.render_widget(
        Paragraph.from_string(f"  {_MONTH_NAMES[_month]}  {_year}")
        .block(Block().bordered().title(" Calendar Widget Demo "))
        .centered()
        .style(Style().fg(Color.cyan()).bold()),
        outer[0],
    )

    # Body: calendar left, legend right
    body = (
        Layout()
        .direction(Direction.Horizontal)
        .constraints([Constraint.length(30), Constraint.fill(1)])
        .split(outer[1])
    )

    # Suppress invalid dates (e.g. Feb 30) safely
    with suppress(ValueError):
        cal_date = CalendarDate.from_ymd(_year, _month, 1)
        store = make_store(_year, _month)

        frame.render_widget(
            Monthly(cal_date, store)
            .block(Block().bordered().title(" Monthly "))
            .show_month_header(Style().bold().fg(Color.cyan()))
            .show_weekdays_header(Style().italic().fg(Color.light_blue()))
            .show_surrounding(Style().dim())
            .default_style(Style().fg(Color.white())),
            body[0],
        )

    legend = Text(
        [
            Line([Span("Legend:", Style().bold().fg(Color.white()))]),
            Line([]),
            Line([Span("  today", Style().fg(Color.green()).bold())]),
            Line([Span("  weekend", Style().fg(Color.yellow()).dim())]),
            Line([Span("  1st of month", Style().fg(Color.cyan()).bold())]),
            Line([Span("  mid-month (15)", Style().fg(Color.magenta()).bold())]),
            Line([]),
            Line(
                [
                    Span(
                        f"Today: {_today.strftime('%Y-%m-%d')}",
                        Style().fg(Color.dark_gray()),
                    )
                ]
            ),
        ]
    )
    frame.render_widget(
        Paragraph(legend).block(Block().bordered().title(" Legend ")),
        body[1],
    )

    # Controls bar
    frame.render_widget(
        Paragraph.from_string(
            "  ←/→: prev/next month    ↑/↓: prev/next year    t: today    q: quit"
        )
        .block(Block().bordered().title(" Controls "))
        .style(Style().fg(Color.dark_gray())),
        outer[2],
    )


# ── Main ──────────────────────────────────────────────────────────────────────


def main() -> _pydate:
    """
    Run the interactive calendar demo.

    Returns
    -------
    datetime.date
        The currently-viewed date (1st of the displayed month) as a
        standard Python ``datetime.date`` object.
    """
    global _year, _month

    with Terminal() as term:
        while True:
            term.draw(ui)
            ev = term.poll_event(timeout_ms=200)
            if ev is None:
                continue

            if ev.code == "q" or (ev.ctrl and ev.code == "c"):
                break
            elif ev.code == "Left":
                _month -= 1
                if _month < 1:
                    _month = 12
                    _year -= 1
            elif ev.code == "Right":
                _month += 1
                if _month > 12:
                    _month = 1
                    _year += 1
            elif ev.code == "Up":
                _year += 1
            elif ev.code == "Down":
                _year -= 1
            elif ev.code == "t":
                _year = _today.year
                _month = _today.month

    # Return the first day of the currently-viewed month as datetime.date
    return _pydate(_year, _month, 1)


if __name__ == "__main__":
    selected = main()
    print(f"Exited on: {selected.strftime('%B %Y')}")
