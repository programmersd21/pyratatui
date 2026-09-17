// src/widgets/clear.rs
use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::Clear as RClear;

/// A widget that clears its area (paints it with the default style).
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Clear;

impl Clear {
    pub(crate) fn to_ratatui(&self) -> RClear {
        RClear
    }
}

impl Clear {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Clear {
    #[new]
    pub fn new() -> Self {
        Self
    }
    fn __repr__(&self) -> String {
        "Clear()".to_string()
    }
}

pub fn register_clear(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Clear>()?;
    Ok(())
}
