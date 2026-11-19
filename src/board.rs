//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! Size note: this is a copied/adapted upstream module, kept whole by plan1;
//! see `PROVENANCE.md`.
//! (`src/board.rs`): board, move and effect representation; tablebase, NNUE, model, web and UCI code removed.
//! An Atomic position built the way engines build theirs, after
//! Atomic-Stockfish: bitboards with a mailbox and 16-bit moves. The rules,
//! the native position keys and the move generator come from
//! `atomic-movegen`; shakmaty remains the typed boundary for parsing and for
//! the independent checker.
//!
//! Castling is standard: the kings start on the e-file and the rooks in the
//! corners.

use std::mem::MaybeUninit;
use std::ops::ControlFlow;

use atomic_movegen::{
    Board as NativeBoard, Color as NativeColor, Counts, EnPassantMode, Move as NativeMove,
    PieceType, Setup as NativeSetup, Square as NativeSquare, movegen,
};
use shakmaty::{Bitboard, CastlingSide, Color, Piece, Position as _, Role, Setup, Square, attacks};

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
        square(self.0.from_sq())
    }

    pub fn to(self) -> Square {
        square(self.0.to_sq())
    }

    pub fn promotion(self) -> Option<Role> {
        self.0.is_promotion().then(|| role(self.0.promotion_type()))
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
            Some(Role::Pawn | Role::King) => unreachable!("a promotion to pawn or king"),
            None if self.is_en_passant() => 1,
            None if self.is_castle() => 2,
            None => 0,
        };
        from | to << 6 | kind << 12
    }

    /// The move from its proof bits, which only `proof_bits` writes.
    pub(crate) fn from_proof_bits(bits: u16) -> Move {
        let from = native_square(Square::new(u32::from(bits & 63)));
        let to = native_square(Square::new(u32::from((bits >> 6) & 63)));
        let native = match bits >> 12 {
            0 => NativeMove::make_move(from, to),
            1 => NativeMove::make_enpassant(from, to),
            2 => NativeMove::make_castling(from, to),
            3 => NativeMove::make_promotion(from, to, PieceType::Knight),
            4 => NativeMove::make_promotion(from, to, PieceType::Bishop),
            5 => NativeMove::make_promotion(from, to, PieceType::Rook),
            6 => NativeMove::make_promotion(from, to, PieceType::Queen),
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

/// Castling rights as bits: the white king side, the white queen side, then
/// the black ones; each belongs to a corner.
const RIGHTS: [(Color, CastlingSide, Square); 4] = [
    (Color::White, CastlingSide::KingSide, Square::H1),
    (Color::White, CastlingSide::QueenSide, Square::A1),
    (Color::Black, CastlingSide::KingSide, Square::H8),
    (Color::Black, CastlingSide::QueenSide, Square::A8),
];

fn native_color(color: Color) -> NativeColor {
    if color.is_white() {
        NativeColor::White
    } else {
        NativeColor::Black
    }
}

fn native_square(square: Square) -> NativeSquare {
    NativeSquare::from_u8(square as u8)
}

fn square(square: NativeSquare) -> Square {
    Square::new(square as u32)
}

fn native_piece(piece: Piece) -> atomic_movegen::Piece {
    atomic_movegen::make_piece(native_color(piece.color), native_role(piece.role))
}

fn native_role(role: Role) -> PieceType {
    match role {
        Role::Pawn => PieceType::Pawn,
        Role::Knight => PieceType::Knight,
        Role::Bishop => PieceType::Bishop,
        Role::Rook => PieceType::Rook,
        Role::Queen => PieceType::Queen,
        Role::King => PieceType::Commoner,
    }
}

fn role(piece_type: PieceType) -> Role {
    match piece_type {
        PieceType::Pawn => Role::Pawn,
        PieceType::Knight => Role::Knight,
        PieceType::Bishop => Role::Bishop,
        PieceType::Rook => Role::Rook,
        PieceType::Queen => Role::Queen,
        PieceType::Commoner => Role::King,
        _ => unreachable!("the movegen piece types"),
    }
}

fn piece(piece: atomic_movegen::Piece) -> Option<Piece> {
    let color = piece.color()?;
    let role = role(piece.type_of()?);
    Some(Piece {
        color: if color == NativeColor::White {
            Color::White
        } else {
            Color::Black
        },
        role,
    })
}

#[derive(Clone)]
pub struct Position {
    board: NativeBoard,
    /// The board's key, counting the en passant file only where taking there
    /// is legal.
    hash: u64,
}

impl Position {
    pub fn from_atomic(pos: &shakmaty::variant::Atomic) -> Position {
        let mut setup = NativeSetup::new();
        for (square, piece) in pos.board().iter() {
            setup.place(native_square(square), native_piece(piece));
        }
        let mut castling = 0;
        for (bit, (color, castling_side, _)) in RIGHTS.iter().enumerate() {
            if pos.castles().has(*color, *castling_side) {
                castling |= 1 << bit;
            }
        }
        setup
            .set_side_to_move(native_color(pos.turn()))
            .set_castling_rights(castling)
            .set_ep_square(pos.maybe_ep_square().map(native_square))
            .set_rule50(pos.halfmoves().min(u32::from(u16::MAX)) as u16);
        let board = NativeBoard::from_setup(&setup).expect("a position the rules reached");
        let hash = board.hash_with(EnPassantMode::Legal);
        Position { board, hash }
    }

    pub fn to_atomic(&self) -> shakmaty::variant::Atomic {
        let mut setup = Setup::empty();
        for (sq, native) in self.board.pieces() {
            if let Some(piece) = piece(native) {
                setup.board.set_piece_at(square(sq), piece);
            }
        }
        setup.turn = self.turn();
        setup.castling_rights = RIGHTS
            .iter()
            .enumerate()
            .filter(|(bit, _)| self.board.castling_rights() & (1 << bit) != 0)
            .map(|(_, (_, _, rook))| Bitboard::from(*rook))
            .fold(Bitboard::EMPTY, |rights, rook| rights | rook);
        setup.ep_square = self.maybe_ep_square();
        setup.halfmoves = self.halfmoves();
        // Read as the root was, which may hold more material than a game can.
        crate::from_setup(setup).expect("a position the rules reached")
    }

    pub(crate) fn board(&self) -> &NativeBoard {
        &self.board
    }

    pub fn turn(&self) -> Color {
        if self.board.side_to_move() == NativeColor::White {
            Color::White
        } else {
            Color::Black
        }
    }

    /// The square a pawn just passed over moving two squares, if any. Only
    /// meaningful for the side to move.
    pub fn maybe_ep_square(&self) -> Option<Square> {
        self.board.ep_square().map(square)
    }

    /// An en passant capture from the capturing pawn to the passed square.
    pub fn en_passant_move(&self, from: Square, to: Square) -> Move {
        Move::from_native(NativeMove::make_enpassant(
            native_square(from),
            native_square(to),
        ))
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
        Bitboard(self.board.pieces_color(native_color(color)).0)
    }

    pub fn by_role(&self, role: Role) -> Bitboard {
        Bitboard(self.board.pieces_pt(native_role(role)).0)
    }

    pub fn by_piece(&self, piece: Piece) -> Bitboard {
        Bitboard(
            self.board
                .pieces_color_pt(native_color(piece.color), native_role(piece.role))
                .0,
        )
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        piece(self.board.piece_on(native_square(square)))
    }

    pub fn role_at(&self, square: Square) -> Option<Role> {
        self.piece_at(square).map(|piece| piece.role)
    }

    pub fn king(&self, color: Color) -> Option<Square> {
        let sq = self.board.commoners(native_color(color)).lsb();
        (sq != NativeSquare::NONE).then(|| square(sq))
    }

    /// The pieces of every color attacking a square, with sliders seeing
    /// through `occupied`.
    pub fn attackers_to(&self, square: Square, occupied: Bitboard) -> Bitboard {
        Bitboard(
            self.board
                .attackers_to(native_square(square), atomic_movegen::Bitboard(occupied.0))
                .0,
        )
    }

    /// Whether `attacker` checks the king on `king`, given what is left on the
    /// board after a move: `occupied`, with every piece outside it gone.
    /// Touching kings are never in check.
    pub fn checked_after(&self, king: Square, attacker: Color, occupied: Bitboard) -> bool {
        let theirs = self.by_color(attacker) & occupied;
        let their_kings = theirs & self.by_role(Role::King);
        their_kings.any()
            && (attacks::king_attacks(king) & their_kings).is_empty()
            && (self.attackers_to(king, occupied) & theirs & !self.by_role(Role::King)).any()
    }

    /// Whether a move is pseudo-legal and legal, as `atomic-movegen` judges
    /// it. The candidate loop reads only the two counts, never checkers.
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
        let board = self.board.null_for_advanced(native_color(!self.turn()));
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

    /// Whether either side may still castle.
    pub fn has_castling_rights(&self) -> bool {
        self.board.castling_rights() != 0
    }

    /// Whether the side to move is in check.
    pub fn in_check(&self) -> bool {
        !self.board.checkers().is_empty()
    }

    /// A capture from one square to another, a pawn reaching the last rank
    /// promoting to a queen.
    pub fn capture_move(&self, from: Square, to: Square) -> Move {
        if self.role_at(from) == Some(Role::Pawn) && Bitboard::BACKRANKS.contains(to) {
            Move::from_native(NativeMove::make_promotion(
                native_square(from),
                native_square(to),
                PieceType::Queen,
            ))
        } else {
            Move::from_native(NativeMove::make_move(
                native_square(from),
                native_square(to),
            ))
        }
    }

    /// The move as shakmaty writes it.
    pub fn to_shakmaty(&self, m: Move) -> shakmaty::Move {
        let (from, to) = (m.from(), m.to());
        if m.is_castle() {
            return shakmaty::Move::Castle {
                king: from,
                rook: to,
            };
        }
        if m.is_en_passant() {
            return shakmaty::Move::EnPassant { from, to };
        }
        shakmaty::Move::Normal {
            role: self.role_at(from).expect("a piece to move"),
            from,
            capture: self.role_at(to),
            to,
            promotion: m.promotion(),
        }
    }
}

/// A position read from FEN.
#[cfg(test)]
pub fn from_fen(fen: &str) -> Result<Position, String> {
    Ok(Position::from_atomic(&crate::position(fen)?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use atomic_movegen::EnPassantMode;
    use shakmaty::Position as _;
    use shakmaty::uci::UciMove;

    use super::{Move, Position, from_fen};

    /// Random games, comparing every position's moves, and every position they
    /// lead to, with shakmaty's.
    #[test]
    fn matches_shakmaty() {
        let starts = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rn2k2r/pp3ppp/2p5/3pP3/8/2N5/PPP2PPP/R3K2R w KQkq d6 0 1",
            "r3k2r/1P4pp/8/2pP4/8/8/p5PP/R3K2R w KQkq c6 0 1",
            "rnbqkbnr/ppppp1p1/5p2/7p/8/4PN2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
            "r3k2r/8/8/3qQ3/8/8/8/R3K2R b KQkq - 0 1",
            "8/2P5/8/3k4/8/3K4/5p2/8 w - - 0 1",
            "r1bqBb2/pp1pk3/8/4P1pp/8/2P5/PP3P2/R1BK3R b - - 1 11",
        ];
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut positions = 0;
        for start in starts {
            for _ in 0..150 {
                let mut ours: Position = from_fen(start).unwrap();
                let mut theirs: shakmaty::variant::Atomic = ours.to_atomic();
                for _ in 0..150 {
                    assert_eq!(
                        ours.hash(),
                        ours.board.hash_with(EnPassantMode::Legal),
                        "{start}"
                    );
                    let expected: Vec<shakmaty::Move> = theirs.legal_moves().into_iter().collect();
                    let found: Vec<_> = ours
                        .pseudo_legal()
                        .as_slice()
                        .iter()
                        .copied()
                        .filter(|m| ours.legal(*m))
                        .collect();
                    let converted: BTreeSet<String> = found
                        .iter()
                        .map(|m| UciMove::from_standard(ours.to_shakmaty(*m)).to_string())
                        .collect();
                    let expected: BTreeSet<String> = expected
                        .into_iter()
                        .map(|m| UciMove::from_standard(m).to_string())
                        .collect();
                    assert_eq!(converted, expected);
                    positions += 1;
                    // Every native 16-bit encoding tested against the list,
                    // on one position in 400 and the position passed to the
                    // other side.
                    if positions % 400 == 0 {
                        let passed = (!ours.in_check()).then(|| ours.null());
                        for pos in std::iter::once(&ours).chain(passed.as_ref()) {
                            let mut listed = vec![false; 1 << 16];
                            for m in pos.pseudo_legal().as_slice() {
                                listed[usize::from(m.bits())] = true;
                            }
                            for bits in 0..=u16::MAX {
                                let m = Move::from_bits(bits);
                                let expected = listed[usize::from(bits)];
                                assert_eq!(pos.is_pseudo_legal(m), expected, "{m:?}");
                            }
                        }
                    }
                    if found.is_empty() || theirs.is_variant_end() {
                        break;
                    }
                    if !ours.in_check() {
                        let passed = ours.null();
                        let swapped = theirs.clone().swap_turn().expect("a side to pass");
                        assert_eq!(passed.turn(), swapped.turn());
                    }
                    for m in &found {
                        let next = ours.play(*m);
                        let effect = ours.effect(*m);
                        assert_eq!(effect.hash, next.hash(), "{m:?}");
                        assert_eq!(effect.counter, next.halfmoves(), "{m:?}");
                        assert_eq!(effect.occupied, next.occupied(), "{m:?}");
                        let mut played = theirs.clone();
                        played.play_unchecked(ours.to_shakmaty(*m));
                        assert_eq!(next.to_atomic().board(), played.board(), "{m:?}");
                        assert_eq!(next.halfmoves(), played.halfmoves());
                    }
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    let m = found[(seed % found.len() as u64) as usize];
                    theirs.play_unchecked(ours.to_shakmaty(m));
                    ours = ours.play(m);
                }
            }
        }
        assert!(positions > 20_000, "{positions}");
    }

    /// The passed position advances the clock, as the search's limits need.
    #[test]
    fn null_advances_the_clock() {
        let pos = from_fen("rn2k2r/pp3ppp/2p5/3pP3/8/2N5/PPP2PPP/R3K2R w KQkq d6 7 12").unwrap();
        let passed = pos.null();
        assert_eq!(passed.turn(), !pos.turn());
        assert_eq!(passed.halfmoves(), pos.halfmoves() + 1);
        assert_eq!(passed.maybe_ep_square(), None);
        assert_eq!(passed.hash(), passed.board.hash_with(EnPassantMode::Legal));
    }
}
