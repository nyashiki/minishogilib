use pyo3::prelude::*;

use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Serialize, Deserialize)]
pub struct Record {
    ply: u16,
    sfen_kif: std::vec::Vec<String>,
    mcts_result: (u32, f32, std::vec::Vec<(String, u32)>),
    learning_target_plys: std::vec::Vec<usize>,
    winner: u8,
    timestamp: u32
}

impl Record {
    pub fn from_json(record_json: &str) -> Record {
        serde_json::from_str(record_json).unwrap()
    }
}
