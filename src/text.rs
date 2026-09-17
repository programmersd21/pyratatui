// IMPORTANT: to_ratatui() helpers are in plain `impl` blocks OUTSIDE #[pymethods]
// so PyO3 does not try to expose them to Python.

use crate::style::Style;
use pyo3::prelude::*;
use ratatui::text::{Line as RLine, Span as RSpan, Text as RText};

// ─── Span ────────────────────────────────────────────────────────────────────

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Span {
    pub(crate) content: String,
    pub(crate) style: Option<Style>,
}

// Rust-only helper — NOT in #[pymethods]
impl Span {
    pub(crate) fn to_ratatui(&self) -> RSpan<'static> {
        match &self.style {
            Some(s) => RSpan::styled(self.content.clone(), s.inner),
            None => RSpan::raw(self.content.clone()),
        }
    }
}

#[pymethods]
impl Span {
    #[new]
    #[pyo3(signature = (content, style=None))]
    pub fn new(content: String, style: Option<Style>) -> Self {
        Self { content, style }
    }

    #[getter]
    pub fn content(&self) -> &str {
        &self.content
    }
    #[getter]
    pub fn style(&self) -> Option<Style> {
        self.style.clone()
    }

    pub fn styled(&self, style: &Style) -> Span {
        Span {
            content: self.content.clone(),
            style: Some(style.clone()),
        }
    }
    pub fn width(&self) -> usize {
        self.content.chars().count()
    }
    fn __repr__(&self) -> String {
        format!("Span({:?})", self.content)
    }
}

// ─── Line ────────────────────────────────────────────────────────────────────

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Line {
    pub(crate) spans: Vec<Span>,
    pub(crate) alignment: Option<String>,
    pub(crate) style: Option<Style>,
}

// Rust-only helper
impl Line {
    pub(crate) fn to_ratatui(&self) -> RLine<'static> {
        let spans: Vec<RSpan<'static>> = self.spans.iter().map(|s| s.to_ratatui()).collect();
        let mut line = RLine::from(spans);
        if let Some(ref align) = self.alignment {
            line = match align.as_str() {
                "center" => line.centered(),
                "right" => line.right_aligned(),
                _ => line.left_aligned(),
            };
        }
        if let Some(ref s) = self.style {
            line = line.style(s.inner);
        }
        line
    }
}

#[pymethods]
impl Line {
    #[new]
    #[pyo3(signature = (spans=None, style=None))]
    pub fn new(spans: Option<Vec<Span>>, style: Option<Style>) -> Self {
        Self {
            spans: spans.unwrap_or_default(),
            alignment: None,
            style,
        }
    }

    #[staticmethod]
    pub fn from_string(s: &str) -> Line {
        Line {
            spans: vec![Span::new(s.to_string(), None)],
            alignment: None,
            style: None,
        }
    }

    pub fn left_aligned(&self) -> Line {
        let mut l = self.clone();
        l.alignment = Some("left".into());
        l
    }
    pub fn centered(&self) -> Line {
        let mut l = self.clone();
        l.alignment = Some("center".into());
        l
    }
    pub fn right_aligned(&self) -> Line {
        let mut l = self.clone();
        l.alignment = Some("right".into());
        l
    }

    pub fn styled(&self, style: &Style) -> Line {
        let mut l = self.clone();
        l.style = Some(style.clone());
        l
    }
    pub fn push_span(&mut self, span: Span) {
        self.spans.push(span);
    }

    #[getter]
    pub fn spans(&self) -> Vec<Span> {
        self.spans.clone()
    }
    pub fn width(&self) -> usize {
        self.spans.iter().map(|s| s.width()).sum()
    }
    fn __repr__(&self) -> String {
        format!("Line(spans={})", self.spans.len())
    }
}

// ─── Text ────────────────────────────────────────────────────────────────────

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Text {
    pub(crate) lines: Vec<Line>,
    pub(crate) alignment: Option<String>,
    pub(crate) style: Option<Style>,
}

// Rust-only helper
impl Text {
    pub(crate) fn to_ratatui(&self) -> RText<'static> {
        let rlines: Vec<RLine<'static>> = self.lines.iter().map(|l| l.to_ratatui()).collect();
        let mut t = RText::from(rlines);
        if let Some(ref s) = self.style {
            t = t.style(s.inner);
        }
        if let Some(ref align) = self.alignment {
            t = match align.as_str() {
                "center" => t.centered(),
                "right" => t.right_aligned(),
                _ => t.left_aligned(),
            };
        }
        t
    }
}

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (lines=None, style=None))]
    pub fn new(lines: Option<Vec<Line>>, style: Option<Style>) -> Self {
        Self {
            lines: lines.unwrap_or_default(),
            alignment: None,
            style,
        }
    }

    #[staticmethod]
    pub fn from_string(s: &str) -> Text {
        Text {
            lines: s.lines().map(Line::from_string).collect(),
            alignment: None,
            style: None,
        }
    }

    pub fn push_line(&mut self, line: Line) {
        self.lines.push(line);
    }
    pub fn push_str(&mut self, s: &str) {
        self.lines.push(Line::from_string(s));
    }

    #[getter]
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    pub fn width(&self) -> usize {
        self.lines.iter().map(|l| l.width()).max().unwrap_or(0)
    }

    pub fn centered(&self) -> Text {
        let mut t = self.clone();
        t.alignment = Some("center".into());
        t
    }
    pub fn right_aligned(&self) -> Text {
        let mut t = self.clone();
        t.alignment = Some("right".into());
        t
    }
    pub fn styled(&self, style: &Style) -> Text {
        let mut t = self.clone();
        t.style = Some(style.clone());
        t
    }

    #[getter]
    pub fn lines(&self) -> Vec<Line> {
        self.lines.clone()
    }
    fn __repr__(&self) -> String {
        format!("Text(lines={})", self.lines.len())
    }
}

// ─── Registration ─────────────────────────────────────────────────────────────

pub fn register_text(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Span>()?;
    m.add_class::<Line>()?;
    m.add_class::<Text>()?;
    Ok(())
}
