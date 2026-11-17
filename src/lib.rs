//! `470M1`: an Atomic win prover and an independent proof checker.
//!
//! The crate supplies the rules helpers, the compact proof format, a proof
//! walk, the independent checker, the hand-guided DF-PN search and the `line`
//! viewer. The CLI in `main.rs` exposes them as `prove`, `check`, `line` and
//! `bench`.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! (`src/lib.rs`, `src/search.rs`): tablebase, NNUE, model, web and UCI code
//! removed; `status` and `hash` kept for the walk.

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
pub mod walk;

use shakmaty::{
    CastlingMode, Color, EnPassantMode, KnownOutcome, Outcome, Position as _, PositionError,
    PositionErrorKinds, Setup, fen::Fen, variant::Atomic, zobrist::Zobrist64,
};

/// An Atomic position from its FEN, read as `from_setup` reads one.
pub fn position(fen: &str) -> Result<Atomic, String> {
    let fen: Fen = fen.parse().map_err(|e| format!("bad FEN: {e}"))?;
    from_setup(fen.into_setup()).map_err(|flaws| format!("illegal position: {flaws}"))
}

/// An Atomic position, castling as in standard chess. As Lichess does, it
/// drops castling rights and an en passant square the position cannot have,
/// and lets more material be than a game can reach; any other flaw makes it
/// no position, and the error says which in words.
pub fn from_setup(setup: Setup) -> Result<Atomic, String> {
    let turn = setup.turn;
    setup
        .position(CastlingMode::Standard)
        .or_else(PositionError::ignore_invalid_castling_rights)
        .or_else(PositionError::ignore_invalid_ep_square)
        .or_else(PositionError::ignore_too_much_material)
        .map_err(|error| flaws(error.kinds(), turn))
}

/// Why a setup with `turn` to move is no position.
fn flaws(kinds: PositionErrorKinds, turn: Color) -> String {
    if kinds.contains(PositionErrorKinds::EMPTY_BOARD) {
        return "the board is empty".to_owned();
    }
    let name = |color: Color| if color.is_white() { "White" } else { "Black" };
    let (us, them) = (name(turn), name(!turn));
    let words = [
        (
            PositionErrorKinds::MISSING_KING,
            format!("{them} has no king while {us} is to move"),
        ),
        (
            PositionErrorKinds::TOO_MANY_KINGS,
            "a side has more than one king".to_owned(),
        ),
        (
            PositionErrorKinds::PAWNS_ON_BACKRANK,
            "a pawn stands on the first or last rank".to_owned(),
        ),
        (
            PositionErrorKinds::OPPOSITE_CHECK,
            format!("{them} is in check while {us} is to move"),
        ),
        (
            PositionErrorKinds::IMPOSSIBLE_CHECK,
            format!("no game reaches the check on {us}"),
        ),
        (
            PositionErrorKinds::VARIANT,
            "it breaks a rule of Atomic chess".to_owned(),
        ),
    ];
    let found: Vec<String> = words
        .into_iter()
        .filter(|(kind, _)| kinds.contains(*kind))
        .map(|(_, words)| words)
        .collect();
    if found.is_empty() {
        "no game reaches it".to_owned()
    } else {
        found.join("; ")
    }
}

/// How the game has ended at `pos`, if it has: the winner, none for a draw,
/// and why, as Lichess names it.
pub fn ending(pos: &Atomic) -> Option<(Option<Color>, &'static str)> {
    match pos.variant_outcome() {
        Outcome::Known(KnownOutcome::Decisive { winner }) => {
            return Some((Some(winner), "variant_loss"));
        }
        Outcome::Known(KnownOutcome::Draw) => return Some((None, "variant_draw")),
        Outcome::Unknown => {}
    }
    if pos.legal_moves().is_empty() {
        return Some(if pos.is_check() {
            (Some(!pos.turn()), "checkmate")
        } else {
            (None, "stalemate")
        });
    }
    if pos.is_insufficient_material() {
        return Some((None, "insufficient_material"));
    }
    if pos.halfmoves() >= 100 {
        return Some((None, "fifty_moves"));
    }
    None
}

/// A position's Zobrist hash, including the en passant file only where taking
/// there is legal.
pub fn hash(pos: &Atomic) -> u64 {
    pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal).0
}

/// How a game stands after a move, for the attacker.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Won,
    Over,
    Open,
}

pub fn status(pos: &Atomic, attacker: Color) -> Status {
    match pos.variant_outcome() {
        Outcome::Known(KnownOutcome::Decisive { winner }) if winner == attacker => {
            return Status::Won;
        }
        Outcome::Known(_) => return Status::Over,
        Outcome::Unknown => {}
    }
    if pos.is_check() && pos.legal_moves().is_empty() {
        return if pos.turn() != attacker {
            Status::Won
        } else {
            Status::Over
        };
    }
    if pos.is_insufficient_material() || pos.halfmoves() >= 100 {
        return Status::Over;
    }
    Status::Open
}

#[cfg(test)]
mod tests {
    use super::{ending, position};
    use shakmaty::{Color, EnPassantMode, fen::Fen};

    fn read(fen: &str) -> Result<String, String> {
        position(fen).map(|pos| Fen::from_position(&pos, EnPassantMode::Legal).to_string())
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
                "illegal position: Black has no king while White is to move",
            ),
            (
                "4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1",
                "illegal position: Black is in check while White is to move",
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
