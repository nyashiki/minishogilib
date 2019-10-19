use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::fs::File;

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

        let record = Record::from_json(record_json);

        self.records.push_back(record.clone());
        self.learning_targets.push_back(record.learning_target_plys);

        // ToDo: Write log.
    }

    pub fn load_json(&mut self, path: &str) {
        let file = File::open(path).expect("The file does not exist.");
        let file = BufReader::new(file);

        for line in file.lines().filter_map(|x| x.ok()) {
            self.push(&line);
        }
    }
}
