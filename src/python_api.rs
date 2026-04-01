// src/python_api.rs
use pyo3::prelude::*;
use crate::core::singularity::Singularity;

#[pyclass]
pub struct PySingularity {
    pub(crate) inner: Singularity,
}

#[pymethods]
impl PySingularity {
    #[new]
    pub fn new(state_size: usize, category_sizes: Vec<usize>) -> Self {
        Self {
            inner: Singularity::new(state_size, category_sizes),
        }
    }

    pub fn select_actions(&mut self, state_idx: usize) -> Vec<i32> {
        self.inner.select_actions(state_idx)
    }

    pub fn learn(&mut self, reward: f32) {
        self.inner.learn(reward);
    }

    pub fn set_active_conditions(&mut self, conditions: Vec<i32>) {
        self.inner.set_active_conditions(&conditions);
    }

    pub fn observe_expert(&mut self, state_idx: usize, expert_actions: Vec<usize>, strength: f32) {
        self.inner.observe_expert(state_idx, &expert_actions, strength);
    }

    pub fn suppress_expert(&mut self, bad_actions: Vec<usize>, strength: f32) {
        self.inner.suppress_expert(&bad_actions, strength);
    }

    pub fn save(&self, path: &str) -> PyResult<()> {
        self.inner.save_to_file(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to save: {}", e))
        })
    }

    pub fn load(&mut self, path: &str) -> PyResult<()> {
        self.inner.load_from_file(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load: {}", e))
        })
    }

    #[getter]
    pub fn get_system_temperature(&self) -> f32 {
        self.inner.system_temperature
    }

    #[getter]
    pub fn get_adrenaline(&self) -> f32 {
        self.inner.adrenaline
    }

    #[getter]
    pub fn get_frustration(&self) -> f32 {
        self.inner.frustration
    }
}
