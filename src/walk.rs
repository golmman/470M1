//! A proof's walk: a proof here is a map from attacker boards to their
//! winning moves, one move a board as the prover keeps them, and walking it
//! measures each attacker position's subproof, its tree and its length.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! (`src/walk.rs`): tablebase verdicts removed; status and hashes come from
//! this crate's rules layer.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::Status;
use crate::board::{Move, Position};
use crate::status;
use crate::types::Color;

/// Why a walk cannot measure a proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The line met a position it had already proved.
    Repeat,
    /// An attacker position of the proof has no winning move.
    NoMove,
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Repeat => write!(out, "the proof repeats a position"),
            Error::NoMove => write!(out, "a position of the proof has no winning move"),
        }
    }
}

impl std::error::Error for Error {}

/// Attacker boards and their winning moves.
pub type Moves = HashMap<u64, Option<Move>>;

/// A subproof's tree size and length in attacker moves.
#[derive(Clone, Copy, Debug)]
pub struct Measure {
    pub tree: u64,
    pub length: u32,
}

/// A capture that wins at once, which the checkers find for themselves.
pub fn winning_capture(pos: &Position, attacker: Color) -> Option<Move> {
    pos.legal_moves()
        .into_iter()
        .find(|&m| pos.effect(m).capture && matches!(status(&pos.play(m), attacker), Status::Won))
}

/// What a walk of a proof learns: each attacker position's subproof and the
/// position itself, and the line being walked, to catch a repetition.
#[derive(Default)]
pub struct Walk {
    pub measures: HashMap<u64, Measure>,
    pub positions: HashMap<u64, Position>,
    active: HashSet<u64>,
    /// Whether to leave `positions` empty, for walks that only measure.
    measures_only: bool,
}

impl Walk {
    /// A walk that keeps the measures but not the positions.
    pub fn measures_only() -> Walk {
        Walk {
            measures_only: true,
            ..Walk::default()
        }
    }
}

/// Measures the subproof of an attacker position, memoised by position; an
/// error for a missing move or a cycle.
pub fn measure(
    pos: &Position,
    attacker: Color,
    moves: &Moves,
    walk: &mut Walk,
) -> Result<Measure, Error> {
    let here = pos.hash();
    if walk.active.contains(&here) {
        return Err(Error::Repeat);
    }
    if let Some(measure) = walk.measures.get(&here) {
        return Ok(*measure);
    }
    let measure = if winning_capture(pos, attacker).is_some() {
        Measure { tree: 0, length: 1 }
    } else {
        let m = moves.get(&here).copied().flatten().ok_or(Error::NoMove)?;
        walk.active.insert(here);
        let after = pos.play(m);
        let replies = measure_after(&after, attacker, moves, walk);
        // Off the line whether or not it failed, so that the walk can go on.
        walk.active.remove(&here);
        let replies = replies?;
        if !walk.measures_only {
            walk.positions.insert(here, pos.clone());
        }
        Measure {
            tree: replies.tree.saturating_add(1),
            length: replies.length + 1,
        }
    };
    walk.measures.insert(here, measure);
    Ok(measure)
}

/// The replies' subproofs after an attacker's move: their tree sizes summed
/// and their longest length.
pub fn measure_after(
    after: &Position,
    attacker: Color,
    moves: &Moves,
    walk: &mut Walk,
) -> Result<Measure, Error> {
    let mut total = Measure { tree: 0, length: 0 };
    if !matches!(status(after, attacker), Status::Open) {
        return Ok(total);
    }
    for reply in after.legal_moves() {
        let next = after.play(reply);
        if matches!(status(&next, attacker), Status::Open) {
            let child = measure(&next, attacker, moves, walk)?;
            total.tree = total.tree.saturating_add(child.tree);
            total.length = total.length.max(child.length);
        }
    }
    Ok(total)
}
