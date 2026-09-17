// src/widgets/sparkline.rs
//! ratatui 0.30 breaking change: `Sparkline::data()` now accepts
//! `IntoIterator<Item = SparklineBar>` instead of `&[u64]`.
//! `SparklineBar` has `From<u64>` so we convert our Vec<u64> using `.map()`.

use crate::style::Style;
use crate::widgets::block::Block;
use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::{Sparkline as RSparkline, SparklineBar};

/// A compact single-row sparkline chart.
///
/// ```python
/// from pyratatui import Sparkline, Style, Color
///
/// spark = (Sparkline()
///     .data([10, 20, 15, 35, 25, 40, 30])
///     .style(Style().fg(Color.green())))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Sparkline {
    data: Vec<u64>,
    block: Option<Block>,
    max: Option<u64>,
    style: Option<Style>,
}

impl Sparkline {
    pub(crate) fn to_ratatui(&self) -> RSparkline<'static> {
        // ratatui 0.30: SparklineBar has From<u64>, so convert each element.
        let bars: Vec<SparklineBar> = self.data.iter().copied().map(SparklineBar::from).collect();
        let mut s = RSparkline::default().data(bars);
        if let Some(ref b) = self.block {
            s = s.block(b.to_ratatui());
        }
        if let Some(m) = self.max {
            s = s.max(m);
        }
        if let Some(ref st) = self.style {
            s = s.style(st.inner);
        }
        s
    }
}

impl Sparkline {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Sparkline {
    #[new]
    pub fn new() -> Self {
        Self {
            data: vec![],
            block: None,
            max: None,
            style: None,
        }
    }

    pub fn data(&self, data: Vec<u64>) -> Sparkline {
        let mut s = self.clone();
        s.data = data;
        s
    }
    pub fn block(&self, block: &Block) -> Sparkline {
        let mut s = self.clone();
        s.block = Some(block.clone());
        s
    }
    pub fn max(&self, m: u64) -> Sparkline {
        let mut s = self.clone();
        s.max = Some(m);
        s
    }
    pub fn style(&self, style: &Style) -> Sparkline {
        let mut s = self.clone();
        s.style = Some(style.clone());
        s
    }
    fn __repr__(&self) -> String {
        format!("Sparkline(data_len={})", self.data.len())
    }
}

pub fn register_sparkline(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Sparkline>()?;
    Ok(())
}
