use pyo3::prelude::*;

use types::*;

#[pyclass]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Move {
    pub piece: Piece,
    pub from: usize,          // 移動元 (持ち駒を打つ場合には、打つ場所)
    pub to: usize,            // 移動先
    pub is_hand: bool,       // 持ち駒かどうか
    pub promotion: bool,      // 成/不成
    pub capture_piece: Piece, // 取った相手の駒
}

#[pymethods]
impl Move {
    pub fn sfen(&self) -> String {
        if self.piece == Piece::NO_PIECE {
            return "resign".to_string();
        }

        const HAND_PIECE_TO_CHAR: [char; 7] = ['E', 'E', 'G', 'S', 'B', 'R', 'P'];

        if self.is_hand {
            format!(
                "{}*{}",
                HAND_PIECE_TO_CHAR[self.piece.get_piece_type().as_usize()],
                square_to_sfen(self.to)
            )
        } else {
            if self.promotion {
                format!("{}{}+", square_to_sfen(self.from), square_to_sfen(self.to))
            } else {
                format!("{}{}", square_to_sfen(self.from), square_to_sfen(self.to))
            }
        }
    }

    pub fn csa_sfen(&self) -> String {
        if self.piece == Piece::NO_PIECE {
            return "%TORYO".to_string();
        }

        let csa_piece = [
            "--", "OU", "KI", "GI", "KA", "HI", "FU", "--", "--", "--", "--", "NG", "UM", "RY",
            "TO",
        ];

        if self.is_hand {
            format!(
                "00{}{}",
                square_to_csa_sfen(self.to),
                csa_piece[self.piece.get_piece_type().as_usize()]
            )
        } else {
            let piece = if self.promotion {
                self.piece.get_piece_type().get_promoted()
            } else {
                self.piece.get_piece_type()
            };

            format!(
                "{}{}{}",
                square_to_csa_sfen(self.from),
                square_to_csa_sfen(self.to),
                csa_piece[piece.as_usize()]
            )
        }
    }
}

#[pyproto]
impl pyo3::class::basic::PyObjectProtocol for Move {
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.sfen())
    }
}

#[pymethods]
impl Move {
    pub fn is_null_move(&self) -> bool {
        self.piece == Piece::NO_PIECE
    }

    pub fn get_from(&self) -> usize {
        self.from
    }

    pub fn get_to(&self) -> usize {
        self.to
    }

    pub fn get_promotion(&self) -> bool {
        self.promotion
    }

    pub fn get_hand_index(&self) -> usize {
        self.piece.get_piece_type().as_usize() - 2
    }
}

impl Move {
    pub fn board_move(
        piece: Piece,
        from: usize,
        to: usize,
        promotion: bool,
        capture_piece: Piece,
    ) -> Move {
        Move {
            piece: piece,
            from: from,
            to: to,
            is_hand: false,
            promotion: promotion,
            capture_piece: capture_piece,
        }
    }

    pub fn hand_move(piece: Piece, to: usize) -> Move {
        Move {
            piece: piece,
            from: 0,
            to: to,
            is_hand: true,
            promotion: false,
            capture_piece: Piece::NO_PIECE,
        }
    }
}

pub static NULL_MOVE: Move = Move {
    piece: Piece::NO_PIECE,
    from: SQUARE_NB,
    to: 0,
    is_hand: false,
    promotion: false,
    capture_piece: Piece::NO_PIECE,
};

pub fn square_to_sfen(square: usize) -> String {
    format!(
        "{}{}",
        "54321".as_bytes()[square % 5 as usize] as char,
        "abcde".as_bytes()[square / 5 as usize] as char
    )
}

pub fn square_to_csa_sfen(square: usize) -> String {
    format!(
        "{}{}",
        "54321".as_bytes()[square % 5 as usize] as char,
        "12345".as_bytes()[square / 5 as usize] as char
    )
}

pub fn sfen_to_square(sfen: &str) -> usize {
    ((sfen.as_bytes()[1] - ('a' as u8)) * 5 + (('5' as u8) - sfen.as_bytes()[0])) as usize
}

lazy_static! {
    /// 2つの座標を受け取り、その方向と距離を返す
    /// e.g. RELATION_TABLE[20][15] = (Direction::N, 1)
    static ref RELATION_TABLE: [[(Direction, usize); SQUARE_NB]; SQUARE_NB] = {
        let mut table = [[(Direction::N, 0usize); SQUARE_NB]; SQUARE_NB];

        const MOVE_DIRS: [Direction; 8] = [Direction::N, Direction::NE, Direction::E, Direction::SE, Direction::S, Direction::SW, Direction::W, Direction::NW];
        const MOVE_DIFF: [(i8, i8); 8] = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)];

        for from in 0..SQUARE_NB {
            let y = (from as i8) / 5;
            let x = (from as i8) % 5;

            for dir in 0..8 {
                for amount in 1..5 {
                    let ny = y + MOVE_DIFF[dir].0 * amount;
                    let nx = x + MOVE_DIFF[dir].1 * amount;

                    if ny < 0 || ny >= 5 || nx < 0 || nx >= 5 {
                        break;
                    }

                    table[(5 * y + x) as usize][(5 * ny + nx) as usize] = (MOVE_DIRS[dir], amount as usize);
                }
            }
        }

        return table;
    };
}

pub fn init() {
    lazy_static::initialize(&RELATION_TABLE);
}

pub fn get_relation(square1: usize, square2: usize) -> (Direction, usize) {
    return RELATION_TABLE[square1][square2];
}

#[test]
fn get_relation_test() {
    assert_eq!(get_relation(20, 15), (Direction::N, 1));

    assert_eq!(get_relation(20, 4), (Direction::NE, 4));
    assert_eq!(get_relation(4, 20), (Direction::SW, 4));
    assert_eq!(get_relation(0, 24), (Direction::SE, 4));
    assert_eq!(get_relation(24, 0), (Direction::NW, 4));

    assert_eq!(get_relation(20, 0), (Direction::N, 4));
    assert_eq!(get_relation(0, 20), (Direction::S, 4));
    assert_eq!(get_relation(0, 4), (Direction::E, 4));
    assert_eq!(get_relation(4, 0), (Direction::W, 4));

    assert_eq!(get_relation(21, 9), (Direction::NE, 3));
}
