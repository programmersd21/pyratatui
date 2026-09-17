// src/widgets/mascot.rs
//! Python bindings for Ratatui's mascot widget.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::{MascotEyeColor as RMascotEyeColor, RatatuiMascot as RRatatuiMascot};

/// State for the mascot's eye.
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum MascotEyeColor {
    Default,
    Red,
}

impl MascotEyeColor {
    fn to_ratatui(&self) -> RMascotEyeColor {
        match self {
            MascotEyeColor::Default => RMascotEyeColor::Default,
            MascotEyeColor::Red => RMascotEyeColor::Red,
        }
    }
}

#[pymethods]
impl MascotEyeColor {
    fn __repr__(&self) -> &'static str {
        match self {
            MascotEyeColor::Default => "MascotEyeColor.Default",
            MascotEyeColor::Red => "MascotEyeColor.Red",
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            MascotEyeColor::Default => "Default",
            MascotEyeColor::Red => "Red",
        }
    }
}

/// A widget that renders the Ratatui mascot.
///
/// This is a small easter egg shipped upstream in ratatui itself.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct RatatuiMascot {
    eye: MascotEyeColor,
}

impl RatatuiMascot {
    pub(crate) fn to_ratatui(&self) -> RRatatuiMascot {
        RRatatuiMascot::new().set_eye(self.eye.to_ratatui())
    }
}

impl RatatuiMascot {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl RatatuiMascot {
    #[new]
    #[pyo3(signature = (eye_color=None))]
    pub fn new(eye_color: Option<&MascotEyeColor>) -> Self {
        Self {
            eye: eye_color.cloned().unwrap_or(MascotEyeColor::Default),
        }
    }

    /// Set the eye color/state.
    pub fn set_eye(&self, rat_eye: &MascotEyeColor) -> RatatuiMascot {
        let mut s = self.clone();
        s.eye = rat_eye.clone();
        s
    }

    fn __repr__(&self) -> String {
        format!("RatatuiMascot(eye_color={:?})", self.eye)
    }
}

pub fn register_mascot(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MascotEyeColor>()?;
    m.add_class::<RatatuiMascot>()?;
    Ok(())
}
