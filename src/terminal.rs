//! `Terminal`, `Frame`, and key events.
//!
//! `Terminal` owns the crossterm backend and drives the render loop.
//! Each `draw` call hands the callback a short-lived `Frame`.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame as RFrame, Terminal as RTerminal, TerminalOptions, Viewport, backend::CrosstermBackend,
};

use crate::calendar::Monthly;
use crate::canvas::{Canvas, Map};
use crate::chart::Chart;
use crate::errors::{io_err_to_py, render_err_to_py};
use crate::layout::Rect;
use crate::prompts::{PasswordPrompt, TextPrompt, TextState};
use crate::widgets::{
    BarChart, Block, Clear, Gauge, LineGauge, List, ListState, Paragraph, RatatuiMascot, Scrollbar,
    ScrollbarState, Sparkline, Table, TableState, Tabs,
};

/// A keyboard event from `Terminal.poll_event`.
///
/// `code` is the key name: a printable character (`"a"`, `"Z"`, `"5"`),
/// or one of `Enter`, `Esc`, `Backspace`, `Delete`, `Tab`, `BackTab`,
/// `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`,
/// `Insert`, `F1`–`F12`, `Null`, `Unknown`. The `ctrl`/`alt`/`shift`
/// flags report held modifiers.
#[pyclass(module = "pyratatui", name = "KeyEvent", from_py_object)]
#[derive(Clone, Debug)]
pub struct PyKeyEvent {
    /// Key code as a string (e.g. `"a"`, `"Enter"`, `"Esc"`, `"Up"`, `"F1"`).
    #[pyo3(get)]
    pub code: String,
    #[pyo3(get)]
    pub ctrl: bool,
    #[pyo3(get)]
    pub alt: bool,
    #[pyo3(get)]
    pub shift: bool,
}

pub(crate) fn key_code_str(kc: &KeyCode) -> String {
    match kc {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::BackTab => "BackTab".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        KeyCode::Insert => "Insert".into(),
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Null => "Null".into(),
        _ => "Unknown".into(),
    }
}

#[pymethods]
impl PyKeyEvent {
    fn __repr__(&self) -> String {
        format!(
            "KeyEvent(code={:?}, ctrl={}, alt={}, shift={})",
            self.code, self.ctrl, self.alt, self.shift
        )
    }
}

/// A single render frame, valid only inside the `draw` callback.
///
/// ```python
/// def ui(frame):
///     frame.render_widget(Paragraph.from_string("Hello!"), frame.area)
/// ```
#[pyclass(module = "pyratatui", unsendable)]
pub struct Frame {
    ptr: *mut RFrame<'static>,
}

// The pointer always refers to the frame ratatui lends us for the duration
// of the `draw` closure, and is never used outside it.
unsafe impl Send for Frame {}

impl Frame {
    fn get(&mut self) -> &mut RFrame<'static> {
        unsafe { &mut *self.ptr }
    }
}

#[pymethods]
impl Frame {
    /// The full terminal area available for this frame.
    #[getter]
    pub fn area(&mut self) -> Rect {
        Rect {
            inner: self.get().area(),
        }
    }

    /// Render a widget into the given area.
    pub fn render_widget(&mut self, widget: &Bound<'_, PyAny>, area: &Rect) -> PyResult<()> {
        let frame = self.get();
        let a = area.inner;

        macro_rules! try_render {
            ($($T:ty),*) => {{
                $(
                    if let Ok(w) = widget.extract::<PyRef<$T>>() {
                        w.render_raw(frame, a)?;
                        return Ok(());
                    }
                )*
            }};
        }

        try_render!(
            Block,
            Paragraph,
            Gauge,
            LineGauge,
            BarChart,
            Sparkline,
            Clear,
            Tabs,
            Monthly,
            RatatuiMascot,
            List,
            Table,
            Canvas,
            Map,
            Chart
        );

        Err(render_err_to_py(format!(
            "Unknown widget type: {}",
            widget
                .get_type()
                .qualname()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "?".to_string())
        )))
    }

    /// Render a `List` with mutable selection state.
    pub fn render_stateful_list(
        &mut self,
        widget: &List,
        area: &Rect,
        state: &mut ListState,
    ) -> PyResult<()> {
        self.get()
            .render_stateful_widget(widget.to_ratatui(), area.inner, &mut state.inner);
        Ok(())
    }

    /// Render a `Table` with mutable selection state.
    pub fn render_stateful_table(
        &mut self,
        widget: &Table,
        area: &Rect,
        state: &mut TableState,
    ) -> PyResult<()> {
        self.get()
            .render_stateful_widget(widget.to_ratatui(), area.inner, &mut state.inner);
        Ok(())
    }

    /// Render a `Scrollbar` with its scroll state.
    pub fn render_stateful_scrollbar(
        &mut self,
        widget: &Scrollbar,
        area: &Rect,
        state: &mut ScrollbarState,
    ) -> PyResult<()> {
        self.get()
            .render_stateful_widget(widget.to_ratatui(), area.inner, &mut state.inner);
        Ok(())
    }

    /// Render a `TextPrompt` with the given `TextState`.
    pub fn render_text_prompt(&mut self, prompt: &TextPrompt, area: &Rect, state: &TextState) {
        prompt.render_raw(self.get(), area.inner, state);
    }

    /// Render a `PasswordPrompt` with the given `TextState`.
    pub fn render_password_prompt(
        &mut self,
        prompt: &PasswordPrompt,
        area: &Rect,
        state: &TextState,
    ) {
        prompt.render_raw(self.get(), area.inner, state);
    }

    fn __repr__(&self) -> String {
        "Frame(<active>)".to_string()
    }
}

type RatTerminal = RTerminal<CrosstermBackend<Stdout>>;

/// The terminal driver.
///
/// Enters the alternate screen and owns the whole terminal by default; pass
/// `inline_height` to draw in a block of that many lines inside the normal
/// buffer instead, leaving scrollback in place.
///
/// `Terminal` must only be used from the thread that created it. Async
/// applications should use `AsyncTerminal` (see `pyratatui.app`) instead.
///
/// ```python
/// with Terminal() as term:
///     while True:
///         term.draw(ui)
///         ev = term.poll_event(timeout_ms=100)
///         if ev and ev.code == "q":
///             break
/// ```
#[pyclass(module = "pyratatui", unsendable)]
pub struct Terminal {
    inner: Option<RatTerminal>,
    entered: bool,
    inline_height: Option<u16>,
    alt_screen: bool,
}

impl Terminal {
    fn start(&mut self) -> PyResult<()> {
        enable_raw_mode().map_err(io_err_to_py)?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = match self.inline_height {
            Some(height) => {
                let options = TerminalOptions {
                    viewport: Viewport::Inline(height),
                };
                RTerminal::with_options(backend, options).map_err(io_err_to_py)?
            }
            None => {
                execute!(io::stdout(), EnterAlternateScreen).map_err(io_err_to_py)?;
                self.alt_screen = true;
                RTerminal::new(backend).map_err(io_err_to_py)?
            }
        };
        self.inner = Some(terminal);
        self.entered = true;
        Ok(())
    }

    fn get(&mut self) -> PyResult<&mut RatTerminal> {
        self.inner.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Terminal not initialised — use `with Terminal() as t:`")
        })
    }
}

#[pymethods]
impl Terminal {
    #[new]
    #[pyo3(signature = (inline_height=None))]
    pub fn new(inline_height: Option<u16>) -> Self {
        Self {
            inner: None,
            entered: false,
            inline_height,
            alt_screen: false,
        }
    }

    pub fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        slf.start()?;
        Ok(slf)
    }

    pub fn __exit__(
        &mut self,
        _exc_type: &Bound<'_, PyAny>,
        _exc_val: &Bound<'_, PyAny>,
        _exc_tb: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        self.restore()?;
        Ok(false)
    }

    pub fn restore(&mut self) -> PyResult<()> {
        if self.entered {
            if self.alt_screen {
                disable_raw_mode().map_err(io_err_to_py)?;
                execute!(io::stdout(), LeaveAlternateScreen).map_err(io_err_to_py)?;
                self.alt_screen = false;
            } else {
                // An inline viewport draws in the normal buffer: clear the
                // block it owns so what is printed next starts on a clean line.
                if let Some(term) = self.inner.as_mut() {
                    term.clear().map_err(io_err_to_py)?;
                    term.show_cursor().map_err(io_err_to_py)?;
                }
                disable_raw_mode().map_err(io_err_to_py)?;
            }
            self.entered = false;
        }
        Ok(())
    }

    pub fn draw(&mut self, draw_fn: &Bound<'_, PyAny>) -> PyResult<()> {
        let term = self.get()?;
        let mut err = None;

        term.draw(|frame| {
            // Widen the borrowed frame to 'static for the holder object.
            // The holder never outlives this closure: it is created and
            // dropped inside a single Python::attach call below.
            let py_frame = Frame {
                ptr: unsafe {
                    std::mem::transmute::<*mut RFrame<'_>, *mut RFrame<'static>>(
                        frame as *mut RFrame<'_>,
                    )
                },
            };
            Python::attach(|py| {
                if let Ok(obj) = Py::new(py, py_frame)
                    && let Err(e) = draw_fn.call1((obj,))
                {
                    err = Some(e);
                }
            });
        })
        .map_err(io_err_to_py)?;

        if let Some(e) = err { Err(e) } else { Ok(()) }
    }

    #[pyo3(signature = (timeout_ms=0))]
    pub fn poll_event(&self, timeout_ms: u64) -> PyResult<Option<PyKeyEvent>> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        if event::poll(timeout).map_err(io_err_to_py)?
            && let Event::Key(key) = event::read().map_err(io_err_to_py)?
            && key.kind == KeyEventKind::Press
        {
            return Ok(Some(PyKeyEvent {
                code: key_code_str(&key.code),
                ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                alt: key.modifiers.contains(KeyModifiers::ALT),
                shift: key.modifiers.contains(KeyModifiers::SHIFT),
            }));
        }
        Ok(None)
    }

    /// The current terminal area.
    pub fn area(&mut self) -> PyResult<Rect> {
        Ok(Rect {
            inner: self.get()?.size().map_err(io_err_to_py)?.into(),
        })
    }

    pub fn clear(&mut self) -> PyResult<()> {
        self.get()?.clear().map_err(io_err_to_py)
    }

    pub fn hide_cursor(&mut self) -> PyResult<()> {
        self.get()?.hide_cursor().map_err(io_err_to_py)
    }

    pub fn show_cursor(&mut self) -> PyResult<()> {
        self.get()?.show_cursor().map_err(io_err_to_py)
    }

    fn __repr__(&self) -> String {
        format!("Terminal(active={})", self.entered)
    }
}

pub fn register_terminal(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Terminal>()?;
    m.add_class::<Frame>()?;
    m.add_class::<PyKeyEvent>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn key_codes_have_stable_names() {
        let cases = [
            (KeyCode::Char('a'), "a"),
            (KeyCode::Char('Z'), "Z"),
            (KeyCode::Char('5'), "5"),
            (KeyCode::Enter, "Enter"),
            (KeyCode::Esc, "Esc"),
            (KeyCode::Backspace, "Backspace"),
            (KeyCode::Delete, "Delete"),
            (KeyCode::Tab, "Tab"),
            (KeyCode::BackTab, "BackTab"),
            (KeyCode::Up, "Up"),
            (KeyCode::Down, "Down"),
            (KeyCode::Left, "Left"),
            (KeyCode::Right, "Right"),
            (KeyCode::Home, "Home"),
            (KeyCode::End, "End"),
            (KeyCode::PageUp, "PageUp"),
            (KeyCode::PageDown, "PageDown"),
            (KeyCode::Insert, "Insert"),
            (KeyCode::F(1), "F1"),
            (KeyCode::F(12), "F12"),
            (KeyCode::Null, "Null"),
        ];
        for (code, expected) in cases {
            assert_eq!(key_code_str(&code), expected);
        }
    }
}
