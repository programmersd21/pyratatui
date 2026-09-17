"""tests for the inline viewport of Terminal and AsyncTerminal"""

import os
import sys
import textwrap

import pytest

from pyratatui import AsyncTerminal, Terminal

ALT_SCREEN_ON = b"\x1b[?1049h"
ALT_SCREEN_OFF = b"\x1b[?1049l"
CURSOR_REPORT = b"\x1b[6n"

APP = """
    import pyratatui as rt

    print("BEFORE-THE-APP")
    with rt.Terminal({height}) as term:
        for _ in range(2):
            term.draw(
                lambda frame: frame.render_widget(
                    rt.Paragraph.from_string("INSIDE-THE-APP"), frame.area
                )
            )
    print("AFTER-THE-APP")
"""

ASYNC_APP = """
    import asyncio
    import pyratatui as rt

    async def main():
        async with rt.AsyncTerminal(inline_height=4) as term:
            async for _ in term.events(fps=30):
                term.draw(
                    lambda frame: frame.render_widget(
                        rt.Paragraph.from_string("ASYNC-INLINE"), frame.area
                    )
                )
                break

    asyncio.run(main())
"""

posix_only = pytest.mark.skipif(os.name != "posix", reason="needs a pty")


def run_in_pty(source, rows=24, cols=80, timeout=20.0):
    """Run source in a child process on a pty and return everything it drew."""
    import fcntl
    import pty
    import select
    import struct
    import termios
    import time

    pid, fd = pty.fork()
    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        os.environ["PYTHONPATH"] = os.pathsep.join(sys.path)
        os.execv(sys.executable, [sys.executable, "-c", textwrap.dedent(source)])

    # A pty starts out 0x0, and a backend with no room to draw in fails
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
    os.set_blocking(fd, False)

    chunks = []
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.2)
        if not ready:
            if os.waitpid(pid, os.WNOHANG)[0]:
                break
            continue
        try:
            data = os.read(fd, 65536)
        except OSError:
            break
        if not data:
            break
        chunks.append(data)
        if CURSOR_REPORT in data:
            # An inline viewport asks where the cursor is before its first frame
            os.write(fd, b"\x1b[20;1R")

    os.close(fd)
    try:
        os.waitpid(pid, 0)
    except ChildProcessError:
        pass
    return b"".join(chunks)


class TestInlineHeightArgument:
    def test_terminal_takes_an_inline_height(self):
        assert "active=false" in repr(Terminal(inline_height=5))

    def test_terminal_still_defaults_to_fullscreen(self):
        assert "active=false" in repr(Terminal())

    def test_async_terminal_takes_an_inline_height(self):
        assert "active=False" in repr(AsyncTerminal(inline_height=5))

    def test_the_height_may_be_given_by_position(self):
        assert repr(Terminal(5)) == repr(Terminal(inline_height=5))


@pytest.fixture(scope="module")
def inline():
    return run_in_pty(APP.format(height="inline_height=5"))


@pytest.fixture(scope="module")
def fullscreen():
    return run_in_pty(APP.format(height=""))


@posix_only
class TestDrawingInline:
    def test_inline_stays_out_of_the_alternate_screen(self, inline):
        assert ALT_SCREEN_ON not in inline
        assert ALT_SCREEN_OFF not in inline

    def test_inline_draws_what_it_was_told_to(self, inline):
        assert b"INSIDE-THE-APP" in inline

    def test_inline_keeps_what_was_printed_around_it(self, inline):
        assert b"BEFORE-THE-APP" in inline
        assert b"AFTER-THE-APP" in inline

    def test_the_default_still_takes_over_the_screen(self, fullscreen):
        assert ALT_SCREEN_ON in fullscreen
        assert ALT_SCREEN_OFF in fullscreen

    def test_the_screen_it_took_over_is_given_back(self, fullscreen):
        assert fullscreen.index(ALT_SCREEN_ON) < fullscreen.index(ALT_SCREEN_OFF)

    def test_async_terminal_draws_inline_too(self):
        drawn = run_in_pty(ASYNC_APP)

        assert b"ASYNC-INLINE" in drawn
        assert ALT_SCREEN_ON not in drawn
