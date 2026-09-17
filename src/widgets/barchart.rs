// src/widgets/barchart.rs
//! Python bindings for `BarChart`, `Bar`, and `BarGroup`.
//!
//! ratatui 0.30 breaking changes:
//! - `Bar::label()` and `BarGroup::label()` now accept `Into<Line<'a>>` instead
//!   of `Into<Cow<'a, str>>`.  Callers that passed `"foo".into()` now just pass `"foo"`.
//!   Our code creates `Line::from(string)` explicitly — that still compiles.
//! - `Bar::text_value()` now accepts `Into<Cow<'static, str>>` — unchanged for us.
//! - `Bar` now implements `Styled`.
//! - New constructors: `Bar::new(label, value)`, `BarGroup::new(label, bars)`,
//!   `BarChart::grouped(groups)`, `BarChart::vertical(groups)`, etc.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Direction as RDirection;
use ratatui::layout::Rect as RRect;
use ratatui::text::Line as RLine;
use ratatui::widgets::{Bar as RBar, BarChart as RBarChart, BarGroup as RBarGroup};

use crate::layout::Direction;
use crate::style::Style;
use crate::widgets::block::Block;

// ─── Bar ─────────────────────────────────────────────────────────────────────

/// A single bar in a bar chart.
///
/// ```python
/// from pyratatui import Bar
///
/// bar = Bar(10, "January")
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Bar {
    label: Option<String>,
    value: u64,
    style: Option<Style>,
    value_style: Option<Style>,
    text_value: Option<String>,
}

impl Bar {
    pub(crate) fn to_ratatui(&self) -> RBar<'static> {
        let mut b = RBar::default().value(self.value);
        if let Some(ref l) = self.label {
            // ratatui 0.30: label() accepts Into<Line<'a>>.
            // RLine implements Into<Line> so this is fine.
            b = b.label(RLine::from(l.clone()));
        }
        if let Some(ref s) = self.style {
            b = b.style(s.inner);
        }
        if let Some(ref s) = self.value_style {
            b = b.value_style(s.inner);
        }
        if let Some(ref tv) = self.text_value {
            // text_value() still accepts Into<Cow<'static, str>>.
            b = b.text_value(tv.clone());
        }
        b
    }
}

#[pymethods]
impl Bar {
    #[new]
    #[pyo3(signature = (value, label=None))]
    pub fn new(value: u64, label: Option<String>) -> Self {
        Self {
            label,
            value,
            style: None,
            value_style: None,
            text_value: None,
        }
    }

    pub fn style(&self, style: &Style) -> Bar {
        let mut b = self.clone();
        b.style = Some(style.clone());
        b
    }
    pub fn value_style(&self, style: &Style) -> Bar {
        let mut b = self.clone();
        b.value_style = Some(style.clone());
        b
    }
    pub fn text_value(&self, tv: &str) -> Bar {
        let mut b = self.clone();
        b.text_value = Some(tv.to_string());
        b
    }
    fn __repr__(&self) -> String {
        format!("Bar(value={})", self.value)
    }
}

// ─── BarGroup ─────────────────────────────────────────────────────────────────

/// A labelled group of bars.
///
/// ```python
/// from pyratatui import BarGroup, Bar
///
/// group = BarGroup([Bar(10, "Jan"), Bar(15, "Feb")], "Q1")
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct BarGroup {
    label: Option<String>,
    bars: Vec<Bar>,
}

impl BarGroup {
    pub(crate) fn to_ratatui(&self) -> RBarGroup<'static> {
        let bars: Vec<RBar<'static>> = self.bars.iter().map(|b| b.to_ratatui()).collect();
        let mut g = RBarGroup::default().bars(&bars);
        if let Some(ref l) = self.label {
            g = g.label(RLine::from(l.clone()));
        }
        g
    }
}

#[pymethods]
impl BarGroup {
    #[new]
    #[pyo3(signature = (bars, label=None))]
    pub fn new(bars: Vec<PyRef<Bar>>, label: Option<String>) -> Self {
        Self {
            label,
            bars: bars.iter().map(|b| (**b).clone()).collect(),
        }
    }
    fn __repr__(&self) -> String {
        format!("BarGroup(bars={})", self.bars.len())
    }
}

// ─── BarChart ─────────────────────────────────────────────────────────────────

/// A vertical or horizontal bar chart.
///
/// ```python
/// from pyratatui import BarChart, BarGroup, Bar
///
/// chart = (BarChart()
///     .data(BarGroup([Bar(10, "Jan"), Bar(15, "Feb"), Bar(12, "Mar")]))
///     .bar_width(5)
///     .max(20))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct BarChart {
    data: Vec<BarGroup>,
    block: Option<Block>,
    bar_width: u16,
    bar_gap: u16,
    group_gap: u16,
    max: Option<u64>,
    style: Option<Style>,
    bar_style: Option<Style>,
    value_style: Option<Style>,
    label_style: Option<Style>,
    direction: Direction,
}

impl BarChart {
    pub(crate) fn to_ratatui(&self) -> RBarChart<'static> {
        let dir = match self.direction {
            Direction::Vertical => RDirection::Vertical,
            _ => RDirection::Horizontal,
        };
        let groups: Vec<RBarGroup<'static>> = self.data.iter().map(|g| g.to_ratatui()).collect();
        let mut chart = RBarChart::default()
            .bar_width(self.bar_width)
            .bar_gap(self.bar_gap)
            .group_gap(self.group_gap)
            .direction(dir);

        for g in groups {
            chart = chart.data(g);
        }

        if let Some(m) = self.max {
            chart = chart.max(m);
        }
        if let Some(ref b) = self.block {
            chart = chart.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            chart = chart.style(s.inner);
        }
        if let Some(ref s) = self.bar_style {
            chart = chart.bar_style(s.inner);
        }
        if let Some(ref s) = self.value_style {
            chart = chart.value_style(s.inner);
        }
        if let Some(ref s) = self.label_style {
            chart = chart.label_style(s.inner);
        }
        chart
    }
}

impl BarChart {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl BarChart {
    #[new]
    pub fn new() -> Self {
        Self {
            data: vec![],
            block: None,
            bar_width: 3,
            bar_gap: 1,
            group_gap: 3,
            max: None,
            style: None,
            bar_style: None,
            value_style: None,
            label_style: None,
            direction: Direction::Vertical,
        }
    }

    pub fn data(&self, group: &BarGroup) -> BarChart {
        let mut c = self.clone();
        c.data.push(group.clone());
        c
    }
    pub fn block(&self, block: &Block) -> BarChart {
        let mut c = self.clone();
        c.block = Some(block.clone());
        c
    }
    pub fn bar_width(&self, w: u16) -> BarChart {
        let mut c = self.clone();
        c.bar_width = w;
        c
    }
    pub fn bar_gap(&self, g: u16) -> BarChart {
        let mut c = self.clone();
        c.bar_gap = g;
        c
    }
    pub fn group_gap(&self, g: u16) -> BarChart {
        let mut c = self.clone();
        c.group_gap = g;
        c
    }
    pub fn max(&self, m: u64) -> BarChart {
        let mut c = self.clone();
        c.max = Some(m);
        c
    }
    pub fn style(&self, s: &Style) -> BarChart {
        let mut c = self.clone();
        c.style = Some(s.clone());
        c
    }
    pub fn bar_style(&self, s: &Style) -> BarChart {
        let mut c = self.clone();
        c.bar_style = Some(s.clone());
        c
    }
    pub fn value_style(&self, s: &Style) -> BarChart {
        let mut c = self.clone();
        c.value_style = Some(s.clone());
        c
    }
    pub fn label_style(&self, s: &Style) -> BarChart {
        let mut c = self.clone();
        c.label_style = Some(s.clone());
        c
    }
    fn __repr__(&self) -> String {
        format!("BarChart(groups={})", self.data.len())
    }
}

pub fn register_barchart(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Bar>()?;
    m.add_class::<BarGroup>()?;
    m.add_class::<BarChart>()?;
    Ok(())
}
