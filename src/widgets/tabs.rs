// src/widgets/tabs.rs
use crate::style::Style;
use crate::widgets::block::Block;
use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::Tabs as RTabs;

/// A tab bar widget.
///
/// ```python
/// from pyratatui import Tabs, Style, Color
///
/// tabs = (Tabs(["Overview", "Logs", "Metrics"])
///     .select(1)
///     .highlight_style(Style().fg(Color.yellow()).bold())
///     .divider(" | "))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Tabs {
    titles: Vec<String>,
    selected: usize,
    block: Option<Block>,
    style: Option<Style>,
    highlight_style: Option<Style>,
    divider: String,
    padding_left: String,
    padding_right: String,
}

impl Tabs {
    pub(crate) fn to_ratatui(&self) -> RTabs<'static> {
        let titles: Vec<ratatui::text::Line<'static>> = self
            .titles
            .iter()
            .map(|t| ratatui::text::Line::from(t.clone()))
            .collect();
        let mut tabs = RTabs::new(titles)
            .select(self.selected)
            .divider(ratatui::text::Span::raw(self.divider.clone()))
            .padding(self.padding_left.clone(), self.padding_right.clone());
        if let Some(ref b) = self.block {
            tabs = tabs.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            tabs = tabs.style(s.inner);
        }
        if let Some(ref s) = self.highlight_style {
            tabs = tabs.highlight_style(s.inner);
        }
        tabs
    }
}

impl Tabs {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Tabs {
    #[new]
    pub fn new(titles: Vec<String>) -> Self {
        Self {
            titles,
            selected: 0,
            block: None,
            style: None,
            highlight_style: None,
            divider: "│".into(),
            padding_left: " ".into(),
            padding_right: " ".into(),
        }
    }
    pub fn select(&self, index: usize) -> Tabs {
        let mut t = self.clone();
        t.selected = index;
        t
    }
    pub fn block(&self, block: &Block) -> Tabs {
        let mut t = self.clone();
        t.block = Some(block.clone());
        t
    }
    pub fn style(&self, style: &Style) -> Tabs {
        let mut t = self.clone();
        t.style = Some(style.clone());
        t
    }
    pub fn highlight_style(&self, style: &Style) -> Tabs {
        let mut t = self.clone();
        t.highlight_style = Some(style.clone());
        t
    }
    pub fn divider(&self, d: &str) -> Tabs {
        let mut t = self.clone();
        t.divider = d.to_string();
        t
    }
    pub fn padding(&self, left: &str, right: &str) -> Tabs {
        let mut t = self.clone();
        t.padding_left = left.to_string();
        t.padding_right = right.to_string();
        t
    }
    fn __repr__(&self) -> String {
        format!(
            "Tabs(tabs={}, selected={})",
            self.titles.len(),
            self.selected
        )
    }
}

pub fn register_tabs(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tabs>()?;
    Ok(())
}
