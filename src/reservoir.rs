use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::fs::File;

use pyo3::prelude::*;
use rand::{distributions::Uniform, Rng}; // 0.7.0
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

    pub fn sample(&self, mini_batch_size: usize) {
        let mut cumulative_plys = vec![0; self.max_size + 1];

        for i in 0..self.max_size {
            cumulative_plys[i + 1] = cumulative_plys[i] + self.learning_targets[i].len();
        }

        let range = Uniform::from(0..cumulative_plys[self.max_size]);
        let mut indicies: std::vec::Vec<usize> = rand::thread_rng().sample_iter(&range).take(mini_batch_size).collect();

        indicies.sort();

        let mut target_plys = vec![(0, 0); mini_batch_size];

        let mut lo = 0;
        for i in 0..mini_batch_size {
            let mut ok = lo;
            let mut ng = self.max_size + 1;

            while ng - ok > 1 {
                let mid = (ok + ng) / 2;

                if indicies[i] >= cumulative_plys[ok] {
                    ok = mid;
                } else {
                    ng = mid;
                }
            }

            let ply = self.learning_targets[ok][indicies[i] - cumulative_plys[ok]];
            target_plys[i] = (ok, ply);

            lo = ok;
        }
    }
}
