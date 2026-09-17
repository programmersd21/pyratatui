//! Python bindings for interactive prompt widgets.
//!
//! Provides `TextPrompt` and `PasswordPrompt` widgets with readline-style
//! key bindings and a clean stateful API.
//!
//! # Quick-start
//! ```python
//! import time
//! from pyratatui import (
//!     Terminal, Layout, Constraint, Direction,
//!     TextState, TextPrompt, PasswordPrompt, PromptStatus,
//! )
//!
//! username = TextState()
//! username.focus()
//!
//! with Terminal() as term:
//!     term.hide_cursor()
//!     while username.is_pending():
//!         def ui(frame, _s=username):
//!             frame.render_text_prompt(
//!                 TextPrompt("Username: "),
//!                 frame.area,
//!                 _s,
//!             )
//!         term.draw(ui)
//!         ev = term.poll_event(timeout_ms=50)
//!         if ev:
//!             username.handle_key(ev)
//!
//! if username.is_complete():
//!     print(f"Hello, {username.value()}!")
//! ```
//!
//! # Blocking convenience helpers
//! ```python
//! from pyratatui import prompt_text, prompt_password
//!
//! name   = prompt_text("Name: ")
//! passwd = prompt_password("Password: ")
//! ```

use pyo3::prelude::*;
use ratatui::{
    Frame as RFrame,
    layout::Rect as RRect,
    style::{Color as RColor, Modifier as RModifier, Style as RStyle},
    text::{Line as RLine, Span as RSpan, Text as RText},
    widgets::Paragraph as RParagraph,
};

use crate::terminal::key_code_str;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use pyo3::exceptions::PyRuntimeError;
use ratatui::Terminal as RTerminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use std::io;

// ── PromptStatus ─────────────────────────────────────────────────────────────

/// Current lifecycle status of a prompt.
///
/// ```python
/// if state.status == PromptStatus.Complete:
///     use_value(state.value())
/// ```
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum PromptStatus {
    /// Input is still in progress.
    Pending,
    /// User pressed Enter — value is available via `state.value()`.
    Complete,
    /// User pressed Esc or Ctrl+C — input was abandoned.
    Aborted,
}

#[pymethods]
impl PromptStatus {
    fn __repr__(&self) -> &'static str {
        match self {
            PromptStatus::Pending => "PromptStatus.Pending",
            PromptStatus::Complete => "PromptStatus.Complete",
            PromptStatus::Aborted => "PromptStatus.Aborted",
        }
    }
}

// ── TextRenderStyle ──────────────────────────────────────────────────────────

/// Controls how input characters are displayed inside a `TextPrompt`.
///
/// ```python
/// prompt = TextPrompt("Token: ", TextRenderStyle.Password)
/// ```
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum TextRenderStyle {
    /// Display characters exactly as typed (default).
    Normal,
    /// Replace every character with `*`.
    Password,
    /// Show nothing — input is entirely hidden.
    Invisible,
}

// ── TextState ────────────────────────────────────────────────────────────────

/// Mutable state owned by a [`TextPrompt`] or [`PasswordPrompt`].
///
/// Stores the current input buffer, cursor position, focus flag, and
/// completion status.  Pass an instance of this class to both the render
/// method and the event handler on each frame.
///
/// # Key bindings (readline/emacs style)
///
/// | Key            | Action                            |
/// |----------------|-----------------------------------|
/// | Enter          | Complete the prompt               |
/// | Esc / Ctrl+C   | Abort the prompt                  |
/// | Backspace      | Delete character before cursor    |
/// | Delete         | Delete character at cursor        |
/// | Left / Ctrl+B  | Move cursor left                  |
/// | Right / Ctrl+F | Move cursor right                 |
/// | Home / Ctrl+A  | Move cursor to beginning          |
/// | End / Ctrl+E   | Move cursor to end                |
/// | Ctrl+K         | Delete to end of line             |
/// | Ctrl+U         | Delete entire line                |
///
/// ```python
/// state = TextState()
/// state.focus()
/// state.handle_key(key_event)
/// if state.is_complete():
///     print(state.value())
/// ```
#[pyclass(module = "pyratatui", unsendable)]
#[derive(Debug)]
pub struct TextState {
    pub(crate) chars: Vec<char>,
    pub(crate) cursor: usize,
    pub(crate) focused: bool,
    pub(crate) status: PromptStatus,
}

#[pymethods]
impl TextState {
    /// Create a new `TextState`, optionally pre-filled with `initial`.
    #[new]
    #[pyo3(signature = (initial = ""))]
    pub fn new(initial: &str) -> Self {
        let chars: Vec<char> = initial.chars().collect();
        let cursor = chars.len();
        Self {
            chars,
            cursor,
            focused: false,
            status: PromptStatus::Pending,
        }
    }

    /// Give keyboard focus to this prompt.
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Remove keyboard focus from this prompt.
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Whether this prompt currently holds focus.
    #[getter]
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Return the current text buffer as a `str`.
    pub fn value(&self) -> String {
        self.chars.iter().collect()
    }

    /// Replace the text buffer with `v`; cursor moves to end.
    pub fn set_value(&mut self, v: &str) {
        self.chars = v.chars().collect();
        self.cursor = self.chars.len();
    }

    /// Erase the input buffer and reset cursor to 0.
    pub fn clear_input(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    /// Current cursor position as a 0-based character index.
    #[getter]
    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }

    /// The current [`PromptStatus`].
    #[getter]
    pub fn status(&self) -> PromptStatus {
        self.status.clone()
    }

    /// `True` while the prompt is still waiting for input.
    pub fn is_pending(&self) -> bool {
        self.status == PromptStatus::Pending
    }

    /// `True` after the user pressed Enter.
    pub fn is_complete(&self) -> bool {
        self.status == PromptStatus::Complete
    }

    /// `True` after the user pressed Esc or Ctrl+C.
    pub fn is_aborted(&self) -> bool {
        self.status == PromptStatus::Aborted
    }

    /// Reset the status to `Pending` without clearing the input buffer.
    pub fn reset_status(&mut self) {
        self.status = PromptStatus::Pending;
    }

    /// Clear the input buffer and reset status to `Pending`.
    pub fn reset(&mut self) {
        self.chars.clear();
        self.cursor = 0;
        self.status = PromptStatus::Pending;
    }

    /// Handle a key event using readline-style bindings.
    ///
    /// Accepts any object with `code: str`, `ctrl: bool`, `alt: bool`
    /// attributes — compatible with `pyratatui.KeyEvent`.
    ///
    /// Returns `True` if the event was consumed by this prompt.
    pub fn handle_key(&mut self, ev: &Bound<'_, PyAny>) -> PyResult<bool> {
        if self.status != PromptStatus::Pending {
            return Ok(false);
        }

        let code: String = ev.getattr("code")?.extract()?;
        let ctrl: bool = ev.getattr("ctrl")?.extract()?;
        let alt: bool = ev.getattr("alt")?.extract()?;

        Ok(self.apply_key(&code, ctrl, alt))
    }

    fn __repr__(&self) -> String {
        format!(
            "TextState(value={:?}, cursor={}, focused={}, status={:?})",
            self.value(),
            self.cursor,
            self.focused,
            self.status,
        )
    }
}

impl TextState {
    /// Internal key dispatch — separated from Python glue so the blocking
    /// helper can call it without a `Bound<PyAny>`.
    pub(crate) fn apply_key(&mut self, code: &str, ctrl: bool, alt: bool) -> bool {
        if self.status != PromptStatus::Pending {
            return false;
        }

        // ── named keys ───────────────────────────────────────────────────────
        match code {
            "Enter" => {
                self.status = PromptStatus::Complete;
                return true;
            }
            "Esc" => {
                self.status = PromptStatus::Aborted;
                return true;
            }
            "Backspace" => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.chars.remove(self.cursor);
                }
                return true;
            }
            "Delete" => {
                if self.cursor < self.chars.len() {
                    self.chars.remove(self.cursor);
                }
                return true;
            }
            "Left" => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                return true;
            }
            "Right" => {
                if self.cursor < self.chars.len() {
                    self.cursor += 1;
                }
                return true;
            }
            "Home" => {
                self.cursor = 0;
                return true;
            }
            "End" => {
                self.cursor = self.chars.len();
                return true;
            }
            _ => {}
        }

        // ── Ctrl combos ───────────────────────────────────────────────────────
        if ctrl {
            match code {
                "a" => {
                    self.cursor = 0;
                    return true;
                }
                "b" => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                    return true;
                }
                "c" => {
                    self.status = PromptStatus::Aborted;
                    return true;
                }
                "e" => {
                    self.cursor = self.chars.len();
                    return true;
                }
                "f" => {
                    if self.cursor < self.chars.len() {
                        self.cursor += 1;
                    }
                    return true;
                }
                // Ctrl+H = backspace on many terminals
                "h" => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.chars.remove(self.cursor);
                    }
                    return true;
                }
                // Ctrl+K: kill to end of line
                "k" => {
                    self.chars.truncate(self.cursor);
                    return true;
                }
                // Ctrl+U: kill whole line
                "u" => {
                    self.chars.clear();
                    self.cursor = 0;
                    return true;
                }
                // Ctrl+W: delete word before cursor (simple whitespace-based)
                "w" => {
                    while self.cursor > 0 && self.chars.get(self.cursor - 1) == Some(&' ') {
                        self.cursor -= 1;
                        self.chars.remove(self.cursor);
                    }
                    while self.cursor > 0 && self.chars.get(self.cursor - 1) != Some(&' ') {
                        self.cursor -= 1;
                        self.chars.remove(self.cursor);
                    }
                    return true;
                }
                _ => return false,
            }
        }

        // ── Printable single characters ───────────────────────────────────────
        let mut chars = code.chars();
        match (alt, chars.next(), chars.next()) {
            (false, Some(ch), None) if !ch.is_control() => {
                self.chars.insert(self.cursor, ch);
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }
}

// ── Shared render helper ──────────────────────────────────────────────────────

/// Construct a [`Paragraph`] widget that displays the prompt.
///
/// The cursor is rendered as a highlighted block when `state.is_focused()`.
fn build_paragraph(label: &str, display: &str, state: &TextState) -> RParagraph<'static> {
    let label_style = RStyle::default()
        .fg(RColor::Cyan)
        .add_modifier(RModifier::BOLD);
    let text_style = RStyle::default().fg(RColor::White);
    let cursor_style = RStyle::default()
        .fg(RColor::Black)
        .bg(RColor::White)
        .add_modifier(RModifier::BOLD);

    let display_chars: Vec<char> = display.chars().collect();
    let cursor_col = state.cursor.min(display_chars.len());

    let mut spans: Vec<RSpan<'static>> = vec![RSpan::styled(label.to_owned(), label_style)];

    if state.focused {
        let before: String = display_chars[..cursor_col].iter().collect();
        let at: String = if cursor_col < display_chars.len() {
            display_chars[cursor_col].to_string()
        } else {
            " ".to_string()
        };
        let after: String = if cursor_col < display_chars.len() {
            display_chars[cursor_col + 1..].iter().collect()
        } else {
            String::new()
        };

        if !before.is_empty() {
            spans.push(RSpan::styled(before, text_style));
        }
        spans.push(RSpan::styled(at, cursor_style));
        if !after.is_empty() {
            spans.push(RSpan::styled(after, text_style));
        }
    } else {
        spans.push(RSpan::styled(display.to_owned(), text_style));
    }

    RParagraph::new(RText::from(RLine::from(spans)))
}

// ── TextPrompt ───────────────────────────────────────────────────────────────

/// A single-line text input prompt widget.
///
/// Render it with `frame.render_text_prompt(prompt, area, state)`.
///
/// ```python
/// from pyratatui import TextPrompt, TextState
///
/// state = TextState()
/// state.focus()
///
/// def ui(frame):
///     frame.render_text_prompt(TextPrompt("Name: "), frame.area, state)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct TextPrompt {
    pub(crate) label: String,
    pub(crate) render_style: TextRenderStyle,
}

#[pymethods]
impl TextPrompt {
    /// Create a new `TextPrompt`.
    ///
    /// Args:
    ///     label: Prompt text displayed before the input area.
    ///     render_style: How to display typed characters (default: Normal).
    #[new]
    #[pyo3(signature = (label, render_style = None))]
    pub fn new(label: &str, render_style: Option<TextRenderStyle>) -> Self {
        Self {
            label: label.to_string(),
            render_style: render_style.unwrap_or(TextRenderStyle::Normal),
        }
    }

    /// Return a new `TextPrompt` with the given render style (builder).
    pub fn with_render_style(&self, style: TextRenderStyle) -> Self {
        Self {
            label: self.label.clone(),
            render_style: style,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "TextPrompt(label={:?}, style={:?})",
            self.label, self.render_style
        )
    }
}

impl TextPrompt {
    /// Render into a ratatui `Frame` (called by `Frame.render_text_prompt`).
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect, state: &TextState) {
        let display = match &self.render_style {
            TextRenderStyle::Normal => state.value(),
            TextRenderStyle::Password => "*".repeat(state.chars.len()),
            TextRenderStyle::Invisible => String::new(),
        };
        let p = build_paragraph(&self.label, &display, state);
        frame.render_widget(p, area);
    }
}

// ── PasswordPrompt ───────────────────────────────────────────────────────────

/// A password input prompt — identical to `TextPrompt` but always masks input.
///
/// ```python
/// from pyratatui import PasswordPrompt, TextState
///
/// state = TextState()
/// state.focus()
///
/// def ui(frame):
///     frame.render_password_prompt(PasswordPrompt("Password: "), frame.area, state)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct PasswordPrompt {
    pub(crate) label: String,
}

#[pymethods]
impl PasswordPrompt {
    /// Create a new `PasswordPrompt`.
    ///
    /// Args:
    ///     label: Prompt text displayed before the masked input.
    #[new]
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    fn __repr__(&self) -> String {
        format!("PasswordPrompt(label={:?})", self.label)
    }
}

impl PasswordPrompt {
    /// Render into a ratatui `Frame` (called by `Frame.render_password_prompt`).
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect, state: &TextState) {
        let display = "*".repeat(state.chars.len());
        let p = build_paragraph(&self.label, &display, state);
        frame.render_widget(p, area);
    }
}

// ── Blocking helpers ──────────────────────────────────────────────────────────

/// Run the blocking prompt event loop.
fn run_blocking(label: &str, style: TextRenderStyle) -> PyResult<Option<String>> {
    enable_raw_mode().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    execute!(io::stdout(), EnterAlternateScreen)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal =
        RTerminal::new(backend).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let mut state = TextState::new("");
    state.focus();
    let prompt = TextPrompt::new(label, Some(style));
    let owned_label = prompt.label.clone();

    let result: PyResult<Option<String>> = (|| {
        loop {
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    // Center the prompt in a three-row vertical layout.
                    let rows = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(45),
                            Constraint::Length(3),
                            Constraint::Min(0),
                        ])
                        .split(area);
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title_top(format!(" {} ", owned_label));
                    let inner = block.inner(rows[1]);
                    frame.render_widget(block, rows[1]);
                    prompt.render_raw(frame, inner, &state);
                })
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            if event::poll(std::time::Duration::from_millis(50))
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
                && let Ok(Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                })) = event::read()
            {
                let code_str = key_code_str(&code);
                let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                let alt = modifiers.contains(KeyModifiers::ALT);
                state.apply_key(&code_str, ctrl, alt);
            }

            if state.is_complete() {
                return Ok(Some(state.value()));
            }
            if state.is_aborted() {
                return Ok(None);
            }
        }
    })();

    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    result
}

/// Run a blocking text prompt and return the user's input.
///
/// Opens a minimal full-screen TUI, collects a line of text, then restores
/// the terminal.  Returns the entered string, or `None` if the user aborted
/// with Esc or Ctrl+C.
///
/// ```python
/// from pyratatui import prompt_text
///
/// name = prompt_text("Enter your name: ")
/// if name:
///     print(f"Hello, {name}!")
/// ```
#[pyfunction]
pub fn prompt_text(label: &str) -> PyResult<Option<String>> {
    run_blocking(label, TextRenderStyle::Normal)
}

/// Run a blocking password prompt and return the user's input.
///
/// Like `prompt_text` but each character is displayed as `*`.
/// Returns `None` if the user aborted.
///
/// ```python
/// from pyratatui import prompt_password
///
/// token = prompt_password("API token: ")
/// ```
#[pyfunction]
pub fn prompt_password(label: &str) -> PyResult<Option<String>> {
    run_blocking(label, TextRenderStyle::Password)
}

// ── Module registration ───────────────────────────────────────────────────────

pub fn register_prompts(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PromptStatus>()?;
    m.add_class::<TextRenderStyle>()?;
    m.add_class::<TextState>()?;
    m.add_class::<TextPrompt>()?;
    m.add_class::<PasswordPrompt>()?;
    m.add_function(wrap_pyfunction!(prompt_text, m)?)?;
    m.add_function(wrap_pyfunction!(prompt_password, m)?)?;
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn focused(text: &str) -> TextState {
        let mut state = TextState::new(text);
        state.focus();
        state
    }

    #[test]
    fn typing_inserts_at_cursor() {
        let mut s = focused("");
        assert!(s.apply_key("h", false, false));
        assert!(s.apply_key("i", false, false));
        assert_eq!(s.value(), "hi");
        assert_eq!(s.cursor_pos(), 2);
    }

    #[test]
    fn enter_completes_esc_aborts() {
        let mut s = focused("draft");
        assert!(s.is_pending());
        assert!(s.apply_key("Enter", false, false));
        assert!(s.is_complete());
        assert_eq!(s.value(), "draft");

        let mut s = focused("draft");
        assert!(s.apply_key("Esc", false, false));
        assert!(s.is_aborted());

        let mut s = focused("draft");
        assert!(s.apply_key("c", true, false));
        assert!(s.is_aborted());
    }

    #[test]
    fn editing_keys_move_and_delete() {
        let mut s = focused("abc");
        assert!(s.apply_key("Left", false, false));
        assert!(s.apply_key("Backspace", false, false));
        assert_eq!(s.value(), "ac");
        assert!(s.apply_key("Delete", false, false));
        assert_eq!(s.value(), "a");
        assert!(s.apply_key("Home", false, false));
        assert!(s.apply_key("End", false, false));
        assert_eq!(s.cursor_pos(), 1);
    }

    #[test]
    fn readline_ctrl_combos() {
        let mut s = focused("hello");
        assert!(s.apply_key("a", true, false));
        assert_eq!(s.cursor_pos(), 0);
        assert!(s.apply_key("e", true, false));
        assert_eq!(s.cursor_pos(), 5);
        assert!(s.apply_key("k", true, false));
        assert_eq!(s.value(), "hello");

        let mut s = focused("hello world");
        s.apply_key("a", true, false);
        s.apply_key("f", true, false);
        s.apply_key("f", true, false);
        assert!(s.apply_key("k", true, false));
        assert_eq!(s.value(), "he");

        let mut s = focused("hello");
        assert!(s.apply_key("u", true, false));
        assert_eq!(s.value(), "");

        let mut s = focused("hello world  ");
        assert!(s.apply_key("w", true, false));
        assert_eq!(s.value(), "hello ");
        // At a word boundary the next rubout takes the space and the word,
        // matching readline's unix-word-rubout.
        assert!(s.apply_key("w", true, false));
        assert_eq!(s.value(), "");
    }

    #[test]
    fn rejected_keys_leave_state_untouched() {
        let mut s = focused("ab");
        assert!(!s.apply_key("x", false, true)); // alt+printable
        assert!(!s.apply_key("q", true, false)); // unbound ctrl combo
        assert!(!s.apply_key("\u{7}", false, false)); // control character
        assert_eq!(s.value(), "ab");
        assert_eq!(s.cursor_pos(), 2);
    }

    #[test]
    fn settled_prompts_ignore_keys() {
        let mut s = focused("done");
        s.apply_key("Enter", false, false);
        assert!(!s.apply_key("x", false, false));
        assert_eq!(s.value(), "done");
    }
}
