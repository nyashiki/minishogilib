use pyo3::prelude::*;

use position::*;
use r#move::*;
use types::*;

#[pymethods]
impl Position {
    pub fn solve_checkmate_dfs(&mut self, depth: i32) -> (bool, Move) {
        for i in (1..depth + 1).step_by(2) {
            let (checkmate, m) = attack(self, i as i32);

            if checkmate {
                return (true, m);
            }
        }

        return (false, NULL_MOVE);
    }
}

/// 詰みがある場合は詰み手順を返す
fn attack(position: &mut Position, depth: i32) -> (bool, Move) {
    if depth <= 0 {
        return (false, NULL_MOVE);
    }

    if position.ply == MAX_PLY as u16 {
        return (false, NULL_MOVE);
    }

    let moves = position.generate_moves_with_option(true, true, false, true);

    for m in &moves {
        position.do_move(m);

        if position.get_check_bb() == 0 {
            position.undo_move();
            continue;
        }

        let (repetition, check_repetition) = position.is_repetition();

        if repetition {
            if !check_repetition && position.side_to_move == Color::WHITE {
                position.undo_move();
                return (true, *m);
            }

            position.undo_move();
            continue;
        }

        let (checkmate, _) = defense(position, depth - 1);

        position.undo_move();

        if checkmate {
            return (true, *m);
        }
    }

    return (false, NULL_MOVE);
}

fn defense(position: &mut Position, depth: i32) -> (bool, Move) {
    if position.ply == MAX_PLY as u16 {
        return (false, NULL_MOVE);
    }

    let moves = position.generate_moves();

    if moves.len() == 0
        && position.kif[position.ply as usize - 1].piece.get_piece_type() == PieceType::PAWN
        && position.kif[position.ply as usize - 1].amount == 0
    {
        // 打ち歩詰め
        return (false, NULL_MOVE);
    }

    for m in &moves {
        position.do_move(m);

        let (repetition, check_repetition) = position.is_repetition();
        if repetition {
            position.undo_move();

            if check_repetition {
                continue;
            }

            if position.side_to_move == Color::BLACK {
                return (false, NULL_MOVE);
            }

            continue;
        }

        let (checkmate, _) = attack(position, depth - 1);

        position.undo_move();

        if !checkmate {
            return (false, NULL_MOVE);
        }
    }

    return (true, NULL_MOVE); // ToDo: take the longest path
}

#[test]
fn checkmate_test() {
    let mut position = Position::empty_board();

    {
        position.set_sfen("2k2/5/2P2/5/2K2 b G 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("5/5/2k2/5/2K2 b 2GS 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("5/5/2k2/5/2K2 b 2G 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, false);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("2k2/5/2B2/5/2K2 b GSBRgsr2p 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("2G1k/5/4G/5/2K2 b P 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, false);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("4k/5/4B/5/2K1R b - 1");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("4k/4p/5/5/K4 b BG 1");
        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("5/4k/3pp/5/K4 b RG 1");
        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, true);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }

    {
        position.set_sfen("rbsgk/4p/5/P4/KGSBR b - 1 moves 2e3d 2a2b 4e4d 4a3b 3d4e 5a4a 1e2e 2b2a 5d5c 3a2b 3e3d 3b1d 2e2d 1d3b 4d3c 2b3c 3d3c G*2b 3c2b 2a2b S*4d S*2a G*3e 3b4c 2d2b 2a2b 4d4c 4a4c 4e5d S*3b B*4d R*3a G*4e 4c4d 3e4d 3b2c R*2e B*3d 2e3e 3d4e+ 4d4e 3a3e+ 4e3e R*3a R*4e 2b3c B*1e G*3b 1e3c 3b3c S*4b 3c3b 4b3a 3b3a 3e4d S*3d R*5a 3d4e+ 5d4e R*3e S*3c B*4a 5c5b 3e2e+ 5e5d 2c3b 3c3b 4a3b S*4c 3b4a 4d3d S*4b 4c4b 3a4b 5a4a 4b4a B*4d 1a2a 3d3c R*2d S*2b 2d2b 3c2b 2e2b 4d2b S*4c 5d5e S*3d R*2d 3d4e 2b1a+ 2a3b 1a2a 3b4b 2a4c 4b4c S*4d 4c5b R*5c 5b4b 5e4e 4b3a S*2b 3a4b 2b3c 4b3a 3c2b 3a4b 2b3c 4b3a 3c2b 3a4b 5c4c 4b5b");

        let start = std::time::Instant::now();
        let (checkmate, checkmate_move) = position.solve_checkmate_dfs(7);
        let elapsed = start.elapsed();

        assert_eq!(checkmate, false);
        println!(
            "{} ... {}.{} sec.",
            checkmate_move.sfen(),
            elapsed.as_secs(),
            elapsed.subsec_nanos() / 1000000
        );
    }
}
