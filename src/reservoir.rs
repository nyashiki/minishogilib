use std::collections::VecDeque;

use pyo3::prelude::*;
use record::*;

#[pyclass]
pub struct Reservoir {
    records: VecDeque<Record>,
    learning_targets: VecDeque<std::vec::Vec<usize>>,
    json_path: String,
    max_size: usize
}

#[pymethods]
impl Reservoir {
    #[new]
    pub fn new(obj: &PyRawObject, json_path: &str, max_size: usize) {
        obj.init(Reservoir {
            records: VecDeque::new(),
            learning_targets: VecDeque::new(),
            json_path: json_path.to_string(),
            max_size: max_size
        });
    }


    pub fn push(&mut self, record_json: &str) {
        if self.records.len() == self.max_size {
            self.records.pop_front();
            self.learning_targets.pop_front();
        }

        // self.records.push_back(record);
        // self.learning_targets.push_back(record.learning_target_plys);

        // ToDo: Write log.
    }
}
