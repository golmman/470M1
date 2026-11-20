//! The three fixed bench cells, proved at the 10 s / 192 MB protocol twice:
//! each proof must pass our checker and the second run must repeat the first
//! run's counters exactly.

use std::time::{Duration, Instant};

use atomic470mi::check::check_proof;
use atomic470mi::dfpn::{self, Dfpn, Guide};
use atomic470mi::{format, position};

const CELLS: [(&str, &str); 3] = [
    (
        "core",
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    ),
    (
        "easy-win",
        "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    ),
    (
        "deep-win",
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
    ),
];

#[derive(PartialEq, Eq, Debug)]
struct Counters {
    mate: Option<u32>,
    nodes: u64,
    evals: u64,
    expansions: u64,
}

struct Run {
    counters: Counters,
    proof: Vec<u8>,
}

fn run(fen: &str) -> Run {
    let root = position(fen).unwrap();
    let seconds = 10.0;
    let needed = dfpn::proofs_needed(Some(seconds), None);
    let (table, proof_mb) = dfpn::split(192, needed);
    let mut search = Dfpn::try_new(
        &root,
        Guide::Hand,
        Instant::now() + Duration::from_secs(10),
        (table, proof_mb),
    )
    .unwrap();
    let mate = search.solve(&root);
    let counters = Counters {
        mate,
        nodes: search.nodes,
        evals: search.evals,
        expansions: search.expansions,
    };
    let proof = if mate.is_some() {
        let (_, bytes) = format::encode(&root, root.turn(), |pos| {
            let key = (pos.hash(), pos.halfmoves());
            search.winning_move(pos, &key)
        })
        .unwrap();
        bytes
    } else {
        Vec::new()
    };
    Run { counters, proof }
}

#[test]
#[ignore = "slow: proves the three bench cells at 10 s twice"]
fn bench_cells_prove_and_repeat_their_counters() {
    for (name, fen) in CELLS {
        let first = run(fen);
        assert!(first.counters.mate.is_some(), "{name}: no win at 10 s");
        let checked = check_proof(name, &first.proof);
        assert_eq!(checked["valid"], true, "{name}: {checked}");

        let second = run(fen);
        assert_eq!(second.counters, first.counters, "{name}: counters moved");
        let checked = check_proof(name, &second.proof);
        assert_eq!(checked["valid"], true, "{name}: {checked}");
    }
}
