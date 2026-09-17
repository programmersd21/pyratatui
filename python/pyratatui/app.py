"""Async terminal driver and small application loop helpers."""

from __future__ import annotations

import asyncio
import contextlib
import time
from collections.abc import AsyncIterator, Callable

from ._pyratatui import Frame, KeyEvent, Terminal


class AsyncTerminal:
    """Asyncio-compatible wrapper around :class:`Terminal`.

    All calls into the native terminal happen on the asyncio event-loop
    thread. ``run_in_executor`` is deliberately never used: ``Terminal`` is
    a PyO3 ``unsendable`` object and calling it from a thread-pool thread
    panics.

    Polling is non-blocking; frame pacing comes from ``await
    asyncio.sleep`` between ticks, which also lets background coroutines run.

    ```python
    async with AsyncTerminal() as term:
        async for event in term.events(fps=30):
            term.draw(lambda f: f.render_widget(widget, f.area))
            if event and event.code == "q":
                break
    ```
    """

    def __init__(self, inline_height: int | None = None) -> None:
        """Wrap a terminal driver.

        Args:
            inline_height: Draw in a block of that many lines inside the
                normal buffer instead of taking over the screen. ``None``
                (the default) uses the alternate screen.
        """
        self._term: Terminal | None = None
        self._inline_height = inline_height

    async def __aenter__(self) -> AsyncTerminal:
        self._term = Terminal(self._inline_height)
        self._term.__enter__()
        return self

    async def __aexit__(self, *args: object) -> bool:
        if self._term is not None:
            with contextlib.suppress(Exception):
                self._term.restore()
            self._term = None
        return False

    def draw(self, draw_fn: Callable[[Frame], None]) -> None:
        """Render one frame on the event-loop thread."""
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active — use `async with`")
        self._term.draw(draw_fn)

    async def poll_event(self, timeout_ms: int = 0) -> KeyEvent | None:
        """Poll for a key event without leaving the event-loop thread."""
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        await asyncio.sleep(0)
        return self._term.poll_event(0)

    async def events(
        self,
        fps: float = 30.0,
        *,
        stop_on_quit: bool = False,
    ) -> AsyncIterator[KeyEvent | None]:
        """Yield one tick per frame at the requested rate.

        Each tick polls for an event (or yields ``None``) and then sleeps
        for the rest of the frame interval.

        Args:
            fps: Target frames per second.
            stop_on_quit: Stop iteration on ``q`` or Ctrl+C when enabled.
        """
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")

        frame_interval = 1.0 / max(1.0, fps)

        while True:
            start = time.monotonic()
            ev = self._term.poll_event(0)

            if stop_on_quit and ev is not None and (ev.code == "q" or (ev.code == "c" and ev.ctrl)):
                return

            yield ev

            elapsed = time.monotonic() - start
            await asyncio.sleep(max(0.0, frame_interval - elapsed))

    def area(self) -> object:
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        return self._term.area()

    def clear(self) -> None:
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        self._term.clear()

    def hide_cursor(self) -> None:
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        self._term.hide_cursor()

    def show_cursor(self) -> None:
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        self._term.show_cursor()

    def __repr__(self) -> str:
        return f"AsyncTerminal(active={self._term is not None})"


def run_app(
    ui_fn: Callable[[Frame], None],
    *,
    fps: float = 30.0,
    on_key: Callable[[KeyEvent], bool] | None = None,
) -> None:
    """Run a simple synchronous TUI application.

    Args:
        ui_fn: Renders the UI each tick; receives the current ``Frame``.
        fps: Target frames per second.
        on_key: Called with each key event; return ``True`` to quit.

    ```python
    from pyratatui import Paragraph, run_app

    def ui(frame):
        frame.render_widget(
            Paragraph.from_string("Hello! Press q to quit."),
            frame.area,
        )

    run_app(ui, on_key=lambda ev: ev.code == "q")
    ```
    """
    timeout_ms = max(1, int(1000 / fps))
    with Terminal() as term:
        while True:
            term.draw(ui_fn)
            ev = term.poll_event(timeout_ms=timeout_ms)
            if ev is not None and on_key is not None and on_key(ev):
                break


async def run_app_async(
    ui_fn: Callable[[Frame], None],
    *,
    fps: float = 30.0,
    on_key: Callable[[KeyEvent], bool] | None = None,
) -> None:
    """Run a simple async TUI application.

    Args:
        ui_fn: Renders the UI each tick; receives the current ``Frame``.
        fps: Target frames per second.
        on_key: Called with each key event; return ``True`` to quit.

    ```python
    import asyncio
    from pyratatui import Paragraph, run_app_async

    async def main():
        def ui(frame):
            frame.render_widget(Paragraph.from_string("Hi!"), frame.area)

        await run_app_async(ui, on_key=lambda ev: ev.code == "q")

    asyncio.run(main())
    ```
    """
    async with AsyncTerminal() as term:
        async for ev in term.events(fps=fps, stop_on_quit=False):
            term.draw(ui_fn)
            if ev is not None and on_key is not None and on_key(ev):
                break
