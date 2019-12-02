use std::collections::VecDeque;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use neuralnetwork;
use numpy::PyArray1;
use position::*;
use pyo3::prelude::*;
use rand::Rng;
use rayon::prelude::*;
use record::*;
use types::*;

#[pyclass]
pub struct Reservoir {
    records: VecDeque<Record>,
    learning_targets: VecDeque<std::vec::Vec<usize>>,
    json_path: String,
    max_size: usize,
}

#[pymethods]
impl Reservoir {
    #[new]
    pub fn new(obj: &PyRawObject, json_path: &str, max_size: usize) {
        obj.init(Reservoir {
            records: VecDeque::new(),
            learning_targets: VecDeque::new(),
            json_path: json_path.to_string(),
            max_size: max_size,
        });
    }

    pub fn push_with_option(&mut self, record_json: &str, log: bool) {
        if self.records.len() == self.max_size {
            self.records.pop_front();
            self.learning_targets.pop_front();
        }

        let record = Record::from_json(record_json);

        self.records.push_back(record.clone());
        self.learning_targets.push_back(record.learning_target_plys);

        if log {
            let mut file =
                OpenOptions::new().create(true).append(true).open(&self.json_path).unwrap();
            file.write(record_json.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
        }
    }

    pub fn push(&mut self, record_json: &str) {
        self.push_with_option(record_json, true);
    }

    pub fn load(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let file = BufReader::new(file);

        let mut line_count = 0;

        println!("");
        for line in file.lines().filter_map(|x| x.ok()) {
            print!("\rloading ... ({} entries).", line_count);

            self.push_with_option(&line, false);
            line_count += 1;
        }
        println!("\r{}[2Kloading ... ok.", 27 as char);
    }

    pub fn sample(
        &self,
        py: Python,
        mini_batch_size: usize,
    ) -> (Py<PyArray1<f32>>, Py<PyArray1<f32>>, Py<PyArray1<f32>>) {
        let mut cumulative_plys = vec![0; self.max_size + 1];

        for i in 0..self.max_size {
            cumulative_plys[i + 1] = cumulative_plys[i] + self.learning_targets[i].len();
        }

        let mut rng = rand::thread_rng();
        let range = cumulative_plys[self.max_size];

        let mut targets = vec![(0, 0); mini_batch_size];

        let white_win_target_count_max = mini_batch_size / 2;
        let black_win_target_count_max = mini_batch_size - white_win_target_count_max;

        let mut white_win_target_count = 0;
        let mut black_win_target_count = 0;
        let mut counter = 0;

        while counter < mini_batch_size {
            let mut ok = 0;
            let mut ng = self.max_size + 1;

            let index = rng.gen_range(0, range);

            while ng - ok > 1 {
                let mid = (ok + ng) / 2;

                if index >= cumulative_plys[mid] {
                    ok = mid;
                } else {
                    ng = mid;
                }
            }

            if Color(self.records[ok].winner) == Color::WHITE {
                if white_win_target_count == white_win_target_count_max {
                    continue;
                }

                white_win_target_count += 1;
            } else if Color(self.records[ok].winner) == Color::BLACK {
                if black_win_target_count == black_win_target_count_max {
                    continue;
                }

                black_win_target_count += 1;
            } else {
                continue;
            }

            let ply = self.learning_targets[ok][index - cumulative_plys[ok]];
            targets[counter] = (ok, ply);
            counter += 1;
        }

        let data: std::vec::Vec<_> = targets
            .par_iter()
            .map(move |&target| {
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

                let mut policy = [0f32; 69 * 5 * 5];
                // Policy.
                let (sum_n, q, playouts) = &self.records[index].mcts_result[ply];

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

                let scaled_q = q * 2.0 - 1.0;
                let value = 0.5 * value + 0.5 * scaled_q;

                (nninput, policy, value)
            })
            .collect();

        let mut ins = std::vec::Vec::with_capacity(mini_batch_size * (neuralnetwork::HISTORY * 33 + 2) * SQUARE_NB);
        let mut policies = std::vec::Vec::with_capacity(mini_batch_size * 69 * SQUARE_NB);
        let mut values = std::vec::Vec::with_capacity(mini_batch_size);

        for (_b, batch) in data.iter().enumerate() {
            ins.extend_from_slice(&batch.0);
            policies.extend_from_slice(&batch.1);
            values.push(batch.2);
        }

        (
            PyArray1::from_slice(py, &ins).to_owned(),
            PyArray1::from_slice(py, &policies).to_owned(),
            PyArray1::from_slice(py, &values).to_owned(),
        )
    }
}
