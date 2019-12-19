use pyo3::prelude::*;
#[cfg(test)]
use rand::seq::SliceRandom;

use bitboard::*;
use r#move::*;
use types::*;

/// A position is represented here.
#[pyclass(module = "minishogilib")]
#[derive(Copy, Clone)]
pub struct Position {
    /// Which player to move.
    pub side_to_move: Color,
    /// The array of the board.
    pub board: [Piece; SQUARE_NB],
    /// The array of hand pieces (prisoners).
    pub hand: [[u8; 5]; 2],
    /// The number of plies.
    pub ply: u16,

    /// Move history.
    pub kif: [Move; MAX_PLY + 1],
    /// The hash value of positions (including history positions).
    pub hash: [(u64, u64); MAX_PLY + 1],

    /// The bitwise pawn flags that is used not to generate double-pawn (2fu) moves.
    pub pawn_flags: [u8; 2],
    /// The bitboard for each piece.
    pub piece_bb: [Bitboard; Piece::B_PAWN_X.as_usize() + 1],
    /// The bitboard for each player.
    pub player_bb: [Bitboard; 2],
    /// The bitboard of pieces which are checking by neighbors.
    pub adjacent_check_bb: [Bitboard; MAX_PLY + 1],
    /// The bitboard of pieces which are checking not by neighbors.
    pub long_check_bb: [Bitboard; MAX_PLY + 1],
    /// The number of sequential check (including history positions).
    pub sequent_check_count: [[u8; 2]; MAX_PLY + 1],
}

#[pymethods]
impl Position {
    #[new]
    pub fn new(obj: &PyRawObject) {
        obj.init(Position::empty_board());
    }

    fn __getstate__(&self) -> PyResult<(String)> {
        Ok(self.sfen(true))
    }

    fn __setstate__(&mut self, sfen: &str) -> PyResult<()> {
        self.set_sfen(&sfen);
        Ok(())
    }

    /// Copy the `position`
    ///
    /// Arguments:
    /// * `entire`: If true, historical positions are also copied.
    pub fn copy(&self, entire: bool) -> Position {
        if entire {
            return *self;
        }

        let mut position = Position::empty_board();
        position.side_to_move = self.side_to_move;
        for i in 0..SQUARE_NB {
            position.board[i] = self.board[i]
        }
        for i in 0..2 {
            for j in 0..5 {
                position.hand[i][j] = self.hand[i][j];
            }
            position.pawn_flags[i] = self.pawn_flags[i];
            position.player_bb[i] = self.player_bb[i];
        }

        for piece in &PIECE_ALL {
            position.piece_bb[piece.as_usize()] = self.piece_bb[piece.as_usize()];
        }

        position.hash[0] = self.hash[self.ply as usize];
        position.adjacent_check_bb[0] = self.adjacent_check_bb[self.ply as usize];
        position.long_check_bb[0] = self.long_check_bb[self.ply as usize];
        position.sequent_check_count[0] = self.sequent_check_count[self.ply as usize];

        return position;
    }

    /// Output the position.
    pub fn print(&self) {
        println!("side_to_move: {:?}", self.side_to_move);

        for y in 0..5 {
            for x in 0..5 {
                print!("{}", self.board[y * 5 + x]);
            }
            println!("");
        }

        let hand_str = ["G", "S", "B", "R", "P"];

        print!("WHITE HAND: ");
        for i in 0..5 {
            print!("{}: {}, ", hand_str[i], self.hand[(Color::WHITE.as_usize())][i]);
        }
        println!("");

        print!("BLACK HAND: ");
        for i in 0..5 {
            print!("{}: {}, ", hand_str[i], self.hand[(Color::BLACK.as_usize())][i]);
        }
        println!("");

        println!("ply: {}", self.ply);

        {
            let hash = self.get_hash();
            println!("hash: ({:x}, {:x})", hash.0, hash.1);
        }

        println!("repetition: {}", self.get_repetition());
    }

    /// Return the sfen representation of the position.
    pub fn sfen(&self, history: bool) -> String {
        if history {
            let mut position = *self;

            for _ in 0..self.ply {
                position.undo_move();
            }

            let mut sfen_position = position.get_sfen_position();

            if self.ply > 0 {
                sfen_position.push_str(" moves");
            }

            for i in 0..self.ply {
                sfen_position.push_str(&format!(" {}", self.kif[i as usize].sfen()));
            }

            return sfen_position;
        } else {
            return self.get_sfen_position();
        }
    }

    pub fn get_kif(&self) -> std::vec::Vec<String> {
        self.kif[0..self.ply as usize].to_vec().into_iter().map(|x| x.sfen()).collect()
    }

    pub fn get_csa_kif(&self) -> std::vec::Vec<String> {
        self.kif[0..self.ply as usize].to_vec().into_iter().map(|x| x.csa_sfen()).collect()
    }

    /// Set the position by sfen string.
    ///
    /// Arguments:
    /// * `sfen`: The sfen representation of a position.
    /// * `incremental_update`: If false, historical variables (check bitboards, etc...) are not set.
    pub fn _set_sfen_with_option(&mut self, sfen: &str, incremental_update: bool) {
        // 初期化
        for i in 0..SQUARE_NB {
            self.board[i] = Piece::NO_PIECE;
        }
        for i in 0..2 {
            for j in 0..5 {
                self.hand[i][j] = 0;
            }

            self.pawn_flags[i] = 0;
        }

        let mut square: usize = 0;
        let mut promote: bool = false;

        let mut sfen_split = sfen.split_whitespace();

        // sfenから盤面を設定
        for c in sfen_split.next().unwrap().chars() {
            if c == '+' {
                promote = true;
                continue;
            }

            if c == '/' {
                continue;
            }

            if c.is_ascii_digit() {
                square += ((c as u8) - ('0' as u8)) as usize;
                continue;
            }

            let mut piece = char_to_piece(c);

            if promote {
                piece = piece.get_promoted();
            }

            self.board[square] = piece;

            if piece == Piece::W_PAWN {
                self.pawn_flags[Color::WHITE.as_usize()] |= 1 << (square % 5);
            } else if piece == Piece::B_PAWN {
                self.pawn_flags[Color::BLACK.as_usize()] |= 1 << (square % 5);
            }

            promote = false;
            square += 1;
        }

        // 手番を設定
        if sfen_split.next() == Some("b") {
            self.side_to_move = Color::WHITE;
        } else {
            self.side_to_move = Color::BLACK;
        }

        // 持ち駒を設定
        let mut count: u8 = 1;
        for c in sfen_split.next().unwrap().chars() {
            if c == '-' {
                continue;
            }

            if c.is_ascii_digit() {
                count = (c as u8) - ('0' as u8);
                continue;
            }

            let piece = char_to_piece(c);
            let color = piece.get_color();
            let piece_type = piece.get_piece_type();
            let hand_index = (piece_type.as_usize()) - 2;

            self.hand[color.as_usize()][hand_index] = count;

            count = 1;
        }

        self.set_bitboard();
        self.set_check_bb();
        self.hash[0] = self.calculate_hash();

        self.ply = 0;

        sfen_split.next(); // sfenプロトコルで常に1が格納されているはずなので、読み飛ばす

        if sfen_split.next() == Some("moves") {
            loop {
                let sfen_move = sfen_split.next();

                if !sfen_move.is_some() {
                    break;
                }

                let m = self.sfen_to_move(sfen_move.unwrap());
                self._do_move_with_option(&m, incremental_update);
            }
        }
    }

    /// Set a position by the sfen.
    pub fn set_sfen(&mut self, sfen: &str) {
        self._set_sfen_with_option(sfen, true);
    }

    /// Set a position by the sfen, ignoring historical positions.
    pub fn set_sfen_simple(&mut self, sfen: &str) {
        self._set_sfen_with_option(sfen, false);
        self.set_flags();
    }

    /// Set the position to the initial position.
    pub fn set_start_position(&mut self) {
        self.set_sfen_without_startpos("");
    }

    /// Set the position by sfen consisted only by moves.
    pub fn set_sfen_without_startpos(&mut self, sfen: &str) {
        static START_POSITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1";
        let sfen_kif = format!("{} moves {}", START_POSITION_SFEN, sfen);

        self.set_sfen(&sfen_kif);
    }

    /// Set the position by sfen consisted only by moves, ignoring historical positions.
    pub fn set_sfen_without_startpos_simple(&mut self, sfen: &str) {
        static START_POSITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1";
        let sfen_kif = format!("{} moves {}", START_POSITION_SFEN, sfen);

        self.set_sfen_simple(&sfen_kif);
    }

    /// Convert a sfen represented move to a `Move` struct instance.
    pub fn sfen_to_move(&self, sfen: &str) -> Move {
        if sfen.as_bytes()[1] as char == '*' {
            let piece = char_to_piece(sfen.as_bytes()[0] as char)
                .get_piece_type()
                .get_piece(self.side_to_move);
            let to = sfen_to_square(&sfen[2..4]);

            Move::hand_move(piece, to)
        } else {
            let from = sfen_to_square(&sfen[0..2]);
            let to = sfen_to_square(&sfen[2..4]);
            let promotion = sfen.len() == 5;
            let piece = self.board[from];
            let capture_piece = self.board[to];

            Move::board_move(piece, from, to, promotion, capture_piece)
        }
    }

    pub fn get_side_to_move(&self) -> u8 {
        return self.side_to_move.as_usize() as u8;
    }

    pub fn get_ply(&self) -> u16 {
        return self.ply;
    }

    /// Generate legal moves.
    ///
    /// Note: A move that cause immediate checkmate by a pawn (Utifu-dume) is included.
    pub fn generate_moves(&self) -> std::vec::Vec<Move> {
        return self.generate_moves_with_option(true, true, false, false);
    }

    /// Whether the king is in check.
    pub fn is_in_check(&self) -> bool {
        return self.get_check_bb() != 0;
    }

    /// Set bitboards, etc...
    pub fn set_flags(&mut self) {
        self.pawn_flags = [0; 2];
        self.piece_bb = [0; Piece::B_PAWN_X.as_usize() + 1];
        self.player_bb = [0; 2];
        self.adjacent_check_bb = [0; MAX_PLY + 1];
        self.long_check_bb = [0; MAX_PLY + 1];
        self.sequent_check_count = [[0; 2]; MAX_PLY + 1];

        for i in 0..SQUARE_NB {
            if self.board[i] == Piece::W_PAWN {
                self.pawn_flags[Color::WHITE.as_usize()] |= 1 << (i % 5);
            } else if self.board[i] == Piece::B_PAWN {
                self.pawn_flags[Color::BLACK.as_usize()] |= 1 << (i % 5);
            }

            if self.board[i] != Piece::NO_PIECE {
                self.piece_bb[self.board[i].as_usize()] |= 1 << i;
                self.player_bb[self.board[i].get_color().as_usize()] |= 1 << i;
            }
        }

        self.set_check_bb();
    }

    /// Do a move.
    ///
    /// Arguments:
    /// * `move`: The move to do.
    /// * `incremental_update`: If false, historical variables (check bitboards, etc...) are not set.
    pub fn _do_move_with_option(&mut self, m: &Move, incremental_update: bool) {
        assert!(m.capture_piece.get_piece_type() != PieceType::KING);

        self.hash[self.ply as usize + 1] = self.hash[self.ply as usize];

        if m.is_hand {
            // 持ち駒を打つ場合

            self.board[m.to as usize] = m.piece;
            self.hand[self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2] -= 1;

            // Bitboardの更新
            self.piece_bb[m.piece.as_usize()] |= 1 << m.to;
            self.player_bb[self.side_to_move.as_usize()] |= 1 << m.to;

            // 二歩フラグの更新
            if m.piece.get_piece_type() == PieceType::PAWN {
                self.pawn_flags[self.side_to_move.as_usize()] |= 1 << (m.to % 5);
            }

            // hash値の更新
            self.hash[self.ply as usize + 1].0 ^= ::zobrist::BOARD_TABLE[m.to][m.piece.as_usize()];
            self.hash[self.ply as usize + 1].1 ^= ::zobrist::HAND_TABLE
                [self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2]
                [self.hand[self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2]
                    as usize
                    + 1];
            self.hash[self.ply as usize + 1].1 ^= ::zobrist::HAND_TABLE
                [self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2]
                [self.hand[self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2]
                    as usize];
        } else {
            // 盤上の駒を動かす場合

            if m.capture_piece != Piece::NO_PIECE {
                self.hand[self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2] += 1;

                // Bitboardの更新
                self.piece_bb[m.capture_piece.as_usize()] ^= 1 << m.to;
                self.player_bb[self.side_to_move.get_op_color().as_usize()] ^= 1 << m.to;

                // 二歩フラグの更新
                if m.capture_piece.get_piece_type() == PieceType::PAWN {
                    self.pawn_flags[self.side_to_move.get_op_color().as_usize()] ^= 1 << (m.to % 5);
                }

                // hashの更新
                self.hash[self.ply as usize + 1].0 ^=
                    ::zobrist::BOARD_TABLE[m.to][m.capture_piece.as_usize()];
                self.hash[self.ply as usize + 1].1 ^= ::zobrist::HAND_TABLE
                    [self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2][self
                    .hand[self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2]
                    as usize
                    - 1];
                self.hash[self.ply as usize + 1].1 ^= ::zobrist::HAND_TABLE
                    [self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2][self
                    .hand[self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2]
                    as usize];
            }

            if m.promotion {
                self.board[m.to as usize] = m.piece.get_promoted();

                // 二歩フラグの更新
                if m.piece.get_piece_type() == PieceType::PAWN {
                    self.pawn_flags[self.side_to_move.as_usize()] ^= 1 << (m.to % 5);
                }
            } else {
                self.board[m.to as usize] = m.piece;
            }

            self.board[m.from as usize] = Piece::NO_PIECE;

            // Bitboardの更新
            // 移動先
            self.piece_bb[self.board[m.to as usize].as_usize()] |= 1 << m.to;
            self.player_bb[self.side_to_move.as_usize()] |= 1 << m.to;
            // 移動元
            self.piece_bb[m.piece.as_usize()] ^= 1 << m.from;
            self.player_bb[self.side_to_move.as_usize()] ^= 1 << m.from;

            // hash値の更新
            self.hash[self.ply as usize + 1].0 ^=
                ::zobrist::BOARD_TABLE[m.from][m.piece.as_usize()];
            self.hash[self.ply as usize + 1].0 ^=
                ::zobrist::BOARD_TABLE[m.to][self.board[m.to].as_usize()];
        }

        self.hash[self.ply as usize + 1].0 ^= 1; // 手番bitの反転

        // 棋譜に登録
        self.kif[self.ply as usize] = *m;

        // 1手進める
        self.ply += 1;

        // 手番を変える
        self.side_to_move = self.side_to_move.get_op_color();

        if incremental_update {
            // 王手している駒を記録
            self.set_check_bb();

            // 連続王手のカウント
            if self.adjacent_check_bb[self.ply as usize] != 0
                || self.long_check_bb[self.ply as usize] != 0
            {
                self.sequent_check_count[self.ply as usize]
                    [self.side_to_move.get_op_color().as_usize()] = self.sequent_check_count
                    [self.ply as usize - 1][self.side_to_move.get_op_color().as_usize()]
                    + 1;
            } else {
                self.sequent_check_count[self.ply as usize]
                    [self.side_to_move.get_op_color().as_usize()] = 0;
            }
            self.sequent_check_count[self.ply as usize][self.side_to_move.as_usize()] =
                self.sequent_check_count[self.ply as usize - 1][self.side_to_move.as_usize()];
        }
    }

    /// Do a move.
    pub fn do_move(&mut self, m: &Move) {
        self._do_move_with_option(m, true);
    }

    /// Undo the move.
    pub fn undo_move(&mut self) {
        assert!(self.ply > 0);

        // 手数を戻す
        let m = self.kif[(self.ply - 1) as usize];
        self.ply -= 1;

        // 手番を戻す
        self.side_to_move = self.side_to_move.get_op_color();

        if m.is_hand {
            // 持ち駒を打った場合

            self.board[m.to as usize] = Piece::NO_PIECE;
            self.hand[self.side_to_move.as_usize()][m.piece.get_piece_type().as_usize() - 2] += 1;

            // Bitboardのundo
            self.piece_bb[m.piece.as_usize()] ^= 1 << m.to;
            self.player_bb[self.side_to_move.as_usize()] ^= 1 << m.to;

            // 二歩フラグのundo
            if m.piece.get_piece_type() == PieceType::PAWN {
                self.pawn_flags[self.side_to_move.as_usize()] ^= 1 << (m.to % 5);
            }
        } else {
            // 盤上の駒を動かした場合
            assert!(self.board[m.to as usize] != Piece::NO_PIECE);

            // Bitboardのundo
            // 移動先
            self.piece_bb[self.board[m.to as usize].as_usize()] ^= 1 << m.to;
            self.player_bb[self.side_to_move.as_usize()] ^= 1 << m.to;
            // 移動元
            self.piece_bb[m.piece.as_usize()] |= 1 << m.from;
            self.player_bb[self.side_to_move.as_usize()] |= 1 << m.from;

            // 二歩フラグのundo
            if m.piece.get_piece_type() == PieceType::PAWN && m.promotion {
                self.pawn_flags[self.side_to_move.as_usize()] |= 1 << (m.to % 5);
            }

            self.board[m.to as usize] = m.capture_piece;
            self.board[m.from as usize] = m.piece;

            // 相手の駒を取っていた場合には、持ち駒から減らす
            if m.capture_piece != Piece::NO_PIECE {
                self.hand[self.side_to_move.as_usize()]
                    [m.capture_piece.get_piece_type().get_raw().as_usize() - 2] -= 1;

                // Bitboardのundo
                self.piece_bb[m.capture_piece.as_usize()] |= 1 << m.to;
                self.player_bb[self.side_to_move.get_op_color().as_usize()] |= 1 << m.to;

                // 二歩フラグのundo
                if m.capture_piece.get_piece_type() == PieceType::PAWN {
                    self.pawn_flags[self.side_to_move.get_op_color().as_usize()] |= 1 << (m.to % 5);
                }
            }
        }
    }

    /// Whether the position is now under the repetition (sennitite).
    ///
    /// Returns:
    /// * `(repetition, check_repetition)`: If `check_repetition` is true,
    ///                                     the one side has continued check moves which means
    ///                                     immediate the other side's win.
    pub fn is_repetition(&self) -> (bool, bool) {
        if self.ply == 0 {
            return (false, false);
        }

        let mut count = 0;

        let mut ply = self.ply as i32 - 4;
        let mut check_repetition = false;

        while ply >= 0 {
            if self.hash[ply as usize] == self.hash[self.ply as usize] {
                count += 1;

                if count == 1 {
                    if self.sequent_check_count[self.ply as usize][self.side_to_move.as_usize()]
                        >= (self.ply + 1 - ply as u16) as u8 / 2
                        || self.sequent_check_count[self.ply as usize]
                            [self.side_to_move.get_op_color().as_usize()]
                            >= (self.ply + 1 - ply as u16) as u8 / 2
                    {
                        check_repetition = true;
                    }
                }
            }

            // 現在の局面の1手前から数え始めているので、3回 (+ 現在の局面 1回) で千日手
            if count == 3 {
                return (true, check_repetition);
            }

            ply -= 2; // 繰り返し回数は、同じ手番の過去局面だけを見れば良い
        }

        return (false, false);
    }

    /// Return the number of repetition.
    pub fn get_repetition(&self) -> usize {
        let mut count: usize = 0;

        let mut ply = self.ply as i32 - 4;
        while ply >= 0 {
            if self.hash[ply as usize] == self.hash[self.ply as usize] {
                count += 1;
            }

            ply -= 2; // 繰り返し回数は、同じ手番の過去局面だけを見れば良い
        }

        return count;
    }

    /// Output a SVG format image.
    pub fn to_svg(&self) -> String {
        // ToDo:
        //   color_last_move: bool
        //   color_promoted_piece: bool
        //   p1_name: String
        //   p2_name: String

        let mut svg_text: String = String::new();

        svg_text.push_str("<svg width=\"448px\" height=\"384px\"\n     xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">\n");

        svg_text.push_str("  <rect x=\"64\" y=\"32\" width=\"320\" height=\"320\" fill=\"white\" stroke=\"black\" stroke-width=\"3\" />\n");

        for y in 0..5 {
            for x in 0..5 {
                svg_text.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"64\" height=\"64\" fill=\"white\" stroke=\"black\" stroke-width=\"1\" />\n",
                                    64 + 64 * x, 32 + 64 * y));
            }
        }

        for i in 0..SQUARE_NB {
            if self.board[i] != Piece::NO_PIECE {
                let kanji = piece_type_to_kanji(self.board[i].get_piece_type());

                let y = i / 5;
                let x = i % 5;

                if self.board[i].get_color() == Color::WHITE {
                    svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"42\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
                            96 + 64 * x, 64 + 64 * y, kanji));
                } else {
                    svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"42\" text-anchor=\"middle\" dominant-baseline=\"central\" transform=\"rotate(180, {}, {})\">{}</text>\n",
                            96 + 64 * x, 64 + 64 * y, 96 + 64 * x, 64 + 64 * y, kanji));
                }
            }
        }

        {
            svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"36\" writing-mode=\"tb\">&#9751;</text>\n", 420, 32));
            let mut hand_string = String::new();
            for piece_type in &HAND_PIECE_TYPE_ALL {
                if self.hand[Color::WHITE.as_usize()][piece_type.as_usize() - 2] != 0 {
                    hand_string.push_str(&piece_type_to_kanji(*piece_type));
                    if self.hand[Color::WHITE.as_usize()][piece_type.as_usize() - 2] == 2 {
                        hand_string.push_str(&"二".to_string());
                    }
                }
            }

            if !hand_string.is_empty() {
                svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"36\" writing-mode=\"tb\" letter-spacing=\"1\">{}</text>\n", 420, 74, hand_string));
            }
        }

        {
            svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"36\" writing-mode=\"tb\" transform=\"rotate(180, {}, {})\">&#9750;</text>\n", 32, 352, 32, 352));
            let mut hand_string = String::new();
            for piece_type in &HAND_PIECE_TYPE_ALL {
                if self.hand[Color::BLACK.as_usize()][piece_type.as_usize() - 2] != 0 {
                    hand_string.push_str(&piece_type_to_kanji(*piece_type));
                    if self.hand[Color::BLACK.as_usize()][piece_type.as_usize() - 2] == 2 {
                        hand_string.push_str(&"二".to_string());
                    }
                }
            }

            if !hand_string.is_empty() {
                svg_text.push_str(&format!("  <text x=\"{}\" y=\"{}\" font-family=\"serif\" font-size=\"36\" writing-mode=\"tb\" letter-spacing=\"1\" transform=\"rotate(180, {}, {})\">{}</text>\n", 32, 310, 32, 310, hand_string));
            }
        }

        svg_text.push_str("</svg>\n");

        return svg_text;
    }
}

impl Position {
    pub fn empty_board() -> Position {
        Position {
            side_to_move: Color::NO_COLOR,
            board: [Piece::NO_PIECE; SQUARE_NB],
            hand: [[0; 5]; 2],
            pawn_flags: [0; 2],
            piece_bb: [0; Piece::B_PAWN_X.as_usize() + 1],
            player_bb: [0; 2],
            ply: 0,
            kif: [NULL_MOVE; MAX_PLY + 1],
            hash: [(0, 0); MAX_PLY + 1],
            adjacent_check_bb: [0; MAX_PLY + 1],
            long_check_bb: [0; MAX_PLY + 1],
            sequent_check_count: [[0; 2]; MAX_PLY + 1],
        }
    }

    /// 盤上の駒からbitboardを設定する
    fn set_bitboard(&mut self) {
        // 初期化
        for i in 0..Piece::B_PAWN_X.as_usize() + 1 {
            self.piece_bb[i] = 0
        }
        self.player_bb[Color::WHITE.as_usize()] = 0;
        self.player_bb[Color::BLACK.as_usize()] = 0;

        // 盤上の駒に対応する場所のbitを立てる
        for i in 0..SQUARE_NB {
            if self.board[i] != Piece::NO_PIECE {
                self.piece_bb[self.board[i].as_usize()] |= 1 << i;
                self.player_bb[self.board[i].get_color().as_usize()] |= 1 << i;
            }
        }
    }

    fn set_check_bb(&mut self) {
        self.adjacent_check_bb[self.ply as usize] = 0;
        self.long_check_bb[self.ply as usize] = 0;

        let king_square =
            get_square(self.piece_bb[PieceType::KING.get_piece(self.side_to_move).as_usize()]);

        assert!(king_square < SQUARE_NB);

        for piece_type in PIECE_TYPE_ALL.iter() {
            let check_bb = adjacent_attack(king_square, piece_type.get_piece(self.side_to_move))
                & self.piece_bb[piece_type.get_piece(self.side_to_move.get_op_color()).as_usize()];

            if check_bb != 0 {
                self.adjacent_check_bb[self.ply as usize] |= check_bb;
            }
        }

        let player_bb =
            self.player_bb[Color::WHITE.as_usize()] | self.player_bb[Color::BLACK.as_usize()];

        // 角による王手
        let bishop_check_bb = bishop_attack(king_square, player_bb);
        self.long_check_bb[self.ply as usize] |= bishop_check_bb
            & self.piece_bb
                [PieceType::BISHOP.get_piece(self.side_to_move.get_op_color()).as_usize()];
        self.long_check_bb[self.ply as usize] |= bishop_check_bb
            & self.piece_bb
                [PieceType::BISHOP_X.get_piece(self.side_to_move.get_op_color()).as_usize()];

        // 飛車による王手
        let rook_check_bb = rook_attack(king_square, player_bb);
        self.long_check_bb[self.ply as usize] |= rook_check_bb
            & self.piece_bb[PieceType::ROOK.get_piece(self.side_to_move.get_op_color()).as_usize()];
        self.long_check_bb[self.ply as usize] |= rook_check_bb
            & self.piece_bb
                [PieceType::ROOK_X.get_piece(self.side_to_move.get_op_color()).as_usize()];
    }

    fn calculate_hash(&self) -> (u64, u64) {
        let mut hash: u64 = 0;

        for i in 0..SQUARE_NB {
            if self.board[i] != Piece::NO_PIECE {
                hash ^= ::zobrist::BOARD_TABLE[i][self.board[i].as_usize()];
            }
        }

        if self.side_to_move == Color::BLACK {
            hash |= 1;
        }

        let mut hand_hash: u64 = 0;

        for i in 0..2 {
            for j in 0..5 {
                hand_hash ^= ::zobrist::HAND_TABLE[i][j][self.hand[i][j] as usize];
            }
        }

        return (hash, hand_hash);
    }

    fn get_hash(&self) -> (u64, u64) {
        return self.hash[self.ply as usize];
    }

    pub fn get_adjacent_check_bb(&self) -> Bitboard {
        return self.adjacent_check_bb[self.ply as usize];
    }

    pub fn get_long_check_bb(&self) -> Bitboard {
        return self.long_check_bb[self.ply as usize];
    }

    pub fn get_check_bb(&self) -> Bitboard {
        return self.get_adjacent_check_bb() | self.get_long_check_bb();
    }

    pub fn get_sfen_position(&self) -> String {
        let mut sfen_position = String::new();

        let mut empty: u8 = 0;

        for i in 0..SQUARE_NB {
            if self.board[i] == Piece::NO_PIECE {
                empty += 1;
            } else {
                if empty > 0 {
                    sfen_position.push_str(&empty.to_string());
                }
                empty = 0;

                sfen_position.push_str(&piece_to_string(self.board[i]));
            }

            if i % 5 == 4 {
                if empty > 0 {
                    sfen_position.push_str(&empty.to_string());
                }
                empty = 0;

                if i != SQUARE_NB - 1 {
                    sfen_position.push('/');
                }
            }
        }

        sfen_position.push(' ');

        if self.side_to_move == Color::WHITE {
            sfen_position.push('b');
        } else {
            sfen_position.push('w');
        }

        sfen_position.push(' ');

        let mut capture_flag = false;

        for piece_type in &HAND_PIECE_TYPE_ALL {
            if self.hand[Color::WHITE.as_usize()][piece_type.as_usize() - 2] > 0 {
                sfen_position.push_str(
                    &self.hand[Color::WHITE.as_usize()][piece_type.as_usize() - 2].to_string(),
                );
                sfen_position.push_str(&piece_to_string(piece_type.get_piece(Color::WHITE)));
                capture_flag = true;
            }
            if self.hand[Color::BLACK.as_usize()][piece_type.as_usize() - 2] > 0 {
                sfen_position.push_str(
                    &self.hand[Color::BLACK.as_usize()][piece_type.as_usize() - 2].to_string(),
                );
                sfen_position.push_str(&piece_to_string(piece_type.get_piece(Color::BLACK)));
                capture_flag = true;
            }
        }

        if !capture_flag {
            sfen_position.push('-');
        }

        sfen_position.push(' ');
        sfen_position.push('1');

        return sfen_position;
    }

    pub fn generate_moves_with_option(
        &self,
        is_board: bool,
        is_hand: bool,
        allow_illegal: bool,
        check_drop_only: bool,
    ) -> std::vec::Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        if is_board {
            let mut player_bb: Bitboard = self.player_bb[self.side_to_move.as_usize()];

            while player_bb != 0 {
                let i = get_square(player_bb);
                player_bb ^= 1 << i;

                // 両王手がかかっているときは，玉を逃げる以外は非合法手
                if !allow_illegal
                    && get_counts(
                        self.adjacent_check_bb[self.ply as usize]
                            | self.long_check_bb[self.ply as usize],
                    ) > 1
                {
                    if self.board[i].get_piece_type() != PieceType::KING {
                        continue;
                    }
                }

                // 飛び駒以外の駒の移動
                {
                    let mut move_tos: Bitboard = adjacent_attack(i, self.board[i]); // 利きの取得
                    move_tos = move_tos & !self.player_bb[self.side_to_move.as_usize()]; // 自分の駒がある場所には動けない

                    while move_tos != 0 {
                        let move_to: usize = get_square(move_tos); // 行先を1か所取得する

                        // 近接王手がかかっていて，玉以外を動かす場合には，王手している駒を取るしかない
                        if !allow_illegal
                            && self.adjacent_check_bb[self.ply as usize] != 0
                            && self.board[i].get_piece_type() != PieceType::KING
                            && (self.adjacent_check_bb[self.ply as usize] & (1 << move_to)) == 0
                        {
                            move_tos ^= 1 << move_to;
                            continue;
                        }

                        let capture_piece = self.board[move_to];

                        if (self.board[i] == Piece::W_PAWN && move_to < 5)
                            || (self.board[i] == Piece::B_PAWN && move_to >= 20)
                        {
                            // 行き場のない歩の不成の手は生成しない
                        } else {
                            moves.push(Move::board_move(
                                self.board[i],
                                i,
                                move_to,
                                false,
                                capture_piece,
                            ));
                        }

                        // 成る手の生成
                        if self.board[i].is_raw()
                            && self.board[i].is_promotable()
                            && ((self.side_to_move == Color::WHITE && (move_to < 5 || i < 5))
                                || (self.side_to_move == Color::BLACK
                                    && (move_to >= 20 || i >= 20)))
                        {
                            moves.push(Move::board_move(
                                self.board[i],
                                i,
                                move_to,
                                true,
                                capture_piece,
                            ));
                        }

                        move_tos ^= 1 << move_to;
                    }
                }

                let all_player_bb = self.player_bb[Color::WHITE.as_usize()]
                    | self.player_bb[Color::BLACK.as_usize()];

                // 飛び駒の移動
                // 角、馬
                if self.board[i].get_piece_type() == PieceType::BISHOP
                    || self.board[i].get_piece_type() == PieceType::BISHOP_X
                {
                    let mut move_tos: Bitboard = bishop_attack(i, all_player_bb);
                    move_tos &= !self.player_bb[self.side_to_move.as_usize()];

                    while move_tos != 0 {
                        let move_to: usize = get_square(move_tos);

                        if !allow_illegal
                            && self.adjacent_check_bb[self.ply as usize] != 0
                            && self.board[i].get_piece_type() != PieceType::KING
                            && (self.adjacent_check_bb[self.ply as usize] & (1 << move_to)) == 0
                        {
                            move_tos ^= 1 << move_to;
                            continue;
                        }

                        let capture_piece = self.board[move_to];

                        moves.push(Move::board_move(
                            self.board[i],
                            i,
                            move_to,
                            false,
                            capture_piece,
                        ));

                        // 成る手の生成
                        if self.board[i].is_raw()
                            && self.board[i].is_promotable()
                            && ((self.side_to_move == Color::WHITE && (move_to < 5 || i < 5))
                                || (self.side_to_move == Color::BLACK
                                    && (move_to >= 20 || i >= 20)))
                        {
                            moves.push(Move::board_move(
                                self.board[i],
                                i,
                                move_to,
                                true,
                                capture_piece,
                            ));
                        }

                        move_tos ^= 1 << move_to;
                    }
                }
                // 飛、龍
                else if self.board[i].get_piece_type() == PieceType::ROOK
                    || self.board[i].get_piece_type() == PieceType::ROOK_X
                {
                    let mut move_tos: Bitboard = rook_attack(i, all_player_bb);
                    move_tos &= !self.player_bb[self.side_to_move.as_usize()];

                    while move_tos != 0 {
                        let move_to: usize = get_square(move_tos);

                        if !allow_illegal
                            && self.adjacent_check_bb[self.ply as usize] != 0
                            && self.board[i].get_piece_type() != PieceType::KING
                            && (self.adjacent_check_bb[self.ply as usize] & (1 << move_to)) == 0
                        {
                            move_tos ^= 1 << move_to;
                            continue;
                        }

                        let capture_piece = self.board[move_to];

                        moves.push(Move::board_move(
                            self.board[i],
                            i,
                            move_to,
                            false,
                            capture_piece,
                        ));

                        // 成る手の生成
                        if self.board[i].is_raw()
                            && self.board[i].is_promotable()
                            && ((self.side_to_move == Color::WHITE && (move_to < 5 || i < 5))
                                || (self.side_to_move == Color::BLACK
                                    && (move_to >= 20 || i >= 20)))
                        {
                            moves.push(Move::board_move(
                                self.board[i],
                                i,
                                move_to,
                                true,
                                capture_piece,
                            ));
                        }

                        move_tos ^= 1 << move_to;
                    }
                }
            }
        }

        // 近接駒に王手されている場合、持ち駒を打つ手は全て非合法手
        if is_hand && (allow_illegal || self.adjacent_check_bb[self.ply as usize] == 0) {
            // 駒のない升を列挙
            let empty_squares: Bitboard = ONE_BB
                ^ (self.player_bb[Color::WHITE.as_usize()]
                    | self.player_bb[Color::BLACK.as_usize()]);

            for piece_type in HAND_PIECE_TYPE_ALL.iter() {
                if self.hand[self.side_to_move.as_usize()][piece_type.as_usize() - 2] > 0 {
                    let mut empty_squares = empty_squares;

                    if check_drop_only {
                        // 王手となる手のみを生成
                        let op_king_square = get_square(
                            self.piece_bb[PieceType::KING
                                .get_piece(self.side_to_move.get_op_color())
                                .as_usize()],
                        );

                        let mut check_squares: Bitboard = adjacent_attack(
                            op_king_square,
                            piece_type.get_piece(self.side_to_move.get_op_color()),
                        );

                        let player_bb = (self.player_bb[Color::WHITE.as_usize()]
                            | self.player_bb[Color::BLACK.as_usize()])
                            ^ (1 << op_king_square);

                        if *piece_type == PieceType::BISHOP || *piece_type == PieceType::BISHOP_X {
                            check_squares |= bishop_attack(op_king_square, player_bb);
                        }

                        if *piece_type == PieceType::ROOK || *piece_type == PieceType::ROOK_X {
                            check_squares |= rook_attack(op_king_square, player_bb);
                        }

                        empty_squares &= check_squares;
                    }

                    while empty_squares != 0 {
                        let target = get_square(empty_squares);
                        empty_squares ^= 1 << target;

                        // 二歩は禁じ手
                        if *piece_type == PieceType::PAWN
                            && self.pawn_flags[self.side_to_move.as_usize()] & (1 << (target % 5))
                                != 0
                        {
                            continue;
                        }

                        // 行き場のない駒を打たない
                        if *piece_type == PieceType::PAWN
                            && ((self.side_to_move == Color::WHITE && target < 5)
                                || (self.side_to_move == Color::BLACK && target >= 20))
                        {
                            continue;
                        }

                        moves
                            .push(Move::hand_move(piece_type.get_piece(self.side_to_move), target));
                    }
                }
            }
        }

        // 非合法手を取り除く
        if !allow_illegal {
            let king_square =
                get_square(self.piece_bb[PieceType::KING.get_piece(self.side_to_move).as_usize()]);

            let mut index: usize = 0;

            loop {
                if index == moves.len() {
                    break;
                }

                let is_legal = |m: Move| -> bool {
                    if m.is_hand {
                        // 持ち駒を打つ場合
                        let player_bb: Bitboard = self.player_bb[Color::WHITE.as_usize()]
                            | self.player_bb[Color::BLACK.as_usize()]
                            | (1 << m.to);

                        // 角による王手
                        let bishop_check_bb = bishop_attack(king_square, player_bb);
                        if bishop_check_bb
                            & self.piece_bb[PieceType::BISHOP
                                .get_piece(self.side_to_move.get_op_color())
                                .as_usize()]
                            != 0
                            || bishop_check_bb
                                & self.piece_bb[PieceType::BISHOP_X
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                        {
                            return false;
                        }

                        // 飛車による王手
                        let rook_check_bb = rook_attack(king_square, player_bb);
                        if rook_check_bb
                            & self.piece_bb[PieceType::ROOK
                                .get_piece(self.side_to_move.get_op_color())
                                .as_usize()]
                            != 0
                            || rook_check_bb
                                & self.piece_bb[PieceType::ROOK_X
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                        {
                            return false;
                        }
                    } else {
                        // 盤上の駒を動かす場合
                        if m.piece.get_piece_type() == PieceType::KING {
                            // 王を動かす場合
                            let player_bb: Bitboard = (self.player_bb[Color::WHITE.as_usize()]
                                | self.player_bb[Color::BLACK.as_usize()]
                                | (1 << m.to))
                                ^ (1 << m.from);

                            // 角による王手
                            let bishop_check_bb = bishop_attack(m.to as usize, player_bb);

                            if bishop_check_bb
                                & self.piece_bb[PieceType::BISHOP
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                                || bishop_check_bb
                                    & self.piece_bb[PieceType::BISHOP_X
                                        .get_piece(self.side_to_move.get_op_color())
                                        .as_usize()]
                                    != 0
                            {
                                return false;
                            }

                            // 飛車による王手
                            let rook_check_bb = rook_attack(m.to as usize, player_bb);

                            if rook_check_bb
                                & self.piece_bb[PieceType::ROOK
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                                || rook_check_bb
                                    & self.piece_bb[PieceType::ROOK_X
                                        .get_piece(self.side_to_move.get_op_color())
                                        .as_usize()]
                                    != 0
                            {
                                return false;
                            }

                            // 近接王手
                            for piece_type in PIECE_TYPE_ALL.iter() {
                                let check_bb = adjacent_attack(
                                    m.to as usize,
                                    piece_type.get_piece(self.side_to_move),
                                ) & self.piece_bb[piece_type
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()];

                                if check_bb != 0 {
                                    return false;
                                }
                            }
                        } else {
                            // 王以外を動かす場合
                            if get_counts(self.adjacent_check_bb[self.ply as usize]) > 1 {
                                // 近接駒に両王手されている場合は玉を動かさないといけない
                                return false;
                            } else if get_counts(self.adjacent_check_bb[self.ply as usize]) == 1 {
                                // 王手している近接駒を取る手でないといけない
                                if self.adjacent_check_bb[self.ply as usize] & (1 << m.to) == 0 {
                                    return false;
                                }
                            }

                            let player_bb: Bitboard = (self.player_bb[Color::WHITE.as_usize()]
                                | self.player_bb[Color::BLACK.as_usize()]
                                | (1 << m.to))
                                ^ (1 << m.from);

                            // 角による王手
                            let bishop_check_bb =
                                bishop_attack(king_square, player_bb) & !(1 << m.to);
                            if bishop_check_bb
                                & self.piece_bb[PieceType::BISHOP
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                                || bishop_check_bb
                                    & self.piece_bb[PieceType::BISHOP_X
                                        .get_piece(self.side_to_move.get_op_color())
                                        .as_usize()]
                                    != 0
                            {
                                return false;
                            }

                            // 飛車による王手
                            let rook_check_bb = rook_attack(king_square, player_bb) & !(1 << m.to);

                            if rook_check_bb
                                & self.piece_bb[PieceType::ROOK
                                    .get_piece(self.side_to_move.get_op_color())
                                    .as_usize()]
                                != 0
                                || rook_check_bb
                                    & self.piece_bb[PieceType::ROOK_X
                                        .get_piece(self.side_to_move.get_op_color())
                                        .as_usize()]
                                    != 0
                            {
                                return false;
                            }
                        }
                    }

                    return true;
                }(moves[index]);

                if !is_legal {
                    moves.swap_remove(index);

                    continue;
                }

                index += 1;
            }
        }

        return moves;
    }
}

fn char_to_piece(c: char) -> Piece {
    match c {
        'K' => Piece::W_KING,
        'G' => Piece::W_GOLD,
        'S' => Piece::W_SILVER,
        'B' => Piece::W_BISHOP,
        'R' => Piece::W_ROOK,
        'P' => Piece::W_PAWN,

        'k' => Piece::B_KING,
        'g' => Piece::B_GOLD,
        's' => Piece::B_SILVER,
        'b' => Piece::B_BISHOP,
        'r' => Piece::B_ROOK,
        'p' => Piece::B_PAWN,

        _ => Piece::NO_PIECE,
    }
}

fn piece_to_string(piece: Piece) -> String {
    match piece {
        Piece::W_KING => "K".to_string(),
        Piece::W_GOLD => "G".to_string(),
        Piece::W_SILVER => "S".to_string(),
        Piece::W_BISHOP => "B".to_string(),
        Piece::W_ROOK => "R".to_string(),
        Piece::W_PAWN => "P".to_string(),
        Piece::W_SILVER_X => "+S".to_string(),
        Piece::W_BISHOP_X => "+B".to_string(),
        Piece::W_ROOK_X => "+R".to_string(),
        Piece::W_PAWN_X => "+P".to_string(),

        Piece::B_KING => "k".to_string(),
        Piece::B_GOLD => "g".to_string(),
        Piece::B_SILVER => "s".to_string(),
        Piece::B_BISHOP => "b".to_string(),
        Piece::B_ROOK => "r".to_string(),
        Piece::B_PAWN => "p".to_string(),
        Piece::B_SILVER_X => "+s".to_string(),
        Piece::B_BISHOP_X => "+b".to_string(),
        Piece::B_ROOK_X => "+r".to_string(),
        Piece::B_PAWN_X => "+p".to_string(),

        _ => "ERROR".to_string(),
    }
}

fn piece_type_to_kanji(piece_type: PieceType) -> String {
    match piece_type {
        PieceType::KING => "玉".to_string(),
        PieceType::GOLD => "金".to_string(),
        PieceType::SILVER => "銀".to_string(),
        PieceType::BISHOP => "角".to_string(),
        PieceType::ROOK => "飛".to_string(),
        PieceType::PAWN => "歩".to_string(),
        PieceType::SILVER_X => "全".to_string(),
        PieceType::BISHOP_X => "馬".to_string(),
        PieceType::ROOK_X => "龍".to_string(),
        PieceType::PAWN_X => "と".to_string(),

        _ => "".to_string(),
    }
}

#[test]
fn pawn_flags_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 100000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let mut pawn_flag: [[bool; 5]; 2] = [[false; 5]; 2];

            // 二歩フラグの差分更新が正しく動作していることを確認する
            for i in 0..SQUARE_NB {
                if position.board[i] == Piece::W_PAWN {
                    pawn_flag[Color::WHITE.as_usize()][(i % 5) as usize] = true;
                } else if position.board[i] == Piece::B_PAWN {
                    pawn_flag[Color::BLACK.as_usize()][(i % 5) as usize] = true;
                }
            }
            for i in 0..5 {
                assert_eq!(
                    pawn_flag[Color::WHITE.as_usize()][i],
                    (position.pawn_flags[Color::WHITE.as_usize()] & (1 << i)) != 0
                );
                assert_eq!(
                    pawn_flag[Color::BLACK.as_usize()][i],
                    (position.pawn_flags[Color::BLACK.as_usize()] & (1 << i)) != 0
                );
            }

            let moves = position.generate_moves();
            if moves.len() == 0 {
                break;
            }

            // ランダムに局面を進める
            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn move_do_undo_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 10000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            for m in &moves {
                let mut temp_position = position;

                if m.capture_piece.get_piece_type() == PieceType::KING {
                    continue;
                }

                temp_position.do_move(m);
                temp_position.undo_move();

                // do_move -> undo_moveで元の局面と一致するはず
                assert_eq!(position.side_to_move, temp_position.side_to_move);
                for i in 0..SQUARE_NB {
                    assert_eq!(position.board[i], temp_position.board[i]);
                }
                for i in 0..2 {
                    for j in 0..5 {
                        assert_eq!(position.hand[i][j], temp_position.hand[i][j]);
                    }
                }

                for i in 0..Piece::B_PAWN_X.as_usize() + 1 {
                    assert_eq!(position.piece_bb[i], temp_position.piece_bb[i]);
                }
                for i in 0..2 {
                    assert_eq!(position.player_bb[i], temp_position.player_bb[i]);
                }

                for i in 0..2 {
                    assert_eq!(position.pawn_flags[i], temp_position.pawn_flags[i]);
                }

                assert_eq!(position.ply, temp_position.ply);

                for i in 0..position.ply as usize {
                    assert!(position.kif[i] == temp_position.kif[i]);
                }

                assert_eq!(position.get_hash(), temp_position.get_hash());

                for i in 0..position.ply as usize {
                    assert_eq!(position.adjacent_check_bb[i], temp_position.adjacent_check_bb[i]);
                    assert_eq!(position.long_check_bb[i], temp_position.long_check_bb[i]);
                }

                for i in 0..position.ply as usize {
                    for j in 0..2 {
                        assert_eq!(
                            position.sequent_check_count[i][j],
                            temp_position.sequent_check_count[i][j]
                        );
                    }
                }
            }

            if moves.len() == 0 {
                break;
            }

            // ランダムに局面を進める
            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn sfen_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 1000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            {
                let mut temp_position = Position::empty_board();
                temp_position.set_sfen(&position.sfen(true));

                assert_eq!(position.side_to_move, temp_position.side_to_move);
                for i in 0..SQUARE_NB {
                    assert_eq!(position.board[i], temp_position.board[i]);
                }
                for i in 0..2 {
                    for j in 0..5 {
                        assert_eq!(position.hand[i][j], temp_position.hand[i][j]);
                    }
                }

                for i in 0..Piece::B_PAWN_X.as_usize() + 1 {
                    assert_eq!(position.piece_bb[i], temp_position.piece_bb[i]);
                }
                for i in 0..2 {
                    assert_eq!(position.player_bb[i], temp_position.player_bb[i]);
                }

                for i in 0..2 {
                    assert_eq!(position.pawn_flags[i], temp_position.pawn_flags[i]);
                }

                assert_eq!(position.ply, temp_position.ply);

                for i in 0..position.ply as usize {
                    assert!(position.kif[i] == temp_position.kif[i]);
                }

                assert_eq!(position.get_hash(), temp_position.get_hash());

                for i in 0..position.ply as usize {
                    assert_eq!(position.adjacent_check_bb[i], temp_position.adjacent_check_bb[i]);
                    assert_eq!(position.long_check_bb[i], temp_position.long_check_bb[i]);
                }

                for i in 0..position.ply as usize {
                    for j in 0..2 {
                        assert_eq!(
                            position.sequent_check_count[i][j],
                            temp_position.sequent_check_count[i][j]
                        );
                    }
                }
            }

            {
                let mut temp_position = Position::empty_board();
                temp_position.set_sfen(&position.sfen(false));

                assert_eq!(position.side_to_move, temp_position.side_to_move);
                for i in 0..SQUARE_NB {
                    assert_eq!(position.board[i], temp_position.board[i]);
                }
                for i in 0..2 {
                    for j in 0..5 {
                        assert_eq!(position.hand[i][j], temp_position.hand[i][j]);
                    }
                }

                for i in 0..Piece::B_PAWN_X.as_usize() + 1 {
                    assert_eq!(position.piece_bb[i], temp_position.piece_bb[i]);
                }
                for i in 0..2 {
                    assert_eq!(position.player_bb[i], temp_position.player_bb[i]);
                }

                for i in 0..2 {
                    assert_eq!(position.pawn_flags[i], temp_position.pawn_flags[i]);
                }
            }

            if moves.len() == 0 {
                break;
            }

            // ランダムに局面を進める
            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn bitboard_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 100000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            for i in 0..SQUARE_NB {
                if position.board[i] == Piece::NO_PIECE {
                    continue;
                }

                assert!(position.piece_bb[position.board[i].as_usize()] & (1 << i) != 0);
            }

            let moves = position.generate_moves();
            if moves.len() == 0 {
                break;
            }

            // ランダムに局面を進める
            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn no_legal_move_test() {
    ::bitboard::init();

    static CHECKMATE_SFEN1: &str = "5/5/2p2/2g2/2K2 b P 1";
    static CHECKMATE_SFEN2: &str = "4k/1s1gp/p4/g1BS1/1KR2 b BRg 1";
    static CHECKMATE_SFEN3: &str = "4k/2G2/5/5/4R w - 1";
    static CHECKMATE_SFEN4: &str = "r4/5/5/2g2/K4 b - 1";
    static CHECKMATE_SFEN5: &str = "2G1k/5/4P/5/B4 w - 1";
    static CHECKMATE_SFEN6: &str = "4b/5/p4/5/K1g2 b - 1";
    static CHECKMATE_SFEN7: &str = "k1G2/5/P4/5/4B w - 1";
    static CHECKMATE_SFEN8: &str = "b4/5/4p/5/2g1K b - 1";
    static CHECKMATE_SFEN9: &str = "R4/2G1k/5/4P/1B3 w - 1";
    static CHECKMATE_SFEN10: &str = "r4/2g1K/5/4g/1b3 b - 1";

    let mut position = Position::empty_board();

    position.set_sfen(CHECKMATE_SFEN1);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN2);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN3);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN4);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN5);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN6);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN7);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN8);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN9);
    assert_eq!(position.generate_moves().len(), 0);

    position.set_sfen(CHECKMATE_SFEN10);
    assert_eq!(position.generate_moves().len(), 0);
}

#[test]
fn not_checkmate_positions() {
    ::bitboard::init();

    static NOT_CHECKMATE_SFEN1: &str = "rb1gk/1s2R/5/P1B2/KGS2 w P 1";

    let mut position = Position::empty_board();

    position.set_sfen(NOT_CHECKMATE_SFEN1);
    assert!(position.generate_moves().len() > 0);
}

#[test]
fn no_king_capture_move_in_legal_moves_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 100000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            for m in &moves {
                // 玉が取られる手は生成しないはず
                // -> 玉が取れる局面に遭遇しないはず
                assert!(m.capture_piece.get_piece_type() != PieceType::KING);
            }

            // ランダムに局面を進める
            if moves.len() == 0 {
                break;
            }

            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn generate_moves_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 10000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();
            let allow_illegal_moves = position.generate_moves_with_option(true, true, true, false);

            let mut legal_move_count = allow_illegal_moves.len();
            for m in allow_illegal_moves {
                position.do_move(&m);

                let all_moves = position.generate_moves_with_option(true, true, true, false);

                for m2 in all_moves {
                    if m2.capture_piece.get_piece_type() == PieceType::KING {
                        legal_move_count -= 1;
                        break;
                    }
                }

                position.undo_move();
            }

            assert_eq!(moves.len(), legal_move_count);

            // ランダムに局面を進める
            if moves.len() == 0 {
                break;
            }
            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn hash_test() {
    ::bitboard::init();
    ::zobrist::init();

    const LOOP_NUM: i32 = 100000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            if moves.len() == 0 {
                break;
            }

            // 差分計算と全計算の値が一致することを確認する
            assert_eq!(position.get_hash(), position.calculate_hash());

            // 手番bitと手番が一致することを確認する
            assert_eq!(position.side_to_move == Color::BLACK, position.get_hash().0 & 1 != 0);

            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn is_repetition_test() {
    ::bitboard::init();
    ::zobrist::init();

    let mut position = Position::empty_board();

    static START_POSITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1";
    static REPETITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1 moves 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a";
    static REPETITION_SFEN2: &str = "rbsgk/4p/5/P4/KGSBR b - 1 moves 3e2d 3a4b 2e3d 2a2b 4e4d 4a3b 5e4e 5a4a 3d5b 4a5a 5b3d 5a4a 3d5b 4a5a 5b2e 5a4a 2e5b 4a5a 5b3d 5a4a 3d5b";
    static CHECK_REPETITION_SFEN: &str = "2k2/5/5/5/2K2 b R 1 moves R*3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c";
    static NOT_REPETITION_SFEN: &str =
        "rbsgk/4p/5/P4/KGSBR b - 1 moves 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a";
    static NOT_CHECK_REPETITION_SFEN: &str =
        "2k2/5/5/5/2K2 b R 1 moves R*3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a";

    position.set_sfen(START_POSITION_SFEN);
    assert_eq!(position.is_repetition(), (false, false));

    position.set_sfen(REPETITION_SFEN);
    assert_eq!(position.is_repetition(), (true, false));

    position.set_sfen(REPETITION_SFEN2);
    assert_eq!(position.is_repetition(), (true, false));

    position.set_sfen(CHECK_REPETITION_SFEN);
    assert_eq!(position.is_repetition(), (true, true));

    position.set_sfen(NOT_REPETITION_SFEN);
    assert_eq!(position.is_repetition(), (false, false));

    position.set_sfen(NOT_CHECK_REPETITION_SFEN);
    assert_eq!(position.is_repetition(), (false, false));
}

#[test]
fn get_repetition_test() {
    ::bitboard::init();
    ::zobrist::init();

    let mut position = Position::empty_board();

    static START_POSITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1";
    static REPETITION_SFEN: &str = "rbsgk/4p/5/P4/KGSBR b - 1 moves 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a";
    static CHECK_REPETITION_SFEN: &str = "2k2/5/5/5/2K2 b R 1 moves R*3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c";
    static NOT_REPETITION_SFEN: &str =
        "rbsgk/4p/5/P4/KGSBR b - 1 moves 5e4d 1a2b 4d5e 2b1a 5e4d 1a2b 4d5e 2b1a";
    static NOT_CHECK_REPETITION_SFEN: &str =
        "2k2/5/5/5/2K2 b R 1 moves R*3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a 2c3c 3a2a 3c2c 2a3a";

    position.set_sfen(START_POSITION_SFEN);
    assert_eq!(position.get_repetition(), 0);

    position.set_sfen(REPETITION_SFEN);
    assert_eq!(position.get_repetition(), 3);

    position.set_sfen(CHECK_REPETITION_SFEN);
    assert_eq!(position.get_repetition(), 3);

    position.set_sfen(NOT_REPETITION_SFEN);
    assert_eq!(position.get_repetition(), 2);

    position.set_sfen(NOT_CHECK_REPETITION_SFEN);
    assert_eq!(position.get_repetition(), 2);
}

#[test]
fn sfen_to_move_test() {
    ::bitboard::init();
    ::zobrist::init();

    const LOOP_NUM: i32 = 10000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            if moves.len() == 0 {
                break;
            }

            for m in &moves {
                let sfen_move = position.sfen_to_move(&m.sfen());
                assert_eq!(sfen_move, *m);
            }

            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[test]
fn init_position_moves_test() {
    ::bitboard::init();
    ::zobrist::init();

    let mut position = Position::empty_board();
    position.set_start_position();
    let moves = position.generate_moves();

    assert_eq!(moves.len(), 14);
}

#[test]
fn do_move_simple_test() {
    ::bitboard::init();

    const LOOP_NUM: i32 = 100000;

    let mut position = Position::empty_board();

    let mut rng = rand::thread_rng();

    for _ in 0..LOOP_NUM {
        position.set_start_position();

        while position.ply < MAX_PLY as u16 {
            let moves = position.generate_moves();

            let mut simple_position = Position::empty_board();
            simple_position.set_start_position();

            for ply in 0..position.ply {
                simple_position._do_move_with_option(&position.kif[ply as usize], false);
            }

            simple_position.set_flags();

            assert_eq!(position.side_to_move, simple_position.side_to_move);
            assert_eq!(position.ply, simple_position.ply);
            for i in 0..SQUARE_NB {
                assert_eq!(position.board[i], simple_position.board[i]);
            }
            for i in 0..5 {
                assert_eq!(
                    position.hand[Color::WHITE.as_usize()][i],
                    simple_position.hand[Color::WHITE.as_usize()][i]
                );
                assert_eq!(
                    position.hand[Color::BLACK.as_usize()][i],
                    simple_position.hand[Color::BLACK.as_usize()][i]
                );
            }
            for i in 0..position.ply as usize {
                assert_eq!(position.kif[i], simple_position.kif[i]);
                assert_eq!(position.hash[i], simple_position.hash[i]);
            }
            assert_eq!(
                position.pawn_flags[Color::WHITE.as_usize()],
                simple_position.pawn_flags[Color::WHITE.as_usize()]
            );
            assert_eq!(
                position.pawn_flags[Color::BLACK.as_usize()],
                simple_position.pawn_flags[Color::BLACK.as_usize()]
            );
            for i in 0..Piece::B_PAWN_X.as_usize() + 1 {
                assert_eq!(position.piece_bb[i], simple_position.piece_bb[i]);
            }
            assert_eq!(
                position.player_bb[Color::WHITE.as_usize()],
                simple_position.player_bb[Color::WHITE.as_usize()]
            );
            assert_eq!(
                position.player_bb[Color::BLACK.as_usize()],
                simple_position.player_bb[Color::BLACK.as_usize()]
            );
            assert_eq!(
                position.adjacent_check_bb[position.ply as usize],
                simple_position.adjacent_check_bb[position.ply as usize]
            );
            assert_eq!(
                position.long_check_bb[position.ply as usize],
                simple_position.long_check_bb[position.ply as usize]
            );

            // ランダムに局面を進める
            if moves.len() == 0 {
                break;
            }

            let random_move = moves.choose(&mut rng).unwrap();
            position.do_move(random_move);
        }
    }
}

#[cfg(test)]
fn count_nodes(position: &mut Position, limit: u8) -> u64 {
    if limit == 0 {
        return 1;
    }

    if position.is_repetition().0 {
        return 1;
    }

    let moves = position.generate_moves();
    let mut count = 0;

    for m in &moves {
        position.do_move(m);

        count += count_nodes(position, limit - 1);

        position.undo_move();
    }

    return count;
}

#[test]
fn perft() {
    let mut position: Position = Position::empty_board();
    position.set_start_position();

    assert_eq!(count_nodes(&mut position, 1), 14);
    assert_eq!(count_nodes(&mut position, 2), 181);
    assert_eq!(count_nodes(&mut position, 3), 2512);
    assert_eq!(count_nodes(&mut position, 4), 35401);
    assert_eq!(count_nodes(&mut position, 5), 533203);
    assert_eq!(count_nodes(&mut position, 6), 8276188);
    assert_eq!(count_nodes(&mut position, 7), 132680698);
}
