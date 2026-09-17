// src/terminal/mod.rs
//! Python bindings for `Terminal` and `Frame`.
//!
//! ratatui 0.30 breaking changes handled here:
//! - `Frame::size()` → `Frame::area()` (size() deprecated and removed)
//! - `Terminal::get_frame()` replaced by `Terminal::size()` returning Result<Rect, B::Error>
//! - Backend now requires an associated `Error` type and `clear_region` method
//!   (CrosstermBackend satisfies both automatically)

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend, Frame as RFrame, Terminal as RTerminal, TerminalOptions, Viewport,
};

use crate::bar_graph::BarGraph as PyBarGraph;
use crate::button_widget::Button as PyButton;
use crate::canvas_widget::Canvas as PyCanvas;
use crate::chart_widget::Chart as PyChart;
use crate::checkbox_widget::Checkbox as PyCheckbox;
use crate::effects::{Effect as PyEffect, EffectManager as PyEffectManager};
use crate::errors::{io_err_to_py, render_err_to_py};
use crate::image_widget::{ImageState as PyImageState, ImageWidget as PyImageWidget};
use crate::layout::Rect;
use crate::logger::{TuiLoggerWidget as PyTuiLoggerWidget, TuiWidgetState as PyTuiWidgetState};
use crate::map_widget::Map as PyMap;
use crate::menu_widget::{Menu as PyMenu, MenuState as PyMenuState};
use crate::piechart_widget::PieChart as PyPieChart;
use crate::popups::{
    render_popup_text, render_stateful_popup_text, Popup as PyPopup, PopupState as PyPopupState,
};
use crate::prompts::{PasswordPrompt, TextPrompt, TextState};
use crate::qrcode::QrCodeWidget as PyQrCodeWidget;
use crate::scrollview::{ScrollView as PyScrollView, ScrollViewState as PyScrollViewState};
use crate::textarea::TextArea as PyTextArea;
use crate::throbber_widget::Throbber as PyThrobber;
use crate::tree_widget::{Tree as PyTree, TreeState as PyTreeState};
use crate::widgets::{
    BarChart, Block, Clear, Gauge, LineGauge, List, ListState, Monthly, Paragraph, RatatuiMascot,
    Scrollbar, ScrollbarState, Sparkline, Table, TableState, Tabs,
};

// ─── KeyEvent wrapper ─────────────────────────────────────────────────────────

/// A keyboard event returned from `Terminal.poll_event()`.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct PyKeyEvent {
    /// Key code as a string (e.g. "a", "Enter", "Esc", "Up", "F1").
    #[pyo3(get)]
    pub code: String,
    /// Whether Ctrl was held.
    #[pyo3(get)]
    pub ctrl: bool,
    /// Whether Alt was held.
    #[pyo3(get)]
    pub alt: bool,
    /// Whether Shift was held.
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
        KeyCode::F(n) => format!("F{}", n),
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

// ─── Frame ────────────────────────────────────────────────────────────────────

/// A single render frame passed to the draw callback.
///
/// Use the `render_*` methods to draw widgets onto the screen.
///
/// ```python
/// def ui(frame):
///     area = frame.area
///     frame.render_widget(
///         Paragraph.from_string("Hello!").block(Block().bordered()),
///         area
///     )
/// ```
///
/// **Note:** `Frame` is only valid inside the `draw` callback. Do not store it.
#[pyclass(module = "pyratatui", unsendable)]
pub struct Frame {
    ptr: *mut RFrame<'static>,
}

unsafe impl Send for Frame {}

impl Frame {
    fn get(&mut self) -> &mut RFrame<'static> {
        unsafe { &mut *self.ptr }
    }
}

#[pymethods]
impl Frame {
    /// The full terminal area available for this frame.
    ///
    /// ratatui 0.30: `Frame::size()` removed, use `Frame::area()`.
    #[getter]
    pub fn area(&mut self) -> Rect {
        Rect {
            inner: self.get().area(),
        }
    }

    /// Alias for `area` (backwards compatible).
    #[getter]
    pub fn size(&mut self) -> Rect {
        self.area()
    }

    /// Render a widget into the given area.
    pub fn render_widget(&mut self, widget: &Bound<'_, PyAny>, area: &Rect) -> PyResult<()> {
        let frame = self.get();
        let a = area.inner;

        macro_rules! try_widget {
            ($T:ty, $to_rat:ident) => {
                if let Ok(w) = widget.extract::<PyRef<$T>>() {
                    frame.render_widget(w.$to_rat(), a);
                    return Ok(());
                }
            };
        }

        try_widget!(Block, to_ratatui);
        try_widget!(Paragraph, to_ratatui);
        try_widget!(Gauge, to_ratatui);
        try_widget!(LineGauge, to_ratatui);
        try_widget!(BarChart, to_ratatui);
        try_widget!(Sparkline, to_ratatui);
        try_widget!(Clear, to_ratatui);
        try_widget!(Tabs, to_ratatui);
        try_widget!(Monthly, to_ratatui);
        try_widget!(RatatuiMascot, to_ratatui);
        try_widget!(PyBarGraph, to_ratatui);

        if let Ok(w) = widget.extract::<PyRef<List>>() {
            frame.render_widget(w.to_ratatui(), a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<Table>>() {
            frame.render_widget(w.to_ratatui(), a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyCanvas>>() {
            frame.render_widget(&*w, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyMap>>() {
            frame.render_widget(&*w, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyButton>>() {
            frame.render_widget(&*w, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyThrobber>>() {
            frame.render_widget(&*w, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyPieChart>>() {
            frame.render_widget(w.to_ratatui()?, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyCheckbox>>() {
            frame.render_widget(w.to_ratatui()?, a);
            return Ok(());
        }
        if let Ok(w) = widget.extract::<PyRef<PyChart>>() {
            frame.render_widget(w.to_ratatui()?, a);
            return Ok(());
        }

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

    /// Render a `Menu` with mutable `MenuState`.
    pub fn render_stateful_menu(
        &mut self,
        widget: &PyMenu,
        area: &Rect,
        mut state: PyRefMut<'_, PyMenuState>,
    ) -> PyResult<()> {
        self.get()
            .render_stateful_widget(widget.to_ratatui(), area.inner, &mut state.inner);
        Ok(())
    }

    fn __repr__(&self) -> String {
        "Frame(<active>)".to_string()
    }

    /// Apply a TachyonFX `Effect` to this frame's buffer.
    pub fn apply_effect(&mut self, effect: &mut PyEffect, elapsed_ms: u64, area: &Rect) {
        let dur: tachyonfx::Duration = std::time::Duration::from_millis(elapsed_ms).into();
        let buf = unsafe { &mut *self.buffer_mut_ptr() };
        effect.inner.process(dur, buf, area.inner);
    }

    /// Apply an `EffectManager` to this frame's buffer.
    pub fn apply_effect_manager(
        &mut self,
        manager: &mut PyEffectManager,
        elapsed_ms: u64,
        area: &Rect,
    ) {
        let buf = unsafe { &mut *self.buffer_mut_ptr() };
        manager.process_raw(elapsed_ms, buf, area.inner);
    }

    /// Render a [`TextPrompt`] with the given [`TextState`] into `area`.
    pub fn render_text_prompt(&mut self, prompt: &TextPrompt, area: &Rect, state: &TextState) {
        prompt.render_raw(self.get(), area.inner, state);
    }

    /// Render a [`PasswordPrompt`] with the given [`TextState`] into `area`.
    pub fn render_password_prompt(
        &mut self,
        prompt: &PasswordPrompt,
        area: &Rect,
        state: &TextState,
    ) {
        prompt.render_raw(self.get(), area.inner, state);
    }

    /// Render a `Popup` widget (stateless — always centered).
    pub fn render_popup(&mut self, popup: &PyPopup, area: &Rect) {
        render_popup_text(self.get(), popup, area.inner);
    }

    /// Render a `Popup` widget with `PopupState` (draggable / moveable).
    pub fn render_stateful_popup(
        &mut self,
        popup: &PyPopup,
        area: &Rect,
        state: &mut PyPopupState,
    ) {
        render_stateful_popup_text(self.get(), popup, area.inner, state);
    }

    /// Render a `TextArea` multi-line text editor widget.
    ///
    /// In ratatui 0.30 / tui-textarea 0.7, `&TextArea` implements `Widget`
    /// so we take an immutable reference.
    pub fn render_textarea(&mut self, ta: &PyTextArea, area: &Rect) {
        ta.render_raw(self.get(), area.inner);
    }

    /// Render a `ScrollView` with its mutable `ScrollViewState`.
    pub fn render_stateful_scrollview(
        &mut self,
        sv: &PyScrollView,
        area: &Rect,
        mut state: PyRefMut<'_, PyScrollViewState>,
    ) {
        sv.render_into_frame(self.get(), area.inner, &mut state);
    }

    /// Render a `QrCodeWidget` QR code into the given area.
    pub fn render_qrcode(&mut self, qr: &PyQrCodeWidget, area: &Rect) -> PyResult<()> {
        qr.render_raw(self.get(), area.inner)
    }

    /// Render a `Tree` widget with its mutable `TreeState`.
    pub fn render_stateful_tree(
        &mut self,
        tree: &PyTree,
        area: &Rect,
        mut state: PyRefMut<'_, PyTreeState>,
    ) {
        use crate::tree_widget::build_ratatui_items;
        use tui_tree_widget::Tree as RTree;

        // Cache items in state so key_up/key_down work correctly.
        state.items = tree.items.clone();

        let mut counter = 0usize;
        let items = build_ratatui_items(&tree.items, &mut counter);
        let mut rtree = match RTree::new(&items) {
            Ok(t) => t,
            Err(_) => return,
        };
        if let Some(ref b) = tree.block {
            rtree = rtree.block(b.to_ratatui());
        }
        if let Some(ref s) = tree.highlight_style {
            rtree = rtree.highlight_style(s.inner);
        }
        if let Some(ref sym) = tree.highlight_symbol {
            rtree = rtree.highlight_symbol(sym.as_str());
        }
        self.get()
            .render_stateful_widget(rtree, area.inner, &mut state.inner);
    }

    /// Render a `TuiLoggerWidget` with a `TuiWidgetState`.
    ///
    /// Note: `TuiLoggerWidget` is a regular Widget (state embedded via builder),
    /// so we use `render_widget` not `render_stateful_widget`.
    pub fn render_logger(
        &mut self,
        widget: &PyTuiLoggerWidget,
        area: &Rect,
        state: PyRef<'_, PyTuiWidgetState>,
    ) {
        let rw = widget.to_ratatui(&state.inner);
        self.get().render_widget(rw, area.inner);
    }

    /// Render a stateful `ImageWidget` with its mutable `ImageState`.
    pub fn render_stateful_image(
        &mut self,
        _widget: &PyImageWidget,
        area: &Rect,
        mut state: PyRefMut<'_, PyImageState>,
    ) {
        let img = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(Some(
            image::imageops::FilterType::Lanczos3,
        )));
        self.get()
            .render_stateful_widget(img, area.inner, &mut state.protocol);
    }
}

impl Frame {
    #[allow(dead_code)]
    pub(crate) fn buffer_mut_ptr(&mut self) -> *mut ratatui::buffer::Buffer {
        self.get().buffer_mut() as *mut _
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_mut(&mut self) -> &mut ratatui::buffer::Buffer {
        self.get().buffer_mut()
    }
}

// ─── Terminal ─────────────────────────────────────────────────────────────────

type RatTerminal = RTerminal<CrosstermBackend<Stdout>>;

/// The main terminal driver.
///
/// Initialises the crossterm backend and drives the render loop. By default it
/// enters the alternate screen and owns the whole terminal; pass
/// `inline_height` to draw in a block of that many lines inside the normal
/// buffer instead, leaving the scrollback and whatever is already on screen
/// alone.
///
/// ```python
/// from pyratatui import Terminal, Paragraph, Text
///
/// with Terminal() as term:
///     while True:
///         def ui(frame):
///             frame.render_widget(
///                 Paragraph.from_string("Hello! Press q to quit."),
///                 frame.area
///             )
///         term.draw(ui)
///         ev = term.poll_event(timeout_ms=100)
///         if ev and ev.code == "q":
///             break
/// ```
#[pyclass(module = "pyratatui", unsendable)]
pub struct Terminal {
    inner: Option<RatTerminal>,
    entered: bool,
    /// Height of the inline viewport, or `None` for the alternate screen.
    inline_height: Option<u16>,
    /// Whether the alternate screen was entered and has to be left again.
    alt_screen: bool,
}

impl Terminal {
    /// Enter raw mode and build the backing terminal, inline or fullscreen.
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
}

#[pymethods]
impl Terminal {
    /// `Terminal()` takes over the screen; `Terminal(inline_height=12)` draws a
    /// block of that many lines where the cursor is.
    #[new]
    #[pyo3(signature = (inline_height=None))]
    pub fn new(inline_height: Option<u16>) -> PyResult<Self> {
        Ok(Self {
            inner: None,
            entered: false,
            inline_height,
            alt_screen: false,
        })
    }

    // ── Context manager ──────────────────────────────────────────────────────

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
                // An inline viewport draws in the normal buffer: clear the block it
                // owns and put the cursor under it, so what is printed next starts on
                // a clean line instead of over the last frame.
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

    // ── Drawing ──────────────────────────────────────────────────────────────

    pub fn draw(&mut self, draw_fn: &Bound<'_, PyAny>) -> PyResult<()> {
        let term = self.inner.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("Terminal not initialised — use `with Terminal() as t:`")
        })?;

        let mut err = None;

        term.draw(|frame| {
            let py_frame = Frame {
                ptr: unsafe {
                    std::mem::transmute::<*mut RFrame<'_>, *mut RFrame<'static>>(
                        frame as *mut RFrame<'_>,
                    )
                },
            };
            Python::attach(|py| {
                if let Ok(obj) = Py::new(py, py_frame) {
                    if let Err(e) = draw_fn.call1((obj,)) {
                        err = Some(e);
                    }
                }
            });
        })
        .map_err(io_err_to_py)?;

        if let Some(e) = err {
            Err(e)
        } else {
            Ok(())
        }
    }

    // ── Events ───────────────────────────────────────────────────────────────

    #[pyo3(signature = (timeout_ms=0))]
    pub fn poll_event(&self, timeout_ms: u64) -> PyResult<Option<PyKeyEvent>> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        if event::poll(timeout).map_err(io_err_to_py)? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) = event::read().map_err(io_err_to_py)?
            {
                return Ok(Some(PyKeyEvent {
                    code: key_code_str(&code),
                    ctrl: modifiers.contains(KeyModifiers::CONTROL),
                    alt: modifiers.contains(KeyModifiers::ALT),
                    shift: modifiers.contains(KeyModifiers::SHIFT),
                }));
            }
        }
        Ok(None)
    }

    // ── Sizing ───────────────────────────────────────────────────────────────

    /// The current terminal area.
    ///
    /// ratatui 0.30: `Terminal::size()` returns `Result<Rect, B::Error>`.
    pub fn area(&mut self) -> PyResult<Rect> {
        let term = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Terminal not initialised"))?;
        // Terminal::size() → Result<Rect, io::Error> for CrosstermBackend.
        Ok(Rect {
            inner: term.size().map_err(io_err_to_py)?.into(),
        })
    }

    pub fn clear(&mut self) -> PyResult<()> {
        let term = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Terminal not initialised"))?;
        term.clear().map_err(io_err_to_py)
    }

    pub fn hide_cursor(&mut self) -> PyResult<()> {
        let term = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Terminal not initialised"))?;
        term.hide_cursor().map_err(io_err_to_py)
    }

    pub fn show_cursor(&mut self) -> PyResult<()> {
        let term = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Terminal not initialised"))?;
        term.show_cursor().map_err(io_err_to_py)
    }

    // ── Async helpers ─────────────────────────────────────────────────────────

    pub fn __aenter__<'a>(
        mut slf: PyRefMut<'a, Self>,
        _py: Python<'_>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        slf.start()?;
        Ok(slf)
    }

    pub fn __aexit__(
        &mut self,
        py: Python<'_>,
        _exc_type: &Bound<'_, PyAny>,
        _exc_val: &Bound<'_, PyAny>,
        _exc_tb: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        self.restore()?;
        Ok(py.None())
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
