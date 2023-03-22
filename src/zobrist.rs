use types::*;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use once_cell::sync::Lazy;

pub static BOARD_TABLE: Lazy<[[u64; Piece::B_PAWN_X.as_usize() + 1]; SQUARE_NB]> = Lazy::new(||{
    let mut table: [[u64; Piece::B_PAWN_X.as_usize() + 1]; SQUARE_NB] =
        [[0; Piece::B_PAWN_X.as_usize() + 1]; SQUARE_NB];

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);

    for i in 0..SQUARE_NB {
        for j in 0..Piece::B_PAWN_X.as_usize() + 1 {
            table[i][j] = rng.gen::<u64>() << 1;
        }
    }

    return table;
});

pub static HAND_TABLE: Lazy<[[[u64; 3]; 5]; 2]> = Lazy::new(||{
    let mut table: [[[u64; 3]; 5]; 2] = [[[0; 3]; 5]; 2];

    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);

    for i in 0..2 {
        for j in 0..5 {
            for k in 0..3 {
                table[i][j][k] = rng.gen::<u64>();
            }
        }
    }

    return table;
});
