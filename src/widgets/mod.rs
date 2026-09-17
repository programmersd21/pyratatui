//! Python bindings for ratatui's built-in widgets.

use pyo3::prelude::*;

mod barchart;
mod block;
mod clear;
mod gauge;
mod list;
mod mascot;
mod paragraph;
mod scrollbar;
mod sparkline;
mod table;
mod tabs;

pub use barchart::BarChart;
pub use block::Block;
pub use clear::Clear;
pub use gauge::{Gauge, LineGauge};
pub use list::{List, ListState};
pub use mascot::RatatuiMascot;
pub use paragraph::Paragraph;
pub use scrollbar::{Scrollbar, ScrollbarState};
pub use sparkline::Sparkline;
pub use table::{Table, TableState};
pub use tabs::Tabs;

pub fn register_widgets(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    block::register_block(py, m)?;
    paragraph::register_paragraph(py, m)?;
    list::register_list(py, m)?;
    table::register_table(py, m)?;
    gauge::register_gauge(py, m)?;
    barchart::register_barchart(py, m)?;
    sparkline::register_sparkline(py, m)?;
    clear::register_clear(py, m)?;
    scrollbar::register_scrollbar(py, m)?;
    tabs::register_tabs(py, m)?;
    mascot::register_mascot(py, m)?;
    Ok(())
}
