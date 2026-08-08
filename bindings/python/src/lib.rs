use pyo3::prelude::*;

use internetarchive_core::{sanitize_windows_filename, validate_s3_identifier};

#[pyfunction]
fn validate_s3_identifier_py(identifier: &str) -> PyResult<()> {
    validate_s3_identifier(identifier)
        .map_err(|err| pyo3::exceptions::PyValueError::new_err(err.to_string()))
}

#[pyfunction]
fn sanitize_windows_filename_py(name: &str) -> (String, bool) {
    sanitize_windows_filename(name)
}

/// Optional PyO3 bindings for the Rust core.
#[pymodule]
fn internetarchive_rust(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_s3_identifier_py, m)?)?;
    m.add_function(wrap_pyfunction!(sanitize_windows_filename_py, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_filename() {
        let (name, modified) = sanitize_windows_filename("AUX.txt");
        assert!(modified);
        assert!(name.contains('%'));
    }
}
