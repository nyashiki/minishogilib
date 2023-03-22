#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    pub const WHITE: Color = Color(0);
    pub const BLACK: Color = Color(1);
    pub const NO_COLOR: Color = Color(2);

    pub const fn get_op_color(self) -> Color {
        Color(self.0 ^ 1)
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[test]
fn get_op_color_test() {
    assert!(Color::WHITE.get_op_color() == Color::BLACK);
    assert!(Color::BLACK.get_op_color() == Color::WHITE);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Piece(pub u8);

impl Piece {
    pub const NO_PIECE: Piece = Piece(0);

    pub const W_KING: Piece = Piece(0b00001);
    pub const W_GOLD: Piece = Piece(0b00010);
    pub const W_SILVER: Piece = Piece(0b00011);
    pub const W_BISHOP: Piece = Piece(0b00100);
    pub const W_ROOK: Piece = Piece(0b00101);
    pub const W_PAWN: Piece = Piece(0b00110);
    pub const W_SILVER_X: Piece = Piece(0b01011);
    pub const W_BISHOP_X: Piece = Piece(0b01100);
    pub const W_ROOK_X: Piece = Piece(0b01101);
    pub const W_PAWN_X: Piece = Piece(0b01110);

    pub const B_KING: Piece = Piece(0b10001);
    pub const B_GOLD: Piece = Piece(0b10010);
    pub const B_SILVER: Piece = Piece(0b10011);
    pub const B_BISHOP: Piece = Piece(0b10100);
    pub const B_ROOK: Piece = Piece(0b10101);
    pub const B_PAWN: Piece = Piece(0b10110);
    pub const B_SILVER_X: Piece = Piece(0b11011);
    pub const B_BISHOP_X: Piece = Piece(0b11100);
    pub const B_ROOK_X: Piece = Piece(0b11101);
    pub const B_PAWN_X: Piece = Piece(0b11110);

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    pub const fn get_promoted(self) -> Piece {
        Piece(self.0 | 0b01000)
    }

    pub const fn is_promoted(self) -> bool {
        (self.0 & 0b01000) != 0
    }

    pub fn is_promotable(self) -> bool {
        self.get_piece_type().is_promotable()
    }

    pub const fn get_raw(self) -> Piece {
        Piece(self.0 & 0b10111)
    }

    pub const fn is_raw(self) -> bool {
        (self.0 & 0b01000) == 0
    }

    pub fn get_color(self) -> Color {
        if self == Piece::NO_PIECE {
            Color::NO_COLOR
        } else {
            Color(self.0 >> 4)
        }
    }

    pub const fn get_piece_type(self) -> PieceType {
        PieceType(self.0 & 0b01111)
    }

    pub fn get_op_piece(self) -> Piece {
        if self == Piece::NO_PIECE {
            Piece::NO_PIECE
        } else {
            Piece(self.0 ^ 0b10000)
        }
    }

    pub fn get_move_dirs(self) -> std::vec::Vec<Direction> {
        match self {
            Piece::W_KING => vec![
                Direction::N,
                Direction::NE,
                Direction::E,
                Direction::SE,
                Direction::S,
                Direction::SW,
                Direction::W,
                Direction::NW,
            ],
            Piece::W_GOLD => vec![
                Direction::N,
                Direction::NE,
                Direction::E,
                Direction::S,
                Direction::W,
                Direction::NW,
            ],
            Piece::W_SILVER => {
                vec![Direction::N, Direction::NE, Direction::SE, Direction::SW, Direction::NW]
            }
            Piece::W_PAWN => vec![Direction::N],
            Piece::W_SILVER_X => vec![
                Direction::N,
                Direction::NE,
                Direction::E,
                Direction::S,
                Direction::W,
                Direction::NW,
            ],
            Piece::W_BISHOP_X => vec![Direction::N, Direction::E, Direction::S, Direction::W],
            Piece::W_ROOK_X => vec![Direction::NE, Direction::SE, Direction::SW, Direction::NW],
            Piece::W_PAWN_X => vec![
                Direction::N,
                Direction::NE,
                Direction::E,
                Direction::S,
                Direction::W,
                Direction::NW,
            ],

            Piece::B_KING => vec![
                Direction::N,
                Direction::NE,
                Direction::E,
                Direction::SE,
                Direction::S,
                Direction::SW,
                Direction::W,
                Direction::NW,
            ],
            Piece::B_GOLD => vec![
                Direction::N,
                Direction::E,
                Direction::SE,
                Direction::S,
                Direction::SW,
                Direction::W,
            ],
            Piece::B_SILVER => {
                vec![Direction::NE, Direction::SE, Direction::S, Direction::SW, Direction::NW]
            }
            Piece::B_PAWN => vec![Direction::S],
            Piece::B_SILVER_X => vec![
                Direction::N,
                Direction::E,
                Direction::SE,
                Direction::S,
                Direction::SW,
                Direction::W,
            ],
            Piece::B_BISHOP_X => vec![Direction::N, Direction::E, Direction::S, Direction::W],
            Piece::B_ROOK_X => vec![Direction::NE, Direction::SE, Direction::SW, Direction::NW],
            Piece::B_PAWN_X => vec![
                Direction::N,
                Direction::E,
                Direction::SE,
                Direction::S,
                Direction::SW,
                Direction::W,
            ],

            _ => vec![],
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Piece::NO_PIECE => write!(f, " * "),

            Piece::W_KING => write!(f, "\x1b[38;2;0;100;200m K \x1b[0m"),
            Piece::W_GOLD => write!(f, "\x1b[38;2;0;100;200m G \x1b[0m"),
            Piece::W_SILVER => write!(f, "\x1b[38;2;0;100;200m S \x1b[0m"),
            Piece::W_BISHOP => write!(f, "\x1b[38;2;0;100;200m B \x1b[0m"),
            Piece::W_ROOK => write!(f, "\x1b[38;2;0;100;200m R \x1b[0m"),
            Piece::W_PAWN => write!(f, "\x1b[38;2;0;100;200m P \x1b[0m"),
            Piece::W_SILVER_X => write!(f, "\x1b[38;2;0;100;200m Sx\x1b[0m"),
            Piece::W_BISHOP_X => write!(f, "\x1b[38;2;0;100;200m Bx\x1b[0m"),
            Piece::W_ROOK_X => write!(f, "\x1b[38;2;0;100;200m Rx\x1b[0m"),
            Piece::W_PAWN_X => write!(f, "\x1b[38;2;0;100;200m Px\x1b[0m"),

            Piece::B_KING => write!(f, "\x1b[38;2;250;200;50mvK \x1b[0m"),
            Piece::B_GOLD => write!(f, "\x1b[38;2;250;200;50mvG \x1b[0m"),
            Piece::B_SILVER => write!(f, "\x1b[38;2;250;200;50mvS \x1b[0m"),
            Piece::B_BISHOP => write!(f, "\x1b[38;2;250;200;50mvB \x1b[0m"),
            Piece::B_ROOK => write!(f, "\x1b[38;2;250;200;50mvR \x1b[0m"),
            Piece::B_PAWN => write!(f, "\x1b[38;2;250;200;50mvP \x1b[0m"),
            Piece::B_SILVER_X => write!(f, "\x1b[38;2;250;200;50mvSx\x1b[0m"),
            Piece::B_BISHOP_X => write!(f, "\x1b[38;2;250;200;50mvBx\x1b[0m"),
            Piece::B_ROOK_X => write!(f, "\x1b[38;2;250;200;50mvRx\x1b[0m"),
            Piece::B_PAWN_X => write!(f, "\x1b[38;2;250;200;50mvPx\x1b[0m"),

            _ => write!(f, "ERROR"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PieceType(u8);

impl PieceType {
    pub const NO_PIECE_TYPE: PieceType = PieceType(0);

    pub const KING: PieceType = PieceType(0b0001);
    pub const GOLD: PieceType = PieceType(0b0010);
    pub const SILVER: PieceType = PieceType(0b0011);
    pub const BISHOP: PieceType = PieceType(0b0100);
    pub const ROOK: PieceType = PieceType(0b0101);
    pub const PAWN: PieceType = PieceType(0b0110);
    pub const SILVER_X: PieceType = PieceType(0b1011);
    pub const BISHOP_X: PieceType = PieceType(0b1100);
    pub const ROOK_X: PieceType = PieceType(0b1101);
    pub const PAWN_X: PieceType = PieceType(0b1110);

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const fn get_promoted(self) -> PieceType {
        PieceType(self.0 | 0b1000)
    }

    pub const fn is_promoted(self) -> bool {
        (self.0 & 0b1000) != 0
    }

    pub fn is_promotable(self) -> bool {
        self.0 > PieceType::GOLD.as_usize() as u8 && self.0 <= PieceType::PAWN.as_usize() as u8
    }

    pub const fn get_raw(self) -> PieceType {
        PieceType(self.0 & 0b0111)
    }

    pub const fn is_raw(self) -> bool {
        self.0 & 0b1000 == 0
    }

    pub fn get_piece(self, color: Color) -> Piece {
        if self == PieceType::NO_PIECE_TYPE {
            Piece::NO_PIECE
        } else {
            if color == Color::WHITE {
                Piece(self.0)
            } else {
                Piece(self.0 | 0b10000)
            }
        }
    }
}

#[test]
fn get_promoted_test() {
    // Piece
    assert!(Piece::W_SILVER.get_promoted() == Piece::W_SILVER_X);
    assert!(Piece::W_BISHOP.get_promoted() == Piece::W_BISHOP_X);
    assert!(Piece::W_ROOK.get_promoted() == Piece::W_ROOK_X);
    assert!(Piece::W_PAWN.get_promoted() == Piece::W_PAWN_X);
    assert!(Piece::B_SILVER.get_promoted() == Piece::B_SILVER_X);
    assert!(Piece::B_BISHOP.get_promoted() == Piece::B_BISHOP_X);
    assert!(Piece::B_ROOK.get_promoted() == Piece::B_ROOK_X);
    assert!(Piece::B_PAWN.get_promoted() == Piece::B_PAWN_X);

    // PieceType
    assert!(PieceType::SILVER.get_promoted() == PieceType::SILVER_X);
    assert!(PieceType::BISHOP.get_promoted() == PieceType::BISHOP_X);
    assert!(PieceType::ROOK.get_promoted() == PieceType::ROOK_X);
    assert!(PieceType::PAWN.get_promoted() == PieceType::PAWN_X);
}

#[test]
fn is_promoted_test() {
    // Piece
    assert!(!Piece::W_KING.is_promoted());
    assert!(!Piece::W_GOLD.is_promoted());
    assert!(!Piece::W_SILVER.is_promoted());
    assert!(!Piece::W_BISHOP.is_promoted());
    assert!(!Piece::W_ROOK.is_promoted());
    assert!(!Piece::W_PAWN.is_promoted());
    assert!(Piece::W_SILVER_X.is_promoted());
    assert!(Piece::W_BISHOP_X.is_promoted());
    assert!(Piece::W_ROOK_X.is_promoted());
    assert!(Piece::W_PAWN_X.is_promoted());
    assert!(!Piece::B_KING.is_promoted());
    assert!(!Piece::B_GOLD.is_promoted());
    assert!(!Piece::B_SILVER.is_promoted());
    assert!(!Piece::B_BISHOP.is_promoted());
    assert!(!Piece::B_ROOK.is_promoted());
    assert!(!Piece::B_PAWN.is_promoted());
    assert!(Piece::B_SILVER_X.is_promoted());
    assert!(Piece::B_BISHOP_X.is_promoted());
    assert!(Piece::B_ROOK_X.is_promoted());
    assert!(Piece::B_PAWN_X.is_promoted());

    // PieceType
    assert!(!PieceType::KING.is_promoted());
    assert!(!PieceType::GOLD.is_promoted());
    assert!(!PieceType::SILVER.is_promoted());
    assert!(!PieceType::BISHOP.is_promoted());
    assert!(!PieceType::ROOK.is_promoted());
    assert!(!PieceType::PAWN.is_promoted());
    assert!(PieceType::SILVER_X.is_promoted());
    assert!(PieceType::BISHOP_X.is_promoted());
    assert!(PieceType::ROOK_X.is_promoted());
    assert!(PieceType::PAWN_X.is_promoted());
}

#[test]
fn get_raw_test() {
    // Piece
    assert!(Piece::W_SILVER_X.get_raw() == Piece::W_SILVER);
    assert!(Piece::W_BISHOP_X.get_raw() == Piece::W_BISHOP);
    assert!(Piece::W_ROOK_X.get_raw() == Piece::W_ROOK);
    assert!(Piece::W_PAWN_X.get_raw() == Piece::W_PAWN);
    assert!(Piece::B_SILVER_X.get_raw() == Piece::B_SILVER);
    assert!(Piece::B_BISHOP_X.get_raw() == Piece::B_BISHOP);
    assert!(Piece::B_ROOK_X.get_raw() == Piece::B_ROOK);
    assert!(Piece::B_PAWN_X.get_raw() == Piece::B_PAWN);

    // PieceType
    assert!(PieceType::SILVER_X.get_raw() == PieceType::SILVER);
    assert!(PieceType::BISHOP_X.get_raw() == PieceType::BISHOP);
    assert!(PieceType::ROOK_X.get_raw() == PieceType::ROOK);
    assert!(PieceType::PAWN_X.get_raw() == PieceType::PAWN);
}

#[test]
fn is_raw_test() {
    // Piece
    assert!(Piece::W_KING.is_raw());
    assert!(Piece::W_GOLD.is_raw());
    assert!(Piece::W_BISHOP.is_raw());
    assert!(Piece::W_ROOK.is_raw());
    assert!(Piece::W_PAWN.is_raw());
    assert!(Piece::B_KING.is_raw());
    assert!(Piece::B_GOLD.is_raw());
    assert!(Piece::B_SILVER.is_raw());
    assert!(Piece::B_BISHOP.is_raw());
    assert!(Piece::B_ROOK.is_raw());
    assert!(Piece::B_PAWN.is_raw());

    // PieceType
    assert!(PieceType::KING.is_raw());
    assert!(PieceType::GOLD.is_raw());
    assert!(PieceType::SILVER.is_raw());
    assert!(PieceType::BISHOP.is_raw());
    assert!(PieceType::ROOK.is_raw());
    assert!(PieceType::PAWN.is_raw());
}

#[test]
fn get_piece_test() {
    assert!(PieceType::NO_PIECE_TYPE.get_piece(Color::WHITE) == Piece::NO_PIECE);

    // White
    assert!(PieceType::KING.get_piece(Color::WHITE) == Piece::W_KING);
    assert!(PieceType::GOLD.get_piece(Color::WHITE) == Piece::W_GOLD);
    assert!(PieceType::SILVER.get_piece(Color::WHITE) == Piece::W_SILVER);
    assert!(PieceType::BISHOP.get_piece(Color::WHITE) == Piece::W_BISHOP);
    assert!(PieceType::ROOK.get_piece(Color::WHITE) == Piece::W_ROOK);
    assert!(PieceType::PAWN.get_piece(Color::WHITE) == Piece::W_PAWN);
    assert!(PieceType::SILVER_X.get_piece(Color::WHITE) == Piece::W_SILVER_X);
    assert!(PieceType::BISHOP_X.get_piece(Color::WHITE) == Piece::W_BISHOP_X);
    assert!(PieceType::ROOK_X.get_piece(Color::WHITE) == Piece::W_ROOK_X);
    assert!(PieceType::PAWN_X.get_piece(Color::WHITE) == Piece::W_PAWN_X);

    // Black
    assert!(PieceType::KING.get_piece(Color::BLACK) == Piece::B_KING);
    assert!(PieceType::GOLD.get_piece(Color::BLACK) == Piece::B_GOLD);
    assert!(PieceType::SILVER.get_piece(Color::BLACK) == Piece::B_SILVER);
    assert!(PieceType::BISHOP.get_piece(Color::BLACK) == Piece::B_BISHOP);
    assert!(PieceType::ROOK.get_piece(Color::BLACK) == Piece::B_ROOK);
    assert!(PieceType::PAWN.get_piece(Color::BLACK) == Piece::B_PAWN);
    assert!(PieceType::SILVER_X.get_piece(Color::BLACK) == Piece::B_SILVER_X);
    assert!(PieceType::BISHOP_X.get_piece(Color::BLACK) == Piece::B_BISHOP_X);
    assert!(PieceType::ROOK_X.get_piece(Color::BLACK) == Piece::B_ROOK_X);
    assert!(PieceType::PAWN_X.get_piece(Color::BLACK) == Piece::B_PAWN_X);
}

#[test]
fn get_op_piece_test() {
    assert!(Piece::NO_PIECE.get_op_piece() == Piece::NO_PIECE);

    // White
    assert!(Piece::W_KING.get_op_piece() == Piece::B_KING);
    assert!(Piece::W_GOLD.get_op_piece() == Piece::B_GOLD);
    assert!(Piece::W_SILVER.get_op_piece() == Piece::B_SILVER);
    assert!(Piece::W_BISHOP.get_op_piece() == Piece::B_BISHOP);
    assert!(Piece::W_ROOK.get_op_piece() == Piece::B_ROOK);
    assert!(Piece::W_PAWN.get_op_piece() == Piece::B_PAWN);
    assert!(Piece::W_SILVER_X.get_op_piece() == Piece::B_SILVER_X);
    assert!(Piece::W_BISHOP_X.get_op_piece() == Piece::B_BISHOP_X);
    assert!(Piece::W_ROOK_X.get_op_piece() == Piece::B_ROOK_X);
    assert!(Piece::W_PAWN_X.get_op_piece() == Piece::B_PAWN_X);

    // Black
    assert!(Piece::B_KING.get_op_piece() == Piece::W_KING);
    assert!(Piece::B_GOLD.get_op_piece() == Piece::W_GOLD);
    assert!(Piece::B_SILVER.get_op_piece() == Piece::W_SILVER);
    assert!(Piece::B_BISHOP.get_op_piece() == Piece::W_BISHOP);
    assert!(Piece::B_ROOK.get_op_piece() == Piece::W_ROOK);
    assert!(Piece::B_PAWN.get_op_piece() == Piece::W_PAWN);
    assert!(Piece::B_SILVER_X.get_op_piece() == Piece::W_SILVER_X);
    assert!(Piece::B_BISHOP_X.get_op_piece() == Piece::W_BISHOP_X);
    assert!(Piece::B_ROOK_X.get_op_piece() == Piece::W_ROOK_X);
    assert!(Piece::B_PAWN_X.get_op_piece() == Piece::W_PAWN_X);
}

#[test]
fn get_color_test() {
    assert!(Piece::NO_PIECE.get_color() == Color::NO_COLOR);

    assert!(Piece::W_KING.get_color() == Color::WHITE);
    assert!(Piece::W_GOLD.get_color() == Color::WHITE);
    assert!(Piece::W_SILVER.get_color() == Color::WHITE);
    assert!(Piece::W_BISHOP.get_color() == Color::WHITE);
    assert!(Piece::W_ROOK.get_color() == Color::WHITE);
    assert!(Piece::W_PAWN.get_color() == Color::WHITE);
    assert!(Piece::W_SILVER_X.get_color() == Color::WHITE);
    assert!(Piece::W_BISHOP_X.get_color() == Color::WHITE);
    assert!(Piece::W_ROOK_X.get_color() == Color::WHITE);
    assert!(Piece::W_PAWN_X.get_color() == Color::WHITE);

    assert!(Piece::B_KING.get_color() == Color::BLACK);
    assert!(Piece::B_GOLD.get_color() == Color::BLACK);
    assert!(Piece::B_SILVER.get_color() == Color::BLACK);
    assert!(Piece::B_BISHOP.get_color() == Color::BLACK);
    assert!(Piece::B_ROOK.get_color() == Color::BLACK);
    assert!(Piece::B_PAWN.get_color() == Color::BLACK);
    assert!(Piece::B_SILVER_X.get_color() == Color::BLACK);
    assert!(Piece::B_BISHOP_X.get_color() == Color::BLACK);
    assert!(Piece::B_ROOK_X.get_color() == Color::BLACK);
    assert!(Piece::B_PAWN_X.get_color() == Color::BLACK);
}

#[test]
fn get_piece_type_test() {
    assert!(Piece::NO_PIECE.get_piece_type() == PieceType::NO_PIECE_TYPE);

    assert!(Piece::W_KING.get_piece_type() == PieceType::KING);
    assert!(Piece::W_GOLD.get_piece_type() == PieceType::GOLD);
    assert!(Piece::W_SILVER.get_piece_type() == PieceType::SILVER);
    assert!(Piece::W_BISHOP.get_piece_type() == PieceType::BISHOP);
    assert!(Piece::W_ROOK.get_piece_type() == PieceType::ROOK);
    assert!(Piece::W_PAWN.get_piece_type() == PieceType::PAWN);
    assert!(Piece::W_SILVER_X.get_piece_type() == PieceType::SILVER_X);
    assert!(Piece::W_BISHOP_X.get_piece_type() == PieceType::BISHOP_X);
    assert!(Piece::W_ROOK_X.get_piece_type() == PieceType::ROOK_X);
    assert!(Piece::W_PAWN_X.get_piece_type() == PieceType::PAWN_X);

    assert!(Piece::B_KING.get_piece_type() == PieceType::KING);
    assert!(Piece::B_GOLD.get_piece_type() == PieceType::GOLD);
    assert!(Piece::B_SILVER.get_piece_type() == PieceType::SILVER);
    assert!(Piece::B_BISHOP.get_piece_type() == PieceType::BISHOP);
    assert!(Piece::B_ROOK.get_piece_type() == PieceType::ROOK);
    assert!(Piece::B_PAWN.get_piece_type() == PieceType::PAWN);
    assert!(Piece::B_SILVER_X.get_piece_type() == PieceType::SILVER_X);
    assert!(Piece::B_BISHOP_X.get_piece_type() == PieceType::BISHOP_X);
    assert!(Piece::B_ROOK_X.get_piece_type() == PieceType::ROOK_X);
    assert!(Piece::B_PAWN_X.get_piece_type() == PieceType::PAWN_X);
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Direction {
    N = 0,
    NE = 1,
    E = 2,
    SE = 3,
    S = 4,
    SW = 5,
    W = 6,
    NW = 7,
}

pub const PIECE_ALL: [Piece; 20] = [
    Piece::W_KING,
    Piece::W_GOLD,
    Piece::W_SILVER,
    Piece::W_BISHOP,
    Piece::W_ROOK,
    Piece::W_PAWN,
    Piece::W_SILVER_X,
    Piece::W_BISHOP_X,
    Piece::W_ROOK_X,
    Piece::W_PAWN_X,
    Piece::B_KING,
    Piece::B_GOLD,
    Piece::B_SILVER,
    Piece::B_BISHOP,
    Piece::B_ROOK,
    Piece::B_PAWN,
    Piece::B_SILVER_X,
    Piece::B_BISHOP_X,
    Piece::B_ROOK_X,
    Piece::B_PAWN_X,
];
pub const PIECE_TYPE_ALL: [PieceType; 10] = [
    PieceType::KING,
    PieceType::GOLD,
    PieceType::SILVER,
    PieceType::BISHOP,
    PieceType::ROOK,
    PieceType::PAWN,
    PieceType::SILVER_X,
    PieceType::BISHOP_X,
    PieceType::ROOK_X,
    PieceType::PAWN_X,
];
pub const HAND_PIECE_TYPE_ALL: [PieceType; 5] =
    [PieceType::GOLD, PieceType::SILVER, PieceType::BISHOP, PieceType::ROOK, PieceType::PAWN];
pub const DIRECTION_ALL: [Direction; 8] = [
    Direction::N,
    Direction::NE,
    Direction::E,
    Direction::SE,
    Direction::S,
    Direction::SW,
    Direction::W,
    Direction::NW,
];

pub const SQUARE_NB: usize = 5 * 5;
pub const MAX_PLY: usize = 512;
