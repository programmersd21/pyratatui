//! Canvas and map widgets.
//!
//! `Canvas` collects simple shapes (lines, points, rectangles) and renders
//! them through ratatui's canvas widget. `Map` renders a world map.

use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols::Marker as RMarker,
    widgets::{
        Widget,
        canvas::{
            Canvas as RCanvas, Line, Map as RMap, MapResolution as RMapResolution, Points,
            Rectangle,
        },
    },
};

#[derive(Clone, Debug)]
enum Shape {
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    Point { x: f64, y: f64 },
    Rect { x: f64, y: f64, w: f64, h: f64 },
}

/// A drawable canvas: collect shapes, then render with `frame.render_widget`.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug, Default)]
pub struct Canvas {
    width: u16,
    height: u16,
    shapes: Vec<Shape>,
}

impl Canvas {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: Rect) -> PyResult<()> {
        frame.render_widget(self, area);
        Ok(())
    }
}

#[pymethods]
impl Canvas {
    #[new]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            shapes: Vec::new(),
        }
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.shapes.push(Shape::Line { x1, y1, x2, y2 });
    }

    pub fn draw_point(&mut self, x: f64, y: f64) {
        self.shapes.push(Shape::Point { x, y });
    }

    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.shapes.push(Shape::Rect { x, y, w, h });
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    #[getter]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "Canvas(width={}, height={}, shapes={})",
            self.width,
            self.height,
            self.shapes.len()
        )
    }
}

impl Widget for &Canvas {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x_bounds = [0.0, f64::from(self.width.max(1))];
        let y_bounds = [0.0, f64::from(self.height.max(1))];
        RCanvas::default()
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .paint(|ctx| {
                for shape in &self.shapes {
                    match shape {
                        Shape::Line { x1, y1, x2, y2 } => ctx.draw(&Line {
                            x1: *x1,
                            y1: *y1,
                            x2: *x2,
                            y2: *y2,
                            color: Color::White,
                        }),
                        Shape::Point { x, y } => {
                            ctx.draw(&Points {
                                coords: &[(*x, *y)],
                                color: Color::White,
                            });
                        }
                        Shape::Rect { x, y, w, h } => ctx.draw(&Rectangle {
                            x: *x,
                            y: *y,
                            width: *w,
                            height: *h,
                            color: Color::White,
                        }),
                    }
                }
            })
            .render(area, buf);
    }
}

/// Detail level of the world map.
#[pyclass(module = "pyratatui", eq, eq_int, from_py_object)]
#[derive(Clone, Debug, PartialEq)]
pub enum MapResolution {
    Low,
    High,
}

impl MapResolution {
    fn to_ratatui(&self) -> RMapResolution {
        match self {
            MapResolution::Low => RMapResolution::Low,
            MapResolution::High => RMapResolution::High,
        }
    }
}

/// A world map rendered onto a canvas.
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Map {
    resolution: MapResolution,
}

impl Map {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: Rect) -> PyResult<()> {
        frame.render_widget(self, area);
        Ok(())
    }
}

#[pymethods]
impl Map {
    #[new]
    #[pyo3(signature = (resolution=None))]
    pub fn new(resolution: Option<MapResolution>) -> Self {
        Self {
            resolution: resolution.unwrap_or(MapResolution::Low),
        }
    }

    pub fn resolution(&self, resolution: MapResolution) -> Self {
        Self { resolution }
    }

    fn __repr__(&self) -> String {
        format!("Map(resolution={:?})", self.resolution)
    }
}

impl Widget for &Map {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let resolution = self.resolution.to_ratatui();
        let marker = match resolution {
            RMapResolution::Low => RMarker::Dot,
            RMapResolution::High => RMarker::Braille,
        };
        RCanvas::default()
            .marker(marker)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&RMap {
                    resolution,
                    color: Color::White,
                });
            })
            .render(area, buf);
    }
}

pub fn register_canvas(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Canvas>()?;
    m.add_class::<Map>()?;
    m.add_class::<MapResolution>()?;
    Ok(())
}
