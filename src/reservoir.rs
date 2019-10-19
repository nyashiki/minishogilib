use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

use numpy::PyArray1;
use pyo3::prelude::*;
use rand::{distributions::Uniform, Rng};
use rayon::prelude::*;
use record::*;
use position::*;

use std::time::{Duration, Instant};

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

    pub fn sample(&self, py: Python, mini_batch_size: usize) -> (Py<PyArray1<f32>>, Py<PyArray1<f32>>, Py<PyArray1<f32>>) {
        let start = Instant::now();

        let mut cumulative_plys = vec![0; self.max_size + 1];

        for i in 0..self.max_size {
            cumulative_plys[i + 1] = cumulative_plys[i] + self.learning_targets[i].len();
        }

        let end = start.elapsed();
        println!("cumulative {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let range = Uniform::from(0..cumulative_plys[self.max_size]);
        let mut indicies: std::vec::Vec<usize> = rand::thread_rng().sample_iter(&range).take(mini_batch_size).collect();

        indicies.sort();

        let end = start.elapsed();
        println!("sort {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let mut targets = vec![(0, 0); mini_batch_size];

        let mut lo = 0;
        for i in 0..mini_batch_size {
            let mut ok = lo;
            let mut ng = self.max_size + 1;

            while ng - ok > 1 {
                let mid = (ok + ng) / 2;

                if indicies[i] >= cumulative_plys[mid] {
                    ok = mid;
                } else {
                    ng = mid;
                }
            }

            let ply = self.learning_targets[ok][indicies[i] - cumulative_plys[ok]];
            targets[i] = (ok, ply);

            lo = ok;
        }

        let end = start.elapsed();
        println!("targets {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let end = start.elapsed();
        println!("allocate {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let data: std::vec::Vec<_> = targets.par_iter().map(move |&target| {
            let index = target.0;
            let ply = target.1;

            let mut position = Position::empty_board();
            position.set_start_position();

            for (i, m) in self.records[index].sfen_kif.iter().enumerate() {
                if i == ply {
                    break;
                }

                let m = position.sfen_to_move(m);
                position.do_move(&m);
            }

            let nninput = position.to_alphazero_input_array();

            let mut policy = vec![0f32; 69 * 5 * 5];
            // Policy.
            let (sum_n, _q, playouts) = &self.records[index].mcts_result[ply];

            for playout in playouts {
                let m = position.sfen_to_move(&playout.0);
                let n = playout.1;

                policy[m.to_policy_index()] = n as f32 / *sum_n as f32;
            }

            // Value.
            let value = if self.records[index].winner == 2 {
                0.0
            } else if self.records[index].winner == position.get_side_to_move() {
                1.0
            } else {
                -1.0
            };

            (nninput, policy, value)
        }).collect();

        let end = start.elapsed();
        println!("sample {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let mut ins = vec![0f32; mini_batch_size * (8 * 33 + 2) * 5 * 5];
        let mut policies = vec![0f32; mini_batch_size * (69 * 5 * 5)];
        let mut values = vec![0f32; mini_batch_size];

        for (i, batch) in data.iter().enumerate() {
            /*
            let (left, _right) = ins[(i * (8 * 33 + 2) * 5 * 5)..].split_at_mut((8 * 33 + 2) * 5 * 5);
            left.clone_from_slice(&batch.0);

            let (left, _right) = policies[(i * 69 * 5 * 5)..].split_at_mut(69 * 5 * 5);
            left.clone_from_slice(&batch.1);
            */

            values[i] = batch.2;
        }

        let end = start.elapsed();
        println!("copy {}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1000000);

        let end = start.elapsed();

        (PyArray1::from_slice(py, &ins).to_owned(),
         PyArray1::from_slice(py, &policies).to_owned(),
         PyArray1::from_slice(py, &values).to_owned())
    }
}
