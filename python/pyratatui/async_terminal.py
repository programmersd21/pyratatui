"""
async_terminal.py — AsyncTerminal: asyncio-friendly wrapper around Terminal.

Also exports ``run_app`` and ``run_app_async`` convenience helpers so that
``__init__.py`` can do a single import from this module.

Threading model
---------------
``Terminal`` (and ``Frame``, ``Effect``, ``EffectManager``) are PyO3
``unsendable`` objects.  PyO3 records the OS thread ID at construction time
and **panics** if any method is called from a different thread::

    pyo3_runtime.PanicException: assertion `left == right` failed:
    pyratatui::terminal::Terminal is unsendable, but sent to another thread

Python's default asyncio event-loop runs entirely on **one thread** (the
thread that called ``asyncio.run()``).  As long as we never use
``loop.run_in_executor`` (or ``asyncio.to_thread``) with Terminal methods,
we are safe.

The previous implementation used ``run_in_executor`` to avoid blocking the
event loop during ``poll_event``.  The fix is dead simple:

* Use **non-blocking** ``poll_event(timeout_ms=0)`` on the main thread.
* Use ``await asyncio.sleep(frame_interval)`` to yield to the event loop
  between frames — this lets background tasks (metric updates, timers, …)
  run without ever touching Terminal from another thread.
"""

from __future__ import annotations

import asyncio
import contextlib
import time
from collections.abc import AsyncIterator, Callable

from ._pyratatui import Frame
from ._pyratatui import PyKeyEvent as KeyEvent
from ._pyratatui import Terminal


class AsyncTerminal:
    """
    Asyncio-compatible terminal driver.

    All calls to ``Terminal`` happen on the **asyncio event-loop thread**.
    ``run_in_executor`` is intentionally absent — it would send Terminal to
    a thread-pool thread and trigger PyO3's unsendable panic.

    ```python
    async with AsyncTerminal() as term:
        async for event in term.events(fps=30):
            term.draw(lambda f: f.render_widget(..., f.area))
            if event and event.code == "q":
                break
    ```
    """

    def __init__(self, inline_height: int | None = None) -> None:
        """Wrap a terminal driver.

        Args:
            inline_height: Draw in a block of that many lines inside the normal
                buffer instead of taking over the screen. ``None`` (the default)
                uses the alternate screen, as before.
        """
        self._term: Terminal | None = None
        self._inline_height = inline_height

    # ── Context manager ──────────────────────────────────────────────────────

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

    # ── Drawing ──────────────────────────────────────────────────────────────

    def draw(self, draw_fn: Callable[[Frame], None]) -> None:
        """Render one frame on the main event-loop thread."""
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active — use `async with`")
        self._term.draw(draw_fn)

    # ── Events ───────────────────────────────────────────────────────────────

    async def poll_event(self, timeout_ms: int = 0) -> KeyEvent | None:
        """
        Poll for a keyboard event without leaving the main thread.

        Yields to the event loop via ``await asyncio.sleep(0)`` then does a
        non-blocking poll.  The ``timeout_ms`` arg is kept for API compat but
        frame pacing is handled by ``events()``.
        """
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")
        # Yield control so other coroutines run, then poll on the same thread.
        # NEVER use run_in_executor here — that sends Terminal to another thread.
        await asyncio.sleep(0)
        return self._term.poll_event(0)

    async def events(
        self,
        fps: float = 30.0,
        *,
        stop_on_quit: bool = False,
    ) -> AsyncIterator[KeyEvent | None]:
        """
        Async generator yielding one tick per frame at the requested rate.

        Each tick:
          1. Non-blocking event poll on the main thread.
          2. Yield the event (or ``None``).
          3. Sleep the remainder of the frame interval — gives background
             coroutines CPU time without touching Terminal from another thread.

        Args:
            fps:           Target frames per second.
            stop_on_quit:  Auto-stop on ``q`` or Ctrl-C when explicitly enabled.
        """
        if self._term is None:
            raise RuntimeError("AsyncTerminal is not active")

        frame_interval = 1.0 / max(1.0, fps)

        while True:
            t_start = time.monotonic()

            # Non-blocking poll — always on the main event-loop thread.
            ev = self._term.poll_event(0)

            if (
                stop_on_quit
                and ev is not None
                and (ev.code == "q" or (ev.code == "c" and ev.ctrl))
            ):
                return

            yield ev

            # Sleep for the rest of the frame interval.
            elapsed = time.monotonic() - t_start
            sleep_s = max(0.0, frame_interval - elapsed)
            await asyncio.sleep(sleep_s if sleep_s > 0 else 0)

    # ── Utilities ────────────────────────────────────────────────────────────

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


# ── Convenience helpers ───────────────────────────────────────────────────────


def run_app(
    ui_fn: Callable[[Frame], None],
    *,
    fps: float = 30.0,
    on_key: Callable[[object], bool] | None = None,
) -> None:
    """
    Run a simple synchronous TUI application.

    Args:
        ui_fn:  A callable that receives ``Frame`` and renders the UI each tick.
        fps:    Target frames per second.
        on_key: Optional callback receiving a ``KeyEvent``. Return ``True`` to
                quit. ``run_app`` no longer handles ``q`` or Ctrl-C for you;
                implement the desired quit logic in ``on_key``.

    Example::

        from pyratatui import run_app, Paragraph

        def ui(frame):
            frame.render_widget(
                Paragraph.from_string("Hello! Press q to quit."),
                frame.area,
            )

        run_app(ui, on_key=lambda ev: ev.code == "q")
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
    on_key: Callable[[object], bool] | None = None,
) -> None:
    """
    Run a simple async TUI application.

    Args:
        ui_fn:  A callable that receives ``Frame`` and renders the UI each tick.
        fps:    Target frames per second.
        on_key: Optional callback receiving a ``KeyEvent``. Return ``True`` to
                quit. Provide ``on_key`` to explicitly handle quitting; ``run_app_async``
                no longer exits automatically when ``q`` or Ctrl-C are pressed.

    Example::

        import asyncio
        from pyratatui import run_app_async, Paragraph

        async def main():
            tick = 0
            def ui(frame):
                frame.render_widget(
                    Paragraph.from_string(f"Async tick: {tick}"),
                    frame.area,
                )

            await run_app_async(ui, on_key=lambda ev: ev.code == "q")

        asyncio.run(main())
    """
    async with AsyncTerminal() as term:
        async for ev in term.events(fps=fps, stop_on_quit=False):
            term.draw(ui_fn)
            if ev is not None and on_key is not None and on_key(ev):
                break
