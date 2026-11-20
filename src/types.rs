//! The prover's rules vocabulary, after `M8`: `atomic-movegen`'s types plus
//! the few conveniences this crate's geometry needs.
//!
//! Nothing here is a second rules implementation. The bitboard helpers are
//! set algebra over the crate's own `Bitboard`; the ray table is a compile-time
//! geometry table used by the move scoring, and `aligned` is the same
//! "three squares on one ray" predicate the old boundary borrowed.

pub use atomic_movegen::{
    Bitboard, CastlingSide, Color, Piece, PieceType, Square, attacks, make_piece,
};

/// The piece role the prover names; the crate calls an Atomic king a commoner.
pub type Role = PieceType;

/// The roles the evaluation counts, commoner excluded.
pub const ROLES: [Role; 5] = [
    Role::Pawn,
    Role::Knight,
    Role::Bishop,
    Role::Rook,
    Role::Queen,
];

/// The squares on the first and last ranks, where a pawn promotes.
pub const BACKRANKS: Bitboard = Bitboard(0xFF00_0000_0000_00FF);

/// The squares a dark-squared bishop keeps to (`a1` is dark).
pub const DARK_SQUARES: Bitboard = Bitboard(0xAA55_AA55_AA55_AA55);
/// The squares a light-squared bishop keeps to.
pub const LIGHT_SQUARES: Bitboard = Bitboard(0x55AA_55AA_55AA_55AA);

/// The attack set of one role, dispatched once and always inlined into the
/// caller's loop.
#[inline(always)]
pub fn attacks_from(role: Role, color: Color, square: Square, occupied: Bitboard) -> Bitboard {
    match role {
        Role::Pawn => attacks::pawn_attacks(color, square),
        Role::Knight => attacks::knight_attacks(square),
        Role::Bishop => attacks::bishop_attacks(square, occupied),
        Role::Rook => attacks::rook_attacks(square, occupied),
        Role::Queen => attacks::queen_attacks(square, occupied),
        _ => attacks::king_attacks(square),
    }
}

/// Whether `color` is White.
#[inline]
pub fn is_white(color: Color) -> bool {
    color == Color::White
}

/// A `CastlingSide` from the side's queen-side flag.
#[inline]
pub fn castle_side(queen_side: bool) -> CastlingSide {
    if queen_side {
        CastlingSide::QueenSide
    } else {
        CastlingSide::KingSide
    }
}

/// The square the commoner lands on when castling to `side`.
#[inline]
pub fn castle_king_to(side: CastlingSide, color: Color) -> Square {
    let back = if is_white(color) {
        [Square::C1, Square::G1]
    } else {
        [Square::C8, Square::G8]
    };
    match side {
        CastlingSide::QueenSide => back[0],
        _ => back[1],
    }
}

/// The square the rook lands on when castling to `side`.
#[inline]
pub fn castle_rook_to(side: CastlingSide, color: Color) -> Square {
    let back = if is_white(color) {
        [Square::D1, Square::F1]
    } else {
        [Square::D8, Square::F8]
    };
    match side {
        CastlingSide::QueenSide => back[0],
        _ => back[1],
    }
}

/// The English name of a color, as the proof header writes it.
pub fn color_name(color: Color) -> &'static str {
    if is_white(color) { "white" } else { "black" }
}

/// A color from the proof header's name.
pub fn parse_color(name: &str) -> Option<Color> {
    match name {
        "white" => Some(Color::White),
        "black" => Some(Color::Black),
        _ => None,
    }
}

/// Whether the three squares lie on one rank, file or diagonal.
///
/// This is the undirected line predicate the old boundary borrowed: it says
/// nothing about which square is between which.
#[inline]
pub fn aligned(a: Square, b: Square, c: Square) -> bool {
    LINES[a as usize][b as usize] & (1u64 << (c as u8)) != 0
}

/// The squares strictly between `a` and `b`, bounds excluded.
#[inline]
pub fn between(a: Square, b: Square) -> Bitboard {
    attacks::between(a, b)
}

/// A positional ray table, built at compile time like the crate's own.
const fn compute_rays() -> [[u64; 64]; 64] {
    let mut table = [[0u64; 64]; 64];
    let mut a = 0u8;
    while a < 64 {
        let (af, ar) = ((a % 8) as i8, (a / 8) as i8);
        let mut b = 0u8;
        while b < 64 {
            let (bf, br) = ((b % 8) as i8, (b / 8) as i8);
            let (df, dr) = (bf - af, br - ar);
            if a != b && (df == 0 || dr == 0 || df.abs() == dr.abs()) {
                let sf = if df > 0 {
                    1
                } else if df < 0 {
                    -1
                } else {
                    0
                };
                let sr = if dr > 0 {
                    1
                } else if dr < 0 {
                    -1
                } else {
                    0
                };
                let (mut f, mut r) = (af + sf, ar + sr);
                let mut bb = 0u64;
                while f >= 0 && f < 8 && r >= 0 && r < 8 {
                    bb |= 1u64 << ((r as u8) * 8 + (f as u8));
                    f += sf;
                    r += sr;
                }
                table[a as usize][b as usize] = bb;
            }
            b += 1;
        }
        a += 1;
    }
    table
}

/// The full rank, file or diagonal through the two squares, both included.
const fn compute_lines() -> [[u64; 64]; 64] {
    let rays = compute_rays();
    let mut table = [[0u64; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let mut b = 0usize;
        while b < 64 {
            table[a][b] = rays[a][b] | rays[b][a];
            b += 1;
        }
        a += 1;
    }
    table
}

static LINES: [[u64; 64]; 64] = compute_lines();

/// The conveniences the prover uses on the crate's bitboards.
pub trait BitboardExt {
    /// Whether any square is set.
    fn any(self) -> bool;
    /// Whether more than one square is set.
    fn more_than_one(self) -> bool;
    /// Whether `square` is set.
    fn contains(self, square: Square) -> bool;
    /// This set with `square` removed.
    fn without(self, square: Square) -> Bitboard;
    /// This set with `square` added.
    fn with(self, square: Square) -> Bitboard;
    /// Add `square` in place.
    fn add(&mut self, square: Square);
    /// The least significant square, if any.
    fn first(self) -> Option<Square>;
    /// The set squares, least significant first.
    fn squares(self) -> Squares;
}

impl BitboardExt for Bitboard {
    #[inline]
    fn any(self) -> bool {
        self.0 != 0
    }

    #[inline]
    fn more_than_one(self) -> bool {
        self.0 & self.0.wrapping_sub(1) != 0
    }

    #[inline]
    fn contains(self, square: Square) -> bool {
        self.0 & (1u64 << (square as u8)) != 0
    }

    #[inline]
    fn without(self, square: Square) -> Bitboard {
        Bitboard(self.0 & !(1u64 << (square as u8)))
    }

    #[inline]
    fn with(self, square: Square) -> Bitboard {
        Bitboard(self.0 | (1u64 << (square as u8)))
    }

    #[inline]
    fn add(&mut self, square: Square) {
        self.0 |= 1u64 << (square as u8);
    }

    #[inline]
    fn first(self) -> Option<Square> {
        (self.0 != 0).then(|| Square::from_u8(self.0.trailing_zeros() as u8))
    }

    #[inline]
    fn squares(self) -> Squares {
        Squares(self)
    }
}

/// A bitboard's squares, least significant first.
pub struct Squares(Bitboard);

impl Iterator for Squares {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0.0 == 0 {
            return None;
        }
        let square = Square::from_u8(self.0.0.trailing_zeros() as u8);
        self.0.0 &= self.0.0 - 1;
        Some(square)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.0.count() as usize;
        (count, Some(count))
    }
}

/// A colored piece of `role`.
#[inline]
pub fn piece_of(color: Color, role: Role) -> Piece {
    make_piece(color, role)
}

/// The piece type a piece has, when it has one.
#[inline]
pub fn role_of(piece: Piece) -> Option<Role> {
    piece.type_of()
}
