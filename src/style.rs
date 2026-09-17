//! Python bindings for ratatui's style primitives.
//!
//! Exposes:
//! - `Color` — terminal colour (named, indexed, RGB, Reset)
//! - `Modifier` — text modifier flags (bold, italic, underlined, …)
//! - `Style` — combined colour + modifier descriptor
//! - `Stylize` mixin applied as builder methods on `Style`

use pyo3::prelude::*;
use ratatui::style::{Color as RColor, Modifier as RModifier, Style as RStyle};

// ─── Color ────────────────────────────────────────────────────────────────────

/// A terminal colour value.
///
/// Can be constructed with named constants, an index (0-255), or RGB triplet:
///
/// ```python
/// from pyratatui import Color
/// c1 = Color.red()
/// c2 = Color.indexed(196)
/// c3 = Color.rgb(255, 128, 0)
/// c4 = Color.reset()
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Color {
    pub(crate) inner: RColor,
}

#[pymethods]
impl Color {
    /// The terminal default/reset colour.
    #[staticmethod]
    pub fn reset() -> Self {
        Self {
            inner: RColor::Reset,
        }
    }

    #[staticmethod]
    pub fn black() -> Self {
        Self {
            inner: RColor::Black,
        }
    }
    #[staticmethod]
    pub fn red() -> Self {
        Self { inner: RColor::Red }
    }
    #[staticmethod]
    pub fn green() -> Self {
        Self {
            inner: RColor::Green,
        }
    }
    #[staticmethod]
    pub fn yellow() -> Self {
        Self {
            inner: RColor::Yellow,
        }
    }
    #[staticmethod]
    pub fn blue() -> Self {
        Self {
            inner: RColor::Blue,
        }
    }
    #[staticmethod]
    pub fn magenta() -> Self {
        Self {
            inner: RColor::Magenta,
        }
    }
    #[staticmethod]
    pub fn cyan() -> Self {
        Self {
            inner: RColor::Cyan,
        }
    }
    #[staticmethod]
    pub fn gray() -> Self {
        Self {
            inner: RColor::Gray,
        }
    }
    #[staticmethod]
    pub fn dark_gray() -> Self {
        Self {
            inner: RColor::DarkGray,
        }
    }
    #[staticmethod]
    pub fn light_red() -> Self {
        Self {
            inner: RColor::LightRed,
        }
    }
    #[staticmethod]
    pub fn light_green() -> Self {
        Self {
            inner: RColor::LightGreen,
        }
    }
    #[staticmethod]
    pub fn light_yellow() -> Self {
        Self {
            inner: RColor::LightYellow,
        }
    }
    #[staticmethod]
    pub fn light_blue() -> Self {
        Self {
            inner: RColor::LightBlue,
        }
    }
    #[staticmethod]
    pub fn light_magenta() -> Self {
        Self {
            inner: RColor::LightMagenta,
        }
    }
    #[staticmethod]
    pub fn light_cyan() -> Self {
        Self {
            inner: RColor::LightCyan,
        }
    }
    #[staticmethod]
    pub fn white() -> Self {
        Self {
            inner: RColor::White,
        }
    }

    /// 256-colour indexed palette entry.
    #[staticmethod]
    pub fn indexed(index: u8) -> Self {
        Self {
            inner: RColor::Indexed(index),
        }
    }

    /// True-colour RGB value.
    #[staticmethod]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            inner: RColor::Rgb(r, g, b),
        }
    }

    fn __repr__(&self) -> String {
        format!("Color({:?})", self.inner)
    }

    fn __eq__(&self, other: &Color) -> bool {
        self.inner == other.inner
    }
}

// ─── Modifier ─────────────────────────────────────────────────────────────────

/// Bitfield of text modifiers (bold, italic, underlined, …).
///
/// ```python
/// from pyratatui import Modifier
/// m = Modifier.bold() | Modifier.italic()
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Modifier {
    pub(crate) inner: RModifier,
}

#[pymethods]
impl Modifier {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RModifier::empty(),
        }
    }

    #[staticmethod]
    pub fn bold() -> Self {
        Self {
            inner: RModifier::BOLD,
        }
    }
    #[staticmethod]
    pub fn dim() -> Self {
        Self {
            inner: RModifier::DIM,
        }
    }
    #[staticmethod]
    pub fn italic() -> Self {
        Self {
            inner: RModifier::ITALIC,
        }
    }
    #[staticmethod]
    pub fn underlined() -> Self {
        Self {
            inner: RModifier::UNDERLINED,
        }
    }
    #[staticmethod]
    pub fn slow_blink() -> Self {
        Self {
            inner: RModifier::SLOW_BLINK,
        }
    }
    #[staticmethod]
    pub fn rapid_blink() -> Self {
        Self {
            inner: RModifier::RAPID_BLINK,
        }
    }
    #[staticmethod]
    pub fn reversed() -> Self {
        Self {
            inner: RModifier::REVERSED,
        }
    }
    #[staticmethod]
    pub fn hidden() -> Self {
        Self {
            inner: RModifier::HIDDEN,
        }
    }
    #[staticmethod]
    pub fn crossed_out() -> Self {
        Self {
            inner: RModifier::CROSSED_OUT,
        }
    }

    /// Bitwise OR — combine modifiers.
    pub fn __or__(&self, other: &Modifier) -> Modifier {
        Modifier {
            inner: self.inner | other.inner,
        }
    }

    /// Bitwise AND — intersect modifiers.
    pub fn __and__(&self, other: &Modifier) -> Modifier {
        Modifier {
            inner: self.inner & other.inner,
        }
    }

    fn __repr__(&self) -> String {
        format!("Modifier({:?})", self.inner)
    }
}

// ─── Style ────────────────────────────────────────────────────────────────────

/// A complete style descriptor: foreground colour, background colour, modifiers.
///
/// All builder methods return `self` for fluent chaining:
///
/// ```python
/// from pyratatui import Style, Color, Modifier
///
/// style = (Style()
///     .fg(Color.red())
///     .bg(Color.black())
///     .add_modifier(Modifier.bold()))
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Style {
    pub(crate) inner: RStyle,
}

#[pymethods]
impl Style {
    /// Create a new empty `Style`.
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RStyle::default(),
        }
    }

    /// Set the foreground colour.
    pub fn fg(&self, color: &Color) -> Style {
        Style {
            inner: self.inner.fg(color.inner),
        }
    }

    /// Set the background colour.
    pub fn bg(&self, color: &Color) -> Style {
        Style {
            inner: self.inner.bg(color.inner),
        }
    }

    /// Add a modifier (bold, italic, …).
    pub fn add_modifier(&self, modifier: &Modifier) -> Style {
        Style {
            inner: self.inner.add_modifier(modifier.inner),
        }
    }

    /// Remove a modifier.
    pub fn remove_modifier(&self, modifier: &Modifier) -> Style {
        Style {
            inner: self.inner.remove_modifier(modifier.inner),
        }
    }

    /// Patch this style with another, overriding only set fields.
    pub fn patch(&self, other: &Style) -> Style {
        Style {
            inner: self.inner.patch(other.inner),
        }
    }

    // Convenience shortcuts matching ratatui's Stylize trait.
    pub fn bold(&self) -> Style {
        self.add_modifier(&Modifier::bold())
    }
    pub fn italic(&self) -> Style {
        self.add_modifier(&Modifier::italic())
    }
    pub fn underlined(&self) -> Style {
        self.add_modifier(&Modifier::underlined())
    }
    pub fn dim(&self) -> Style {
        self.add_modifier(&Modifier::dim())
    }
    pub fn reversed(&self) -> Style {
        self.add_modifier(&Modifier::reversed())
    }
    pub fn hidden(&self) -> Style {
        self.add_modifier(&Modifier::hidden())
    }
    pub fn crossed_out(&self) -> Style {
        self.add_modifier(&Modifier::crossed_out())
    }
    pub fn slow_blink(&self) -> Style {
        self.add_modifier(&Modifier::slow_blink())
    }
    pub fn rapid_blink(&self) -> Style {
        self.add_modifier(&Modifier::rapid_blink())
    }

    /// Return the foreground colour, or `None` if unset.
    #[getter]
    pub fn foreground(&self) -> Option<Color> {
        self.inner.fg.map(|c| Color { inner: c })
    }

    /// Return the background colour, or `None` if unset.
    #[getter]
    pub fn background(&self) -> Option<Color> {
        self.inner.bg.map(|c| Color { inner: c })
    }

    fn __repr__(&self) -> String {
        format!(
            "Style(fg={:?}, bg={:?}, mods={:?})",
            self.inner.fg, self.inner.bg, self.inner.add_modifier
        )
    }

    fn __eq__(&self, other: &Style) -> bool {
        self.inner == other.inner
    }
}

// ─── Module registration ───────────────────────────────────────────────────────

pub fn register_style(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Color>()?;
    m.add_class::<Modifier>()?;
    m.add_class::<Style>()?;
    Ok(())
}
