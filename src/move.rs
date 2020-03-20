use pyo3::prelude::*;

use types::*;

#[pyclass]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Move {
    pub _data: u32, // 00 -- 07 bit 動かす駒
                    // 08 -- 12 bit 移動元の座標
                    // 13 -- 17 bit 移動先の座標
                    // 18 -- 18 bit 持ち駒を打つ手かどうか
                    // 19 -- 19 bit 成る手かどうか
                    // 20 -- 27 bit 取る相手の駒


    // pub piece: Piece,
    // pub from: usize,          // 移動元
    // pub to: usize,            // 移動先
    // pub is_hand: bool,        // 持ち駒を打つ手かどうか
    // pub promotion: bool,      // 成/不成
    // pub capture_piece: Piece, // 取る相手の駒
}

#[pymethods]
impl Move {
    pub fn sfen(&self) -> String {
        if self.get_piece() == Piece::NO_PIECE {
            return "resign".to_string();
        }

        const HAND_PIECE_TO_CHAR: [char; 7] = ['E', 'E', 'G', 'S', 'B', 'R', 'P'];

        if self.is_hand() {
            format!(
                "{}*{}",
                HAND_PIECE_TO_CHAR[self.get_piece().get_piece_type().as_usize()],
                square_to_sfen(self.get_to())
            )
        } else {
            if self.is_promotion() {
                format!("{}{}+", square_to_sfen(self.get_from()), square_to_sfen(self.get_to()))
            } else {
                format!("{}{}", square_to_sfen(self.get_from()), square_to_sfen(self.get_to()))
            }
        }
    }

    pub fn csa(&self) -> String {
        if self.get_piece() == Piece::NO_PIECE {
            return "%TORYO".to_string();
        }

        let csa_piece = [
            "--", "OU", "KI", "GI", "KA", "HI", "FU", "--", "--", "--", "--", "NG", "UM", "RY",
            "TO",
        ];

        if self.is_hand() {
            format!(
                "00{}{}",
                square_to_csa(self.get_to()),
                csa_piece[self.get_piece().get_piece_type().as_usize()]
            )
        } else {
            let piece = if self.is_promotion() {
                self.get_piece().get_piece_type().get_promoted()
            } else {
                self.get_piece().get_piece_type()
            };

            format!(
                "{}{}{}",
                square_to_csa(self.get_from()),
                square_to_csa(self.get_to()),
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
        self.get_piece() == Piece::NO_PIECE
    }

    pub fn get_from(&self) -> usize {
        ((self._data & 0b1111100000000) >> 8) as usize
    }

    pub fn set_from(&mut self, from: usize) {
        self._data = (self._data & !0b1111100000000) | (from << 8) as u32;
    }

    pub fn get_to(&self) -> usize {
        ((self._data & 0b111110000000000000) >> 13) as usize
    }

    pub fn set_to(&mut self, to: usize) {
        self._data = (self._data & !0b111110000000000000) | (to << 13) as u32;
    }

    pub fn is_hand(&self) -> bool {
        ((self._data & 0b1000000000000000000) >> 18) != 0
    }

    pub fn is_promotion(&self) -> bool {
        ((self._data & 0b10000000000000000000) >> 19) != 0
    }

    pub fn get_hand_index(&self) -> usize {
        self.get_piece().get_piece_type().as_usize() - 2
    }
}

impl Move {
    pub fn get_piece(&self) -> Piece {
        Piece((self._data & 0b11111111) as u8)
    }

    pub fn get_capture_piece(&self) -> Piece {
        Piece(((self._data & 0b1111111100000000000000000000) >> 20) as u8)
    }

    pub fn board_move(
        piece: Piece,
        from: usize,
        to: usize,
        promotion: bool,
        capture_piece: Piece,
    ) -> Move {
        Move {
            _data: piece.as_u32() |
                   (from as u32) << 8 |
                   (to as u32) << 13  |
                   (promotion as u32) << 19 |
                   (capture_piece.as_u32()) << 20
        }
    }

    pub fn hand_move(piece: Piece, to: usize) -> Move {
        Move {
            _data: piece.as_u32() |
                   (to as u32) << 13 |
                   1 << 18
        }
    }

    pub fn flip(&self) -> Move {
        let mut m = *self;

        if !self.is_hand() {
            let nfrom = {
                let y = self.get_from() / 5;
                let x = 4 - self.get_from() % 5;

                y * 5 + x
            };

            m.set_from(nfrom);
        }

        let nto = {
            let y = self.get_to() / 5;
            let x = 4 - self.get_to() % 5;

            y * 5 + x
        };
        m.set_to(nto);

        return m;
    }
}

pub static NULL_MOVE: Move = Move {
    _data: 0
};

pub fn square_to_sfen(square: usize) -> String {
    format!(
        "{}{}",
        "54321".as_bytes()[square % 5 as usize] as char,
        "abcde".as_bytes()[square / 5 as usize] as char
    )
}

pub fn square_to_csa(square: usize) -> String {
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

#[test]
fn flip_test() {
    {
        let m = Move::board_move(Piece::NO_PIECE, 20, 15, false, Piece::NO_PIECE).flip();
        assert_eq!(m.get_from(), 24);
        assert_eq!(m.get_to(), 19);
    }

    {
        let m = Move::board_move(Piece::NO_PIECE, 23, 11, false, Piece::NO_PIECE).flip();
        assert_eq!(m.get_from(), 21);
        assert_eq!(m.get_to(), 13);
    }

    {
        let m = Move::hand_move(Piece::NO_PIECE, 15).flip();

        assert_eq!(m.get_from(), 0);
        assert_eq!(m.get_to(), 19);
    }
}
