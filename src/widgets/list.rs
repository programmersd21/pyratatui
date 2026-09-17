// src/widgets/list.rs
//! Python bindings for the `List` widget and `ListState`.
//!
//! ratatui 0.30 breaking change:
//! - `List::highlight_symbol()` now accepts `Into<Line>` instead of `&str`.
//!   A plain `&str` still satisfies `Into<Line<'_>>`, so no source change needed
//!   at the Rust call site, but we use `.to_string()` to keep 'static lifetime.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::text::Line as RLine;
use ratatui::widgets::{
    List as RList, ListDirection as RListDirection, ListItem as RListItem, ListState as RListState,
};

use crate::style::Style;
use crate::text::Text;
use crate::widgets::block::Block;

/// Direction in which the list scrolls.
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum ListDirection {
    TopToBottom,
    BottomToTop,
}

impl ListDirection {
    fn to_ratatui(&self) -> RListDirection {
        match self {
            ListDirection::TopToBottom => RListDirection::TopToBottom,
            ListDirection::BottomToTop => RListDirection::BottomToTop,
        }
    }
}

/// A single item in a `List`.
///
/// ```python
/// from pyratatui import ListItem, Style, Color
///
/// item = ListItem("Server A", Style().fg(Color.green()))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct ListItem {
    text: Text,
    style: Option<Style>,
}

impl ListItem {
    pub(crate) fn to_ratatui(&self) -> RListItem<'static> {
        let mut item = RListItem::new(self.text.to_ratatui());
        if let Some(ref s) = self.style {
            item = item.style(s.inner);
        }
        item
    }
}

#[pymethods]
impl ListItem {
    #[new]
    #[pyo3(signature = (text, style=None))]
    pub fn new(text: &str, style: Option<&Style>) -> Self {
        Self {
            text: Text::from_string(text),
            style: style.cloned(),
        }
    }

    #[staticmethod]
    pub fn from_text(text: &Text) -> ListItem {
        ListItem {
            text: text.clone(),
            style: None,
        }
    }

    pub fn style(&self, style: &Style) -> ListItem {
        let mut i = self.clone();
        i.style = Some(style.clone());
        i
    }

    fn __repr__(&self) -> String {
        format!("ListItem(width={})", self.text.width())
    }
}

/// Mutable selection state for a `List`.
///
/// ```python
/// from pyratatui import ListState
///
/// state = ListState()
/// state.select(2)     # select index 2
/// state.select_next() # move down
/// print(state.selected)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct ListState {
    pub(crate) inner: RListState,
}

#[pymethods]
impl ListState {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RListState::default(),
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
        format!("ListState(selected={:?})", self.inner.selected())
    }
}

/// A scrollable, selectable list of items.
///
/// ```python
/// from pyratatui import List, ListItem, ListState, Style, Color, Block
///
/// items = [ListItem(f"Item {i}") for i in range(20)]
/// lst = (List(items)
///     .block(Block().bordered().title("Items"))
///     .highlight_style(Style().fg(Color.yellow()).bold())
///     .highlight_symbol("▶ "))
///
/// state = ListState()
/// state.select(0)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct List {
    items: Vec<ListItem>,
    block: Option<Block>,
    style: Option<Style>,
    highlight_style: Option<Style>,
    highlight_symbol: Option<String>,
    direction: ListDirection,
    repeat_highlight_symbol: bool,
}

impl List {
    pub(crate) fn to_ratatui(&self) -> RList<'_> {
        let items: Vec<RListItem<'static>> = self.items.iter().map(|i| i.to_ratatui()).collect();
        let mut lst = RList::new(items);

        if let Some(ref b) = self.block {
            lst = lst.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            lst = lst.style(s.inner);
        }
        if let Some(ref s) = self.highlight_style {
            lst = lst.highlight_style(s.inner);
        }
        if let Some(ref sym) = self.highlight_symbol {
            // ratatui 0.30: highlight_symbol() now takes Into<Line>.
            // RLine::from(&str) implements Into<Line>, and String → &str is cheap.
            lst = lst.highlight_symbol(RLine::from(sym.clone()));
        }
        lst = lst.direction(self.direction.to_ratatui());
        if self.repeat_highlight_symbol {
            lst = lst.repeat_highlight_symbol(true);
        }
        lst
    }
}

impl List {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl List {
    #[new]
    pub fn new(items: Vec<PyRef<ListItem>>) -> Self {
        Self {
            items: items.iter().map(|i| (**i).clone()).collect(),
            block: None,
            style: None,
            highlight_style: None,
            highlight_symbol: None,
            direction: ListDirection::TopToBottom,
            repeat_highlight_symbol: false,
        }
    }

    pub fn block(&self, block: &Block) -> List {
        let mut l = self.clone();
        l.block = Some(block.clone());
        l
    }
    pub fn style(&self, style: &Style) -> List {
        let mut l = self.clone();
        l.style = Some(style.clone());
        l
    }
    pub fn highlight_style(&self, style: &Style) -> List {
        let mut l = self.clone();
        l.highlight_style = Some(style.clone());
        l
    }
    pub fn highlight_symbol(&self, sym: &str) -> List {
        let mut l = self.clone();
        l.highlight_symbol = Some(sym.to_string());
        l
    }
    pub fn direction(&self, dir: &ListDirection) -> List {
        let mut l = self.clone();
        l.direction = dir.clone();
        l
    }
    pub fn repeat_highlight_symbol(&self, v: bool) -> List {
        let mut l = self.clone();
        l.repeat_highlight_symbol = v;
        l
    }

    fn __repr__(&self) -> String {
        format!("List(items={})", self.items.len())
    }
}

pub fn register_list(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ListItem>()?;
    m.add_class::<ListState>()?;
    m.add_class::<List>()?;
    m.add_class::<ListDirection>()?;
    Ok(())
}
