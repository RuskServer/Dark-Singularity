// src/lib.rs
pub mod core;
pub mod jni_api;

#[cfg(feature = "python")]
pub mod python_api;

#[cfg(feature = "python")]
use pyo3::prelude::*;

// Python モジュールの定義
#[cfg(feature = "python")]
#[pymodule]
fn dark_singularity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python_api::PySingularity>()?;
    Ok(())
}
