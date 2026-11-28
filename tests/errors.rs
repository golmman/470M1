//! The layers' typed errors: their messages, their sources, the checker's
//! verdict split, and the source guard that keeps `String` errors and bare
//! panic-like calls out of the library.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use atomic470mi::board::{Color, FenError, Flaw};
use atomic470mi::check::{self, CheckOptions, Rejection};
use atomic470mi::walk;
use atomic470mi::{format, lines, position, search};
use serde_json::{Value, json};

/// One snapshot per variant: the Display text a driver or a test may match.
#[test]
fn display_snapshots_are_unchanged() {
    let cases: Vec<(String, &str)> = vec![
        (
            FenError::Syntax(atomic_movegen::FenLoadError::Syntax(
                atomic_movegen::board::FenError::TooShort { parts: 1 },
            ))
            .to_string(),
            "bad FEN: FEN too short: expected at least 4 parts, got 1",
        ),
        (
            FenError::Illegal(vec![Flaw::EmptyBoard]).to_string(),
            "illegal position: the board is empty",
        ),
        (
            FenError::Illegal(vec![Flaw::MissingKing(Color::White)]).to_string(),
            "illegal position: White has no king",
        ),
        (
            FenError::Illegal(vec![Flaw::MissingKing(Color::Black)]).to_string(),
            "illegal position: Black has no king",
        ),
        (
            FenError::Illegal(vec![Flaw::TooManyKings(Color::White)]).to_string(),
            "illegal position: White has more than one king",
        ),
        (
            FenError::Illegal(vec![Flaw::TooManyKings(Color::Black)]).to_string(),
            "illegal position: Black has more than one king",
        ),
        (
            FenError::Illegal(vec![Flaw::PawnOnBackRank]).to_string(),
            "illegal position: a pawn stands on the first or last rank",
        ),
        (
            FenError::Illegal(vec![Flaw::OppositeCheck]).to_string(),
            "illegal position: the side that just moved is in check",
        ),
        (
            FenError::Illegal(vec![Flaw::ImpossibleCheck]).to_string(),
            "illegal position: no game reaches the check",
        ),
        (
            FenError::Illegal(vec![Flaw::Unreachable]).to_string(),
            "illegal position: no game reaches it",
        ),
        (
            FenError::Illegal(vec![Flaw::MissingKing(Color::White), Flaw::PawnOnBackRank])
                .to_string(),
            "illegal position: White has no king; a pawn stands on the first or last rank",
        ),
        (
            walk::Error::Repeat.to_string(),
            "the proof repeats a position",
        ),
        (
            walk::Error::NoMove.to_string(),
            "a position of the proof has no winning move",
        ),
        (format::Error::Json("json".into()).to_string(), "json"),
        (format::Error::Zstd("zstd".into()).to_string(), "zstd"),
        (format::Error::Header("header".into()).to_string(), "header"),
        (
            format::Error::Position("position".into()).to_string(),
            "position",
        ),
        (format::Error::Move("move".into()).to_string(), "move"),
        (format::Error::Walk("walk".into()).to_string(), "walk"),
        (
            format::Error::Tablebases("tablebases".into()).to_string(),
            "tablebases",
        ),
        (
            format::Error::Mismatch("mismatch".into()).to_string(),
            "mismatch",
        ),
        (
            lines::Error::Walk(walk::Error::Repeat).to_string(),
            "the proof repeats a position",
        ),
        (
            lines::Error::NoMove { fen: "fen".into() }.to_string(),
            "the proof has no move for fen",
        ),
        (
            lines::Error::Illegal {
                token: "e2e4".into(),
            }
            .to_string(),
            "e2e4 is not a legal move here",
        ),
        (
            lines::Error::NotProof {
                played: "Nf3".into(),
                token: "e4".into(),
            }
            .to_string(),
            "the proof plays Nf3 here, not e4",
        ),
        (lines::Error::Over.to_string(), "the game is over here"),
        (
            lines::Error::NoWin.to_string(),
            "the proof ends without a win",
        ),
        (
            search::Error::RootOver.to_string(),
            "the root has ended without the attacker winning",
        ),
        (
            search::Error::NoMove { fen: "fen".into() }.to_string(),
            "no winning move recorded for fen",
        ),
        (
            check::Error::Options("options".into()).to_string(),
            "options",
        ),
        (check::Error::Thread("thread".into()).to_string(), "thread"),
        (
            check::Error::Incomplete.to_string(),
            "a proof was not checked",
        ),
        (Rejection::Invalid("invalid".into()).to_string(), "invalid"),
        (
            Rejection::Unchecked("unchecked".into()).to_string(),
            "unchecked",
        ),
    ];
    for (shown, want) in cases {
        assert_eq!(shown, want);
    }
    let read = format::Error::Read {
        path: PathBuf::from("a.pvr"),
        source: io::Error::new(io::ErrorKind::NotFound, "gone"),
    };
    assert_eq!(read.to_string(), "a.pvr: gone");
}

/// A read failure keeps its I/O error as `source`, through the proof error and
/// the app's wrapper.
#[test]
fn an_io_error_keeps_its_source() {
    let error = format::Error::Read {
        path: PathBuf::from("a.pvr"),
        source: io::Error::new(io::ErrorKind::NotFound, "gone"),
    };
    let source = std::error::Error::source(&error).expect("a source");
    assert_eq!(source.to_string(), "gone");
    assert!(std::error::Error::source(source).is_none());
}

/// A malformed proof is false; one needing tablebases or too many moves is
/// null; the keys and their order are the ones a driver reads.
#[test]
fn the_checker_keeps_its_verdict_split() {
    let malformed = check::check_proof("malformed", b"{}");
    assert_eq!(malformed["valid"], json!(false));
    assert_eq!(malformed["error"], json!("no root"));
    assert_eq!(keys(&malformed), ["error", "proof", "valid"]);

    let tablebases = check::check_proof("tablebases", &with_header(json!({"tablebases": 5})));
    assert_eq!(tablebases["valid"], Value::Null);
    assert_eq!(
        tablebases["error"],
        json!("cannot check: needs the tablebases up to 5 pieces")
    );
    assert_eq!(keys(&tablebases), ["error", "proof", "valid"]);

    let too_many = check::check_proof("too-many", &with_header(json!({"positions": 20_000_001})));
    assert_eq!(too_many["valid"], Value::Null);
    assert_eq!(
        too_many["error"],
        json!("cannot check: more than 20000000 moves written")
    );
    assert_eq!(keys(&too_many), ["error", "proof", "valid"]);

    let good = fs::read("tests/fixtures/proofs/core.pvr").expect("fixture");
    let valid = check::check_proof("core", &good);
    assert_eq!(valid["valid"], json!(true));
    assert_eq!(
        keys(&valid),
        [
            "highest_counter",
            "moves",
            "proof",
            "valid",
            "won_positions"
        ]
    );
}

/// Out-of-range options are the checker's entry error, not a per-proof line.
#[test]
fn bad_check_options_are_an_entry_error() {
    let path = PathBuf::from("a.pvr");
    let error = check::check_paths(
        std::slice::from_ref(&path),
        CheckOptions {
            workers: 0,
            seconds: None,
            compact_only: false,
        },
    )
    .expect_err("zero workers");
    assert!(matches!(error, check::Error::Options(_)));
    assert_eq!(error.to_string(), "workers is 1..=1024, not 0");
    let error = check::check_paths(
        &[],
        CheckOptions {
            workers: 1,
            seconds: None,
            compact_only: false,
        },
    )
    .expect_err("no proofs");
    assert_eq!(error.to_string(), "no proofs given");
}

/// The rules layer's delegates keep their messages.
#[test]
fn position_delegates_to_the_rules_error() {
    assert_eq!(
        position("8").err().expect("bad FEN").to_string(),
        "bad FEN: FEN too short: expected at least 4 parts, got 1"
    );
    assert_eq!(
        position("8/8/8/8/8/8/8/8 w - - 0 1")
            .err()
            .expect("empty board")
            .to_string(),
        "illegal position: the board is empty"
    );
}

/// No non-test source line names a `String` error or a bare panic-like call;
/// every `expect` is a stated invariant.
#[test]
fn the_source_guard_holds() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk_sources(&root, &mut files);
    assert!(!files.is_empty());
    for path in files {
        let source = fs::read_to_string(&path).expect("source");
        let code = source.split("#[cfg(test)]").next().expect("a prefix");
        assert!(
            !has_string_result(code),
            "{}: a Result with a String error",
            path.display()
        );
        assert_invariant_expects(code, &path);
    }
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .expect("a JSON object")
        .keys()
        .map(String::as_str)
        .collect()
}

/// A compact proof fixture with its header changed.
fn with_header(changed: Value) -> Vec<u8> {
    let bytes = fs::read("tests/fixtures/proofs/core.pvr").expect("fixture");
    let end = bytes
        .iter()
        .position(|&byte| byte == b'\n')
        .expect("header");
    let mut header: Value = serde_json::from_slice(&bytes[..end]).expect("header JSON");
    for (key, value) in changed.as_object().expect("an object") {
        header[key] = value.clone();
    }
    let mut proof = header.to_string().into_bytes();
    proof.push(b'\n');
    proof.extend(&bytes[end + 1..]);
    proof
}

fn walk_sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            walk_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// Whether the code has a `Result<..., String>`: a top-level `String` after
/// the last top-level comma of the `Result`'s arguments.
fn has_string_result(code: &str) -> bool {
    let bytes = code.as_bytes();
    let mut i = 0;
    while let Some(at) = code[i..].find("Result<") {
        let start = i + at + "Result<".len();
        let mut depth = 1;
        let mut commas = Vec::new();
        let mut j = start;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'<' => depth += 1,
                b'>' => depth -= 1,
                b',' if depth == 1 => commas.push(j),
                _ => {}
            }
            j += 1;
        }
        if depth == 0 {
            let from = commas.last().map_or(start, |comma| comma + 1);
            if code[from..j - 1].trim() == "String" {
                return true;
            }
        }
        i = j;
    }
    false
}

/// Every `unwrap(`, `panic!` or `unreachable!` fails; every `expect(` must
/// start with `"invariant: `.
fn assert_invariant_expects(code: &str, path: &Path) {
    for bad in ["unwrap(", "panic!", "unreachable!"] {
        assert!(!code.contains(bad), "{}: a bare {bad}", path.display());
    }
    let mut from = 0;
    while let Some(at) = code[from..].find("expect(") {
        let start = from + at + "expect(".len();
        let tail = code[start..].trim_start();
        assert!(
            tail.starts_with("\"invariant: "),
            "{}: an expect without a stated invariant: {}",
            path.display(),
            &tail[..tail.len().min(60)]
        );
        from = start;
    }
}
