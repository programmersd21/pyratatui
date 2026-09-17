"""
async_app.py — Async reactive UI with background data fetching.

Demonstrates: AsyncTerminal, asyncio tasks, reactive data updates, live counter.
"""

import asyncio
import contextlib
import random
import time
from typing import TypedDict

from pyratatui import (
    AsyncTerminal,
    Block,
    Color,
    Constraint,
    Direction,
    Frame,
    Gauge,
    Layout,
    Line,
    Paragraph,
    Span,
    Sparkline,
    Style,
    Text,
)

# Shared state (updated by background task)


class Metrics(TypedDict):
    cpu: int
    mem: int
    requests: int
    history: list[int]
    tick: int
    log: list[str]


state: Metrics = {
    "cpu": 0,
    "mem": 0,
    "requests": 0,
    "history": [0] * 30,
    "tick": 0,
    "log": [],
}


async def simulate_metrics() -> None:
    """Background task: simulate live server metrics."""
    while True:
        await asyncio.sleep(0.3)
        state["cpu"] = max(0, min(100, state["cpu"] + random.randint(-8, 10)))
        state["mem"] = max(10, min(95, state["mem"] + random.randint(-3, 4)))
        state["requests"] += random.randint(10, 50)
        state["tick"] += 1
        state["history"].append(state["cpu"])
        state["history"] = state["history"][-30:]
        ts = time.strftime("%H:%M:%S")
        if state["tick"] % 5 == 0:
            state["log"].append(f"[{ts}] Metrics tick {state['tick']}")
            state["log"] = state["log"][-6:]


async def main() -> None:
    metrics_task = asyncio.create_task(simulate_metrics())

    async with AsyncTerminal() as term:
        term.hide_cursor()
        async for ev in term.events(fps=20):
            if ev is not None and (ev.code == "q" or (ev.code == "c" and ev.ctrl)):
                break
            cpu = state["cpu"]
            mem = state["mem"]
            reqs = state["requests"]
            hist = list(state["history"])
            log = list(state["log"])
            tick = state["tick"]

            def ui(
                frame: Frame,
                _cpu: int = cpu,
                _mem: int = mem,
                _reqs: int = reqs,
                _hist: list[int] = hist,
                _log: list[str] = log,
                _tick: int = tick,
            ) -> None:
                area = frame.area
                outer = (
                    Layout()
                    .direction(Direction.Vertical)
                    .constraints(
                        [
                            Constraint.length(3),
                            Constraint.length(3),
                            Constraint.length(5),
                            Constraint.fill(1),
                            Constraint.length(1),
                        ]
                    )
                    .split(area)
                )

                cpu_color = (
                    Color.green() if _cpu < 50 else Color.yellow() if _cpu < 80 else Color.red()
                )

                # CPU gauge
                frame.render_widget(
                    Gauge()
                    .percent(_cpu)
                    .label(f"CPU: {_cpu}%  (tick {_tick})")
                    .style(Style().fg(cpu_color))
                    .gauge_style(Style().fg(Color.dark_gray()))
                    .block(Block().bordered().title("CPU")),
                    outer[0],
                )

                # MEM gauge
                frame.render_widget(
                    Gauge()
                    .percent(_mem)
                    .label(f"MEM: {_mem}%")
                    .style(Style().fg(Color.blue()))
                    .gauge_style(Style().fg(Color.dark_gray()))
                    .block(Block().bordered().title("Memory")),
                    outer[1],
                )

                # Sparkline
                frame.render_widget(
                    Sparkline()
                    .data([int(h) for h in _hist])
                    .max(100)
                    .style(Style().fg(cpu_color))
                    .block(Block().bordered().title("CPU History")),
                    outer[2],
                )

                # Metrics + log
                body = (
                    Layout()
                    .direction(Direction.Horizontal)
                    .constraints([Constraint.percentage(40), Constraint.fill(1)])
                    .split(outer[3])
                )

                frame.render_widget(
                    Paragraph(
                        Text(
                            [
                                Line(
                                    [
                                        Span("Requests: ", Style().bold()),
                                        Span(str(_reqs), Style().fg(Color.cyan())),
                                    ]
                                ),
                                Line(
                                    [
                                        Span("Uptime:   ", Style().bold()),
                                        Span(
                                            f"{_tick * 0.3:.1f}s",
                                            Style().fg(Color.green()),
                                        ),
                                    ]
                                ),
                            ]
                        )
                    ).block(Block().bordered().title("Stats")),
                    body[0],
                )

                frame.render_widget(
                    Paragraph.from_string("\n".join(_log) or "(waiting…)")
                    .block(Block().bordered().title("Log"))
                    .style(Style().fg(Color.gray())),
                    body[1],
                )

                frame.render_widget(
                    Paragraph.from_string(" q: Quit  (auto-refreshing)").style(
                        Style().fg(Color.dark_gray())
                    ),
                    outer[4],
                )

            term.draw(ui)

    metrics_task.cancel()
    with contextlib.suppress(asyncio.CancelledError):
        await metrics_task


asyncio.run(main())
