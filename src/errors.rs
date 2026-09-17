//! Custom Python exception hierarchy for pyratatui.
//!
//! All exceptions derive from `PyratatuiError` so callers can catch them
//! at any granularity they choose.

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

// Root exception
create_exception!(
    pyratatui,
    PyratatuiError,
    PyException,
    "Base class for all pyratatui errors."
);

// Domain-specific exceptions
create_exception!(
    pyratatui,
    BackendError,
    PyratatuiError,
    "Raised when the terminal backend encounters an I/O or init error."
);

create_exception!(
    pyratatui,
    LayoutError,
    PyratatuiError,
    "Raised when layout constraints cannot be satisfied."
);

create_exception!(
    pyratatui,
    RenderError,
    PyratatuiError,
    "Raised when a render/draw cycle fails."
);

create_exception!(
    pyratatui,
    AsyncError,
    PyratatuiError,
    "Raised for async runtime or callback bridging failures."
);

create_exception!(
    pyratatui,
    StyleError,
    PyratatuiError,
    "Raised when an invalid style value is provided."
);

/// Register all exceptions onto the top-level module.
pub fn register_errors(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("PyratatuiError", py.get_type::<PyratatuiError>())?;
    m.add("BackendError", py.get_type::<BackendError>())?;
    m.add("LayoutError", py.get_type::<LayoutError>())?;
    m.add("RenderError", py.get_type::<RenderError>())?;
    m.add("AsyncError", py.get_type::<AsyncError>())?;
    m.add("StyleError", py.get_type::<StyleError>())?;
    Ok(())
}

/// Convert a generic std::io::Error into a Python BackendError.
pub fn io_err_to_py(e: std::io::Error) -> PyErr {
    BackendError::new_err(e.to_string())
}

/// Convert a ratatui layout error string to LayoutError.
pub fn layout_err_to_py(msg: impl Into<String>) -> PyErr {
    LayoutError::new_err(msg.into())
}

/// Convert a render failure to RenderError.
pub fn render_err_to_py(msg: impl Into<String>) -> PyErr {
    RenderError::new_err(msg.into())
}
