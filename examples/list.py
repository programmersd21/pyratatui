"""
list.py — Keyboard-navigable list.

Demonstrates: List, ListItem, ListState, keyboard input handling.
↑/↓  Home  End  to navigate,  q  to quit.
"""

from typing import TypedDict

from pyratatui import (
    Block,
    Color,
    Constraint,
    Direction,
    Frame,
    Layout,
    Line,
    List,
    ListItem,
    ListState,
    Paragraph,
    Span,
    Style,
    Terminal,
    Text,
)

# ── Planet data (no emoji — use reliable ASCII/single-width symbols) ──────────


class Planet(TypedDict):
    name: str
    status: str
    order: int
    dist: str
    moons: int
    desc: list[str]


PLANETS: list[Planet] = [
    {
        "name": "Earth",
        "status": "Online",
        "order": 3,
        "dist": "1.00 AU",
        "moons": 1,
        "desc": [
            "Our home.  Third rock from the Sun,",
            "and the only known harbour of life.",
        ],
    },
    {
        "name": "Mars",
        "status": "Degraded",
        "order": 4,
        "dist": "1.52 AU",
        "moons": 2,
        "desc": [
            "The Red Planet.  Home to Olympus Mons,",
            "the tallest volcano in the Solar System.",
        ],
    },
    {
        "name": "Saturn",
        "status": "Online",
        "order": 6,
        "dist": "9.58 AU",
        "moons": 146,
        "desc": [
            "Lord of the rings.  Its iconic ring system",
            "is made mostly of ice and rock.",
        ],
    },
    {
        "name": "Jupiter",
        "status": "Online",
        "order": 5,
        "dist": "5.20 AU",
        "moons": 95,
        "desc": [
            "King of planets — largest in the Solar System.",
            "The Great Red Spot is a storm >350 years old.",
        ],
    },
    {
        "name": "Neptune",
        "status": "Offline",
        "order": 8,
        "dist": "30.07 AU",
        "moons": 16,
        "desc": [
            "Windiest planet in the Solar System.",
            "Gusts reach up to 2,100 km/h.",
        ],
    },
    {
        "name": "Pluto",
        "status": "Unknown",
        "order": 9,
        "dist": "39.48 AU",
        "moons": 5,
        "desc": [
            "Dwarf planet in the Kuiper Belt.",
            "Demoted from full planet status in 2006.",
        ],
    },
]


def status_color(status: str) -> Color:
    return {
        "Online": Color.green(),
        "Degraded": Color.yellow(),
        "Offline": Color.red(),
        "Unknown": Color.dark_gray(),
    }.get(status, Color.white())


def status_symbol(status: str) -> str:
    # Single-width ASCII glyphs only — safe in every terminal
    return {
        "Online": "[+]",
        "Degraded": "[~]",
        "Offline": "[-]",
        "Unknown": "[?]",
    }.get(status, "[ ]")


def main() -> None:
    state = ListState()
    state.select(0)

    with Terminal() as term:
        while True:
            # ── Hard clamp — never let index escape valid range ───────────────
            # ratatui's select_next() / select_previous() wrap around, so we
            # do all navigation manually with min/max instead.
            sel = state.selected
            if sel is None or sel < 0:
                sel = 0
                state.select(sel)
            elif sel >= len(PLANETS):
                sel = len(PLANETS) - 1
                state.select(sel)

            planet = PLANETS[sel]

            def ui(frame: Frame, _sel: int = sel, _planet: Planet = planet) -> None:
                area = frame.area
                chunks = (
                    Layout()
                    .direction(Direction.Horizontal)
                    .constraints(
                        [
                            Constraint.percentage(44),
                            Constraint.fill(1),
                        ]
                    )
                    .split(area)
                )

                # ── Left: planet list ─────────────────────────────────────────
                items = [
                    ListItem(
                        f"  {status_symbol(p['status'])}  {p['name']:<10s}  {p['status']}",
                        Style().fg(status_color(p["status"])),
                    )
                    for p in PLANETS
                ]
                frame.render_stateful_list(
                    List(items)
                    .block(Block().bordered().title(" Solar System "))
                    .highlight_style(Style().fg(Color.cyan()).bold())
                    .highlight_symbol("▶ "),
                    chunks[0],
                    state,
                )

                # ── Right: detail panel ───────────────────────────────────────
                sc = status_color(_planet["status"])
                sym = status_symbol(_planet["status"])

                lines: list[Line] = [
                    Line(
                        [
                            Span(f"  {_planet['name']}", Style().fg(Color.white()).bold()),
                            Span(
                                f"   planet #{_planet['order']}",
                                Style().fg(Color.dark_gray()),
                            ),
                        ]
                    ),
                    Line([]),
                    Line(
                        [
                            Span("  Status   : ", Style().fg(Color.dark_gray())),
                            Span(f"{sym}  {_planet['status']}", Style().fg(sc).bold()),
                        ]
                    ),
                    Line(
                        [
                            Span("  Distance : ", Style().fg(Color.dark_gray())),
                            Span(_planet["dist"], Style().fg(Color.cyan())),
                        ]
                    ),
                    Line(
                        [
                            Span("  Moons    : ", Style().fg(Color.dark_gray())),
                            Span(str(_planet["moons"]), Style().fg(Color.cyan())),
                        ]
                    ),
                    Line(
                        [
                            Span("  Index    : ", Style().fg(Color.dark_gray())),
                            Span(
                                f"{_sel + 1} / {len(PLANETS)}",
                                Style().fg(Color.white()),
                            ),
                        ]
                    ),
                    Line([]),
                ]
                for desc_line in _planet["desc"]:
                    lines.append(Line([Span(f"  {desc_line}", Style().fg(Color.white()))]))
                lines += [
                    Line([]),
                    Line(
                        [
                            Span("  ↑/↓", Style().fg(Color.yellow()).bold()),
                            Span(": Navigate   ", Style().fg(Color.dark_gray())),
                            Span("Home/End", Style().fg(Color.yellow()).bold()),
                            Span(": Jump   ", Style().fg(Color.dark_gray())),
                            Span("q", Style().fg(Color.yellow()).bold()),
                            Span(": Quit", Style().fg(Color.dark_gray())),
                        ]
                    ),
                ]

                frame.render_widget(
                    Paragraph(Text(lines)).block(
                        Block().bordered().title(f" {_planet['name']} — Detail ")
                    ),
                    chunks[1],
                )

            term.draw(ui)

            ev = term.poll_event(timeout_ms=50)
            if ev:
                if ev.code == "q" or (ev.code == "c" and ev.ctrl):
                    break
                elif ev.code == "Down":
                    # Manual clamp — no wrap-around
                    state.select(min(sel + 1, len(PLANETS) - 1))
                elif ev.code == "Up":
                    state.select(max(sel - 1, 0))
                elif ev.code == "Home":
                    state.select(0)
                elif ev.code == "End":
                    state.select(len(PLANETS) - 1)


if __name__ == "__main__":
    main()
