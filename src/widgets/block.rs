// src/widgets/block.rs
//! Python binding for the `Block` widget.
//!
//! ratatui 0.30 breaking changes handled here:
//! - `block::Title` struct has been **removed**. Use `Line` + `title_top()` / `title_bottom()`.
//! - `Block::title_alignment()` is still available but now accepts `HorizontalAlignment`.
//!   A type-alias `Alignment = HorizontalAlignment` is provided for backwards compatibility.
//! - New `BorderType` variants: `LightDoubleDashed`, `HeavyDoubleDashed`, etc. Added below.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::HorizontalAlignment;
use ratatui::layout::Rect as RRect;
use ratatui::text::{Line as RLine, Span as RSpan};
use ratatui::widgets::{Block as RBlock, BorderType as RBorderType, Borders, Padding as RPadding};

use crate::layout::Rect;
use crate::style::Style;

// ─── BorderType ───────────────────────────────────────────────────────────────

/// The visual style of a block border.
///
/// ```python
/// from pyratatui import Block, BorderType
///
/// block = Block().bordered().border_type(BorderType.Rounded)
/// ```
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum BorderType {
    Plain,
    Rounded,
    Double,
    Thick,
    QuadrantInside,
    QuadrantOutside,
    // New in ratatui 0.30:
    LightDoubleDashed,
    HeavyDoubleDashed,
    LightTripleDashed,
    HeavyTripleDashed,
    LightQuadrupleDashed,
    HeavyQuadrupleDashed,
}

impl BorderType {
    fn to_ratatui(&self) -> RBorderType {
        match self {
            BorderType::Plain => RBorderType::Plain,
            BorderType::Rounded => RBorderType::Rounded,
            BorderType::Double => RBorderType::Double,
            BorderType::Thick => RBorderType::Thick,
            BorderType::QuadrantInside => RBorderType::QuadrantInside,
            BorderType::QuadrantOutside => RBorderType::QuadrantOutside,
            BorderType::LightDoubleDashed => RBorderType::LightDoubleDashed,
            BorderType::HeavyDoubleDashed => RBorderType::HeavyDoubleDashed,
            BorderType::LightTripleDashed => RBorderType::LightTripleDashed,
            BorderType::HeavyTripleDashed => RBorderType::HeavyTripleDashed,
            BorderType::LightQuadrupleDashed => RBorderType::LightQuadrupleDashed,
            BorderType::HeavyQuadrupleDashed => RBorderType::HeavyQuadrupleDashed,
        }
    }
}

#[pymethods]
impl BorderType {
    fn __repr__(&self) -> String {
        format!("BorderType.{:?}", self)
    }
}

// ─── Block ────────────────────────────────────────────────────────────────────

/// A bordered container widget.
///
/// ```python
/// from pyratatui import Block, Style, Color, BorderType
///
/// block = (Block()
///     .title("My App")
///     .bordered()
///     .border_type(BorderType.Rounded)
///     .style(Style().fg(Color.cyan())))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Block {
    title: Option<String>,
    title_bottom: Option<String>,
    borders: u8, // bit flags: 1=top, 2=right, 4=bottom, 8=left
    border_type: BorderType,
    style: Option<Style>,
    title_style: Option<Style>,
    border_style: Option<Style>,
    padding_left: u16,
    padding_right: u16,
    padding_top: u16,
    padding_bottom: u16,
    title_alignment: String,
}

impl Block {
    fn borders_flags(&self) -> Borders {
        let mut b = Borders::NONE;
        if self.borders & 1 != 0 {
            b |= Borders::TOP;
        }
        if self.borders & 2 != 0 {
            b |= Borders::RIGHT;
        }
        if self.borders & 4 != 0 {
            b |= Borders::BOTTOM;
        }
        if self.borders & 8 != 0 {
            b |= Borders::LEFT;
        }
        b
    }

    pub(crate) fn to_ratatui(&self) -> RBlock<'static> {
        let mut block = RBlock::default()
            .borders(self.borders_flags())
            .border_type(self.border_type.to_ratatui())
            .padding(RPadding {
                left: self.padding_left,
                right: self.padding_right,
                top: self.padding_top,
                bottom: self.padding_bottom,
            });

        // ratatui 0.30: block::Title struct removed.
        // Use title_top(Line) with alignment set on the Line itself.
        if let Some(ref t) = self.title {
            let align = match self.title_alignment.as_str() {
                "center" => HorizontalAlignment::Center,
                "right" => HorizontalAlignment::Right,
                _ => HorizontalAlignment::Left,
            };
            let line: RLine<'static> = if let Some(ref ts) = self.title_style {
                RLine::from(RSpan::styled(t.clone(), ts.inner))
            } else {
                RLine::from(t.clone())
            };
            // title_top() positions the title on the top border.
            // title_alignment() still works as a block-level default.
            block = block.title_top(line).title_alignment(align);
        }

        if let Some(ref b) = self.title_bottom {
            let line: RLine<'static> = RLine::from(b.clone());
            block = block.title_bottom(line);
        }

        if let Some(ref s) = self.style {
            block = block.style(s.inner);
        }
        if let Some(ref s) = self.border_style {
            block = block.border_style(s.inner);
        }
        block
    }
}

impl Block {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Block {
    /// Create an empty block with no borders or title.
    #[new]
    pub fn new() -> Self {
        Self {
            title: None,
            title_bottom: None,
            borders: 0,
            border_type: BorderType::Plain,
            style: None,
            title_style: None,
            border_style: None,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
            padding_bottom: 0,
            title_alignment: "left".into(),
        }
    }

    /// Set the top title.  Returns `self` for chaining.
    pub fn title(&self, title: &str) -> Block {
        let mut b = self.clone();
        b.title = Some(title.to_string());
        b
    }

    /// Set a title at the bottom of the block.
    pub fn title_bottom(&self, title: &str) -> Block {
        let mut b = self.clone();
        b.title_bottom = Some(title.to_string());
        b
    }

    /// Enable all four borders.
    pub fn bordered(&self) -> Block {
        let mut b = self.clone();
        b.borders = 0b1111;
        b
    }

    /// Enable specific borders.
    #[pyo3(signature = (top=true, right=true, bottom=true, left=true))]
    pub fn borders(&self, top: bool, right: bool, bottom: bool, left: bool) -> Block {
        let mut b = self.clone();
        b.borders =
            (top as u8) | ((right as u8) << 1) | ((bottom as u8) << 2) | ((left as u8) << 3);
        b
    }

    /// Set the border visual type.
    pub fn border_type(&self, bt: &BorderType) -> Block {
        let mut b = self.clone();
        b.border_type = bt.clone();
        b
    }

    /// Set the block background / text style.
    pub fn style(&self, style: &Style) -> Block {
        let mut b = self.clone();
        b.style = Some(style.clone());
        b
    }

    /// Style applied to the title text.
    pub fn title_style(&self, style: &Style) -> Block {
        let mut b = self.clone();
        b.title_style = Some(style.clone());
        b
    }

    /// Style applied to the border characters.
    pub fn border_style(&self, style: &Style) -> Block {
        let mut b = self.clone();
        b.border_style = Some(style.clone());
        b
    }

    /// Set inner padding.
    #[pyo3(signature = (left=0, right=0, top=0, bottom=0))]
    pub fn padding(&self, left: u16, right: u16, top: u16, bottom: u16) -> Block {
        let mut b = self.clone();
        b.padding_left = left;
        b.padding_right = right;
        b.padding_top = top;
        b.padding_bottom = bottom;
        b
    }

    /// Set the horizontal alignment of the title ("left", "center", "right").
    pub fn title_alignment(&self, alignment: &str) -> Block {
        let mut b = self.clone();
        b.title_alignment = alignment.to_string();
        b
    }

    /// Return the inner area of the block for a given outer `area`.
    ///
    /// This subtracts the borders and padding to give the area available
    /// for content rendered inside the block.
    ///
    /// ```python
    /// block = Block().bordered().title("Example")
    /// inner = block.inner(frame.area)
    /// frame.render_widget(block, frame.area)
    /// frame.render_widget(content, inner)
    /// ```
    pub fn inner(&self, area: &Rect) -> Rect {
        let rblock = self.to_ratatui();
        Rect {
            inner: rblock.inner(area.inner),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Block(title={:?}, borders=0b{:04b})",
            self.title, self.borders
        )
    }
}

pub fn register_block(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Block>()?;
    m.add_class::<BorderType>()?;
    Ok(())
}
