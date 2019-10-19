use pyo3::prelude::*;

use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub ply: u16,
    pub sfen_kif: std::vec::Vec<String>,
    pub mcts_result: std::vec::Vec<(u32, f32, std::vec::Vec<(String, u32)>)>,
    pub learning_target_plys: std::vec::Vec<usize>,
    pub winner: u8,
    pub timestamp: u32
}

impl Record {
    pub fn from_json(record_json: &str) -> Record {
        serde_json::from_str(record_json).unwrap()
    }
}

#[pymethods]
impl Record {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
