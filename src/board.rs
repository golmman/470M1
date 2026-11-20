//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! Size note: this is a copied/adapted upstream module, kept whole by plan1;
//! see `PROVENANCE.md`.
//! (`src/board.rs`): board, move and effect representation; tablebase, NNUE, model, web and UCI code removed.
//! An Atomic position built the way engines build theirs, after
//! Atomic-Stockfish: bitboards with a mailbox and 16-bit moves. The rules,
//! the native position keys and the move generator come from
//! `atomic-movegen`; the parser, FEN writing and every value type are the
//! crate's, so the prover does not use shakmaty.
//!
//! Castling is standard: the kings start on the e-file and the rooks in the
//! corners.

use std::mem::MaybeUninit;
use std::ops::ControlFlow;

use atomic_movegen::{
    Board as NativeBoard, Counts, EnPassantMode, Move as NativeMove, Square as NativeSquare,
    movegen,
};

use crate::types::{self, BitboardExt, role_of};
pub use crate::types::{Bitboard, CastlingSide, Color, Piece, PieceType as Role, Square};

/// A move in `atomic-movegen`'s stable 16-bit wire encoding. The native value
/// is kept whole, so the move list, legality and effect calls need no
/// translation; the proof table projects it to 15 bits at its own boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move(NativeMove);

impl Default for Move {
    fn default() -> Move {
        Move(NativeMove::NONE)
    }
}

impl Move {
    #[inline]
    pub(crate) fn from_native(m: NativeMove) -> Move {
        Move(m)
    }

    /// The native move; the sentinel for a malformed 16-bit value.
    #[inline]
    pub(crate) fn native(self) -> NativeMove {
        self.0
    }

    /// The move's native bits: 0 for none, never from a square to itself.
    pub fn bits(self) -> u16 {
        self.0.to_bits()
    }

    pub fn from_bits(bits: u16) -> Move {
        Move(NativeMove::from_bits(bits).unwrap_or(NativeMove::NONE))
    }

    pub fn from(self) -> Square {
        self.0.from_sq()
    }

    pub fn to(self) -> Square {
        self.0.to_sq()
    }

    pub fn promotion(self) -> Option<Role> {
        self.0.is_promotion().then(|| self.0.promotion_type())
    }

    /// The move in UCI notation, castling as the commoner's destination.
    pub fn to_uci(self) -> String {
        self.0.to_uci()
    }

    /// The move from UCI notation, resolving castling and en passant against
    /// `pos`.
    pub fn from_uci(pos: &Position, text: &str) -> Option<Move> {
        pos.board.move_from_uci(text).map(Move::from_native)
    }

    pub fn is_en_passant(self) -> bool {
        self.0.is_en_passant()
    }

    pub fn is_castle(self) -> bool {
        self.0.is_castling()
    }

    /// The compact 15-bit form the proof tables keep: origin, destination and
    /// a kind in `0..=6`, so a proof entry still fits its 32-bit field.
    pub(crate) fn proof_bits(self) -> u16 {
        let from = self.from() as u16;
        let to = self.to() as u16;
        let kind = match self.promotion() {
            Some(Role::Knight) => 3,
            Some(Role::Bishop) => 4,
            Some(Role::Rook) => 5,
            Some(Role::Queen) => 6,
            Some(_) => unreachable!("a promotion to pawn or commoner"),
            None if self.is_en_passant() => 1,
            None if self.is_castle() => 2,
            None => 0,
        };
        from | to << 6 | kind << 12
    }

    /// The move from its proof bits, which only `proof_bits` writes.
    pub(crate) fn from_proof_bits(bits: u16) -> Move {
        let from = Square::from_u8((bits & 63) as u8);
        let to = Square::from_u8(((bits >> 6) & 63) as u8);
        let native = match bits >> 12 {
            0 => NativeMove::make_move(from, to),
            1 => NativeMove::make_enpassant(from, to),
            2 => NativeMove::make_castling(from, to),
            3 => NativeMove::make_promotion(from, to, Role::Knight),
            4 => NativeMove::make_promotion(from, to, Role::Bishop),
            5 => NativeMove::make_promotion(from, to, Role::Rook),
            6 => NativeMove::make_promotion(from, to, Role::Queen),
            _ => unreachable!("a proof move kind"),
        };
        Move::from_native(native)
    }
}

/// What a move changes, worked out without playing it.
#[derive(Clone, Copy)]
pub struct Effect {
    /// The child's key, with the en passant file counted only where taking
    /// there is legal.
    pub hash: u64,
    /// The child's 50-move counter.
    pub counter: u32,
    /// The board's occupancy after the move.
    pub occupied: Bitboard,
    pub capture: bool,
}

/// Moves of one position, on the stack. Only the moves pushed are written,
/// so the list is not zeroed each time.
pub struct Moves {
    moves: [MaybeUninit<Move>; 256],
    len: usize,
}

impl Moves {
    fn new() -> Moves {
        Moves {
            moves: [const { MaybeUninit::uninit() }; 256],
            len: 0,
        }
    }

    fn push(&mut self, m: Move) {
        self.moves[self.len].write(m);
        self.len += 1;
    }

    pub fn as_slice(&self) -> &[Move] {
        // SAFETY: the first `len` moves were written by `push`, and a
        // `MaybeUninit<Move>` has a `Move`'s layout.
        unsafe { std::slice::from_raw_parts(self.moves.as_ptr().cast(), self.len) }
    }
}

fn piece(piece: atomic_movegen::Piece) -> Option<Piece> {
    piece.color().map(|_| piece)
}

#[derive(Clone)]
pub struct Position {
    board: NativeBoard,
    /// The board's key, counting the en passant file only where taking there
    /// is legal.
    hash: u64,
}

impl Position {
    /// A position from the parser's native board, with its legal-en-passant
    /// key.
    pub fn from_board(board: NativeBoard) -> Position {
        let hash = board.hash_with(EnPassantMode::Legal);
        Position { board, hash }
    }

    /// A position read from FEN under the Lichess policy.
    pub fn from_fen(fen: &str) -> Result<Position, String> {
        crate::position(fen)
    }

    /// The position as FEN, with the en passant file only where taking there
    /// is legal.
    pub fn to_fen(&self) -> String {
        self.board.fen_with(EnPassantMode::Legal)
    }

    pub(crate) fn board(&self) -> &NativeBoard {
        &self.board
    }

    pub fn turn(&self) -> Color {
        self.board.side_to_move()
    }

    /// The game's move number, from one.
    pub fn fullmoves(&self) -> u32 {
        u32::from(self.board.fullmove_number())
    }

    /// The square a pawn just passed over moving two squares, if any. Only
    /// meaningful for the side to move.
    pub fn maybe_ep_square(&self) -> Option<Square> {
        self.board.ep_square()
    }

    /// An en passant capture from the capturing pawn to the passed square.
    pub fn en_passant_move(&self, from: Square, to: Square) -> Move {
        Move::from_native(NativeMove::make_enpassant(from, to))
    }

    pub fn halfmoves(&self) -> u32 {
        u32::from(self.board.rule50())
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn occupied(&self) -> Bitboard {
        Bitboard(self.board.occupied().0)
    }

    pub fn by_color(&self, color: Color) -> Bitboard {
        Bitboard(self.board.pieces_color(color).0)
    }

    pub fn by_role(&self, role: Role) -> Bitboard {
        Bitboard(self.board.pieces_pt(role).0)
    }

    pub fn by_piece(&self, piece: Piece) -> Bitboard {
        let (Some(color), Some(role)) = (piece.color(), piece.type_of()) else {
            return Bitboard::EMPTY;
        };
        self.by_color_role(color, role)
    }

    /// The pieces of one color and role, without packing them into a `Piece`.
    pub fn by_color_role(&self, color: Color, role: Role) -> Bitboard {
        Bitboard(self.board.pieces_color_pt(color, role).0)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        piece(self.board.piece_on(square))
    }

    pub fn role_at(&self, square: Square) -> Option<Role> {
        role_of(self.piece_at(square)?)
    }

    pub fn king(&self, color: Color) -> Option<Square> {
        let sq = self.board.commoners(color).lsb();
        (sq != NativeSquare::NONE).then_some(sq)
    }

    /// The pieces of every color attacking a square, with sliders seeing
    /// through `occupied`.
    pub fn attackers_to(&self, square: Square, occupied: Bitboard) -> Bitboard {
        Bitboard(
            self.board
                .attackers_to(square, atomic_movegen::Bitboard(occupied.0))
                .0,
        )
    }

    /// Whether a move is pseudo-legal and legal, as `atomic-movegen` judges
    /// it. This query reads only the two counts; `Mover`'s cached context is
    /// check-aware because it is built from a populated `StateInfo`.
    pub fn legal(&self, m: Move) -> bool {
        let native = m.native();
        let context = self.board.legality_context(self.board.counts());
        self.board.is_pseudo_legal(native) && context.legal(native)
    }

    pub(crate) fn is_double_push(&self, m: Move) -> bool {
        self.board.is_double_push(m.native())
    }

    /// Whether `pseudo_legal` lists a move, without listing them all, as a
    /// replay needs for the one move it follows.
    pub fn is_pseudo_legal(&self, m: Move) -> bool {
        let native = m.native();
        native != NativeMove::NONE && self.board.is_pseudo_legal(native)
    }

    /// Every move that is legal but for where it leaves the mover's king.
    pub fn pseudo_legal(&self) -> Moves {
        let mut moves = Moves::new();
        let _ = movegen::for_each_pseudo_legal(&self.board, &mut |m| {
            moves.push(Move::from_native(m));
            ControlFlow::<()>::Continue(())
        });
        moves
    }

    /// The position with the other side to move, as if the side to move had
    /// passed: what a threat would win.
    pub fn null(&self) -> Position {
        let board = self.board.null_for_advanced(self.turn().flip());
        let hash = board.hash();
        Position { board, hash }
    }

    /// What a legal move changes, as `play` would change it.
    pub fn effect(&self, m: Move) -> Effect {
        let counts = self.board.counts();
        self.effect_with_occupancy(m, self.occupied(), counts)
    }

    /// The child effect from a parent occupancy and counts the caller already
    /// holds: one pass on the parent's bitboards, no child board or view.
    pub(crate) fn effect_with_occupancy(
        &self,
        m: Move,
        occupied: Bitboard,
        counts: Counts,
    ) -> Effect {
        let child = self
            .board
            .child_effect_with_occupancy(
                m.native(),
                atomic_movegen::Bitboard(occupied.0),
                counts,
                EnPassantMode::Legal,
            )
            .expect("a well-formed move");
        Effect {
            hash: child.key,
            counter: u32::from(child.rule50),
            occupied: Bitboard(child.occupied.0),
            capture: child.capture,
        }
    }

    /// The position after a legal move, with its key.
    pub fn play(&self, m: Move) -> Position {
        let mut board = self.board.clone();
        let mut state = atomic_movegen::StateInfo::new();
        board.do_move(m.native(), &mut state);
        let hash = board.hash_with(EnPassantMode::Legal);
        Position { board, hash }
    }

    /// Whether the side to move is in check.
    pub fn in_check(&self) -> bool {
        !self.board.checkers().is_empty()
    }

    /// A capture from one square to another, a pawn reaching the last rank
    /// promoting to a queen.
    pub fn capture_move(&self, from: Square, to: Square) -> Move {
        if self.role_at(from) == Some(Role::Pawn) && types::BACKRANKS.contains(to) {
            Move::from_native(NativeMove::make_promotion(from, to, Role::Queen))
        } else {
            Move::from_native(NativeMove::make_move(from, to))
        }
    }

    /// Every legal move, in the order the generator lists them.
    pub fn legal_moves(&self) -> Vec<Move> {
        self.pseudo_legal()
            .as_slice()
            .iter()
            .copied()
            .filter(|&m| self.legal(m))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use atomic_movegen::EnPassantMode;

    use super::{Move, Position};

    /// The moved list, every legal move once, and the native key and effect
    /// agree with the position the move leads to.
    #[test]
    fn moves_and_effects_agree() {
        let start = "rn2k2r/pp3ppp/2p5/3pP3/8/2N5/PPP2PPP/R3K2R w KQkq d6 0 1";
        let pos = Position::from_fen(start).unwrap();
        assert_eq!(pos.hash(), pos.board.hash_with(EnPassantMode::Legal));
        let legal = pos.legal_moves();
        assert!(!legal.is_empty());
        for m in legal {
            let next = pos.play(m);
            let effect = pos.effect(m);
            assert_eq!(effect.hash, next.hash(), "{m:?}");
            assert_eq!(effect.counter, next.halfmoves(), "{m:?}");
            assert_eq!(effect.occupied, next.occupied(), "{m:?}");
            assert!(pos.is_pseudo_legal(m));
            let bits = m.bits();
            assert_eq!(Move::from_bits(bits), m);
        }
    }

    /// The passed position advances the clock, as the search's limits need.
    #[test]
    fn null_advances_the_clock() {
        let pos = Position::from_fen("rn2k2r/pp3ppp/2p5/3pP3/8/2N5/PPP2PPP/R3K2R w KQkq d6 7 12")
            .unwrap();
        let passed = pos.null();
        assert_eq!(passed.turn(), pos.turn().flip());
        assert_eq!(passed.halfmoves(), pos.halfmoves() + 1);
        assert_eq!(passed.maybe_ep_square(), None);
        assert_eq!(passed.hash(), passed.board.hash_with(EnPassantMode::Legal));
    }
}
