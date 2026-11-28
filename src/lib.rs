//! `470MI`: an Atomic win prover and an independent proof checker.
//!
//! The crate supplies the rules helpers, the compact proof format, a proof
//! walk, the independent checker, the hand-guided DF-PN search and the `line`
//! viewer. The CLI in `main.rs` exposes them as `prove`, `check`, `line` and
//! `bench`.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! (`src/lib.rs`, `src/search.rs`): tablebase, NNUE, model, web and UCI code
//! removed; `ending` and `status` are kept for the walk and the rules
//! differential.

pub mod board;
pub mod check;
pub mod clock;
pub mod dfpn;
pub mod eval;
pub mod format;
pub mod lines;
pub mod moves;
pub mod proofs;
pub mod search;
pub mod table;
pub mod types;
pub mod walk;

use atomic_movegen::{Board, FenPolicy};

use crate::board::{FenError, Position};

/// An Atomic position from its FEN, read as Lichess reads one: a castling
/// right not backed by its pieces and an impossible en passant square are
/// dropped, more material than a game can reach is allowed, and any other
/// flaw makes it no position. The error says which in words.
pub fn position(fen: &str) -> Result<Position, FenError> {
    let board = Board::from_fen_with(fen, FenPolicy::Lichess).map_err(FenError::from)?;
    Ok(Position::from_board(board))
}

/// How the game has ended at `pos`, if it has: the winner, none for a draw,
/// and why, as Lichess names it.
pub fn ending(pos: &Position) -> Option<(Option<board::Color>, &'static str)> {
    for color in [board::Color::White, board::Color::Black] {
        if pos.king(color).is_none() {
            return Some((Some(color.flip()), "variant_loss"));
        }
    }
    if !moves::has_legal_move(pos) {
        return Some(if pos.in_check() {
            (Some(pos.turn().flip()), "checkmate")
        } else {
            (None, "stalemate")
        });
    }
    if moves::cannot_win(pos, board::Color::White) && moves::cannot_win(pos, board::Color::Black) {
        return Some((None, "insufficient_material"));
    }
    if pos.halfmoves() >= 100 {
        return Some((None, "fifty_moves"));
    }
    None
}

/// How a game stands after a move, for the attacker.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Won,
    Over,
    Open,
}

pub fn status(pos: &Position, attacker: board::Color) -> Status {
    if pos.king(attacker).is_none() {
        return Status::Over;
    }
    if pos.king(attacker.flip()).is_none() {
        return Status::Won;
    }
    // Only a position in check can be mate; a stalemate stays open and fails
    // when the search finds no move in it.
    if pos.in_check() && !moves::has_legal_move(pos) {
        return if pos.turn() != attacker {
            Status::Won
        } else {
            Status::Over
        };
    }
    if (moves::cannot_win(pos, board::Color::White) && moves::cannot_win(pos, board::Color::Black))
        || pos.halfmoves() >= 100
    {
        return Status::Over;
    }
    Status::Open
}

#[cfg(test)]
mod tests {
    use super::{ending, position};
    use crate::board::Color;

    fn read(fen: &str) -> Result<String, String> {
        position(fen)
            .map(|pos| pos.to_fen())
            .map_err(|error| error.to_string())
    }

    #[test]
    fn reads_fens_as_lichess_does() {
        assert_eq!(
            read("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w KQkq e6 0 1"),
            Ok("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1".to_owned())
        );
        for (fen, error) in [
            (
                "rnbq1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
                "illegal position: Black has no king",
            ),
            (
                "4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1",
                "illegal position: the side that just moved is in check",
            ),
        ] {
            assert_eq!(read(fen), Err(error.to_owned()), "{fen}");
        }
    }

    #[test]
    fn tells_how_a_game_ended() {
        let ended = |fen: &str| ending(&position(fen).unwrap());
        assert_eq!(
            ended("rnbq1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQ - 0 1"),
            Some((Some(Color::White), "variant_loss"))
        );
        assert_eq!(
            ended("k7/8/1Q6/8/8/8/8/6K1 b - - 0 1"),
            Some((None, "stalemate"))
        );
        assert_eq!(
            ended("4k3/8/8/8/8/8/2QQ4/4K3 w - - 100 80"),
            Some((None, "fifty_moves"))
        );
    }
}
