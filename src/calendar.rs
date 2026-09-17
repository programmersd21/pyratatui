//! Python bindings for ratatui's built-in Calendar widget.
//!
//! Exposes:
//! - `CalendarDate`       — wraps `time::Date` (year/month/day)
//! - `CalendarEventStore` — HashMap-based `DateStyler` for marking events
//! - `Monthly`            — the monthly calendar widget
//!
//! Requires the `widget-calendar` feature on ratatui.
//! Also requires the `time` crate as a direct dependency.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ratatui::Frame as RFrame;
use ratatui::layout::Rect as RRect;
use ratatui::widgets::calendar::{CalendarEventStore as REventStore, Monthly as RMonthly};
use time::{Date as TDate, Month as TMonth, OffsetDateTime};

use crate::style::Style;
use crate::widgets::Block;

// ─── CalendarDate ─────────────────────────────────────────────────────────────

/// A calendar date (year, month, day) used with the Monthly calendar widget.
///
/// ```python
/// from pyratatui import CalendarDate
///
/// d = CalendarDate.from_ymd(2024, 3, 15)   # March 15, 2024
/// t = CalendarDate.today()                   # today's UTC date
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CalendarDate {
    pub(crate) inner: TDate,
}

#[pymethods]
impl CalendarDate {
    /// Construct a date from year, month (1-12), and day.
    ///
    /// Raises `ValueError` if the combination is invalid.
    #[staticmethod]
    pub fn from_ymd(year: i32, month: u8, day: u8) -> PyResult<Self> {
        let m = int_to_month(month)?;
        TDate::from_calendar_date(year, m, day)
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(format!("Invalid date: {e}")))
    }

    /// Today's date in UTC.
    #[staticmethod]
    pub fn today() -> Self {
        Self {
            inner: OffsetDateTime::now_utc().date(),
        }
    }

    /// Year component.
    #[getter]
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Month component (1-12).
    #[getter]
    pub fn month(&self) -> u8 {
        self.inner.month() as u8
    }

    /// Day component.
    #[getter]
    pub fn day(&self) -> u8 {
        self.inner.day()
    }

    fn __repr__(&self) -> String {
        format!(
            "CalendarDate({}-{:02}-{:02})",
            self.inner.year(),
            self.inner.month() as u8,
            self.inner.day()
        )
    }

    fn __str__(&self) -> String {
        format!(
            "{}-{:02}-{:02}",
            self.inner.year(),
            self.inner.month() as u8,
            self.inner.day()
        )
    }

    fn __eq__(&self, other: &CalendarDate) -> bool {
        self.inner == other.inner
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.inner.hash(&mut h);
        h.finish()
    }
}

fn int_to_month(m: u8) -> PyResult<TMonth> {
    match m {
        1 => Ok(TMonth::January),
        2 => Ok(TMonth::February),
        3 => Ok(TMonth::March),
        4 => Ok(TMonth::April),
        5 => Ok(TMonth::May),
        6 => Ok(TMonth::June),
        7 => Ok(TMonth::July),
        8 => Ok(TMonth::August),
        9 => Ok(TMonth::September),
        10 => Ok(TMonth::October),
        11 => Ok(TMonth::November),
        12 => Ok(TMonth::December),
        _ => Err(PyValueError::new_err(format!(
            "Month must be 1-12, got {m}"
        ))),
    }
}

// ─── CalendarEventStore ───────────────────────────────────────────────────────

/// A store for calendar events: maps dates to display styles.
///
/// Implements the `DateStyler` trait so it can be passed to `Monthly`.
///
/// ```python
/// from pyratatui import CalendarDate, CalendarEventStore, Style, Color
///
/// store = CalendarEventStore()
/// store.add(CalendarDate.from_ymd(2024, 3, 15), Style().fg(Color.red()).bold())
/// store.add_today(Style().fg(Color.green()).bold())
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug, Default)]
pub struct CalendarEventStore {
    pub(crate) inner: REventStore,
}

#[pymethods]
impl CalendarEventStore {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: REventStore::default(),
        }
    }

    /// Mark a date with a style.
    pub fn add(&mut self, date: &CalendarDate, style: &Style) {
        self.inner.add(date.inner, style.inner);
    }

    /// Highlight today's date with the given style.
    pub fn add_today(&mut self, style: &Style) {
        let today = OffsetDateTime::now_utc().date();
        self.inner.add(today, style.inner);
    }

    /// Return a new store with today highlighted in the given style.
    #[staticmethod]
    pub fn today_highlighted(style: &Style) -> Self {
        Self {
            inner: REventStore::today(style.inner),
        }
    }

    fn __repr__(&self) -> String {
        format!("CalendarEventStore(<{} events>)", self.inner.0.len())
    }
}

// ─── Monthly ──────────────────────────────────────────────────────────────────

/// Monthly calendar widget.
///
/// Displays a single calendar month with optional surrounding days, weekday
/// headers, a month/year header, and per-day event styling.
///
/// ```python
/// from pyratatui import CalendarDate, CalendarEventStore, Monthly, Style, Color
///
/// today = CalendarDate.today()
/// store = CalendarEventStore.today_highlighted(Style().fg(Color.green()).bold())
///
/// calendar = (Monthly(today, store)
///     .show_month_header(Style().bold())
///     .show_weekdays_header(Style().italic())
///     .show_surrounding(Style().dim()))
///
/// # Render via frame.render_widget(calendar, area)
/// ```
#[pyclass(module = "pyratatui", from_py_object)]
#[derive(Clone, Debug)]
pub struct Monthly {
    display_date: CalendarDate,
    events: CalendarEventStore,
    block: Option<Block>,
    default_style: Option<Style>,
    show_surrounding: Option<Style>,
    show_month_header: Option<Style>,
    show_weekdays_header: Option<Style>,
}

impl Monthly {
    /// Build the underlying ratatui `Monthly` widget.
    pub(crate) fn to_ratatui(&self) -> RMonthly<'static, REventStore> {
        let mut w = RMonthly::new(self.display_date.inner, self.events.inner.clone());

        if let Some(ref s) = self.default_style {
            w = w.default_style(s.inner);
        }
        if let Some(ref s) = self.show_surrounding {
            w = w.show_surrounding(s.inner);
        }
        if let Some(ref s) = self.show_month_header {
            w = w.show_month_header(s.inner);
        }
        if let Some(ref s) = self.show_weekdays_header {
            w = w.show_weekdays_header(s.inner);
        }
        if let Some(ref b) = self.block {
            w = w.block(b.to_ratatui());
        }
        w
    }
}

impl Monthly {
    pub(crate) fn render_raw(&self, frame: &mut RFrame<'_>, area: RRect) -> PyResult<()> {
        frame.render_widget(self.to_ratatui(), area);
        Ok(())
    }
}

#[pymethods]
impl Monthly {
    /// Create a monthly calendar for the month containing `display_date`.
    ///
    /// Args:
    ///     display_date: Any date in the month to display.
    ///     events:       Event store that provides per-day styles.
    #[new]
    pub fn new(display_date: &CalendarDate, events: &CalendarEventStore) -> Self {
        Self {
            display_date: display_date.clone(),
            events: events.clone(),
            block: None,
            default_style: None,
            show_surrounding: None,
            show_month_header: None,
            show_weekdays_header: None,
        }
    }

    /// Wrap the calendar in a `Block`.
    pub fn block(&self, block: &Block) -> Self {
        let mut c = self.clone();
        c.block = Some(block.clone());
        c
    }

    /// Set the default style applied to all un-styled days.
    pub fn default_style(&self, style: &Style) -> Self {
        let mut c = self.clone();
        c.default_style = Some(style.clone());
        c
    }

    /// Show dates from the preceding/following months in `style`.
    /// Pass `Style()` (default) to show them unstyled.
    pub fn show_surrounding(&self, style: &Style) -> Self {
        let mut c = self.clone();
        c.show_surrounding = Some(style.clone());
        c
    }

    /// Show the month/year header line in `style`.
    pub fn show_month_header(&self, style: &Style) -> Self {
        let mut c = self.clone();
        c.show_month_header = Some(style.clone());
        c
    }

    /// Show the weekday abbreviations header (Su Mo Tu …) in `style`.
    pub fn show_weekdays_header(&self, style: &Style) -> Self {
        let mut c = self.clone();
        c.show_weekdays_header = Some(style.clone());
        c
    }

    fn __repr__(&self) -> String {
        format!(
            "Monthly({}-{:02})",
            self.display_date.inner.year(),
            self.display_date.inner.month() as u8
        )
    }
}

// ─── Registration ─────────────────────────────────────────────────────────────

pub fn register_calendar(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CalendarDate>()?;
    m.add_class::<CalendarEventStore>()?;
    m.add_class::<Monthly>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_dates_roundtrip() {
        let d = CalendarDate::from_ymd(2024, 3, 15).unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (2024, 3, 15));
    }

    #[test]
    fn invalid_dates_err() {
        assert!(CalendarDate::from_ymd(2024, 2, 30).is_err());
        assert!(CalendarDate::from_ymd(2024, 13, 1).is_err());
        assert!(CalendarDate::from_ymd(2024, 0, 10).is_err());
    }

    #[test]
    fn today_matches_time_crate() {
        let today = OffsetDateTime::now_utc().date();
        let d = CalendarDate::today();
        assert_eq!(
            (d.year(), d.month(), d.day()),
            (today.year(), today.month() as u8, today.day())
        );
    }
}
