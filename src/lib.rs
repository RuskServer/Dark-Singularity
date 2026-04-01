// src/lib.rs
pub mod core;
pub mod jni_api;
pub mod python_api;

use pyo3::prelude::*;

// Python モジュールの定義
#[pymodule]
fn dark_singularity(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python_api::PySingularity>()?;
    Ok(())
}
