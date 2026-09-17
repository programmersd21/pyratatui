"""
table.py — Dynamic table with live-updating data.

Demonstrates: Table, Row, Cell, TableState, live data mutation.
↑/↓ to select rows, r to reset data, q to quit.
"""

import random
import time
from typing import TypedDict

from pyratatui import (
    Block,
    Cell,
    Color,
    Constraint,
    Direction,
    Frame,
    Layout,
    Paragraph,
    Row,
    Style,
    Table,
    TableState,
    Terminal,
)


class Service(TypedDict):
    name: str
    cpu: int
    mem: int
    status: str


def random_data() -> list[Service]:
    services = ["nginx", "postgres", "redis", "kafka", "prometheus", "grafana"]
    return [
        {
            "name": s,
            "cpu": random.randint(0, 100),
            "mem": random.randint(0, 100),
            "status": random.choice(["Running", "Running", "Running", "Degraded", "Stopped"]),
        }
        for s in services
    ]


data = random_data()
state = TableState()
state.select(0)
last_update = time.time()


def cpu_style(pct: float) -> Style:
    if pct < 50:
        return Style().fg(Color.green())
    if pct < 80:
        return Style().fg(Color.yellow())
    return Style().fg(Color.red())


def status_style(s: str) -> Style:
    return {
        "Running": Style().fg(Color.green()),
        "Degraded": Style().fg(Color.yellow()),
        "Stopped": Style().fg(Color.red()),
    }.get(s, Style())


with Terminal() as term:
    while True:
        now = time.time()
        if now - last_update > 1.5:
            data = random_data()
            last_update = now

        def ui(frame: Frame, rows: list[Service] = data) -> None:
            chunks = (
                Layout()
                .direction(Direction.Vertical)
                .constraints([Constraint.fill(1), Constraint.length(1)])
                .split(frame.area)
            )

            header = Row(
                [
                    Cell("Service").style(Style().bold()),
                    Cell("CPU %").style(Style().bold()),
                    Cell("MEM %").style(Style().bold()),
                    Cell("Status").style(Style().bold()),
                ]
            ).style(Style().fg(Color.cyan()))

            tbl_rows = [
                Row(
                    [
                        Cell(r["name"]),
                        Cell(f"{r['cpu']:3d}%").style(cpu_style(r["cpu"])),
                        Cell(f"{r['mem']:3d}%").style(cpu_style(r["mem"])),
                        Cell(r["status"]).style(status_style(r["status"])),
                    ]
                )
                for r in rows
            ]

            table = (
                Table(tbl_rows)
                .column_widths(
                    [
                        Constraint.fill(1),
                        Constraint.length(8),
                        Constraint.length(8),
                        Constraint.length(12),
                    ]
                )
                .header(header)
                .block(Block().bordered().title("Services  (auto-refresh 1.5s)"))
                .highlight_style(Style().fg(Color.yellow()).bold())
                .highlight_symbol("▶ ")
            )
            frame.render_stateful_table(table, chunks[0], state)
            frame.render_widget(
                Paragraph.from_string(" ↑/↓: Select  r: Refresh  q: Quit").style(
                    Style().fg(Color.dark_gray())
                ),
                chunks[1],
            )

        term.draw(ui)
        ev = term.poll_event(timeout_ms=100)
        if ev:
            if ev.code == "q":
                break
            elif ev.code == "Down":
                state.select_next()
            elif ev.code == "Up":
                state.select_previous()
            elif ev.code == "r":
                data = random_data()
                last_update = time.time()
