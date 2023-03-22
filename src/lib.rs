extern crate bitintr;
extern crate pyo3;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate once_cell;

pub mod bitboard;
pub mod r#move;
pub mod position;
pub mod types;
pub mod zobrist;

use pyo3::prelude::*;

#[pymodule]
fn minishogilib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<position::Position>()?;
    m.add_class::<r#move::Move>()?;

    Ok(())
}
