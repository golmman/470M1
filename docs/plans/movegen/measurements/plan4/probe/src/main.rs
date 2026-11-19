//! Per-call cost probe for the movegen plan4 acceptance numbers.
//!
//! Ten FENs from the hot-path audit, one operation in a hot loop, then
//! `perf stat -e instructions` around the process divided by the printed
//! `ops`. The operations are the 2.10.0 entry points plus the 2.9.0 adapter
//! shapes they replace.

use std::hint::black_box;

use atomic_movegen as mg;
use mg::EnPassantMode;

const FENS: &[&str] = &[
    "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "rn2k2r/pp3ppp/2p5/3pP3/8/2N5/PPP2PPP/R3K2R w KQkq d6 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    "k7/8/3npn2/4P3/8/8/3nnn2/3nKn2 w - - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r1b1Brk1/ppp5/6pp/3p4/5p2/P3PP2/1P4PP/R4RK1 b - - 1 15",
    "rnbqkbnr/ppp4p/4p1p1/3pNp2/7Q/4P3/PPPP1PPP/RNB1KB1R b KQkq - 1 5",
    "8/6kp/8/6P1/8/8/8/K7 b - - 0 1",
];

fn main() {
    let which = std::env::args().nth(1).expect("op");
    let repeats: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2_000);

    let boards: Vec<mg::Board> = FENS.iter().map(|f| mg::Board::from_fen(f).unwrap()).collect();
    let states: Vec<mg::StateInfo> = boards
        .iter()
        .map(|b| {
            let mut state = mg::StateInfo::new();
            b.populate_state(&mut state);
            state
        })
        .collect();
    let counts: Vec<mg::Counts> = boards.iter().map(mg::Board::counts).collect();
    let lists: Vec<mg::MoveList> = boards
        .iter()
        .map(|b| {
            let mut list = mg::MoveList::new();
            mg::movegen::generate_pseudo_legal(b, &mut list);
            list
        })
        .collect();

    let mut ops: u64 = 0;
    let mut checksum: u64 = 0;
    for _ in 0..repeats {
        match which.as_str() {
            "state" => {
                for b in &boards {
                    let mut state = mg::StateInfo::new();
                    b.populate_state(black_box(&mut state));
                    ops += 1;
                    checksum = checksum.wrapping_add(state.pinned.0);
                }
            }
            "counts" => {
                for b in &boards {
                    let counts = black_box(b.counts());
                    ops += 1;
                    checksum = checksum.wrapping_add(u64::from(counts.commoners()));
                }
            }
            "populate_counts" => {
                for b in &boards {
                    let mut state = mg::StateInfo::new();
                    b.populate_counts(black_box(&mut state));
                    ops += 1;
                    checksum = checksum.wrapping_add(u64::from(state.commoners_count));
                }
            }
            "legal_full" => {
                for ((b, state), list) in boards.iter().zip(&states).zip(&lists) {
                    for &m in list.as_slice() {
                        checksum += u64::from(black_box(b.legal(m, state)));
                        ops += 1;
                    }
                }
            }
            "legal_context" => {
                for (b, counts, list) in itertools(&boards, &counts, &lists) {
                    let context = b.legality_context(*counts);
                    for &m in list.as_slice() {
                        checksum += u64::from(black_box(context.legal(m)));
                        ops += 1;
                    }
                }
            }
            "effect_old" => {
                for (b, list) in boards.iter().zip(&lists) {
                    for &m in list.as_slice() {
                        let (hash, rule50) = b.hash_and_rule50_after(m);
                        let hash = if b.is_double_push(m) {
                            b.hash_after_with(m, EnPassantMode::Legal)
                        } else {
                            hash
                        };
                        checksum = checksum.wrapping_add(hash).wrapping_add(u64::from(rule50));
                        ops += 1;
                    }
                }
            }
            "effect_child" => {
                for (b, counts, list) in itertools(&boards, &counts, &lists) {
                    let context = b.legality_context(*counts);
                    let occupied = context.occupied();
                    for &m in list.as_slice() {
                        let effect = b
                            .child_effect_with_occupancy(
                                m,
                                occupied,
                                *counts,
                                EnPassantMode::Legal,
                            )
                            .unwrap();
                        checksum = checksum
                            .wrapping_add(effect.key)
                            .wrapping_add(effect.occupied.0)
                            .wrapping_add(u64::from(effect.rule50))
                            .wrapping_add(u64::from(effect.capture));
                        ops += 1;
                    }
                }
            }
            "null" => {
                for b in &boards {
                    let child = b.null_for_advanced(b.side_to_move().flip());
                    checksum = checksum
                        .wrapping_add(child.hash())
                        .wrapping_add(u64::from(child.rule50()));
                    ops += 1;
                }
            }
            "native_roundtrip" => {
                for list in &lists {
                    for &m in list.as_slice() {
                        let back = mg::Move::from_bits(m.to_bits()).unwrap();
                        checksum = checksum.wrapping_add(u64::from(back.to_bits()) & 1);
                        ops += 1;
                    }
                }
            }
            other => panic!("unknown op {other}"),
        }
    }
    println!("ops={ops} checksum={checksum}");
}

fn itertools<'a>(
    a: &'a [mg::Board],
    b: &'a [mg::Counts],
    c: &'a [mg::MoveList],
) -> impl Iterator<Item = (&'a mg::Board, &'a mg::Counts, &'a mg::MoveList)> {
    a.iter().zip(b).zip(c).map(|((a, b), c)| (a, b, c))
}
