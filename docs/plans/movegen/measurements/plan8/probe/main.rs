//! Plan 8 per-call screen: V0, VB, V1 ladder, V3 ChildEffect.

use std::hint::black_box;

use atomic_movegen as mg;
use mg::{EnPassantMode, FastLegality, PieceType};

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
    let contexts: Vec<mg::LegalityContext> = boards
        .iter()
        .zip(&counts)
        .map(|(b, c)| b.legality_context(*c))
        .collect();
    let fasts: Vec<FastLegality> = boards
        .iter()
        .zip(&states)
        .map(|(b, s)| FastLegality::for_board(b, s))
        .collect();
    let lists: Vec<mg::MoveList> = boards
        .iter()
        .map(|b| {
            let mut list = mg::MoveList::new();
            mg::movegen::generate_pseudo_legal(b, &mut list);
            list
        })
        .collect();

    let kings: Vec<mg::Square> = boards
        .iter()
        .map(|b| {
            (b.commoners(mg::Color::White) | b.commoners(mg::Color::Black)).lsb()
        })
        .collect();

    let mut ops: u64 = 0;
    let mut checksum: u64 = 0;
    for _ in 0..repeats {
        match which.as_str() {
            "v0" => {
                for ((b, ctx, list), king) in zip3b(&boards, &contexts, &lists).zip(&kings) {
                    let occupied = ctx.occupied();
                    for _ in list.as_slice() {
                        let attackers = b.attackers_to(black_box(*king), black_box(occupied));
                        let roles = b.pieces_pt(PieceType::Queen) | b.pieces_pt(PieceType::Rook);
                        checksum = checksum
                            .wrapping_add(attackers.0)
                            .wrapping_add(roles.0);
                        ops += 1;
                    }
                }
            }
            "vb" => {
                for (ctx, list) in contexts.iter().zip(&lists) {
                    for &m in list.as_slice() {
                        checksum += u64::from(black_box(ctx.legal(black_box(m))));
                        ops += 1;
                    }
                }
            }
            "v1" => {
                for (fast, b, state, list) in zip4(&fasts, &boards, &states, &lists) {
                    for &m in list.as_slice() {
                        checksum += u64::from(black_box(fast.legal_in(black_box(b), black_box(m), black_box(state))));
                        ops += 1;
                    }
                }
            }
            "v3" => {
                for (b, ctx, list) in zip3(&boards, &contexts, &lists) {
                    let occupied = ctx.occupied();
                    let counts = b.counts();
                    for &m in list.as_slice() {
                        let e = b
                            .child_effect_with_occupancy(
                                m,
                                occupied,
                                counts,
                                EnPassantMode::Legal,
                            )
                            .unwrap();
                        checksum = checksum
                            .wrapping_add(e.key)
                            .wrapping_add(e.occupied.0)
                            .wrapping_add(u64::from(e.rule50))
                            .wrapping_add(u64::from(e.capture));
                        ops += 1;
                    }
                }
            }
            other => panic!("unknown op {other}"),
        }
    }
    println!("ops={ops} checksum={checksum}");
}

fn zip4<'a>(
    a: &'a [FastLegality],
    b: &'a [mg::Board],
    c: &'a [mg::StateInfo],
    d: &'a [mg::MoveList],
) -> impl Iterator<Item = (&'a FastLegality, &'a mg::Board, &'a mg::StateInfo, &'a mg::MoveList)> {
    a.iter().zip(b).zip(c).zip(d).map(|(((a, b), c), d)| (a, b, c, d))
}

fn zip3b<'a>(
    a: &'a [mg::Board],
    b: &'a [mg::LegalityContext],
    c: &'a [mg::MoveList],
) -> impl Iterator<Item = (&'a mg::Board, &'a mg::LegalityContext, &'a mg::MoveList)> {
    a.iter().zip(b).zip(c).map(|((a, b), c)| (a, b, c))
}

fn zip3<'a>(
    a: &'a [mg::Board],
    b: &'a [mg::LegalityContext],
    c: &'a [mg::MoveList],
) -> impl Iterator<Item = (&'a mg::Board, &'a mg::LegalityContext, &'a mg::MoveList)> {
    a.iter().zip(b).zip(c).map(|((a, b), c)| (a, b, c))
}
