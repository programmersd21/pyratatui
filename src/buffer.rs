//! Python bindings for ratatui's `Buffer` type.
//!
//! The `Buffer` is ratatui's intermediate rendering surface.
//! Widgets write into it; the terminal backend flushes it.

use pyo3::prelude::*;
use ratatui::buffer::Buffer as RBuffer;

use crate::layout::Rect;
use crate::style::Style;
use crate::text::Span;

/// An in-memory grid of styled cells used as ratatui's render target.
///
/// Direct manipulation is rarely needed from Python — the `Frame` exposes
/// `render_widget` which writes into the buffer automatically.
///
/// ```python
/// from pyratatui import Buffer, Rect
///
/// buf = Buffer(Rect(0, 0, 80, 24))
/// buf.set_string(0, 0, "Hello!", None)
/// content = buf.get_string(0, 0, 6)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone)]
pub struct Buffer {
    pub(crate) inner: RBuffer,
}

#[pymethods]
impl Buffer {
    /// Create a new blank buffer filling the given area.
    #[new]
    pub fn new(area: &Rect) -> Self {
        Self {
            inner: RBuffer::empty(area.inner),
        }
    }

    /// The area this buffer covers.
    #[getter]
    pub fn area(&self) -> Rect {
        Rect {
            inner: self.inner.area,
        }
    }

    /// Write a plain string at `(x, y)` using an optional style.
    #[pyo3(signature = (x, y, text, style=None))]
    pub fn set_string(&mut self, x: u16, y: u16, text: &str, style: Option<&Style>) {
        let s = style.map(|st| st.inner).unwrap_or_default();
        self.inner.set_string(x, y, text, s);
    }

    /// Write a styled span at `(x, y)`.
    pub fn set_span(&mut self, x: u16, y: u16, span: &Span) {
        let rspan = span.to_ratatui();
        self.inner.set_span(x, y, &rspan, span.width() as u16);
    }

    /// Read the text content of `width` cells starting at `(x, y)`.
    pub fn get_string(&self, x: u16, y: u16, width: u16) -> String {
        (x..x + width)
            .map(|cx| {
                let idx = self.inner.index_of(cx, y);
                self.inner.content[idx].symbol().to_string()
            })
            .collect()
    }

    /// Reset all cells to blank.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Merge `other` into `self` (other overwrites non-empty cells).
    pub fn merge(&mut self, other: &Buffer) {
        self.inner.merge(&other.inner);
    }

    fn __repr__(&self) -> String {
        format!("Buffer(area={:?})", self.inner.area)
    }
}

pub fn register_buffer(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Buffer>()?;
    Ok(())
}
