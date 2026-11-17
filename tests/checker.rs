//! Fixture acceptance and rejection, through the public checker only.
//!
//! The compact proofs were written by the pinned provari oracle from the
//! FENs in `tests/fixtures/veto.tsv`; the control is a truncated copy of
//! `core.pvr`. Both checkers must agree.

use std::fs;
use std::path::Path;

use atomic470m1::check::check_proof;

#[test]
fn accepts_every_pinned_proof_fixture() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/proofs");
    let mut seen = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "pvr") {
            let bytes = fs::read(&path).unwrap();
            let line = check_proof(&path.display().to_string(), &bytes);
            assert_eq!(line["valid"], true, "{line}");
            seen += 1;
        }
    }
    assert!(seen >= 5, "only {seen} proof fixtures");
}

#[test]
fn rejects_every_control_fixture() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/controls");
    let mut seen = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "pvr") {
            let bytes = fs::read(&path).unwrap();
            let line = check_proof(&path.display().to_string(), &bytes);
            assert_eq!(line["valid"], false, "{line}");
            seen += 1;
        }
    }
    assert!(seen >= 1, "no controls found");
}
