//! Adapted from provari `v0.1.0` (`src/dfpn.rs`): tablebase, network,
//! provability-model, progress and sampling code removed; the hand guide and
//! the single-threaded search remain. See `PROVENANCE.md`.
//! Size note: this is a copied/adapted upstream module, kept whole by plan2.
//! Depth-first proof-number search (df-pn, Nagai 2002) for Atomic wins.
//!
//! Every position has a proof number, how many unsettled leaves must still be
//! won to win it, and a disproof number, how many must fail to refute it. The
//! search always descends into the child that looks cheapest to settle, within
//! thresholds that keep it depth-first, and uses the 1+ε trick against
//! thrashing. Explosions in one move, either way, are settled without a search,
//! and so are checks and double threats after which every defence lets the
//! king be exploded.
//! Forcing attacker moves start cheaper than quiet ones, so they are tried
//! first while the quiet ones stay available: a check starts at its number of
//! evasions that survive, and a threat to explode the king cheaper the more
//! ways it threatens, unless its piece is cheap to blow up, and a plain move
//! whose piece is cheap to blow up starts dearer than the others. Defences
//! unlikely to refute start harder to refute, and ties go to the moves that
//! proved or refuted most so far (the history heuristic). When a defence is
//! proven, its proof is replayed on the other defences (Kawano's simulation),
//! which proves those it also answers. So is a threat: when a defender
//! position not in check is first expanded, a win the attacker would have at
//! once if the defender passed is replayed on every defence, and only those
//! that parry it stay open.
//!
//! A position keeps its children's numbers while it is searched and updates
//! only the child it searched. Children are scored from their moves' effects;
//! a child position is built only to descend into it. The attacker's plain
//! moves, quiet ones that neither check nor threaten, wait together as one
//! child, scored only when the search first wants one of them (as
//! KomoringHeights delays expansions): most positions never want one. When
//! they are scored, the network evaluates the position after each, and those
//! it likes less than its best few start dearer.
//!
//! Each expanded position is evaluated once, by a hand-crafted evaluation,
//! Atomic-Stockfish's network, or a provability model, and its prospects for
//! the attacker, as a step from behind to ahead, scale its new children's
//! starting numbers (EF-PN, Winands and Schadd): lines that look winnable
//! start cheaper to prove and harder to refute.
//!
//! Positions are keyed without the 50-move counter, so that move orders share
//! their work. A proof records its length in attacker moves and the highest
//! counter it holds at, and only a proof that is shorter or holds further
//! joins it, so lengths shrink along every line of a proof and none can repeat
//! a position. Proofs that come out the same whenever worked out again, such
//! as an explosion at once, are not kept. Repeating a position of the current
//! line, the 50-move limit, and stalemate count as draws. A refutation that
//! relies on a repetition applies only while the repeated position is on the
//! line, and one that relies on the 50-move limit only from the same counter
//! upward.
//!
//! A search may be limited to wins in at most so many attacker moves. Proofs
//! know their lengths, so one counts wherever it fits the moves left; the
//! table keeps what it learns apart for each number of moves left, since a
//! position refuted for lack of moves may be won with more. A tablebase win
//! says nothing of its length, so a limited search plays it out.

use std::cmp::Reverse;
use std::collections::TryReserveError;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::clock::Instant;

use shakmaty::{Color, variant::Atomic};

use crate::board::{Effect, Move, Position};
use crate::moves::{self, Mover};
use crate::proofs::{Proof, Proofs};
use crate::search::{Key, Status, status};
use crate::table::{self, Known, Table};

const INFINITY: u32 = 1 << 30;
/// The starting proof numbers of attacker moves: a check or a threat to
/// explode the king two ways starts at 1, one threat at `THREAT`, and a quiet
/// move at `QUIET`.
const THREAT: u32 = 4;
const QUIET: u32 = 9;
/// A plain move whose piece the defender can blow up without losing
/// material, after Stockfish's ordering, which ranks a quiet move lower when
/// a cheaper piece attacks where it lands.
const HANGING: u32 = 15;
/// A threat from a piece the defender can blow up without losing material.
const WEAK_THREAT: u32 = 8;
/// A check starts at its number of evasions that do not lose at once, up to
/// this many.
const EVASIONS: u32 = 8;
/// A defence unlikely to refute starts at this disproof number, so that
/// likelier ones are tried first.
const UNLIKELY: u32 = 2;
/// How many positions a replay of a sibling's proof may visit.
const SIMULATION: u32 = 16;
/// The highest 50-move counter at which the game goes on.
const LAST: u32 = 99;
/// The depth of a refutation's repetition when it relies on none.
const ANY_LINE: u32 = u32::MAX;
/// Where the table keeps a position's plain moves together.
const GROUP: u64 = 0x2545_f491_4f6c_dd1d;
/// EF-PN's parameters: how much a parent's evaluation weighs on its new
/// children's starting numbers, and the evaluation that decides a position.
const EF_A: u32 = 10;
const EF_B: u32 = 5;
const HAND_DECISIVE: i32 = 500;

/// What guides the search's starting numbers.
#[derive(Clone, Copy)]
pub enum Guide {
    None,
    /// The hand-crafted evaluation, cheap.
    Hand,
}

#[derive(Clone, Copy)]
struct Numbers {
    proof: u32,
    disproof: u32,
}

const WON: Numbers = Numbers {
    proof: 0,
    disproof: INFINITY,
};
const LOST: Numbers = Numbers {
    proof: INFINITY,
    disproof: 0,
};

/// What the search knows about a position on the current line.
#[derive(Clone, Copy)]
struct Value {
    numbers: Numbers,
    /// For a refutation: the depth on the line of the earliest repeated
    /// position it relies on, or `ANY_LINE`.
    cycle: u32,
    /// For a refutation: whether it relies on the 50-move limit.
    fifty: bool,
}

impl Value {
    const WON: Value = Value::open(WON);
    const DRAWN: Value = Value::open(LOST);

    const fn open(numbers: Numbers) -> Value {
        Value {
            numbers,
            cycle: ANY_LINE,
            fifty: false,
        }
    }
}

struct Child {
    m: Move,
    hash: u64,
    counter: u32,
    value: Value,
    /// The move exploded the defender's king.
    exploded: bool,
    /// A defence after which the attacker explodes the king at once.
    winning: bool,
    /// How often the move proved a position, attacking, or refuted one,
    /// defending: the history heuristic, which breaks ties.
    score: u32,
    /// A defence a sibling's proof has been replayed on.
    simulated: bool,
    /// Starting numbers, which the parent's evaluation scales, rather than
    /// known ones.
    fresh: bool,
    /// Not a move: the attacker's plain moves together, scored only when the
    /// search first wants one of them.
    group: bool,
}

/// What `Dfpn::shorten` found: the shortest win's length, the passes it ran,
/// and whether that length is known to be the shortest, by the search's
/// rules.
pub struct Shortening {
    pub length: u32,
    pub passes: u32,
    pub shortest: bool,
}

pub struct Dfpn<'a> {
    attacker: Color,
    guide: Guide,
    table: Table,
    /// The proven positions, in a fixed amount of memory. A proof that comes
    /// out the same whenever it is worked out again (`recomputed`) is not
    /// kept: the attacker exploding the king at once, the defender mated, or
    /// a defence after which every move lets the king be exploded. The search
    /// spots those whenever it scores the move leading there, and proofs
    /// leave out the explosions for the checkers to find. Once the proofs
    /// first fill, a mate in one is not kept either (`drop_mates`).
    proofs: Proofs,
    path: Vec<u64>,
    /// How often each attacker move, by piece and destination, proved a
    /// position, and how often each defence refuted one.
    attack_history: Vec<u32>,
    defense_history: Vec<u32>,

    /// Each depth's children, kept between expansions to spare allocations.
    buffers: Vec<Vec<Child>>,
    /// The legal moves being scored, the expanded position's or its passed
    /// position's, with their effects and whether the filter may hold their
    /// children.
    scratch: Vec<(Move, Effect, bool)>,
    /// The line of a replay (`simulate`), kept between replays.
    line: Vec<u64>,
    /// One bit, by hash, for each position on the line that the moves being
    /// scored could repeat (`mark_repetitions`): a clear bit spares
    /// `repeated` its walk back along the line.
    repetitions: [u64; 16],
    deadline: Instant,
    /// Set from outside to stop the search, as its deadline does.
    stop: Option<&'a AtomicBool>,
    /// The node budget, checked at every node so that it repeats exactly.
    limit: u64,
    /// The child-evaluation budget, checked at every node.
    eval_limit: u64,
    /// The proofs filled their memory, which stopped the search.
    full: bool,
    /// Whether the proofs keep mates in one: until they first fill, when they
    /// drop them so that the search goes on (`drop_mates`).
    mates_kept: bool,
    /// The proofs taken at which they drop their mates in one, if before they
    /// fill: for tests of the search after.
    drop_mates_at: usize,
    /// The root's proof and disproof numbers as the search last combined them.
    root: (u32, u32),
    /// The tie-break seed, spread over all bits; 0 for none.
    seed: u64,
    /// The most attacker moves a win may take, if limited.
    most: Option<u32>,
    /// Whether the attacker is to move at the root, which tells the line's
    /// attacker moves from its plies.
    root_attacking: bool,
    pub nodes: u64,
    /// Child evaluations, the deterministic mechanism counter.
    pub evals: u64,
    /// Expanded positions.
    pub expansions: u64,
    pub stopped: bool,
    /// The nodes searched when the proofs dropped their mates in one.
    pub mates_dropped: Option<u64>,
    /// The root's proof and disproof numbers when the search last stopped.
    pub root_numbers: Option<(u32, u32)>,
}

impl<'a> Dfpn<'a> {
    /// A search of `root` with a table and proofs of these megabytes.
    pub fn new(root: &Atomic, guide: Guide, deadline: Instant, memory: (usize, usize)) -> Dfpn<'a> {
        Dfpn::try_new(root, guide, deadline, memory)
            .unwrap_or_else(|error| panic!("the search's memory: {error}"))
    }

    /// As `new`, unless the memory cannot be had.
    pub fn try_new(
        root: &Atomic,
        guide: Guide,
        deadline: Instant,
        (table_mb, proofs_mb): (usize, usize),
    ) -> Result<Dfpn<'a>, TryReserveError> {
        Ok(Dfpn {
            attacker: shakmaty::Position::turn(root),
            guide,
            table: Table::new(table_mb)?,
            proofs: Proofs::new(proofs_mb)?,
            path: Vec::new(),
            attack_history: vec![0; 7 * 64],
            defense_history: vec![0; 7 * 64],

            buffers: Vec::new(),
            scratch: Vec::new(),
            line: Vec::new(),
            repetitions: [0; 16],
            deadline,
            stop: None,
            limit: u64::MAX,
            eval_limit: u64::MAX,
            full: false,
            mates_kept: true,
            drop_mates_at: usize::MAX,
            root: (1, 1),
            seed: 0,
            most: None,
            root_attacking: true,
            nodes: 0,
            evals: 0,
            expansions: 0,
            stopped: false,
            mates_dropped: None,
            root_numbers: None,
        })
    }

    /// Searches for a win of `attacker`, who need not be the side to move.
    pub fn set_attacker(&mut self, attacker: Color) {
        self.attacker = attacker;
    }

    /// Perturbs the search's tie-breaks: among children with equal deciding
    /// numbers and equal history, the seed picks one by a hash of its
    /// position. 0 leaves every tie as it was.
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed.wrapping_mul(0xD1B5_4A32_D192_ED03);
    }

    /// Stops the search after `nodes` nodes as well as at the deadline: unlike
    /// a time limit, a node budget gives the same search on any machine.
    pub fn set_limit(&mut self, nodes: u64) {
        self.limit = nodes;
    }

    /// Stops the search after `evals` child evaluations.
    pub fn set_eval_limit(&mut self, evals: u64) {
        self.eval_limit = evals;
    }

    /// Stops the search at `deadline` instead of the one it was made with.
    pub fn set_deadline(&mut self, deadline: Instant) {
        self.deadline = deadline;
    }

    /// Stops the search once `stop` is set, as at the deadline: for a search
    /// that another thread ends.
    pub fn set_stop(&mut self, stop: &'a AtomicBool) {
        self.stop = Some(stop);
    }

    /// Accepts only wins in at most `moves` attacker moves.
    pub fn set_most(&mut self, moves: u32) {
        self.most = Some(moves);
    }

    /// How many attacker moves the proven win takes, if one is found in time.
    pub fn solve(&mut self, root: &Atomic) -> Option<u32> {
        // The search settles a position that has ended before reaching it;
        // a root that has ended, such as a sampled defender position, is
        // settled here, for the attacker as set.
        match status(root, self.attacker) {
            Status::Won => {
                self.add_proof(Position::from_atomic(root).hash(), Proof::FINISHED);
                self.root_numbers = Some((0, INFINITY));
                return Some(0);
            }
            Status::Over => {
                self.root_numbers = Some((INFINITY, 0));
                return None;
            }
            Status::Open => {}
        }
        self.root_attacking = shakmaty::Position::turn(root) == self.attacker;
        let root = Position::from_atomic(root);
        let numbers = self.mid(&root, INFINITY, INFINITY).numbers;
        self.root_numbers = Some((numbers.proof, numbers.disproof));
        // Once the proofs are full, a win may rest on a proof never kept.
        if numbers.proof != 0 || self.full {
            return None;
        }
        self.proof_at(root.hash(), root.halfmoves())
            .or_else(|| self.recomputed(&root))
            .map(|proof| proof.length)
    }

    /// After a win of `length`, searches again with a limit one below the
    /// last win's length, so that the tables and proofs carry over: a
    /// position with so many moves left is the same problem whatever the
    /// root's limit, and stored proofs count wherever they fit. Calls
    /// `improved` after each shorter win, with the search and its length.
    /// Stops when the budget ends, the proofs fill, or a pass settles that no
    /// shorter win exists.
    pub fn shorten(
        &mut self,
        root: &Atomic,
        mut length: u32,
        mut improved: impl FnMut(&Self, u32),
    ) -> Shortening {
        let mut passes = 0;
        while length > 1 {
            self.set_most(length - 1);
            passes += 1;
            match self.solve(root) {
                Some(shorter) => {
                    length = shorter;
                    improved(self, shorter);
                }
                None => {
                    let shortest = !self.stopped
                        && !self.full
                        && self.root_numbers.is_some_and(|(_, disproof)| disproof == 0);
                    return Shortening {
                        length,
                        passes,
                        shortest,
                    };
                }
            }
        }
        // No win is shorter than one move.
        Shortening {
            length,
            passes,
            shortest: true,
        }
    }

    /// The winning move for an attacker position, for reading the proof back:
    /// once the proofs have dropped their mates in one, a mate in one found
    /// again.
    pub fn winning_move(&self, pos: &Atomic, key: &Key) -> Option<shakmaty::Move> {
        let pos = Position::from_atomic(pos);
        let m = match self.proof_at(key.0, key.1) {
            Some(proof) => proof.m?,
            None => self.dropped_mate(&pos)?,
        };
        Some(pos.to_shakmaty(m))
    }

    /// Has the proofs drop their mates in one once they hold this share of
    /// their room, rather than when they fill: for tests of the search after.
    pub fn set_drop_mates_at(&mut self, share: f64) {
        self.drop_mates_at = (self.proofs.room() as f64 * share) as usize;
        if self.drop_mates_at == 0 {
            self.drop_mates();
        }
    }

    /// Whether the search stopped because its proofs filled their memory.
    pub fn proofs_full(&self) -> bool {
        self.full
    }

    fn mid(&mut self, pos: &Position, proof_limit: u32, disproof_limit: u32) -> Value {
        let start = self.nodes;
        let hash = pos.hash();
        let counter = pos.halfmoves();
        let remaining = self.remaining(self.path.len());
        // A transposition may have settled the position, or taken it past the
        // limits, since its parent looked.
        if self.proof_within(hash, counter, remaining).is_some() {
            return Value::WON;
        }
        // Out of attacker moves, a game that goes on is lost to the limit.
        if remaining == 0 {
            return Value::DRAWN;
        }
        let key = self.key(hash, remaining);
        let known = self.lookup(key, counter, |m| self.mates_with(pos, m));
        // A mate in one the table notes comes back won, its disproof number
        // past every limit.
        if let Some(value) = known {
            let numbers = value.numbers;
            if numbers.disproof == 0
                || numbers.proof >= proof_limit
                || numbers.disproof >= disproof_limit
            {
                return value;
            }
        }
        self.tick();
        self.expansions += 1;
        let attacking = pos.turn() == self.attacker;
        let level = self.path.len();
        if self.buffers.len() <= level {
            self.buffers.resize_with(level + 1, Vec::new);
        }
        let mut children = std::mem::take(&mut self.buffers[level]);
        let step = self.children(pos, attacking, remaining, &mut children);
        if !attacking && known.is_none() && !children.is_empty() && !pos.in_check() {
            self.replay_threat(pos, remaining, &mut children);
        }
        if children.is_empty() {
            self.buffers[level] = children;
            // A mate is settled before the search reaches it, except at the
            // root; anything else is a stalemate, a draw.
            let value = if pos.in_check() {
                self.mated(pos).unwrap_or(Value::DRAWN)
            } else {
                Value::DRAWN
            };
            return self.settle(key, counter, value, 1);
        }
        let depth = self.path.len() as u32;
        self.path.push(hash);
        let mut opened = None;
        let (value, best) = loop {
            let (value, best, second) = combine(&children, attacking, self.seed);
            let numbers = value.numbers;
            if depth == 0 {
                self.root = (numbers.proof, numbers.disproof);
            }
            if numbers.proof == 0
                || numbers.disproof == 0
                || numbers.proof >= proof_limit
                || numbers.disproof >= disproof_limit
                || self.stopped
            {
                break (value, best);
            }
            if children[best].group {
                children.swap_remove(best);
                opened = Some(children.len());
                self.path.pop();
                self.open_group(pos, remaining, &mut children, step);
                self.path.push(hash);
                continue;
            }
            let current = children[best].value.numbers;
            let (child_proof, child_disproof) = if attacking {
                (
                    proof_limit.min(widen(second)),
                    share(disproof_limit, numbers.disproof, current.disproof),
                )
            } else {
                (
                    share(proof_limit, numbers.proof, current.proof),
                    disproof_limit.min(widen(second)),
                )
            };
            let next = pos.play(children[best].m);
            debug_assert_eq!(next.hash(), children[best].hash);
            let value = self.mid(&next, child_proof, child_disproof);
            children[best].value = value;
            // A defence just proven may prove its siblings too: replay its
            // proof on each defence still open.
            if !attacking && value.numbers.proof == 0 {
                for (index, sibling) in children.iter_mut().enumerate() {
                    let open =
                        sibling.value.numbers.proof != 0 && sibling.value.numbers.disproof != 0;
                    if index == best || sibling.simulated || !open {
                        continue;
                    }
                    sibling.simulated = true;
                    let position = pos.play(sibling.m);
                    let mut left = SIMULATION;
                    let mut line = std::mem::take(&mut self.line);
                    let reached = self.simulate(&position, &next, remaining, &mut left, &mut line);
                    self.line = line;
                    if reached.is_some_and(|(length, _)| length <= remaining) {
                        debug_assert!(
                            self.full
                                || self.proof_at(sibling.hash, sibling.counter).is_some()
                                || self.recomputed(&position).is_some()
                        );
                        sibling.value = Value::WON;
                    }
                }
            }
        };
        self.path.pop();
        if !attacking && value.numbers.disproof == 0 {
            let index = history_index(pos, children[best].m);
            self.defense_history[index] = self.defense_history[index].saturating_add(1);
        }
        // Repeating this position, or one after it, refutes it on any line.
        let value = if value.cycle >= depth {
            Value {
                cycle: ANY_LINE,
                ..value
            }
        } else {
            value
        };
        if let Some(first) = opened
            && value.numbers.proof != 0
            && !self.stopped
        {
            let plain = &children[first..];
            let proof = plain.iter().map(|child| child.value.numbers.proof).min();
            let disproof = plain
                .iter()
                .map(|child| child.value.numbers.disproof)
                .fold(0, u32::saturating_add);
            if let Some(proof) = proof
                && proof != 0
                && proof < INFINITY
                && disproof != 0
            {
                let numbers = Numbers {
                    proof,
                    disproof: disproof.min(INFINITY - 1),
                };
                self.store(
                    key ^ GROUP,
                    counter,
                    Value::open(numbers),
                    self.nodes - start,
                );
            }
        }
        if value.numbers.proof == 0 {
            self.record_proof(pos, hash, &children, attacking);
            debug_assert!(
                self.full
                    || self.proof_at(hash, counter).is_some()
                    || self.recomputed(pos).is_some(),
                "a win without a proof"
            );
        } else if !self.stopped {
            self.store(key, counter, value, self.nodes - start);
        }
        self.buffers[level] = children;
        value
    }

    /// Scores a position's moves into `children`, with `remaining` attacker
    /// moves left, and returns the step its evaluation scaled their starting
    /// numbers by.
    fn children(
        &mut self,
        pos: &Position,
        attacking: bool,
        remaining: u32,
        children: &mut Vec<Child>,
    ) -> Option<f32> {
        let candidates = pos.pseudo_legal();
        let mover = Mover::new(pos);
        // The attacker's plain moves wait together.
        let group = attacking;
        let mut plain = 0;
        let left = remaining.saturating_sub(u32::from(attacking));
        if attacking && left == 0 {
            self.last_children(pos, &mover, children);
            return None;
        }
        // First every legal move's effect and its child's bit in the filter,
        // starting the table reads of the children it may hold: the reads do
        // not wait on one another, so their latencies overlap.
        let mut legal = std::mem::take(&mut self.scratch);
        legal.clear();
        let filtering = self.table.filtering();
        for &m in candidates.as_slice() {
            if group && mover.plain(m) {
                plain += u32::from(mover.legal(m));
                continue;
            }
            if mover.legal(m) {
                let effect = mover.full_effect(m);
                self.prefetch_child(effect.hash, left, filtering);
                legal.push((m, effect, !filtering));
            }
        }
        // Then the filter's bits, their loads under way, and the table reads
        // of the children it may hold.
        if filtering {
            self.read_filter(&mut legal, left);
        }
        children.clear();
        self.mark_repetitions(pos.halfmoves());
        for &(m, effect, seen) in &legal {
            children.push(self.evaluate(pos, &mover, m, &effect, seen, attacking, left, EVASIONS));
        }
        self.scratch = legal;
        if plain > 0 {
            let key = self.key(pos.hash(), remaining) ^ GROUP;
            let counter = pos.halfmoves();
            let known = self
                .lookup(key, counter, |_| false)
                .filter(|value| value.numbers.disproof != 0);
            children.push(Child {
                m: Move::default(),
                hash: key,
                counter,
                value: known.unwrap_or(Value::open(Numbers {
                    proof: QUIET,
                    disproof: plain,
                })),
                exploded: false,
                winning: false,
                score: 0,
                simulated: false,
                fresh: known.is_none(),
                group: true,
            });
        }
        let step = self.step(pos, &mover);
        scale(children, attacking, step);
        step
    }

    /// `children` for an attacker position with one move left, where only a
    /// move that wins at once counts: a plain move neither captures nor
    /// checks, so it cannot and is left out, and every other child is decided
    /// where it is scored, so nothing needs the position's evaluation. Apart
    /// from `children`, so that searches without a limit run the same code.
    #[inline(never)]
    fn last_children(&mut self, pos: &Position, mover: &Mover, children: &mut Vec<Child>) {
        let mut legal = std::mem::take(&mut self.scratch);
        legal.clear();
        let filtering = self.table.filtering();
        for &m in pos.pseudo_legal().as_slice() {
            if mover.plain(m) || !mover.legal(m) {
                continue;
            }
            let effect = mover.full_effect(m);
            self.prefetch_child(effect.hash, 0, filtering);
            legal.push((m, effect, !filtering));
        }
        if filtering {
            self.read_filter(&mut legal, 0);
        }
        children.clear();
        self.mark_repetitions(pos.halfmoves());
        for &(m, effect, seen) in &legal {
            children.push(self.evaluate(pos, mover, m, &effect, seen, true, 0, EVASIONS));
        }
        self.scratch = legal;
    }

    /// The expanded position's evaluation for the attacker, squashed to a
    /// step from 0, the defender clearly ahead, to 2, the attacker clearly
    /// ahead, when an evaluation guides the search.
    fn step(&mut self, pos: &Position, mover: &Mover) -> Option<f32> {
        match self.guide {
            Guide::None => None,
            Guide::Hand => {
                let value = crate::eval::evaluate(pos, mover);
                Some(self.band(pos, value, HAND_DECISIVE))
            }
        }
    }

    /// An evaluation for the side to move, squashed for the attacker to 0
    /// when the defender is clearly ahead, 1 when neither is, or 2.
    fn band(&self, pos: &Position, value: i32, decisive: i32) -> f32 {
        let value = if pos.turn() == self.attacker {
            value
        } else {
            -value
        };
        if value >= decisive {
            2.0
        } else if value <= -decisive {
            0.0
        } else {
            1.0
        }
    }

    /// Scores the plain moves a group held, as `children` would have.
    fn open_group(
        &mut self,
        pos: &Position,
        remaining: u32,
        children: &mut Vec<Child>,
        step: Option<f32>,
    ) {
        let first = children.len();
        let mover = Mover::new(pos);
        let left = remaining.saturating_sub(1);
        // The moves first, their filter words' loads started, as in `children`.
        let mut plain = std::mem::take(&mut self.scratch);
        plain.clear();
        let filtering = self.table.filtering();
        for &m in pos.pseudo_legal().as_slice() {
            if mover.plain(m) && mover.legal(m) {
                let effect = mover.full_effect(m);
                self.prefetch_child(effect.hash, left, filtering);
                plain.push((m, effect, !filtering));
            }
        }
        if filtering {
            self.read_filter(&mut plain, left);
        }
        self.mark_repetitions(pos.halfmoves());
        for &(m, effect, seen) in &plain {
            children.push(self.evaluate(pos, &mover, m, &effect, seen, true, left, EVASIONS));
        }
        self.scratch = plain;
        scale(&mut children[first..], true, step);
    }

    /// What is known about a child before searching it, from its move's
    /// effect, with `remaining` attacker moves left there; the child position
    /// is built only for a check, a tablebase probe, or en passant.
    #[allow(clippy::too_many_arguments)]
    fn evaluate(
        &mut self,
        pos: &Position,
        mover: &Mover,
        m: Move,
        effect: &Effect,
        seen: bool,
        attacking: bool,
        remaining: u32,
        evasions: u32,
    ) -> Child {
        self.evals += 1;
        let counter = effect.counter;
        let hash = effect.hash;
        let key = self.key(hash, remaining);
        let history = if attacking {
            &self.attack_history
        } else {
            &self.defense_history
        };
        let score = history[history_index(pos, m)];
        let child = |value: Value, exploded: bool| Child {
            m,
            hash,
            counter,
            value,
            exploded,
            winning: false,
            score,
            simulated: false,
            fresh: false,
            group: false,
        };
        // Only a capture explodes a king, never the mover's.
        if mover.explodes_them(effect) {
            return if attacking {
                child(Value::WON, true)
            } else {
                child(Value::DRAWN, false)
            };
        }
        // A move after which the defender explodes the attacker's king at
        // once loses on any line.
        if attacking && mover.exposes(m, effect) {
            return child(Value::DRAWN, false);
        }
        if let Some(depth) = self.repeated(hash, counter) {
            let repeated = Value {
                cycle: depth,
                ..Value::DRAWN
            };
            return child(repeated, false);
        }
        // A defence after which the attacker explodes the defender's king at
        // once is won without a search, as a mate in one is, unless it
        // reaches the 50-move limit.
        if !attacking && counter <= LAST && mover.exposes(m, effect) {
            return Child {
                winning: true,
                ..child(Value::WON, false)
            };
        }
        // Most children are new: one bit of the filter says so for both tables.
        if seen && self.proof_within(hash, counter, remaining).is_some() {
            return child(Value::WON, false);
        }
        // Out of attacker moves, only a mate still wins.
        if remaining == 0 {
            let mated = mover.gives_check(m, effect)
                && self
                    .mated(&pos.play(m))
                    .is_some_and(|value| value.numbers.proof == 0);
            return child(if mated { Value::WON } else { Value::DRAWN }, false);
        }
        let check = mover.gives_check(m, effect);
        if counter > LAST {
            // Past the 50-move limit only a mate still counts.
            let value = match check.then(|| self.mated(&pos.play(m))).flatten() {
                Some(value) => self.settle(key, counter, value, 0),
                None => Value {
                    fifty: true,
                    ..Value::DRAWN
                },
            };
            return child(value, false);
        }
        // A mate in one the table notes is checked on the child's position,
        // made only then.
        let known = if seen {
            self.lookup(key, counter, |mate| self.mates_with(&pos.play(m), mate))
        } else {
            None
        };
        if let Some(value) = known
            && value.numbers.disproof == 0
        {
            return child(value, false);
        }
        if let Some(value) = known {
            return child(value, false);
        }
        // Met for the first time: settle mates and positions the attacker
        // cannot win, and otherwise start from how forcing the move was.
        let mut check_proof = 1;
        if check {
            let next = pos.play(m);
            if attacking {
                // A check is as hard to prove as it has evasions that
                // survive, and with none it wins in two moves; with no
                // evasion at all it mates.
                let (any, safe) = moves::replies(&next, evasions);
                if !any || safe == 0 && self.exposed(&next).is_some() {
                    return child(Value::WON, false);
                }
                check_proof = safe.max(1);
            } else if let Some(value) = self.mated(&next) {
                return child(self.settle(key, counter, value, 0), false);
            }
        }
        // Only a capture or a promotion changes the material.
        if (effect.capture || m.promotion().is_some())
            && mover.might_end_material(self.attacker, effect)
            && moves::cannot_win(&pos.play(m), self.attacker)
        {
            return child(self.settle(key, counter, Value::DRAWN, 0), false);
        }
        if !attacking {
            // A defence likely to refute is tried first: a check, a blast
            // that removes an attacking piece, or kings that can touch.
            let attackers = pos.by_color(self.attacker) & !pos.by_role(shakmaty::Role::Pawn);
            let good = check
                || (effect.capture && (attackers & !effect.occupied).any())
                || mover.connectable(m, effect);
            let disproof = if good { 1 } else { UNLIKELY };
            return Child {
                fresh: true,
                ..child(Value::open(Numbers { proof: 1, disproof }), false)
            };
        }
        let threats = if check { 0 } else { mover.threats(m, effect) };
        // So does a double threat no defence parries.
        if threats >= 2 && self.exposed(&pos.play(m)).is_some() {
            return child(Value::WON, false);
        }
        let proof = if check {
            check_proof
        } else {
            match threats {
                0 if mover.plain(m) && mover.blasted_cheaply(m, effect) => HANGING,
                0 => QUIET,
                1 if mover.blasted_cheaply(m, effect) => WEAK_THREAT,
                1 => THREAT,
                _ => 1,
            }
        };
        Child {
            fresh: true,
            ..child(Value::open(Numbers { proof, disproof: 1 }), false)
        }
    }

    /// The depth of a child's position on the current line, if it repeats
    /// one. The line ends with the child's parent's parent, and only positions
    /// since the last capture or pawn move can recur, with the same side to
    /// move: an even number of plies back.
    fn repeated(&self, hash: u64, counter: u32) -> Option<u32> {
        let bit = (hash >> 54) as usize;
        if self.repetitions[bit >> 6] & (1 << (bit & 63)) == 0 {
            return None;
        }
        let len = self.path.len();
        let back = (counter as usize).min(len + 1);
        (2..=back)
            .step_by(2)
            .map(|plies| len + 1 - plies)
            .find(|&depth| self.path[depth] == hash)
            .map(|depth| depth as u32)
    }

    /// Sets the bits `repeated` reads for the moves of a position whose
    /// 50-move counter is `counter`: their children's counters are at most one
    /// more, so only the line's positions that many plies back, with the
    /// children's side to move, could repeat.
    fn mark_repetitions(&mut self, counter: u32) {
        self.repetitions = [0; 16];
        let len = self.path.len();
        let back = (counter as usize + 1).min(len + 1);
        for plies in (2..=back).step_by(2) {
            let bit = (self.path[len + 1 - plies] >> 54) as usize;
            self.repetitions[bit >> 6] |= 1 << (bit & 63);
        }
    }

    /// Tries to prove `pos`, attacker to move, by replaying the proof of
    /// `guide`, a similar position (Kawano's simulation): the attacker plays
    /// the guide's winning moves, and every defence must be answered by a
    /// known proof, an explosion at once, or the same defence in the guide.
    /// The proofs it finds are recorded as the search records its own, so it
    /// is exact; it gives up when the budget runs out. Under a limit, a proof
    /// must take at most `remaining` attacker moves: a known one that takes
    /// more leaves the replay to look for a shorter one.
    fn simulate(
        &mut self,
        pos: &Position,
        guide: &Position,
        remaining: u32,
        budget: &mut u32,
        line: &mut Vec<u64>,
    ) -> Option<(u32, u32)> {
        let counter = pos.halfmoves();
        if let Some(proof) = self.proof_at(pos.hash(), counter)
            && proof.length <= remaining
        {
            return Some((proof.length, proof.limit));
        }
        if counter > LAST || *budget == 0 || remaining == 0 || self.on_line(pos.hash(), line) {
            return None;
        }
        *budget -= 1;
        self.nodes += 1;
        let m = match self.proof_at(guide.hash(), guide.halfmoves()) {
            Some(proof) => proof.m?,
            None => self.explosion(guide).or_else(|| self.dropped_mate(guide))?,
        };
        if !pos.is_pseudo_legal(m) || !pos.legal(m) {
            return None;
        }
        let after = pos.play(m);
        line.push(pos.hash());
        let reached = self.simulate_defences(&after, &guide.play(m), remaining - 1, budget, line);
        line.pop();
        let (length, limit) = reached?;
        let mates = mated(length, limit);
        let limit = if after.halfmoves() == 0 {
            LAST
        } else {
            limit.saturating_sub(1).min(LAST)
        };
        let proof = Proof {
            m: Some(m),
            length: length + 1,
            limit,
        };
        // A capture that explodes the king at once is not kept.
        if after.king(!self.attacker).is_some() {
            self.keep_attacker(pos, proof, mates);
        }
        Some((proof.length, proof.limit))
    }

    /// The defender's side of `simulate`, with `remaining` attacker moves
    /// left.
    fn simulate_defences(
        &mut self,
        pos: &Position,
        guide: &Position,
        remaining: u32,
        budget: &mut u32,
        line: &mut Vec<u64>,
    ) -> Option<(u32, u32)> {
        if pos.king(!self.attacker).is_none() {
            return Some((0, u32::MAX));
        }
        let counter = pos.halfmoves();
        if let Some(proof) = self.proof_at(pos.hash(), counter)
            && proof.length <= remaining
        {
            return Some((proof.length, proof.limit));
        }
        if counter > LAST || self.on_line(pos.hash(), line) {
            return None;
        }
        let mover = Mover::new(pos);
        let mut length = 0;
        let mut limit = LAST;
        let mut defences = 0;
        line.push(pos.hash());
        let mut answered = true;
        // Whether every defence lets the king be exploded at once, which
        // makes the proof one `recomputed` finds again.
        let mut exposed = true;
        for &reply in pos.pseudo_legal().as_slice() {
            if !mover.legal(reply) {
                continue;
            }
            defences += 1;
            let effect = mover.effect(reply);
            if mover.explodes_them(&effect) {
                answered = false;
                break;
            }
            let next = pos.play(reply);
            let exposing = next.halfmoves() <= LAST && mover.exposes(reply, &effect);
            exposed &= exposing;
            let reached = if exposing {
                moves::exploding_capture(&next, self.attacker)
                    .filter(|_| remaining > 0)
                    .map(|_| (1, LAST))
            } else if let Some(proof) = self
                .proof_at(next.hash(), next.halfmoves())
                .filter(|proof| proof.length <= remaining)
            {
                Some((proof.length, proof.limit))
            } else if guide.is_pseudo_legal(reply) && guide.legal(reply) {
                self.simulate(&next, &guide.play(reply), remaining, budget, line)
            } else {
                None
            };
            let Some((reply_length, reply_limit)) = reached else {
                answered = false;
                break;
            };
            length = length.max(reply_length);
            limit = limit.min(if next.halfmoves() == 0 {
                LAST
            } else {
                reply_limit.saturating_sub(1).min(LAST)
            });
        }
        line.pop();
        // No defence at all is a mate if in check, else a stalemate.
        if !answered || (defences == 0 && !pos.in_check()) {
            return None;
        }
        let proof = if defences == 0 {
            Proof::FINISHED
        } else {
            Proof {
                m: None,
                length,
                limit,
            }
        };
        // A mate, or a defence every move of which lets the king be exploded,
        // is not kept: `recomputed` finds it again.
        if defences != 0 && !(exposed && counter < LAST) {
            self.add_proof(pos.hash(), proof);
        }
        debug_assert!(
            defences != 0 && !(exposed && counter < LAST) || self.recomputed(pos) == Some(proof)
        );
        Some((proof.length, proof.limit))
    }

    /// The proof of a position, defender to move, in which every move lets
    /// the attacker explode the king at once, short of the 50-move limit: a
    /// win in two attacker moves, settled without a search and not kept.
    fn exposed(&self, pos: &Position) -> Option<Proof> {
        if pos.halfmoves() >= LAST {
            return None;
        }
        let mover = Mover::new(pos);
        let mut any = false;
        let mut limit = LAST;
        for &reply in pos.pseudo_legal().as_slice() {
            if !mover.legal(reply) {
                continue;
            }
            let effect = mover.effect(reply);
            if mover.explodes_them(&effect) || !mover.exposes(reply, &effect) {
                return None;
            }
            any = true;
            // Where a reply leaves the 50-move counter running, the win holds
            // at one counter less.
            if effect.counter != 0 {
                limit = LAST - 1;
            }
        }
        any.then_some(Proof {
            m: None,
            length: 1,
            limit,
        })
    }

    /// The proof of a position whose proof is not kept because it comes out
    /// the same whenever it is worked out again: the attacker exploding the
    /// king at once, the defender mated, or a defence after which every move
    /// lets the king be exploded; and once the proofs have dropped them, a
    /// mate in one.
    fn recomputed(&self, pos: &Position) -> Option<Proof> {
        if pos.turn() == self.attacker {
            let m = self.explosion(pos).or_else(|| self.dropped_mate(pos))?;
            return Some(Proof {
                m: Some(m),
                length: 1,
                limit: LAST,
            });
        }
        if pos.in_check() && !moves::has_legal_move(pos) {
            return Some(Proof::FINISHED);
        }
        self.exposed(pos)
    }

    /// A check the defender has no move against, before the 50-move limit.
    fn mate_in_one(&self, pos: &Position) -> Option<Move> {
        if pos.halfmoves() > LAST {
            return None;
        }
        let mover = Mover::new(pos);
        pos.pseudo_legal()
            .as_slice()
            .iter()
            .copied()
            .find(|&m| mates(&mover, pos, m))
    }

    /// A mate in one, once the proofs have dropped them and keep none.
    fn dropped_mate(&self, pos: &Position) -> Option<Move> {
        if self.mates_kept {
            None
        } else {
            self.mate_in_one(pos)
        }
    }

    /// Whether a move mates at once in an attacker position, as `mate_in_one`
    /// finds a mate: how a mate in one the table notes is checked.
    fn mates_with(&self, pos: &Position, m: Move) -> bool {
        pos.turn() == self.attacker
            && pos.halfmoves() <= LAST
            && pos.is_pseudo_legal(m)
            && mates(&Mover::new(pos), pos, m)
    }

    /// The proof of the attacker exploding the king at once.
    fn exploding(&self, pos: &Position) -> Option<Proof> {
        self.explosion(pos).map(|m| Proof {
            m: Some(m),
            length: 1,
            limit: LAST,
        })
    }

    /// Keeps an attacker position's proof, mated whether its move mates,
    /// unless it explodes the king at once: those make up most short proofs,
    /// and replays make many of them.
    fn keep_proof(&mut self, pos: &Position, proof: Proof, mated: bool) {
        if proof.length <= 1 && self.exploding(pos) == Some(proof) {
            return;
        }
        self.keep_attacker(pos, proof, mated);
    }

    /// Keeps an attacker position's proof, mated whether its move mates. Once
    /// the proofs have dropped their mates in one, a position that has one
    /// keeps no proof, and drops any it kept before: a parent may count on
    /// the mate, as `recomputed` finds it, and a longer proof found by lookup
    /// would make the parent's win longer than it says. The table notes the
    /// mate instead, so that meeting the position again costs a check of its
    /// move rather than a node.
    fn keep_attacker(&mut self, pos: &Position, proof: Proof, mated: bool) {
        let hash = pos.hash();
        if self.mates_kept {
            if !mated {
                self.add_proof(hash, proof);
                return;
            }
            self.table.note(hash);
            if self.proofs.add_mate(hash, proof) && self.proofs.taken() < self.drop_mates_at {
                return;
            }
            // Dropped with the others.
            self.drop_mates();
        }
        let mate = if mated {
            proof.m
        } else {
            self.mate_in_one(pos)
        };
        match mate {
            Some(m) => {
                self.proofs.remove(hash);
                // With a limit, the table's keys count the moves left.
                if self.most.is_none() {
                    self.table.put(hash, Known::Mate(m.proof_bits()), 1);
                    self.refresh_filter();
                }
            }
            None => self.add_proof(hash, proof),
        }
    }

    /// Replays on a defender position's open defences the win the attacker
    /// would have at once if the defender passed: the first move the tactics
    /// that score moves find winning (an explosion, a check every evasion of
    /// loses, a double threat no defence parries, or a move into a known
    /// proof). It becomes the passed position's proof, and each defence it
    /// still answers is proven by replaying it, as a proven sibling's proof
    /// is replayed; a defence that parries it starts at half its proof
    /// number. Plain moves are not tried: they win at once only into a known
    /// proof.
    fn replay_threat(&mut self, pos: &Position, remaining: u32, children: &mut [Child]) {
        let passed = pos.null();
        let left = remaining.saturating_sub(1);
        let counter = passed.halfmoves();
        if counter > LAST {
            return;
        }
        if self.proof_at(passed.hash(), counter).is_none() {
            let mover = Mover::new(&passed);
            let mut win = None;
            // The moves first, their table reads started, as `children` does.
            let mut moves = std::mem::take(&mut self.scratch);
            moves.clear();
            let filtering = self.table.filtering();
            for &m in passed.pseudo_legal().as_slice() {
                if mover.plain(m) || !mover.legal(m) {
                    continue;
                }
                let effect = mover.full_effect(m);
                self.prefetch_child(effect.hash, left, filtering);
                moves.push((m, effect, !filtering));
            }
            if filtering {
                self.read_filter(&mut moves, left);
            }
            self.mark_repetitions(passed.halfmoves());
            for &(m, effect, seen) in &moves {
                // Only whether the move wins at once counts here, so a check
                // needs one surviving evasion, not their number.
                let child = self.evaluate(&passed, &mover, m, &effect, seen, true, left, 1);
                if child.value.numbers.proof != 0 {
                    continue;
                }
                let (length, limit) = if child.exploded {
                    (0, u32::MAX)
                } else if let Some(proof) = self
                    .proof_at(child.hash, child.counter)
                    .or_else(|| self.recomputed(&passed.play(m)))
                {
                    (proof.length, proof.limit)
                } else {
                    continue;
                };
                let mates = !child.exploded && mated(length, limit);
                let limit = if child.counter == 0 {
                    LAST
                } else {
                    limit.saturating_sub(1).min(LAST)
                };
                let proof = Proof {
                    m: Some(m),
                    length: length + 1,
                    limit,
                };
                win = Some((proof, mates));
                break;
            }
            self.scratch = moves;
            let Some((proof, mates)) = win else {
                return;
            };
            self.keep_proof(&passed, proof, mates);
        }
        for child in children.iter_mut() {
            if child.value.numbers.proof == 0 || child.value.numbers.disproof == 0 {
                continue;
            }
            let position = pos.play(child.m);
            let mut budget = SIMULATION;
            let mut line = std::mem::take(&mut self.line);
            let reached = self.simulate(&position, &passed, remaining, &mut budget, &mut line);
            self.line = line;
            if reached.is_some_and(|(length, _)| length <= remaining) {
                child.value = Value::WON;
                child.simulated = true;
            } else if child.fresh {
                child.value.numbers.proof = (child.value.numbers.proof / 2).max(1);
            }
        }
    }

    /// Whether a position is on the search's line or the replay's.
    fn on_line(&self, hash: u64, line: &[u64]) -> bool {
        self.path.contains(&hash) || line.contains(&hash)
    }

    /// Remembers a finished game that is not won, as a refutation: a won one
    /// is the defender mated, which `recomputed` finds again.
    fn settle(&mut self, hash: u64, counter: u32, value: Value, work: u64) -> Value {
        if value.numbers.proof != 0 {
            self.store(hash, counter, value, work);
        }
        value
    }

    /// The value of a mate, if the side to move, in check, is mated.
    fn mated(&self, pos: &Position) -> Option<Value> {
        if moves::has_legal_move(pos) {
            return None;
        }
        Some(if pos.turn() == self.attacker {
            Value::DRAWN
        } else {
            Value::WON
        })
    }

    /// The attacker moves left `depth` plies from the root.
    fn remaining(&self, depth: usize) -> u32 {
        match self.most {
            None => u32::MAX,
            Some(most) => {
                let made = (depth + usize::from(self.root_attacking)) / 2;
                most.saturating_sub(u32::try_from(made).unwrap_or(u32::MAX))
            }
        }
    }

    /// A position's key in the table with `remaining` attacker moves left:
    /// its hash, apart for each number of moves left under a limit.
    fn key(&self, hash: u64, remaining: u32) -> u64 {
        match self.most {
            None => hash,
            Some(_) => hash ^ (u64::from(remaining) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        }
    }

    /// Whether the proofs or the table may hold a child: its hash for the
    /// proofs, its key for the table.
    fn seen(&self, hash: u64, key: u64) -> bool {
        self.table.may_hold(hash) || (key != hash && self.table.may_hold(key))
    }

    /// Starts loading what a child's scoring reads: its filter words while the
    /// filter is read, else its table cluster and proof slot at once.
    fn prefetch_child(&self, hash: u64, remaining: u32, filtering: bool) {
        if filtering {
            self.prefetch_seen(hash, remaining);
        } else {
            self.table.prefetch(self.key(hash, remaining));
            self.proofs.prefetch(hash);
        }
    }

    /// Reads the filter's bit of every listed child, their words' loads under
    /// way, and starts the table reads of the children it may hold.
    fn read_filter(&self, moves: &mut [(Move, Effect, bool)], remaining: u32) {
        for (_, effect, seen) in moves.iter_mut() {
            let key = self.key(effect.hash, remaining);
            *seen = self.seen(effect.hash, key);
            if *seen {
                self.table.prefetch(key);
                self.proofs.prefetch(effect.hash);
            }
        }
    }

    /// Starts loading the filter's words for a child: for its hash, and for
    /// its key under a limit on moves.
    fn prefetch_seen(&self, hash: u64, remaining: u32) {
        self.table.prefetch_filter(hash);
        let key = self.key(hash, remaining);
        if key != hash {
            self.table.prefetch_filter(key);
        }
    }

    /// The shortest proof of a position that holds at this counter, if it
    /// takes at most `remaining` attacker moves.
    fn proof_within(&self, hash: u64, counter: u32, remaining: u32) -> Option<Proof> {
        self.proof_at(hash, counter)
            .filter(|proof| proof.length <= remaining)
    }

    /// What the table knows about a position at this counter on this line: a
    /// mate in one only where `mates` finds that its move mates.
    fn lookup(&self, hash: u64, counter: u32, mates: impl FnOnce(Move) -> bool) -> Option<Value> {
        match self.table.get(hash)? {
            Known::Mate(bits) => mates(Move::from_proof_bits(bits)).then_some(Value::WON),
            Known::Open { proof, disproof } => Some(Value::open(Numbers { proof, disproof })),
            Known::Refuted { cycle, from } => {
                if counter < from {
                    return None;
                }
                let cycle = match cycle {
                    None => ANY_LINE,
                    Some(repeated) => self
                        .path
                        .iter()
                        .position(|&on_line| table::check(on_line) == repeated)?
                        as u32,
                };
                Some(Value {
                    numbers: LOST,
                    cycle,
                    fifty: from > 0,
                })
            }
        }
    }

    /// Stores a value with the work that found it, which decides how long
    /// the table keeps it. Out of line, with the table's `put` in it: the
    /// other way round costs the search's loops more instructions.
    #[inline(never)]
    fn store(&mut self, hash: u64, counter: u32, value: Value, work: u64) {
        let known = if value.numbers.disproof != 0 {
            Known::Open {
                proof: value.numbers.proof,
                disproof: value.numbers.disproof,
            }
        } else {
            Known::Refuted {
                cycle: (value.cycle != ANY_LINE)
                    .then(|| table::check(self.path[value.cycle as usize])),
                from: if value.fifty { counter } else { 0 },
            }
        };
        self.table.put(hash, known, work);
        self.refresh_filter();
    }

    /// Rebuilds the table's filter once it is stale.
    #[inline(always)]
    fn refresh_filter(&mut self) {
        if self.table.filter_is_stale() {
            self.table
                .rebuild_filter(self.proofs.keys(), self.proofs.taken());
        }
    }

    /// The shortest proof of a position that holds at this counter.
    fn proof_at(&self, hash: u64, counter: u32) -> Option<Proof> {
        if !self.table.may_hold(hash) {
            return None;
        }
        self.proofs.get(hash, counter)
    }

    /// The attacker's capture that explodes the defender's king at once before
    /// the 50-move limit: the proof of a position whose proof is not kept.
    fn explosion(&self, pos: &Position) -> Option<Move> {
        if pos.halfmoves() > LAST {
            return None;
        }
        moves::exploding_capture(pos, self.attacker)
    }

    /// Records a proven position: attacking, through the shortest winning
    /// move; defending, through every defence.
    fn record_proof(&mut self, pos: &Position, hash: u64, children: &[Child], attacking: bool) {
        // A child's proof seen from here: its length, the highest counter
        // here it holds at, which a capture or pawn move frees, and whether it
        // is the defender mated.
        let reach = |child: &Child| -> Option<(u32, u32, bool)> {
            let counter = child.counter;
            let (length, limit) = if child.exploded {
                (0, u32::MAX)
            } else if child.winning {
                (1, LAST)
            } else {
                if child.value.numbers.proof != 0 {
                    return None;
                }
                let proof = self
                    .proof_at(child.hash, counter)
                    .or_else(|| self.recomputed(&pos.play(child.m)))?;
                (proof.length, proof.limit)
            };
            let mates = !child.exploded && mated(length, limit);
            let limit = if counter == 0 {
                LAST
            } else {
                limit.saturating_sub(1).min(LAST)
            };
            Some((length, limit, mates))
        };
        let proof = if attacking {
            children
                .iter()
                .filter(|child| !child.group)
                .filter_map(|child| reach(child).map(|reached| (child.m, reached)))
                .min_by_key(|&(_, (length, limit, _))| (length, Reverse(limit)))
                .map(|(m, (length, limit, mates))| {
                    let proof = Proof {
                        m: Some(m),
                        length: length + 1,
                        limit,
                    };
                    (proof, mates)
                })
        } else {
            children
                .iter()
                .map(reach)
                .try_fold(
                    Proof {
                        m: None,
                        length: 0,
                        limit: LAST,
                    },
                    |proof, reached| {
                        let (length, limit, _) = reached?;
                        Some(Proof {
                            m: None,
                            length: proof.length.max(length),
                            limit: proof.limit.min(limit),
                        })
                    },
                )
                .map(|proof| (proof, false))
        };
        if attacking && let Some((Proof { m: Some(m), .. }, _)) = proof {
            let index = history_index(pos, m);
            self.attack_history[index] = self.attack_history[index].saturating_add(1);
        }
        if let Some((proof, mates)) = proof {
            debug_assert_eq!(hash, pos.hash());
            // An attacker's shortest win longer than one move leaves no mate
            // in one there: one would have been shorter.
            if attacking && proof.length <= 1 {
                self.keep_proof(pos, proof, mates);
            } else if attacking
                || !(pos.halfmoves() < LAST && children.iter().all(|child| child.winning))
            {
                self.add_proof(hash, proof);
            }
        }
    }

    /// Keeps a proof. When the proofs fill, the first time they drop their
    /// mates in one (`drop_mates`), and the second the search stops.
    fn add_proof(&mut self, hash: u64, proof: Proof) {
        self.table.note(hash);
        if self.proofs.add(hash, proof) {
            if self.mates_kept && self.proofs.taken() >= self.drop_mates_at {
                self.drop_mates();
            }
            return;
        }
        if self.mates_kept {
            self.drop_mates();
            if self.proofs.add(hash, proof) {
                return;
            }
        }
        self.full = true;
        self.stopped = true;
    }

    /// Drops the mates in one from the proofs, so that the search goes on once
    /// they fill: from then on it keeps none (`keep_attacker`), and works each
    /// out again where it needs one (`recomputed`).
    fn drop_mates(&mut self) {
        self.mates_kept = false;
        self.proofs.drop_mates();
        self.mates_dropped = Some(self.nodes);
    }

    fn tick(&mut self) {
        self.nodes += 1;
        if self.nodes >= self.limit || self.evals >= self.eval_limit {
            self.stopped = true;
        } else if self.nodes.is_multiple_of(1024) {
            let now = Instant::now();
            if now >= self.deadline || self.stop.is_some_and(|stop| stop.load(Ordering::Relaxed)) {
                self.stopped = true;
            }
        }
    }
}

/// Whether a child's proof, by its length and limit, is a finished game: for
/// a defender position, mated.
fn mated(length: u32, limit: u32) -> bool {
    length == Proof::FINISHED.length && limit == Proof::FINISHED.limit
}

impl Proof {
    /// A finished game: won at any counter.
    const FINISHED: Proof = Proof {
        m: None,
        length: 0,
        limit: u32::MAX,
    };
}

/// How `--memory` divides between the table and the proofs: the proofs get
/// what the search's budget needs (`proofs_needed`), up to three quarters,
/// and at least what is left after the table's largest power of two up to two
/// thirds; the table gets the rest. Proofs that fill stop a search, once their
/// mates in one are dropped, while a full table costs little. Each takes at
/// most `MOST_ONE` megabytes.
pub fn split(memory_mb: usize, needed_mb: usize) -> (usize, usize) {
    let table = match memory_mb.saturating_mul(2) / 3 {
        0 => 0,
        most => (1 << most.ilog2()).min(MOST_ONE),
    };
    let proofs = needed_mb
        .min(memory_mb - memory_mb / 4)
        .max(memory_mb - table)
        .min(MOST_ONE);
    ((memory_mb - proofs).min(MOST_ONE), proofs)
}

/// The proofs' megabytes a search's budget needs, the less of what its time
/// limit and its nodes need.
pub fn proofs_needed(seconds: Option<f64>, nodes: Option<u64>) -> usize {
    let for_time = seconds.map(|seconds| (seconds * PROOF_MB_A_SECOND).ceil() as usize);
    let for_nodes =
        nodes.map(|nodes| nodes.saturating_mul(PROOF_BYTES_A_NODE).div_ceil(1 << 20) as usize);
    match (for_nodes, for_time) {
        (Some(nodes), Some(time)) => nodes.min(time),
        (nodes, time) => nodes.or(time).unwrap_or(0),
    }
}

/// Proof memory a search may need for each second of its time limit and each
/// node of its budget, at 12 bytes a proof.
const PROOF_MB_A_SECOND: f64 = 1.5;
const PROOF_BYTES_A_NODE: u64 = 3;

/// The most megabytes one table may take: no allocation reaches `isize::MAX`
/// bytes, which on 32-bit targets such as WebAssembly is 2 GB.
const MOST_ONE: usize = isize::MAX as usize >> 20;

/// `x.round()`. WebAssembly has no instruction for it and calls a library
/// function, slowly; there, `round_by_truncation`.
#[cfg(target_arch = "wasm32")]
fn round(x: f32) -> f32 {
    round_by_truncation(x)
}

#[cfg(not(target_arch = "wasm32"))]
fn round(x: f32) -> f32 {
    x.round()
}

/// Scales the new children's starting numbers by their parent's evaluation,
/// as EF-PN does (Winands and Schadd): `attacking` when the parent is the
/// attacker's, `step` from 0 (the defender clearly ahead) to 2.
fn scale(children: &mut [Child], attacking: bool, step: Option<f32>) {
    let Some(step) = step else {
        return;
    };
    let (a, b) = (EF_A as f32, EF_B as f32);
    let times = |number: u32, factor: f32| (round(number as f32 * factor) as u32).min(1 << 24);
    for child in children.iter_mut().filter(|child| child.fresh) {
        let numbers = &mut child.value.numbers;
        if attacking {
            numbers.proof = times(numbers.proof, 1.0 + b * (2.0 - step));
            numbers.disproof = times(numbers.disproof, 1.0 + a * step);
        } else {
            // A defence's new children start at proof number 1.
            numbers.proof = times(numbers.proof, 1.0) + round(a * (2.0 - step)) as u32;
            numbers.disproof = times(numbers.disproof, 1.0 + b * step);
        }
    }
}

/// How a seed breaks an otherwise exact tie: lower wins, and 0 is no seed,
/// which leaves the first child the best.
fn tie(hash: u64, seed: u64) -> u64 {
    if seed == 0 {
        0
    } else {
        hash.wrapping_add(seed).wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }
}

/// Combines the children: the position's value, the child to search, and the
/// second-best child's deciding number.
fn combine(children: &[Child], attacking: bool, seed: u64) -> (Value, usize, u32) {
    let mut best_score = 0;
    let mut best_tie = 0;
    let mut best = 0;
    let mut first = u32::MAX;
    let mut second = u32::MAX;
    let mut sum = 0u32;
    // With every child refuted, a refutation relies on what any of them does.
    let mut cycle = ANY_LINE;
    let mut fifty = false;
    // With one refuted, it relies only on what the sturdiest one does.
    let mut sturdiest: Option<(u32, bool)> = None;
    for (index, child) in children.iter().enumerate() {
        let numbers = child.value.numbers;
        // Attacking, a child is best when cheapest to prove; defending, when
        // cheapest to refute.
        let (deciding, summed) = if attacking {
            (numbers.proof, numbers.disproof)
        } else {
            (numbers.disproof, numbers.proof)
        };
        sum = sum.saturating_add(summed);
        // Ties go to the move with more history, then to the seed's pick.
        let child_tie = tie(child.hash, seed);
        if deciding < first
            || (deciding == first
                && (child.score > best_score
                    || (child.score == best_score && child_tie < best_tie)))
        {
            second = first;
            first = deciding;
            best = index;
            best_score = child.score;
            best_tie = child_tie;
        } else if deciding < second {
            second = deciding;
        }
        if numbers.disproof == 0 {
            cycle = cycle.min(child.value.cycle);
            fifty |= child.value.fifty;
            let sturdiness = (child.value.cycle, !child.value.fifty);
            if sturdiest.is_none_or(|known| sturdiness > known) {
                sturdiest = Some(sturdiness);
            }
        }
    }
    let (first, second) = (first.min(INFINITY), second.min(INFINITY));
    // Only a settled child makes a sum infinite: sums over many open ones
    // grow fast, and one that reached infinity would read as settled.
    let sum = if sum >= INFINITY
        && !children.iter().any(|child| {
            let numbers = child.value.numbers;
            (if attacking {
                numbers.disproof
            } else {
                numbers.proof
            }) >= INFINITY
        }) {
        INFINITY - 1
    } else {
        sum.min(INFINITY)
    };
    let numbers = if attacking {
        Numbers {
            proof: first,
            disproof: sum,
        }
    } else {
        Numbers {
            proof: sum,
            disproof: first,
        }
    };
    let value = if numbers.disproof != 0 {
        Value::open(numbers)
    } else if attacking {
        Value {
            numbers,
            cycle,
            fifty,
        }
    } else {
        let (cycle, sturdy) = sturdiest.unwrap_or((ANY_LINE, true));
        Value {
            numbers,
            cycle,
            fifty: !sturdy,
        }
    };
    (value, best, second)
}

/// A child's limit on the number its parent sums: its own number plus what
/// the sum may still grow by. A parent without a limit gives none: its sum may
/// stand just below infinity (see `combine`), and taking that from infinity
/// would leave the child a margin of one, sending it back after every step.
fn share(limit: u32, sum: u32, own: u32) -> u32 {
    if limit >= INFINITY {
        INFINITY
    } else {
        capped(limit - sum + own)
    }
}

fn capped(value: u32) -> u32 {
    value.min(INFINITY)
}

/// The 1+ε trick, ε = 1/8: a child may run until it is somewhat worse than
/// the second-best one, which spares many switches between near-equal
/// children.
fn widen(second: u32) -> u32 {
    if second >= INFINITY {
        INFINITY
    } else {
        capped(second + 1 + second / 8)
    }
}

/// A move's place in the history tables: its piece and destination.
fn history_index(pos: &Position, m: Move) -> usize {
    let role = pos.role_at(m.from()).map_or(0, |role| role as usize);
    role * 64 + m.to() as usize
}

/// Whether a pseudo-legal move mates: a check the other side has no move
/// against, rather than an explosion of its king.
fn mates(mover: &Mover, pos: &Position, m: Move) -> bool {
    // Plain moves check nothing, and telling them costs a bit test.
    if mover.plain(m) || !mover.legal(m) {
        return false;
    }
    let effect = mover.effect(m);
    !mover.explodes_them(&effect)
        && mover.gives_check(m, &effect)
        && !moves::has_legal_move(&pos.play(m))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use shakmaty::Position as _;

    use super::{Child, Dfpn, Guide, INFINITY, Numbers, Value, combine, share, widen};
    use crate::board::Position;

    fn child_at(hash: u64, proof: u32, disproof: u32) -> Child {
        Child {
            m: Default::default(),
            hash,
            counter: 0,
            value: Value::open(Numbers { proof, disproof }),
            exploded: false,
            winning: false,
            score: 0,
            simulated: false,
            fresh: false,
            group: false,
        }
    }

    fn child(proof: u32, disproof: u32) -> Child {
        child_at(0, proof, disproof)
    }

    #[test]
    fn values_and_thresholds() {
        let won = combine(&[child(0, INFINITY)], true, 0);
        assert_eq!((won.0.numbers.proof, won.0.numbers.disproof), (0, INFINITY));
        let lost = combine(&[child(INFINITY, 0)], true, 0);
        assert_eq!(
            (lost.0.numbers.proof, lost.0.numbers.disproof),
            (INFINITY, 0)
        );
        let open = combine(&[child(3, 5), child(7, 2)], true, 0);
        assert_eq!((open.0.numbers.proof, open.0.numbers.disproof), (3, 7));
        assert_eq!(widen(8), 10);
        assert_eq!(widen(INFINITY), INFINITY);
        assert_eq!(share(INFINITY, 5, 2), INFINITY);
        assert_eq!(share(20, 5, 2), 17);
    }

    #[test]
    fn a_seed_breaks_only_exact_ties() {
        let children = [child_at(1, 3, 5), child_at(2, 3, 5)];
        let (_, first, _) = combine(&children, true, 0);
        assert_eq!(first, 0, "seed 0 keeps the first child");
        let mut reordered = false;
        for seed in 1..=20u64 {
            let (_, best, _) = combine(&children, true, seed.wrapping_mul(0xD1B5_4A32_D192_ED03));
            reordered |= best == 1;
        }
        assert!(
            reordered,
            "a seed can reorder children with equal numbers and history"
        );
    }

    fn core_counters(
        seed: u64,
        limit: Option<u64>,
    ) -> (Option<u32>, u64, u64, u64, Option<(u32, u32)>) {
        let root =
            crate::position("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let mut search = Dfpn::new(
            &root,
            Guide::Hand,
            Instant::now() + Duration::from_secs(10),
            (16, 16),
        );
        search.set_seed(seed);
        if let Some(limit) = limit {
            search.set_limit(limit);
        }
        let moves = search.solve(&root);
        (
            moves,
            search.nodes,
            search.evals,
            search.expansions,
            search.root_numbers,
        )
    }

    #[test]
    fn seed_zero_keeps_the_core_tree_unchanged() {
        // One deterministic node budget, pinned so that a guide or tie-break
        // change cannot slip through unnoticed.
        assert_eq!(
            core_counters(0, Some(100_000)),
            (None, 100_005, 1_181_957, 58_248, Some((371, 26_198)))
        );
    }

    #[test]
    fn seeds_are_deterministic() {
        for seed in 1..=2 {
            assert_eq!(
                core_counters(seed, Some(100_000)),
                core_counters(seed, Some(100_000))
            );
        }
    }

    #[test]
    fn a_seed_does_not_change_a_proof() {
        let root =
            crate::position("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        for seed in 1..=2 {
            let mut search = Dfpn::new(
                &root,
                Guide::Hand,
                Instant::now() + Duration::from_secs(10),
                (16, 16),
            );
            search.set_seed(seed);
            let moves = search.solve(&root);
            assert!(moves.is_some(), "seed {seed}: no win");
            let (_, bytes) = crate::format::encode(&root, root.turn(), |pos| {
                let key = (pos.hash(), pos.halfmoves());
                search
                    .winning_move(&pos.to_atomic(), &key)
                    .and_then(|m| crate::format::board_move(pos, &m))
            })
            .unwrap();
            let checked = crate::check::check_proof("core", &bytes);
            assert_eq!(checked["valid"], true, "seed {seed}: {checked}");
        }
    }

    #[test]
    fn a_repeated_position_is_a_draw() {
        let root = crate::position("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let mut search = Dfpn::new(
            &root,
            Guide::None,
            Instant::now() + Duration::from_secs(1),
            (1, 1),
        );
        let hash = Position::from_atomic(&root).hash();
        let bit = (hash >> 54) as usize;
        search.repetitions[bit >> 6] |= 1 << (bit & 63);
        search.path.push(hash);
        assert_eq!(search.repeated(hash, 2), Some(0));
    }

    #[test]
    fn the_fifty_move_limit_is_not_a_win() {
        let root = crate::position("4k3/8/8/8/8/8/4P3/4K3 w - - 100 80").unwrap();
        let mut search = Dfpn::new(
            &root,
            Guide::Hand,
            Instant::now() + Duration::from_secs(1),
            (1, 1),
        );
        assert_eq!(search.solve(&root), None);
    }

    #[test]
    fn proves_core_and_writes_a_checked_proof() {
        let root =
            crate::position("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let mut search = Dfpn::new(
            &root,
            Guide::Hand,
            Instant::now() + Duration::from_secs(10),
            (16, 16),
        );
        assert_eq!(search.solve(&root), Some(14));
        let (header, bytes) = crate::format::encode(&root, root.turn(), |pos| {
            let key = (pos.hash(), pos.halfmoves());
            search
                .winning_move(&pos.to_atomic(), &key)
                .and_then(|m| crate::format::board_move(pos, &m))
        })
        .unwrap();
        assert_eq!(header.moves, 14);
        let checked = crate::check::check_proof("core", &bytes);
        assert_eq!(checked["valid"], true, "{checked}");
    }

    #[test]
    fn a_known_loss_is_not_proved() {
        let root =
            crate::position("rnbqkbnr/ppp1pQ1p/3p1pp1/8/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 1 4")
                .unwrap();
        let mut search = Dfpn::new(
            &root,
            Guide::Hand,
            Instant::now() + Duration::from_secs(2),
            (16, 16),
        );
        assert_eq!(search.solve(&root), None);
    }
}
