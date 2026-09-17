//! Python bindings for ratatui's layout primitives.

use pyo3::prelude::*;
use ratatui::layout::{
    Constraint as RConstraint, Direction as RDirection, Flex, Layout as RLayout, Margin as RMargin,
    Rect as RRect,
};

use crate::errors::layout_err_to_py;

// ─── Rect ─────────────────────────────────────────────────────────────────────

/// A rectangular region on the terminal screen.
///
/// ```python
/// from pyratatui import Rect
///
/// r = Rect(0, 0, 80, 24)
/// inner = r.inner(1, 1)  # shrink by margin
/// ```
#[pyclass(module = "pyratatui", skip_from_py_object)]
#[derive(Clone, Debug, Copy)]
pub struct Rect {
    pub(crate) inner: RRect,
}

#[pymethods]
impl Rect {
    #[new]
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            inner: RRect {
                x,
                y,
                width,
                height,
            },
        }
    }

    #[getter]
    pub fn x(&self) -> u16 {
        self.inner.x
    }
    #[getter]
    pub fn y(&self) -> u16 {
        self.inner.y
    }
    #[getter]
    pub fn width(&self) -> u16 {
        self.inner.width
    }
    #[getter]
    pub fn height(&self) -> u16 {
        self.inner.height
    }
    #[getter]
    pub fn right(&self) -> u16 {
        self.inner.right()
    }
    #[getter]
    pub fn bottom(&self) -> u16 {
        self.inner.bottom()
    }
    #[getter]
    pub fn left(&self) -> u16 {
        self.inner.left()
    }
    #[getter]
    pub fn top(&self) -> u16 {
        self.inner.top()
    }

    pub fn area(&self) -> u32 {
        self.inner.area()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Return the rect shrunk by horizontal/vertical margins.
    #[pyo3(signature = (horizontal=1, vertical=1))]
    pub fn inner(&self, horizontal: u16, vertical: u16) -> Rect {
        Rect {
            inner: self.inner.inner(RMargin {
                horizontal,
                vertical,
            }),
        }
    }

    pub fn contains(&self, other: &Rect) -> bool {
        let (s, o) = (self.inner, other.inner);
        o.x >= s.x
            && o.y >= s.y
            && (o.x + o.width) <= (s.x + s.width)
            && (o.y + o.height) <= (s.y + s.height)
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let i = self.inner.intersection(other.inner);
        if i.is_empty() {
            None
        } else {
            Some(Rect { inner: i })
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            inner: self.inner.union(other.inner),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Rect(x={}, y={}, width={}, height={})",
            self.inner.x, self.inner.y, self.inner.width, self.inner.height
        )
    }
    fn __eq__(&self, other: &Rect) -> bool {
        self.inner == other.inner
    }
}

// ─── Constraint ───────────────────────────────────────────────────────────────

/// A sizing rule for layout children.
///
/// ```python
/// from pyratatui import Constraint
///
/// fixed = Constraint.length(20)
/// pct   = Constraint.percentage(50)
/// fill  = Constraint.fill(1)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Constraint {
    pub(crate) inner: RConstraint,
}

#[pymethods]
impl Constraint {
    #[staticmethod]
    pub fn length(n: u16) -> Self {
        Self {
            inner: RConstraint::Length(n),
        }
    }
    #[staticmethod]
    pub fn percentage(pct: u16) -> Self {
        Self {
            inner: RConstraint::Percentage(pct),
        }
    }
    #[staticmethod]
    pub fn fill(n: u16) -> Self {
        Self {
            inner: RConstraint::Fill(n),
        }
    }
    #[staticmethod]
    pub fn min(n: u16) -> Self {
        Self {
            inner: RConstraint::Min(n),
        }
    }
    #[staticmethod]
    pub fn max(n: u16) -> Self {
        Self {
            inner: RConstraint::Max(n),
        }
    }
    #[staticmethod]
    pub fn ratio(num: u32, den: u32) -> Self {
        Self {
            inner: RConstraint::Ratio(num, den),
        }
    }
    fn __repr__(&self) -> String {
        format!("Constraint({:?})", self.inner)
    }
}

// ─── Direction ────────────────────────────────────────────────────────────────

/// Split axis for a Layout.
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub(crate) fn to_ratatui(&self) -> RDirection {
        match self {
            Direction::Horizontal => RDirection::Horizontal,
            Direction::Vertical => RDirection::Vertical,
        }
    }
}

// ─── Alignment ────────────────────────────────────────────────────────────────

/// Horizontal alignment for text and widgets.
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

// ─── Layout ───────────────────────────────────────────────────────────────────

/// A layout engine that splits a `Rect` into child `Rect`s.
///
/// ```python
/// from pyratatui import Layout, Constraint, Direction, Rect
///
/// area = Rect(0, 0, 80, 24)
/// header, body, footer = (
///     Layout()
///     .direction(Direction.Vertical)
///     .constraints([Constraint.length(3), Constraint.fill(1), Constraint.length(1)])
///     .split(area)
/// )
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Layout {
    constraints: Vec<Constraint>,
    direction: Direction,
    margin: u16,
    spacing: u16,
    flex: String,
}

#[pymethods]
impl Layout {
    #[new]
    pub fn new() -> Self {
        Self {
            constraints: vec![],
            direction: Direction::Vertical,
            margin: 0,
            spacing: 0,
            flex: "start".into(),
        }
    }

    pub fn constraints(&self, constraints: Vec<PyRef<Constraint>>) -> Layout {
        let mut l = self.clone();
        l.constraints = constraints.iter().map(|c| (**c).clone()).collect();
        l
    }
    pub fn direction(&self, direction: &Direction) -> Layout {
        let mut l = self.clone();
        l.direction = direction.clone();
        l
    }
    pub fn margin(&self, margin: u16) -> Layout {
        let mut l = self.clone();
        l.margin = margin;
        l
    }
    pub fn spacing(&self, spacing: u16) -> Layout {
        let mut l = self.clone();
        l.spacing = spacing;
        l
    }
    /// Flex mode: "start" | "end" | "center" | "space_between" | "space_around".
    pub fn flex_mode(&self, mode: &str) -> Layout {
        let mut l = self.clone();
        l.flex = mode.to_string();
        l
    }

    /// Split `area` and return a list of `Rect` — one per constraint.
    pub fn split(&self, area: &Rect) -> PyResult<Vec<Rect>> {
        if self.constraints.is_empty() {
            return Err(layout_err_to_py("No constraints set on Layout"));
        }
        let rc: Vec<RConstraint> = self.constraints.iter().map(|c| c.inner).collect();
        let flex = match self.flex.as_str() {
            "end" => Flex::End,
            "center" => Flex::Center,
            "space_between" => Flex::SpaceBetween,
            "space_around" => Flex::SpaceAround,
            _ => Flex::Start,
        };
        let layout = RLayout::default()
            .direction(self.direction.to_ratatui())
            .constraints(rc)
            .margin(self.margin)
            .spacing(self.spacing)
            .flex(flex);

        Ok(layout
            .split(area.inner)
            .iter()
            .map(|r| Rect { inner: *r })
            .collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "Layout(direction={:?}, constraints={}, margin={})",
            self.direction,
            self.constraints.len(),
            self.margin
        )
    }
}

// ─── Module registration ───────────────────────────────────────────────────────

pub fn register_layout(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Rect>()?;
    m.add_class::<Constraint>()?;
    m.add_class::<Direction>()?;
    m.add_class::<Alignment>()?;
    m.add_class::<Layout>()?;
    Ok(())
}
