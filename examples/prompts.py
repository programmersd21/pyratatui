"""
prompts.py — Interactive text prompt demo.

Demonstrates the stateful TextPrompt / TextState API:
  - TextState holds the mutable input value, cursor and status
  - TextPrompt renders the label + input field with live cursor
  - handle_key() processes readline-style key bindings
  - is_complete() / is_aborted() / value() query the result

Also shows prompt_text() — the one-liner blocking convenience helper.

Press Enter to submit, Esc to abort.
"""

import time

from pyratatui import (
    Block,
    Color,
    Constraint,
    Direction,
    Frame,
    Layout,
    Paragraph,
    Style,
    Terminal,
    TextPrompt,
    TextState,
)

# ── State ─────────────────────────────────────────────────────────────────────

state = TextState()
state.focus()
last_frame = time.monotonic()
result: str | None = None
aborted = False

HINT = (
    "  Readline key bindings:\n\n"
    "  Enter     — submit\n"
    "  Esc       — abort\n"
    "  Ctrl+U    — clear line\n"
    "  Ctrl+K    — kill to end\n"
    "  Ctrl+A/E  — start / end\n"
    "  ← / →    — move cursor\n"
    "  Backspace — delete left\n"
)

# ── Event loop ────────────────────────────────────────────────────────────────

with Terminal() as term:
    term.hide_cursor()

    while state.is_pending():
        now = time.monotonic()
        last_frame = now

        _state = state

        def ui(frame: Frame, _s: TextState = _state) -> None:
            area = frame.area

            # Split: hints on the left, prompt on the right.
            cols = (
                Layout()
                .direction(Direction.Horizontal)
                .constraints([Constraint.percentage(40), Constraint.fill(1)])
                .split(area)
            )

            frame.render_widget(
                Paragraph.from_string(HINT).block(
                    Block().bordered().title(" Key Bindings ").style(Style().fg(Color.gray()))
                ),
                cols[0],
            )

            # Prompt panel: split vertically to centre the input row.
            rows = (
                Layout()
                .direction(Direction.Vertical)
                .constraints(
                    [
                        Constraint.fill(1),
                        Constraint.length(1),
                        Constraint.fill(1),
                    ]
                )
                .split(cols[1])
            )

            frame.render_widget(
                Block().bordered().title(" Enter your name ").style(Style().fg(Color.cyan())),
                cols[1],
            )

            # Render the actual prompt into the middle row.
            frame.render_text_prompt(
                TextPrompt("Name: "),
                rows[1],
                _s,
            )

        term.draw(ui)

        ev = term.poll_event(timeout_ms=50)
        if ev:
            state.handle_key(ev)

    # Capture result before leaving alternate screen.
    if state.is_complete():
        result = state.value()
    else:
        aborted = True

    term.show_cursor()

# ── Display outcome ───────────────────────────────────────────────────────────

print()
if aborted:
    print("  Prompt aborted.")
elif result is not None:
    print(f"  Hello, {result}!")
print()
