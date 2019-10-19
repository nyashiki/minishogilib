#[macro_use]
extern crate lazy_static;
extern crate bitintr;
extern crate numpy;
extern crate pyo3;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;

pub mod bitboard;
pub mod checkmate;
pub mod mcts;
pub mod r#move;
pub mod neuralnetwork;
pub mod position;
pub mod record;
pub mod reservoir;
pub mod types;
pub mod zobrist;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use rayon::prelude::*;

#[pyfunction]
pub fn get_positions_from_sfen_without_startpos(
    sfen_kifs: std::vec::Vec<String>,
) -> std::vec::Vec<position::Position> {
    let positions = sfen_kifs
        .par_iter()
        .map(|x| {
            let mut position = position::Position::empty_board();
            position.set_sfen_without_startpos(x);

            position
        })
        .collect();

    return positions;
}

#[pymodule]
fn minishogilib(_py: Python, m: &PyModule) -> PyResult<()> {
    r#move::init();
    bitboard::init();
    zobrist::init();

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<position::Position>()?;
    m.add_class::<mcts::MCTS>()?;
    m.add_class::<r#move::Move>()?;

    m.add_wrapped(wrap_pyfunction!(get_positions_from_sfen_without_startpos))?;

    Ok(())
}
