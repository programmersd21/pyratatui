use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Alignment as RAlignment;
use ratatui::layout::Rect as RRect;
use ratatui::symbols::Marker as RMarker;
use ratatui::widgets::{
    Axis as RAxis, Chart as RChart, Dataset as RDataset, GraphType as RGraphType,
    LegendPosition as RLegendPosition,
};

use crate::layout::Constraint;
use crate::style::Style;
use crate::widgets::Block;

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct GraphType {
    inner: RGraphType,
}

#[pymethods]
impl GraphType {
    #[classattr]
    #[allow(non_snake_case)]
    pub fn Scatter() -> Self {
        Self {
            inner: RGraphType::Scatter,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Line() -> Self {
        Self {
            inner: RGraphType::Line,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Bar() -> Self {
        Self {
            inner: RGraphType::Bar,
        }
    }

    fn __repr__(&self) -> String {
        format!("GraphType({:?})", self.inner)
    }
}

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Marker {
    inner: RMarker,
}

#[pymethods]
impl Marker {
    #[classattr]
    #[allow(non_snake_case)]
    pub fn Dot() -> Self {
        Self {
            inner: RMarker::Dot,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Block() -> Self {
        Self {
            inner: RMarker::Block,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Bar() -> Self {
        Self {
            inner: RMarker::Bar,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Braille() -> Self {
        Self {
            inner: RMarker::Braille,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn HalfBlock() -> Self {
        Self {
            inner: RMarker::HalfBlock,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Quadrant() -> Self {
        Self {
            inner: RMarker::Quadrant,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Sextant() -> Self {
        Self {
            inner: RMarker::Sextant,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Octant() -> Self {
        Self {
            inner: RMarker::Octant,
        }
    }

    fn __repr__(&self) -> String {
        format!("Marker({:?})", self.inner)
    }
}

#[pyclass(module = "pyratatui", skip_from_py_object)]
#[derive(Clone, Copy, Debug)]
pub struct LegendPosition {
    inner: RLegendPosition,
}

#[pymethods]
impl LegendPosition {
    #[classattr]
    #[allow(non_snake_case)]
    pub fn Top() -> Self {
        Self {
            inner: RLegendPosition::Top,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn TopRight() -> Self {
        Self {
            inner: RLegendPosition::TopRight,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn TopLeft() -> Self {
        Self {
            inner: RLegendPosition::TopLeft,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Left() -> Self {
        Self {
            inner: RLegendPosition::Left,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Right() -> Self {
        Self {
            inner: RLegendPosition::Right,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn Bottom() -> Self {
        Self {
            inner: RLegendPosition::Bottom,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn BottomRight() -> Self {
        Self {
            inner: RLegendPosition::BottomRight,
        }
    }

    #[classattr]
    #[allow(non_snake_case)]
    pub fn BottomLeft() -> Self {
        Self {
            inner: RLegendPosition::BottomLeft,
        }
    }

    fn __repr__(&self) -> String {
        format!("LegendPosition({:?})", self.inner)
    }
}

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Axis {
    title: Option<String>,
    bounds: [f64; 2],
    labels: Vec<String>,
    style: Option<Style>,
    labels_alignment: String,
}

impl Axis {
    fn to_ratatui(&self) -> PyResult<RAxis<'_>> {
        let mut axis = RAxis::default().bounds(self.bounds);
        if let Some(ref title) = self.title {
            axis = axis.title(title.clone());
        }
        if !self.labels.is_empty() {
            axis = axis.labels(self.labels.clone());
        }
        if let Some(ref style) = self.style {
            axis = axis.style(style.inner);
        }
        axis = axis.labels_alignment(parse_alignment(&self.labels_alignment)?);
        Ok(axis)
    }
}

#[pymethods]
impl Axis {
    #[new]
    pub fn new() -> Self {
        Self {
            title: None,
            bounds: [0.0, 0.0],
            labels: Vec::new(),
            style: None,
            labels_alignment: "left".to_string(),
        }
    }

    pub fn title(&self, title: &str) -> Self {
        let mut next = self.clone();
        next.title = Some(title.to_string());
        next
    }

    pub fn bounds(&self, min: f64, max: f64) -> Self {
        let mut next = self.clone();
        next.bounds = [min, max];
        next
    }

    pub fn labels(&self, labels: Vec<String>) -> Self {
        let mut next = self.clone();
        next.labels = labels;
        next
    }

    pub fn style(&self, style: &Style) -> Self {
        let mut next = self.clone();
        next.style = Some(style.clone());
        next
    }

    pub fn labels_alignment(&self, alignment: &str) -> PyResult<Self> {
        parse_alignment(alignment)?;
        let mut next = self.clone();
        next.labels_alignment = normalize(alignment);
        Ok(next)
    }

    fn __repr__(&self) -> String {
        format!("Axis(bounds=[{}, {}])", self.bounds[0], self.bounds[1])
    }
}

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Dataset {
    name: Option<String>,
    data: Vec<(f64, f64)>,
    marker: Marker,
    graph_type: GraphType,
    style: Option<Style>,
}

impl Dataset {
    fn to_ratatui(&self) -> RDataset<'_> {
        let mut dataset = RDataset::default()
            .data(self.data.as_slice())
            .marker(self.marker.inner)
            .graph_type(self.graph_type.inner);
        if let Some(ref name) = self.name {
            dataset = dataset.name(name.clone());
        }
        if let Some(ref style) = self.style {
            dataset = dataset.style(style.inner);
        }
        dataset
    }
}

#[pymethods]
impl Dataset {
    #[new]
    pub fn new(data: Vec<(f64, f64)>) -> Self {
        Self {
            name: None,
            data,
            marker: Marker::Dot(),
            graph_type: GraphType::Scatter(),
            style: None,
        }
    }

    pub fn name(&self, name: &str) -> Self {
        let mut next = self.clone();
        next.name = Some(name.to_string());
        next
    }

    pub fn data(&self, data: Vec<(f64, f64)>) -> Self {
        let mut next = self.clone();
        next.data = data;
        next
    }

    pub fn marker(&self, marker: Marker) -> Self {
        let mut next = self.clone();
        next.marker = marker;
        next
    }

    pub fn graph_type(&self, graph_type: GraphType) -> Self {
        let mut next = self.clone();
        next.graph_type = graph_type;
        next
    }

    pub fn style(&self, style: &Style) -> Self {
        let mut next = self.clone();
        next.style = Some(style.clone());
        next
    }

    fn __repr__(&self) -> String {
        format!(
            "Dataset(points={}, graph_type={:?})",
            self.data.len(),
            self.graph_type.inner
        )
    }
}

#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Chart {
    datasets: Vec<Dataset>,
    block: Option<Block>,
    style: Option<Style>,
    x_axis: Axis,
    y_axis: Axis,
    hidden_legend_constraints: Option<(Constraint, Constraint)>,
    legend_position: Option<LegendPosition>,
}

impl Chart {
    pub(crate) fn to_ratatui(&self) -> PyResult<RChart<'_>> {
        let datasets = self.datasets.iter().map(Dataset::to_ratatui).collect();
        let mut chart = RChart::new(datasets)
            .x_axis(self.x_axis.to_ratatui()?)
            .y_axis(self.y_axis.to_ratatui()?);

        if let Some(ref block) = self.block {
            chart = chart.block(block.to_ratatui());
        }
        if let Some(ref style) = self.style {
            chart = chart.style(style.inner);
        }
        if let Some((ref width, ref height)) = self.hidden_legend_constraints {
            chart = chart.hidden_legend_constraints((width.inner, height.inner));
        }
        chart = chart.legend_position(self.legend_position.map(|pos| pos.inner));
        Ok(chart)
    }
}

impl Chart {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui()?, area);
        Ok(())
    }
}

#[pymethods]
impl Chart {
    #[new]
    pub fn new(datasets: Vec<Dataset>) -> Self {
        Self {
            datasets,
            block: None,
            style: None,
            x_axis: Axis::new(),
            y_axis: Axis::new(),
            hidden_legend_constraints: None,
            legend_position: Some(LegendPosition::TopRight()),
        }
    }

    pub fn datasets(&self, datasets: Vec<Dataset>) -> Self {
        let mut next = self.clone();
        next.datasets = datasets;
        next
    }

    pub fn add_dataset(&self, dataset: &Dataset) -> Self {
        let mut next = self.clone();
        next.datasets.push(dataset.clone());
        next
    }

    pub fn block(&self, block: &Block) -> Self {
        let mut next = self.clone();
        next.block = Some(block.clone());
        next
    }

    pub fn style(&self, style: &Style) -> Self {
        let mut next = self.clone();
        next.style = Some(style.clone());
        next
    }

    pub fn x_axis(&self, axis: &Axis) -> Self {
        let mut next = self.clone();
        next.x_axis = axis.clone();
        next
    }

    pub fn y_axis(&self, axis: &Axis) -> Self {
        let mut next = self.clone();
        next.y_axis = axis.clone();
        next
    }

    pub fn hidden_legend_constraints(&self, width: &Constraint, height: &Constraint) -> Self {
        let mut next = self.clone();
        next.hidden_legend_constraints = Some((width.clone(), height.clone()));
        next
    }

    #[pyo3(signature = (position=None))]
    pub fn legend_position(&self, position: Option<&LegendPosition>) -> Self {
        let mut next = self.clone();
        next.legend_position = position.copied();
        next
    }

    fn __repr__(&self) -> String {
        format!("Chart(datasets={})", self.datasets.len())
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn parse_alignment(value: &str) -> PyResult<RAlignment> {
    match normalize(value).as_str() {
        "left" => Ok(RAlignment::Left),
        "center" => Ok(RAlignment::Center),
        "right" => Ok(RAlignment::Right),
        _ => Err(PyValueError::new_err(
            "Invalid labels_alignment. Use 'left', 'center', or 'right'",
        )),
    }
}

pub fn register_chart(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GraphType>()?;
    m.add_class::<Marker>()?;
    m.add_class::<LegendPosition>()?;
    m.add_class::<Axis>()?;
    m.add_class::<Dataset>()?;
    m.add_class::<Chart>()?;
    Ok(())
}
