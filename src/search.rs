//! Adapted from provari `v0.1.0` (`src/search.rs`): tablebase code removed.
//! See `PROVENANCE.md`.
//! Size note: this is a copied/adapted upstream module, kept whole by plan2.
//! Exhaustive search for a win in N attacker moves, deepened one move at a
//! time: the win-in-N reference baseline, kept for comparison rather than used
//! by the CLI.
//!
//! A position is won when the attacker has a move after which every defence
//! loses in fewer moves. Repeating a position on the current line counts as a
//! draw, and so does reaching the 50-move limit, so every proof also holds
//! under those rules.

use crate::clock::Instant;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

use crate::Status;
use crate::board::{Move, Position};
use crate::status;
use crate::types::{Color, Role};
/// A position and its 50-move counter, which can turn a win into a draw.
pub type Key = (u64, u32);

/// Keys are Zobrist hashes already, so hashing them again is wasted work.
#[derive(Default)]
pub struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = self.0.rotate_left(8) ^ u64::from(*byte);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 ^= value;
    }

    fn write_u32(&mut self, value: u32) {
        self.0 ^= u64::from(value).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}

pub type Table<V> = HashMap<Key, V, BuildHasherDefault<KeyHasher>>;

pub fn key(pos: &Position) -> Key {
    (pos.hash(), pos.halfmoves())
}

#[derive(Clone, Copy)]
enum Bound {
    /// Won within this many attacker moves.
    Won(u32),
    /// Not won within this many.
    NotWithin(u32),
}

pub struct Search {
    attacker: Color,
    known: Table<Bound>,
    /// The winning move of every won attacker position, with its bound.
    strategy: Table<(Move, u32)>,
    path: Vec<u64>,
    capacity: usize,
    deadline: Instant,
    pub nodes: u64,
    pub stopped: bool,
}

impl Search {
    pub fn new(root: &Position, deadline: Instant, capacity: usize) -> Search {
        Search {
            attacker: root.turn(),
            known: Table::default(),
            strategy: Table::default(),
            path: Vec::new(),
            capacity,
            deadline,
            nodes: 0,
            stopped: false,
        }
    }

    /// The fewest attacker moves that win, if a win within `most` is found in
    /// time.
    pub fn solve(&mut self, root: &Position, most: u32) -> Option<u32> {
        match status(root, self.attacker) {
            Status::Won => return Some(0),
            Status::Over => return None,
            Status::Open => {}
        }
        for moves in 1..=most {
            if self.attacker_to_move(root, moves) {
                return Some(moves);
            }
            if self.stopped {
                break;
            }
        }
        None
    }

    fn attacker_to_move(&mut self, pos: &Position, moves: u32) -> bool {
        if self.tick() {
            return false;
        }
        let key = key(pos);
        match self.known.get(&key) {
            Some(Bound::Won(within)) if *within <= moves => return true,
            Some(Bound::NotWithin(within)) if *within >= moves => return false,
            _ => {}
        }
        let legal = pos.legal_moves();
        // A move that ends the game is tried before any deeper line.
        for &m in &legal {
            let child = pos.play(m);
            if matches!(status(&child, self.attacker), Status::Won) {
                self.record(key, m, 1);
                return true;
            }
        }
        if moves > 1 {
            self.path.push(key.0);
            for m in attacker_order(pos, legal) {
                let child = pos.play(m);
                if !matches!(status(&child, self.attacker), Status::Open) || self.repeats(&child) {
                    continue;
                }
                if self.defender_to_move(&child, moves - 1) {
                    self.path.pop();
                    self.record(key, m, moves);
                    return true;
                }
                if self.stopped {
                    break;
                }
            }
            self.path.pop();
        }
        if !self.stopped {
            self.remember(key, Bound::NotWithin(moves));
        }
        false
    }

    /// Every defence loses within `moves` attacker moves.
    fn defender_to_move(&mut self, pos: &Position, moves: u32) -> bool {
        if self.tick() {
            return false;
        }
        let legal = pos.legal_moves();
        if legal.is_empty() {
            return false;
        }
        self.path.push(key(pos).0);
        let mut holds = true;
        for m in defender_order(pos, legal) {
            let child = pos.play(m);
            let lost = match status(&child, self.attacker) {
                Status::Won => true,
                Status::Over => false,
                Status::Open => !self.repeats(&child) && self.attacker_to_move(&child, moves),
            };
            if !lost {
                holds = false;
                break;
            }
        }
        self.path.pop();
        holds
    }

    fn repeats(&self, pos: &Position) -> bool {
        self.path.contains(&key(pos).0)
    }

    /// Keeps the shortest win, so a proof's bounds shrink along every line.
    fn record(&mut self, key: Key, m: Move, within: u32) {
        let shorter = self
            .strategy
            .get(&key)
            .is_none_or(|(_, known)| within < *known);
        if shorter {
            self.strategy.insert(key, (m, within));
        }
        self.remember(key, Bound::Won(within.min(self.strategy[&key].1)));
    }

    fn remember(&mut self, key: Key, bound: Bound) {
        if self.known.len() >= self.capacity {
            self.known.clear();
        }
        self.known.insert(key, bound);
    }

    fn tick(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes.is_multiple_of(4096) && Instant::now() >= self.deadline {
            self.stopped = true;
        }
        self.stopped
    }

    /// The winning move found for a won attacker position.
    pub fn winning_move(&self, key: &Key) -> Option<Move> {
        self.strategy.get(key).map(|(m, _)| *m)
    }
}

/// Reads a proof back from the root, handing `emit` every attacker position
/// the win reaches with the move a search recorded for it, in the order a
/// checker meets them; positions where a capture wins at once (an explosion of
/// the king, as a rule) are skipped, as the checkers find those for themselves.
/// Returns how many positions it handed over.
pub fn read_proof(
    root: &Position,
    mut winning_move: impl FnMut(&Position, &Key) -> Option<Move>,
    mut emit: impl FnMut(&Position, Option<Move>) -> Result<(), String>,
) -> Result<usize, String> {
    let attacker = root.turn();
    match status(root, attacker) {
        Status::Won => return Ok(0),
        Status::Over => return Err("the root has ended without the attacker winning".into()),
        Status::Open => {}
    }
    let mut count = 0;
    let mut seen = HashSet::new();
    let mut pending = vec![root.clone()];
    while let Some(pos) = pending.pop() {
        let key = key(&pos);
        if !seen.insert(key) {
            continue;
        }
        if crate::walk::winning_capture(&pos, attacker).is_some() {
            continue;
        }
        let m = winning_move(&pos, &key)
            .ok_or_else(|| format!("no winning move recorded for {}", pos.to_fen()))?;
        emit(&pos, Some(m))?;
        count += 1;
        let after = pos.play(m);
        if !matches!(status(&after, attacker), Status::Open) {
            continue;
        }
        for reply in after.legal_moves() {
            let next = after.play(reply);
            if matches!(status(&next, attacker), Status::Open) {
                pending.push(next);
            }
        }
    }
    Ok(count)
}

/// Captures first: in Atomic every capture explodes.
fn attacker_order(pos: &Position, legal: Vec<Move>) -> Vec<Move> {
    let mut moves = legal;
    moves.sort_by_key(|&m| if pos.effect(m).capture { 0 } else { 1 });
    moves
}

/// Captures first, then king moves: the likeliest escapes.
fn defender_order(pos: &Position, legal: Vec<Move>) -> Vec<Move> {
    let mut moves = legal;
    moves.sort_by_key(|&m| {
        if pos.effect(m).capture {
            0
        } else if pos.role_at(m.from()) == Some(Role::Commoner) {
            1
        } else {
            2
        }
    });
    moves
}

#[cfg(test)]
mod tests {
    use super::{Search, read_proof};
    use crate::board::Position;
    use std::time::{Duration, Instant};

    #[test]
    fn a_root_draw_precedes_a_capture_that_would_win() {
        for clock in [99, 100] {
            let root = Position::from_fen(&format!(
                "rnbqkb1r/pp1p3p/2pBppp1/8/3PP3/2N5/PPP2P1P/R2QKB1R w KQ - {clock} 2"
            ))
            .unwrap();
            let mut search = Search::new(&root, Instant::now() + Duration::from_secs(1), 100);
            assert_eq!(search.solve(&root, 1), (clock == 99).then_some(1));
            let winning_move = |pos: &Position, _: &_| crate::board::Move::from_uci(pos, "d6f8");
            let proof = read_proof(&root, winning_move, |_, _| Ok(()));
            assert_eq!(proof.is_ok(), clock == 99);
        }
    }
}
