// src/widgets/gauge.rs
//! Python bindings for `Gauge` and `LineGauge`.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::symbols::line;
use ratatui::widgets::{Gauge as RGauge, LineGauge as RLineGauge};

use crate::style::Style;
use crate::widgets::block::Block;

/// A filled progress bar spanning the full area.
///
/// ```python
/// from pyratatui import Gauge, Style, Color, Block
///
/// gauge = (Gauge()
///     .block(Block().bordered().title("Progress"))
///     .percent(42)
///     .style(Style().fg(Color.green()))
///     .label("42%"))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Gauge {
    block: Option<Block>,
    percent: u16,
    style: Option<Style>,
    gauge_style: Option<Style>,
    label: Option<String>,
    ratio: Option<f64>,
    use_unicode: bool,
}

impl Gauge {
    pub(crate) fn to_ratatui(&self) -> RGauge<'static> {
        let ratio = self.ratio.unwrap_or_else(|| self.percent as f64 / 100.0);
        let mut g = RGauge::default().ratio(ratio.clamp(0.0, 1.0));

        if let Some(ref b) = self.block {
            g = g.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            g = g.style(s.inner);
        }
        if let Some(ref s) = self.gauge_style {
            g = g.gauge_style(s.inner);
        }
        if let Some(ref l) = self.label {
            g = g.label(ratatui::text::Span::raw(l.clone()));
        }
        if self.use_unicode {
            g = g.use_unicode(true);
        }
        g
    }
}

impl Gauge {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Gauge {
    #[new]
    pub fn new() -> Self {
        Self {
            block: None,
            percent: 0,
            style: None,
            gauge_style: None,
            label: None,
            ratio: None,
            use_unicode: false,
        }
    }

    pub fn block(&self, block: &Block) -> Gauge {
        let mut g = self.clone();
        g.block = Some(block.clone());
        g
    }
    pub fn percent(&self, pct: u16) -> Gauge {
        let mut g = self.clone();
        g.percent = pct.min(100);
        g.ratio = None;
        g
    }
    pub fn ratio(&self, ratio: f64) -> Gauge {
        let mut g = self.clone();
        g.ratio = Some(ratio.clamp(0.0, 1.0));
        g
    }
    pub fn style(&self, style: &Style) -> Gauge {
        let mut g = self.clone();
        g.style = Some(style.clone());
        g
    }
    pub fn gauge_style(&self, style: &Style) -> Gauge {
        let mut g = self.clone();
        g.gauge_style = Some(style.clone());
        g
    }
    pub fn label(&self, label: &str) -> Gauge {
        let mut g = self.clone();
        g.label = Some(label.to_string());
        g
    }
    pub fn use_unicode(&self, v: bool) -> Gauge {
        let mut g = self.clone();
        g.use_unicode = v;
        g
    }

    fn __repr__(&self) -> String {
        format!("Gauge(percent={})", self.percent)
    }
}

/// A thin single-line progress indicator.
///
/// ```python
/// from pyratatui import LineGauge, Style, Color
///
/// lg = (LineGauge()
///     .ratio(0.65)
///     .style(Style().fg(Color.blue())))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct LineGauge {
    block: Option<Block>,
    ratio: f64,
    style: Option<Style>,
    gauge_style: Option<Style>,
    label: Option<String>,
    line_set: String,
}

impl LineGauge {
    pub(crate) fn to_ratatui(&self) -> RLineGauge<'static> {
        let ls = match self.line_set.as_str() {
            "double" => line::DOUBLE,
            "thick" => line::THICK,
            _ => line::NORMAL,
        };
        let mut g = RLineGauge::default()
            .ratio(self.ratio.clamp(0.0, 1.0))
            .filled_symbol(ls.horizontal)
            .unfilled_symbol(line::NORMAL.horizontal);

        if let Some(ref b) = self.block {
            g = g.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            g = g.style(s.inner);
        }
        if let Some(ref s) = self.gauge_style {
            g = g.filled_style(s.inner);
        }
        if let Some(ref l) = self.label {
            g = g.label(ratatui::text::Span::raw(l.clone()));
        }
        g
    }
}

impl LineGauge {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl LineGauge {
    #[new]
    pub fn new() -> Self {
        Self {
            block: None,
            ratio: 0.0,
            style: None,
            gauge_style: None,
            label: None,
            line_set: "normal".into(),
        }
    }
    pub fn block(&self, block: &Block) -> LineGauge {
        let mut g = self.clone();
        g.block = Some(block.clone());
        g
    }
    pub fn ratio(&self, ratio: f64) -> LineGauge {
        let mut g = self.clone();
        g.ratio = ratio.clamp(0.0, 1.0);
        g
    }
    pub fn percent(&self, pct: u16) -> LineGauge {
        self.ratio(pct as f64 / 100.0)
    }
    pub fn style(&self, style: &Style) -> LineGauge {
        let mut g = self.clone();
        g.style = Some(style.clone());
        g
    }
    pub fn gauge_style(&self, style: &Style) -> LineGauge {
        let mut g = self.clone();
        g.gauge_style = Some(style.clone());
        g
    }
    pub fn label(&self, label: &str) -> LineGauge {
        let mut g = self.clone();
        g.label = Some(label.to_string());
        g
    }
    /// Line set: "normal" | "thick" | "double".
    pub fn line_set(&self, name: &str) -> LineGauge {
        let mut g = self.clone();
        g.line_set = name.to_string();
        g
    }

    fn __repr__(&self) -> String {
        format!("LineGauge(ratio={:.2})", self.ratio)
    }
}

pub fn register_gauge(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Gauge>()?;
    m.add_class::<LineGauge>()?;
    Ok(())
}
