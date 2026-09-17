//! pyratatui — Python bindings for Ratatui.
//!
//! The module layout mirrors the public API: style, text, layout, buffer,
//! widgets, calendar, canvas, chart, prompts, and the terminal driver.

use pyo3::prelude::*;

mod buffer;
mod calendar;
mod canvas;
mod chart;
mod errors;
mod layout;
mod prompts;
mod style;
mod terminal;
mod text;
mod widgets;

#[pymodule]
fn _pyratatui(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    errors::register_errors(py, m)?;
    style::register_style(py, m)?;
    text::register_text(py, m)?;
    layout::register_layout(py, m)?;
    buffer::register_buffer(py, m)?;
    widgets::register_widgets(py, m)?;
    calendar::register_calendar(py, m)?;
    canvas::register_canvas(py, m)?;
    chart::register_chart(py, m)?;
    prompts::register_prompts(py, m)?;
    terminal::register_terminal(py, m)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__ratatui_version__", "0.30.2")?;

    Ok(())
}
