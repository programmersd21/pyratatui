"""
dashboard.py — Full-featured dashboard demo.

Tabs
----
  Overview   — server list, sparkline, process table, bar chart, CPU gauge
  Logs       — colour-coded live log feed with auto-follow and stats

Navigation
----------
  Tab          — cycle tabs
  ↑/↓          — navigate list / table / scroll logs
  Home / End   — (Logs tab) jump to oldest / newest entry
  q / Ctrl-C   — quit
"""

from __future__ import annotations

import math
import random
import time
from typing import TypedDict

from pyratatui import (
    Bar,
    BarChart,
    BarGroup,
    Block,
    BorderType,
    Cell,
    Color,
    Constraint,
    Direction,
    Frame,
    Gauge,
    Layout,
    Line,
    List,
    ListItem,
    ListState,
    Paragraph,
    Row,
    Span,
    Sparkline,
    Style,
    Table,
    TableState,
    Tabs,
    Terminal,
    Text,
)

# ── Simulated server data ─────────────────────────────────────────────────────


class Server(TypedDict):
    name: str
    cpu: int
    mem: int
    status: str


SERVERS: list[Server] = [
    {"name": "web-01", "cpu": 23, "mem": 45, "status": "Running"},
    {"name": "web-02", "cpu": 5, "mem": 30, "status": "Running"},
    {"name": "db-01", "cpu": 67, "mem": 80, "status": "Running"},
    {"name": "cache-01", "cpu": 1, "mem": 12, "status": "Running"},
    {"name": "queue-01", "cpu": 34, "mem": 55, "status": "Stopped"},
]

CPU_HISTORY: list[int] = [int(20 + 30 * abs(math.sin(i * 0.4))) for i in range(40)]

# ── Live log feed ─────────────────────────────────────────────────────────────

LOG_ENTRIES: list[tuple[str, str, str]] = [
    ("12:00:01", "INFO", "Server started on port 8080"),
    ("12:00:02", "INFO", "Database connection established"),
    ("12:00:05", "DEBUG", "Loading config from /etc/app/config.yaml"),
    ("12:00:10", "INFO", "Worker pool initialised with 4 workers"),
    ("12:01:23", "WARN", "Response time 850 ms exceeds threshold (500 ms)"),
    ("12:01:45", "ERROR", "Connection timeout to db-02:5432"),
    ("12:02:01", "INFO", "Retrying connection to db-02:5432"),
    ("12:02:03", "INFO", "Connection to db-02:5432 restored"),
    ("12:03:15", "DEBUG", "Cache hit ratio: 87.3%"),
    ("12:03:30", "WARN", "Memory usage at 78%, approaching limit"),
    ("12:04:00", "INFO", "Scheduled task 'cleanup' started"),
    ("12:04:05", "INFO", "Cleaned up 1,234 expired sessions"),
    ("12:04:06", "INFO", "Scheduled task 'cleanup' completed in 312 ms"),
    ("12:05:12", "ERROR", "Failed to parse request body: unexpected EOF"),
    ("12:05:13", "WARN", "Client 192.168.1.42 sent malformed request"),
    ("12:06:00", "INFO", "Health check passed"),
    ("12:06:30", "DEBUG", "Queue depth: 42 pending messages"),
    ("12:07:15", "INFO", "Processed batch of 250 items in 1.2 s"),
    ("12:08:00", "WARN", "Disk usage at 65% on /var"),
    ("12:08:45", "ERROR", "Rate limit exceeded for API key prod-key-123"),
]

_LOG_TEMPLATES = [
    ("INFO", "Health check passed [{ms} ms]"),
    ("DEBUG", "Cache hit ratio: {pct:.1f}%"),
    ("WARN", "Response time {ms} ms exceeds threshold"),
    ("ERROR", "Connection reset by peer: {host}"),
    ("INFO", "Processed {n} requests in the last minute"),
    ("DEBUG", "Worker {id} idle"),
    ("INFO", "Scheduled job completed in {ms} ms"),
    ("WARN", "Queue depth reached {n}"),
    ("INFO", "Config reload successful"),
    ("DEBUG", "GC collected {n} objects"),
]

_HOSTS = ["db-01:5432", "cache-01:6379", "api.example.com", "10.0.0.5"]

# ── Colour helpers ────────────────────────────────────────────────────────────


def status_color(status: str) -> Color:
    return Color.green() if status == "Running" else Color.red()


def cpu_color(pct: float) -> Color:
    if pct < 40:
        return Color.green()
    if pct < 70:
        return Color.yellow()
    return Color.red()


def log_level_color(level: str) -> Color:
    return {
        "ERROR": Color.red(),
        "WARN": Color.yellow(),
        "INFO": Color.green(),
        "DEBUG": Color.cyan(),
    }.get(level, Color.dark_gray())


# ── Status-bar spans helper ───────────────────────────────────────────────────


def _help_spans(pairs: list[tuple[str, str]]) -> list[Span]:
    spans: list[Span] = [Span("  ")]
    for key, desc in pairs:
        spans.append(Span(key, Style().fg(Color.yellow()).bold()))
        spans.append(Span(f":{desc}   ", Style().fg(Color.dark_gray())))
    return spans


# ── Main ──────────────────────────────────────────────────────────────────────


def main() -> None:
    tab_index = 0
    list_state = ListState()
    list_state.select(0)
    table_state = TableState()
    table_state.select(0)
    log_state = ListState()
    log_state.select(len(LOG_ENTRIES) - 1)
    log_follow = True  # auto-scroll to newest entry
    tick = 0

    with Terminal() as term:
        term.hide_cursor()

        while True:
            t = tick
            ti = tab_index

            # ── Clamp Overview selection ──────────────────────────────────────
            srv_sel = list_state.selected or 0
            srv_sel = max(0, min(srv_sel, len(SERVERS) - 1))
            srv = SERVERS[srv_sel]

            # ── Log follow ────────────────────────────────────────────────────
            if log_follow and LOG_ENTRIES:
                log_state.select(len(LOG_ENTRIES) - 1)

            # ── Clamp log selection ───────────────────────────────────────────
            lsel = log_state.selected or 0
            lsel = max(0, min(lsel, max(0, len(LOG_ENTRIES) - 1)))

            # ── Pre-compute log stats ─────────────────────────────────────────
            log_counts: dict[str, int] = {"ERROR": 0, "WARN": 0, "INFO": 0, "DEBUG": 0}
            for _, lvl, _ in LOG_ENTRIES:
                if lvl in log_counts:
                    log_counts[lvl] += 1

            lf = log_follow
            lc = dict(log_counts)

            # ─────────────────────────────────────────────────────────────────
            def ui(
                frame: Frame,
                _t: int = t,
                _ti: int = ti,
                _srv: Server = srv,
                _lf: bool = lf,
                _lc: dict[str, int] = lc,
            ) -> None:
                area = frame.area

                # Dynamic bottom-panel height:
                #   Tab 0  → 3 rows (single Gauge)
                #   Tab 1  → 5 rows (log stats panel)
                bottom_h = 3 if _ti == 0 else 5

                outer = (
                    Layout()
                    .direction(Direction.Vertical)
                    .constraints(
                        [
                            Constraint.length(3),  # tabs header
                            Constraint.fill(1),  # main body
                            Constraint.length(bottom_h),  # bottom panel
                            Constraint.length(1),  # status bar
                        ]
                    )
                    .split(area)
                )

                # ── Tabs header ───────────────────────────────────────────────
                frame.render_widget(
                    Tabs(["Overview", "Logs"])
                    .select(_ti)
                    .block(
                        Block()
                        .bordered()
                        .title(f" pyratatui Dashboard  ·  {time.strftime('%H:%M:%S')} ")
                    )
                    .highlight_style(Style().fg(Color.cyan()).bold())
                    .style(Style().fg(Color.dark_gray())),
                    outer[0],
                )

                # ── Status bar (tab-aware) ────────────────────────────────────
                help_map = {
                    0: [("↑/↓", "Navigate"), ("Tab", "Switch tab"), ("q", "Quit")],
                    1: [
                        ("↑/↓", "Scroll"),
                        ("Home", "Oldest"),
                        ("End", "Latest"),
                        ("Tab", "Switch tab"),
                        ("q", "Quit"),
                    ],
                }
                frame.render_widget(
                    Paragraph(Text([Line(_help_spans(help_map[_ti]))])),
                    outer[3],
                )

                # =============================================================
                # TAB 0 — Overview
                # =============================================================
                if _ti == 0:
                    body = (
                        Layout()
                        .direction(Direction.Horizontal)
                        .constraints(
                            [
                                Constraint.percentage(35),
                                Constraint.fill(1),
                            ]
                        )
                        .split(outer[1])
                    )

                    left_panels = (
                        Layout()
                        .direction(Direction.Vertical)
                        .constraints([Constraint.fill(1), Constraint.length(8)])
                        .split(body[0])
                    )

                    # Server list
                    srv_items = [
                        ListItem(
                            f"  {'[+]' if s['status'] == 'Running' else '[-]'}"
                            f"  {s['name']:<10s}  CPU:{s['cpu']:3d}%",
                            Style().fg(status_color(s["status"])),
                        )
                        for s in SERVERS
                    ]
                    frame.render_stateful_list(
                        List(srv_items)
                        .block(
                            Block().bordered().title(" Servers ").border_type(BorderType.Rounded)
                        )
                        .highlight_style(Style().fg(Color.yellow()).bold())
                        .highlight_symbol("▶ "),
                        left_panels[0],
                        list_state,
                    )

                    # Sparkline
                    history = CPU_HISTORY[-left_panels[1].width :]
                    frame.render_widget(
                        Sparkline()
                        .data(history)
                        .max(100)
                        .style(Style().fg(cpu_color(_srv["cpu"])))
                        .block(Block().bordered().title(f" CPU History — {_srv['name']} ")),
                        left_panels[1],
                    )

                    right_panels = (
                        Layout()
                        .direction(Direction.Vertical)
                        .constraints([Constraint.fill(1), Constraint.length(10)])
                        .split(body[1])
                    )

                    # Process table (server summary)
                    hdr = Row(
                        [
                            Cell("Server").style(Style().fg(Color.yellow()).bold()),
                            Cell("CPU").style(Style().fg(Color.yellow()).bold()),
                            Cell("Memory").style(Style().fg(Color.yellow()).bold()),
                            Cell("Status").style(Style().fg(Color.yellow()).bold()),
                        ]
                    )
                    rows = [
                        Row(
                            [
                                Cell(s["name"]),
                                Cell(f"{s['cpu']}%").style(Style().fg(cpu_color(s["cpu"]))),
                                Cell(f"{s['mem']}%"),
                                Cell(s["status"]).style(Style().fg(status_color(s["status"]))),
                            ]
                        )
                        for s in SERVERS
                    ]
                    frame.render_stateful_table(
                        Table(
                            rows,
                            column_widths=[
                                Constraint.fill(1),
                                Constraint.length(7),
                                Constraint.length(9),
                                Constraint.length(10),
                            ],
                            header=hdr,
                        )
                        .block(
                            Block()
                            .bordered()
                            .title(" Server Summary ")
                            .border_type(BorderType.Rounded)
                        )
                        .highlight_style(Style().fg(Color.cyan()).bold()),
                        right_panels[0],
                        table_state,
                    )

                    # Bar chart
                    bars = [
                        Bar(_srv2["cpu"], _srv2["name"][:6]).style(
                            Style().fg(cpu_color(_srv2["cpu"]))
                        )
                        for _srv2 in SERVERS
                    ]
                    frame.render_widget(
                        BarChart()
                        .data(BarGroup(bars, "CPU %"))
                        .bar_width(4)
                        .bar_gap(1)
                        .max(100)
                        .value_style(Style().fg(Color.white()).bold())
                        .label_style(Style().fg(Color.dark_gray()))
                        .block(Block().bordered().title(" CPU Overview ")),
                        right_panels[1],
                    )

                    # Bottom gauge — selected server CPU
                    frame.render_widget(
                        Gauge()
                        .percent(_srv["cpu"])
                        .label(f"{_srv['name']}  CPU: {_srv['cpu']}%   MEM: {_srv['mem']}%")
                        .style(Style().fg(cpu_color(_srv["cpu"])))
                        .gauge_style(Style().fg(Color.dark_gray()))
                        .block(Block().bordered().title(" Selected Server ")),
                        outer[2],
                    )

                # =============================================================
                # TAB 1 — Logs
                # =============================================================
                else:
                    # Colour-coded log list
                    log_items = [
                        ListItem(
                            f"  {ts}  [{lvl:<5s}]  {msg}",
                            Style().fg(log_level_color(lvl)),
                        )
                        for ts, lvl, msg in LOG_ENTRIES
                    ]
                    follow_tag = " [live] " if _lf else " [paused] "
                    frame.render_stateful_list(
                        List(log_items)
                        .block(
                            Block()
                            .bordered()
                            .title(f" Logs  ({len(LOG_ENTRIES)} entries){follow_tag}")
                        )
                        .highlight_style(Style().fg(Color.white()).bold()),
                        outer[1],
                        log_state,
                    )

                    # Bottom stats panel
                    e = _lc.get("ERROR", 0)
                    w = _lc.get("WARN", 0)
                    i = _lc.get("INFO", 0)
                    d = _lc.get("DEBUG", 0)
                    stats_lines = [
                        Line(
                            [
                                Span("  Counts:  ", Style().fg(Color.dark_gray())),
                                Span(f"ERROR:{e} ", Style().fg(Color.red()).bold()),
                                Span(f"WARN:{w} ", Style().fg(Color.yellow()).bold()),
                                Span(f"INFO:{i} ", Style().fg(Color.green()).bold()),
                                Span(f"DEBUG:{d}", Style().fg(Color.cyan()).bold()),
                            ]
                        ),
                        Line(
                            [
                                Span("  Scroll:  ", Style().fg(Color.dark_gray())),
                                Span("↑/↓", Style().fg(Color.yellow()).bold()),
                                Span(" move   ", Style().fg(Color.dark_gray())),
                                Span("Home/End", Style().fg(Color.yellow()).bold()),
                                Span(" jump   ", Style().fg(Color.dark_gray())),
                                Span("auto-follow: ", Style().fg(Color.dark_gray())),
                                Span(
                                    "ON " if _lf else "OFF",
                                    Style().fg(Color.green() if _lf else Color.yellow()).bold(),
                                ),
                                Span(
                                    " (scroll up to pause)",
                                    Style().fg(Color.dark_gray()),
                                ),
                            ]
                        ),
                        Line(
                            [
                                Span("  Total:   ", Style().fg(Color.dark_gray())),
                                Span(str(len(LOG_ENTRIES)), Style().fg(Color.white())),
                                Span(" entries", Style().fg(Color.dark_gray())),
                            ]
                        ),
                    ]
                    frame.render_widget(
                        Paragraph(Text(stats_lines)).block(
                            Block().bordered().title(" Log Statistics ")
                        ),
                        outer[2],
                    )

            # ── Draw ──────────────────────────────────────────────────────────
            term.draw(ui)
            tick += 1

            # ── Animate server CPU ────────────────────────────────────────────
            for s in SERVERS:
                s["cpu"] = max(0, min(100, s["cpu"] + (tick % 3 - 1) * 2))
            CPU_HISTORY.append(SERVERS[0]["cpu"])
            if len(CPU_HISTORY) > 100:
                CPU_HISTORY.pop(0)

            # ── Generate a new log entry every ~2 s ───────────────────────────
            if tick % 25 == 0:
                tmpl_level, tmpl_msg = random.choice(_LOG_TEMPLATES)
                msg = tmpl_msg.format(
                    ms=random.randint(10, 2000),
                    pct=random.uniform(50, 95),
                    host=random.choice(_HOSTS),
                    n=random.randint(10, 500),
                    id=random.randint(1, 8),
                )
                LOG_ENTRIES.append((time.strftime("%H:%M:%S"), tmpl_level, msg))
                if len(LOG_ENTRIES) > 500:
                    LOG_ENTRIES.pop(0)
                    # keep log_state index valid
                    lsel2 = log_state.selected or 0
                    if lsel2 > 0:
                        log_state.select(lsel2 - 1)

            # ── Keyboard events ───────────────────────────────────────────────
            ev = term.poll_event(timeout_ms=80)
            if ev:
                if ev.code == "q" or (ev.code == "c" and ev.ctrl):
                    break

                elif ev.code == "Tab":
                    tab_index = (tab_index + 1) % 2

                elif tab_index == 0:
                    # Overview: navigate server list + table in sync
                    if ev.code == "Down":
                        new = min(srv_sel + 1, len(SERVERS) - 1)
                        list_state.select(new)
                        table_state.select(new)
                    elif ev.code == "Up":
                        new = max(srv_sel - 1, 0)
                        list_state.select(new)
                        table_state.select(new)

                elif tab_index == 1:
                    # Logs: scroll + follow mode
                    cur = log_state.selected or 0
                    if ev.code == "Down":
                        new = min(cur + 1, len(LOG_ENTRIES) - 1)
                        log_state.select(new)
                        log_follow = new >= len(LOG_ENTRIES) - 1
                    elif ev.code == "Up":
                        log_state.select(max(cur - 1, 0))
                        log_follow = False  # scrolled up → pause follow
                    elif ev.code == "End":
                        log_state.select(len(LOG_ENTRIES) - 1)
                        log_follow = True
                    elif ev.code == "Home":
                        log_state.select(0)
                        log_follow = False

        term.show_cursor()


if __name__ == "__main__":
    main()
