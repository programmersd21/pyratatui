// src/widgets/paragraph.rs
//! Python binding for the `Paragraph` widget.
//!
//! ratatui 0.30: `Alignment` is now a type alias for `HorizontalAlignment`.
//! Both work via `use ratatui::layout::Alignment as RAlignment`.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Alignment as RAlignment;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::{Paragraph as RParagraph, Wrap};

use crate::style::Style;
use crate::text::Text;
use crate::widgets::block::Block;

/// A widget that renders a block of text with optional wrapping and scrolling.
///
/// ```python
/// from pyratatui import Paragraph, Text, Block, Style, Color
///
/// para = (Paragraph(Text.from_string("Hello, World!"))
///     .block(Block().bordered().title("Info"))
///     .style(Style().fg(Color.white()))
///     .wrap(True))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Paragraph {
    text: Text,
    block: Option<Block>,
    style: Option<Style>,
    wrap: bool,
    trim: bool,
    scroll_x: u16,
    scroll_y: u16,
    alignment: String,
}

impl Paragraph {
    pub(crate) fn to_ratatui(&self) -> RParagraph<'static> {
        let mut para = RParagraph::new(self.text.to_ratatui());
        if let Some(ref b) = self.block {
            para = para.block(b.to_ratatui());
        }
        if let Some(ref s) = self.style {
            para = para.style(s.inner);
        }
        if self.wrap {
            para = para.wrap(Wrap { trim: self.trim });
        }
        para = para.scroll((self.scroll_y, self.scroll_x));
        para = para.alignment(match self.alignment.as_str() {
            "center" => RAlignment::Center,
            "right" => RAlignment::Right,
            _ => RAlignment::Left,
        });
        para
    }
}

impl Paragraph {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Paragraph {
    #[new]
    pub fn new(text: &Text) -> Self {
        Self {
            text: text.clone(),
            block: None,
            style: None,
            wrap: false,
            trim: true,
            scroll_x: 0,
            scroll_y: 0,
            alignment: "left".into(),
        }
    }

    #[staticmethod]
    pub fn from_string(s: &str) -> Paragraph {
        Paragraph::new(&Text::from_string(s))
    }

    pub fn block(&self, block: &Block) -> Paragraph {
        let mut p = self.clone();
        p.block = Some(block.clone());
        p
    }
    pub fn style(&self, style: &Style) -> Paragraph {
        let mut p = self.clone();
        p.style = Some(style.clone());
        p
    }
    #[pyo3(signature = (wrap=true, trim=true))]
    pub fn wrap(&self, wrap: bool, trim: bool) -> Paragraph {
        let mut p = self.clone();
        p.wrap = wrap;
        p.trim = trim;
        p
    }
    #[pyo3(signature = (y=0, x=0))]
    pub fn scroll(&self, y: u16, x: u16) -> Paragraph {
        let mut p = self.clone();
        p.scroll_y = y;
        p.scroll_x = x;
        p
    }
    pub fn alignment(&self, alignment: &str) -> Paragraph {
        let mut p = self.clone();
        p.alignment = alignment.to_string();
        p
    }
    pub fn left_aligned(&self) -> Paragraph {
        self.alignment("left")
    }
    pub fn centered(&self) -> Paragraph {
        self.alignment("center")
    }
    pub fn right_aligned(&self) -> Paragraph {
        self.alignment("right")
    }

    fn __repr__(&self) -> String {
        format!(
            "Paragraph(lines={}, wrap={})",
            self.text.lines.len(),
            self.wrap
        )
    }
}

pub fn register_paragraph(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Paragraph>()?;
    Ok(())
}
