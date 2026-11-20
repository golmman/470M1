//! The search proves the known-win fixtures and refuses the never-prove
//! negatives. Every proof it writes is accepted by the independent checker.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use atomic470mi::check::check_proof;
use atomic470mi::dfpn::{Dfpn, Guide};
use atomic470mi::{format, position};

fn rows(name: &str) -> Vec<(String, String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter(|line| !line.starts_with("name\t"))
        .map(|line| {
            let mut fields = line.split('\t');
            (
                fields.next().unwrap().to_owned(),
                fields.next().unwrap().to_owned(),
                fields.next().unwrap_or("").to_owned(),
            )
        })
        .collect()
}

fn prove(fen: &str, seconds: u64) -> (Option<u32>, Vec<u8>) {
    let root = position(fen).unwrap();
    let mut search = Dfpn::new(
        &root,
        Guide::Hand,
        Instant::now() + Duration::from_secs(seconds),
        (32, 32),
    );
    let moves = search.solve(&root);
    let bytes = if moves.is_some() {
        let (_, bytes) = format::encode(&root, root.turn(), |pos| {
            let key = (pos.hash(), pos.halfmoves());
            search.winning_move(pos, &key)
        })
        .unwrap();
        bytes
    } else {
        Vec::new()
    };
    (moves, bytes)
}

#[test]
fn proves_every_known_win_fixture() {
    for (name, fen, role) in rows("veto.tsv") {
        if role != "prove" && role != "rule50" {
            continue;
        }
        let (moves, bytes) = prove(&fen, 10);
        assert!(moves.is_some(), "{name}: no win");
        let checked = check_proof(&name, &bytes);
        assert_eq!(checked["valid"], true, "{name}: {checked}");
    }
}

#[test]
fn refuses_every_never_prove_negative() {
    for (name, fen, _) in rows("never_prove.tsv") {
        let (moves, _) = prove(&fen, 2);
        assert_eq!(moves, None, "{name}: proved a known loss");
    }
}
