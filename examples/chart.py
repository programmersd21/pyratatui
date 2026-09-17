"""Chart widget demo with line, scatter, and bar datasets."""

from __future__ import annotations

from pyratatui import (
    Axis,
    Block,
    Chart,
    Color,
    Dataset,
    GraphType,
    Marker,
    Style,
    Terminal,
)


def main() -> None:
    line_points = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 5.0), (4.0, 4.0)]
    scatter_points = [(0.4, 1.2), (1.6, 2.6), (2.7, 2.1), (3.8, 4.1)]
    bar_points = [(0.0, 1.4), (1.0, 1.0), (2.0, 2.8), (3.0, 2.0), (4.0, 3.3)]

    datasets = [
        Dataset(line_points)
        .name("Line")
        .graph_type(GraphType.Line)
        .marker(Marker.Braille)
        .style(Style().fg(Color.cyan())),
        Dataset(scatter_points)
        .name("Scatter")
        .graph_type(GraphType.Scatter)
        .marker(Marker.Dot)
        .style(Style().fg(Color.yellow())),
        Dataset(bar_points)
        .name("Bars")
        .graph_type(GraphType.Bar)
        .marker(Marker.Bar)
        .style(Style().fg(Color.green())),
    ]

    chart = (
        Chart(datasets)
        .block(Block().bordered().title(" Multi-Series Chart "))
        .x_axis(Axis().title("X").bounds(0.0, 4.0).labels(["0", "2", "4"]))
        .y_axis(Axis().title("Y").bounds(0.0, 6.0).labels(["0", "3", "6"]))
    )

    with Terminal() as term:
        while True:
            term.draw(lambda frame: frame.render_widget(chart, frame.area))
            event = term.poll_event(timeout_ms=100)
            if event and event.code in {"q", "Esc"}:
                break


if __name__ == "__main__":
    main()
