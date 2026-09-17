// src/widgets/table.rs
//! Python bindings for `Table`, `Row`, `Cell`, and `TableState`.
//!
//! ratatui 0.30 breaking changes:
//! - `Row::height()` is now `Row::height_with_margin()` for rows taller than 1.
//!   The plain `Row::new(cells)` / `.height(n)` builder still exists; no change needed.
//! - `Table::column_spacing()` renamed to `Table::column_spacing()` (unchanged).
//! - All types remain in `ratatui::widgets`.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Constraint as RConstraint;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::{
    Cell as RCell, HighlightSpacing, Row as RRow, Table as RTable, TableState as RTableState,
};

use crate::layout::Constraint;
use crate::style::Style;
use crate::text::Text;
use crate::widgets::block::Block;

// ─── Cell ─────────────────────────────────────────────────────────────────────

/// A single styled cell in a table row.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Cell {
    text: Text,
    style: Option<Style>,
}

impl Cell {
    pub(crate) fn to_ratatui(&self) -> RCell<'static> {
        let mut c = RCell::new(self.text.to_ratatui());
        if let Some(ref s) = self.style {
            c = c.style(s.inner);
        }
        c
    }
}

#[pymethods]
impl Cell {
    #[new]
    #[pyo3(signature = (text, style=None))]
    pub fn new(text: &str, style: Option<&Style>) -> Self {
        Self {
            text: Text::from_string(text),
            style: style.cloned(),
        }
    }
    pub fn style(&self, style: &Style) -> Cell {
        let mut c = self.clone();
        c.style = Some(style.clone());
        c
    }
    fn __repr__(&self) -> String {
        format!("Cell(width={})", self.text.width())
    }
}

// ─── Row ──────────────────────────────────────────────────────────────────────

/// A row of cells in a table.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Row {
    cells: Vec<Cell>,
    style: Option<Style>,
    height: u16,
}

impl Row {
    pub(crate) fn to_ratatui(&self) -> RRow<'static> {
        let cells: Vec<RCell<'static>> = self.cells.iter().map(|c| c.to_ratatui()).collect();
        let mut row = RRow::new(cells).height(self.height);
        if let Some(ref s) = self.style {
            row = row.style(s.inner);
        }
        row
    }
}

#[pymethods]
impl Row {
    #[new]
    pub fn new(cells: Vec<PyRef<Cell>>) -> Self {
        Self {
            cells: cells.iter().map(|c| (**c).clone()).collect(),
            style: None,
            height: 1,
        }
    }
    #[staticmethod]
    pub fn from_strings(texts: Vec<String>) -> Row {
        Row {
            cells: texts.iter().map(|t| Cell::new(t, None)).collect(),
            style: None,
            height: 1,
        }
    }
    pub fn style(&self, style: &Style) -> Row {
        let mut r = self.clone();
        r.style = Some(style.clone());
        r
    }
    pub fn height(&self, height: u16) -> Row {
        let mut r = self.clone();
        r.height = height;
        r
    }
    fn __repr__(&self) -> String {
        format!("Row(cells={})", self.cells.len())
    }
}

// ─── TableState ───────────────────────────────────────────────────────────────

/// Mutable selection state for a `Table`.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct TableState {
    pub(crate) inner: RTableState,
}

#[pymethods]
impl TableState {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RTableState::default(),
        }
    }

    #[pyo3(signature = (index=None))]
    pub fn select(&mut self, index: Option<usize>) {
        self.inner.select(index);
    }

    pub fn select_next(&mut self) {
        self.inner.select_next();
    }
    pub fn select_previous(&mut self) {
        self.inner.select_previous();
    }
    pub fn select_first(&mut self) {
        self.inner.select_first();
    }
    pub fn select_last(&mut self) {
        self.inner.select_last();
    }

    #[getter]
    pub fn selected(&self) -> Option<usize> {
        self.inner.selected()
    }
    #[getter]
    pub fn offset(&self) -> usize {
        self.inner.offset()
    }

    fn __repr__(&self) -> String {
        format!("TableState(selected={:?})", self.inner.selected())
    }
}

// ─── Table ────────────────────────────────────────────────────────────────────

/// A scrollable table widget with row and column selection.
///
/// `column_widths`, `header`, and `footer` can be passed to the constructor
/// or set later with the builder methods of the same name.
///
/// ```python
/// from pyratatui import Table, Row, Cell, Constraint, TableState, Block
///
/// rows = [
///     Row([Cell("Name"), Cell("Value")]),
///     Row([Cell("Alice"), Cell("42")]),
/// ]
/// table = Table(
///     rows,
///     column_widths=[Constraint.length(20), Constraint.fill(1)],
///     header=Row([Cell("Key"), Cell("Val")]),
/// ).block(Block().bordered().title("Data"))
///
/// state = TableState()
/// state.select(0)
/// # render with frame.render_stateful_table(table, area, state)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Table {
    rows: Vec<Row>,
    header: Option<Row>,
    footer: Option<Row>,
    block: Option<Block>,
    column_widths: Vec<Constraint>,
    style: Option<Style>,
    highlight_style: Option<Style>,
    highlight_symbol: Option<String>,
    column_spacing: u16,
}

impl Table {
    pub(crate) fn to_ratatui(&self) -> RTable<'_> {
        let rows: Vec<RRow<'static>> = self.rows.iter().map(|r| r.to_ratatui()).collect();
        let widths: Vec<RConstraint> = self.column_widths.iter().map(|c| c.inner).collect();
        let mut table = RTable::new(rows, widths).column_spacing(self.column_spacing);

        if let Some(ref h) = self.header {
            table = table.header(h.to_ratatui());
        }
        if let Some(ref f) = self.footer {
            table = table.footer(f.to_ratatui());
        }
        if let Some(ref b) = self.block {
            table = table.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            table = table.style(s.inner);
        }
        if let Some(ref s) = self.highlight_style {
            table = table.row_highlight_style(s.inner);
        }
        if let Some(ref sym) = self.highlight_symbol {
            table = table
                .highlight_symbol(sym.as_str())
                .highlight_spacing(HighlightSpacing::Always);
        }
        table
    }
}

impl Table {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Table {
    #[new]
    #[pyo3(signature = (rows, column_widths=None, header=None, footer=None))]
    pub fn new(
        rows: Vec<PyRef<Row>>,
        column_widths: Option<Vec<PyRef<Constraint>>>,
        header: Option<PyRef<Row>>,
        footer: Option<PyRef<Row>>,
    ) -> Self {
        Self {
            rows: rows.iter().map(|r| (**r).clone()).collect(),
            header: header.as_ref().map(|r| (**r).clone()),
            footer: footer.as_ref().map(|r| (**r).clone()),
            block: None,
            column_widths: column_widths
                .map(|w| w.iter().map(|c| (**c).clone()).collect())
                .unwrap_or_default(),
            style: None,
            highlight_style: None,
            highlight_symbol: None,
            column_spacing: 1,
        }
    }

    pub fn header(&self, row: &Row) -> Table {
        let mut t = self.clone();
        t.header = Some(row.clone());
        t
    }
    pub fn footer(&self, row: &Row) -> Table {
        let mut t = self.clone();
        t.footer = Some(row.clone());
        t
    }
    pub fn block(&self, block: &Block) -> Table {
        let mut t = self.clone();
        t.block = Some(block.clone());
        t
    }
    pub fn column_widths(&self, widths: Vec<PyRef<Constraint>>) -> Table {
        let mut t = self.clone();
        t.column_widths = widths.iter().map(|c| (**c).clone()).collect();
        t
    }
    pub fn style(&self, style: &Style) -> Table {
        let mut t = self.clone();
        t.style = Some(style.clone());
        t
    }
    pub fn highlight_style(&self, style: &Style) -> Table {
        let mut t = self.clone();
        t.highlight_style = Some(style.clone());
        t
    }
    pub fn highlight_symbol(&self, sym: &str) -> Table {
        let mut t = self.clone();
        t.highlight_symbol = Some(sym.to_string());
        t
    }
    pub fn column_spacing(&self, n: u16) -> Table {
        let mut t = self.clone();
        t.column_spacing = n;
        t
    }
    fn __repr__(&self) -> String {
        format!("Table(rows={})", self.rows.len())
    }
}

pub fn register_table(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Cell>()?;
    m.add_class::<Row>()?;
    m.add_class::<TableState>()?;
    m.add_class::<Table>()?;
    Ok(())
}
