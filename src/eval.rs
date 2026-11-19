//! Adapted from provari `v0.1.0` (`src/eval.rs`): verbatim apart from this
//! provenance line. See `PROVENANCE.md`.
//! Size note: this is a copied/adapted upstream module, kept whole by plan2.
//! A hand-crafted Atomic evaluation after Multi-Variant Stockfish's Atomic
//! terms, to guide the search as the network does at a fraction of its cost:
//! material at Atomic values, then, unless the kings touch, explosion
//! targets, pieces beside the enemy king, attacks around it, and checks.

use shakmaty::{Bitboard, ByRole, Color, Piece, Role, attacks};

use crate::board::Position;
use crate::moves::Mover;

/// Piece values by role, between Multi-Variant Stockfish's Atomic midgame and
/// endgame values: pawns survive blasts, so the pieces are worth fewer pawns.
const VALUES: [i32; 7] = [0, 305, 545, 634, 930, 1670, 0];
/// One of their pieces beside their king that we attack: a threat to explode
/// the king; two are nearly decisive.
const TARGET: i32 = 400;
const TARGETS: i32 = 1000;
/// One of their pieces beside their king, which a capture would explode.
const FUSE: i32 = 60;
/// Each of our attacks on a square beside their king.
const PRESSURE: i32 = 20;
/// Each of our pieces that can check next move.
const CHECKER: i32 = 60;

/// The evaluation for the side to move. The mover's per-role check masks are
/// already built, so only the defender's are computed, once per role.
pub fn evaluate(pos: &Position, mover: &Mover) -> i32 {
    let us = pos.turn();
    let ours = terms(pos, us, Some(mover.check_squares()));
    let theirs = terms(pos, !us, None);
    ours.score() - theirs.score()
}

/// What one side has: its material and its attack on the other king.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Terms {
    pub material: i32,
    /// Their pieces beside their king that we attack, up to two.
    pub targets: u32,
    /// Their pieces beside their king.
    pub fuses: u32,
    /// Our attacks on the squares beside their king.
    pub pressure: u32,
    /// Our pieces that can check next move.
    pub checkers: u32,
}

impl Terms {
    fn score(&self) -> i32 {
        let targets = match self.targets {
            0 => 0,
            1 => TARGET,
            _ => TARGETS,
        };
        self.material
            + targets
            + FUSE * self.fuses as i32
            + PRESSURE * self.pressure as i32
            + CHECKER * self.checkers as i32
    }
}

/// One side's material at Atomic values, without its attack terms.
pub fn material(pos: &Position, color: Color) -> i32 {
    (pos.by_color(color) & !pos.by_role(Role::King))
        .into_iter()
        .map(|square| pos.role_at(square).map_or(0, |role| VALUES[role as usize]))
        .sum()
}

fn terms(pos: &Position, color: Color, masks: Option<&ByRole<Bitboard>>) -> Terms {
    let ours = pos.by_color(color);
    let theirs = pos.by_color(!color);
    let occupied = pos.occupied();
    let mut terms = Terms::default();
    // Material by role, so the role is known for the attack terms below.
    for role in ROLES {
        terms.material +=
            pos.by_piece(Piece { color, role }).count() as i32 * VALUES[role as usize];
    }
    let (Some(their_king), Some(our_king)) = (pos.king(!color), pos.king(color)) else {
        return terms;
    };
    let ring = attacks::king_attacks(their_king);
    // Touching kings can be neither checked nor exploded.
    if ring.contains(our_king) {
        return terms;
    }
    let computed;
    let masks = match masks {
        Some(masks) => masks,
        None => {
            computed = check_masks(pos, color, their_king, occupied);
            &computed
        }
    };
    let targets = ring & theirs & !attacks::king_attacks(our_king);
    let mut attacked = Bitboard::EMPTY;
    let mut pressure = 0u32;
    let mut checkers = 0u32;
    for role in ROLES {
        let pieces = pos.by_piece(Piece { color, role });
        if pieces.is_empty() {
            continue;
        }
        let checks = *masks.get(role);
        for square in pieces {
            let reach = attacks::attacks(square, role.of(color), occupied);
            attacked |= reach;
            pressure += (reach & ring).count() as u32;
            if (reach & checks & !ours).any() {
                checkers += 1;
            }
        }
    }
    terms.targets = (attacked & targets).count().min(2) as u32;
    terms.fuses = (ring & theirs).count() as u32;
    terms.pressure = pressure;
    terms.checkers = checkers;
    terms
}

/// The roles whose pieces the evaluation counts, king excluded.
const ROLES: [Role; 5] = [
    Role::Pawn,
    Role::Knight,
    Role::Bishop,
    Role::Rook,
    Role::Queen,
];

/// The squares from which one role of `color` would check `their_king`, one
/// mask per role present. The mover's table is already in its `Mover`; this
/// builds the defender's.
fn check_masks(
    pos: &Position,
    color: Color,
    their_king: shakmaty::Square,
    occupied: Bitboard,
) -> ByRole<Bitboard> {
    let mut masks = ByRole::new_with(|_| Bitboard::EMPTY);
    for role in ROLES {
        if (pos.by_piece(Piece { color, role })).any() {
            *masks.get_mut(role) = attacks::attacks(their_king, role.of(!color), occupied);
        }
    }
    masks
}

/// The pre-formulation, one board walk per side, kept as the test witness.
#[cfg(test)]
fn terms_reference(pos: &Position, color: Color) -> Terms {
    let ours = pos.by_color(color);
    let theirs = pos.by_color(!color);
    let occupied = pos.occupied();
    let kings = pos.by_role(Role::King);
    let mut terms = Terms::default();
    for square in ours & !kings {
        terms.material += pos.role_at(square).map_or(0, |role| VALUES[role as usize]);
    }
    let (Some(their_king), Some(our_king)) = (pos.king(!color), pos.king(color)) else {
        return terms;
    };
    let ring = attacks::king_attacks(their_king);
    if ring.contains(our_king) {
        return terms;
    }
    let targets = ring & theirs & !attacks::king_attacks(our_king);
    let mut attacked = Bitboard::EMPTY;
    let mut pressure = 0u32;
    let mut checkers = 0u32;
    for square in ours & !kings {
        let Some(role) = pos.role_at(square) else {
            continue;
        };
        let reach = attacks::attacks(square, role.of(color), occupied);
        attacked |= reach;
        pressure += (reach & ring).count() as u32;
        let checks = attacks::attacks(their_king, role.of(!color), occupied);
        if (reach & checks & !ours).any() {
            checkers += 1;
        }
    }
    terms.targets = (attacked & targets).count().min(2) as u32;
    terms.fuses = (ring & theirs).count() as u32;
    terms.pressure = pressure;
    terms.checkers = checkers;
    terms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Position;

    /// The pre-formulation's `evaluate`, for equality.
    fn evaluate_reference(pos: &Position) -> i32 {
        let us = pos.turn();
        terms_reference(pos, us).score() - terms_reference(pos, !us).score()
    }

    fn check(pos: &Position) {
        let mover = Mover::new(pos);
        for color in [Color::White, Color::Black] {
            assert_eq!(
                terms(
                    pos,
                    color,
                    (color == pos.turn()).then(|| mover.check_squares())
                ),
                terms_reference(pos, color),
                "hash {:#x}: {color:?} terms",
                pos.hash()
            );
        }
        assert_eq!(
            evaluate(pos, &mover),
            evaluate_reference(pos),
            "hash {:#x}: evaluation",
            pos.hash()
        );
    }

    fn corpus_fens() -> Vec<String> {
        let mut fens = Vec::new();
        for row in include_str!("../tests/fixtures/veto.tsv").lines() {
            let mut fields = row.split('\t');
            let _name = fields.next();
            if let Some(fen) = fields.next()
                && !row.starts_with('#')
                && !row.starts_with("name\t")
            {
                fens.push(fen.to_owned());
            }
        }
        for row in include_str!("../tests/fixtures/never_prove.tsv").lines() {
            let mut fields = row.split('\t');
            let _name = fields.next();
            if let Some(fen) = fields.next()
                && !row.starts_with('#')
                && !row.starts_with("name\t")
            {
                fens.push(fen.to_owned());
            }
        }
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../bench/corpus.json")).expect("corpus.json");
        fens.push(corpus["core"]["fen"].as_str().expect("core FEN").to_owned());
        for cell in corpus["sample20"].as_array().expect("sample20") {
            fens.push(cell["fen"].as_str().expect("sample FEN").to_owned());
        }
        fens
    }

    #[test]
    fn terms_match_the_reference_over_the_corpus() {
        let mut positions = 0;
        for fen in corpus_fens() {
            let Ok(atomic) = crate::position(&fen) else {
                continue;
            };
            let pos = Position::from_atomic(&atomic);
            check(&pos);
            positions += 1;
        }
        assert!(positions > 20, "only {positions} corpus positions checked");
    }

    #[test]
    fn terms_match_the_reference_over_seeded_play_outs() {
        struct Lcg(u64);
        impl Lcg {
            fn next(&mut self) -> u64 {
                self.0 = self
                    .0
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                self.0
            }
        }
        let starts: Vec<_> = corpus_fens()
            .iter()
            .filter_map(|fen| crate::position(fen).ok())
            .map(|atomic| Position::from_atomic(&atomic))
            .collect();
        let mut plies = 0;
        for seed in [0x9e37_79b9_7f4a_7c15, 0x2545_f491_4f6c_dd1d] {
            let mut rng = Lcg(seed);
            for start in &starts {
                let mut pos = start.clone();
                for _ in 0..32 {
                    let moves = pos.pseudo_legal();
                    if moves.as_slice().is_empty() {
                        break;
                    }
                    let moves = moves.as_slice();
                    let mut picked = None;
                    for _ in 0..moves.len() {
                        let index = (rng.next() as usize) % moves.len();
                        let m = moves[index];
                        if pos.legal(m) {
                            picked = Some(m);
                            break;
                        }
                    }
                    let Some(m) = picked else { break };
                    pos = pos.play(m);
                    check(&pos);
                    plies += 1;
                }
            }
        }
        assert!(plies > 100, "only {plies} play-out plies checked");
    }
}
