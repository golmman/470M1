//! The CLI's bad flags fail with a usage error, not a panic, and its exit
//! codes match the help text.

use std::process::{Command, Output};

const CORE: &str = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_470m1"))
        .args(args)
        .output()
        .unwrap()
}

fn usage_error(args: &[&str]) {
    let output = run(args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
    assert!(stderr.starts_with("error: "), "{args:?}: {stderr}");
    assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
}

#[test]
fn bad_numeric_flags_are_usage_errors() {
    for seconds in ["NaN", "inf", "0", "-1", "1e300"] {
        usage_error(&["prove", CORE, "--seconds", seconds]);
        usage_error(&[
            "check",
            "tests/fixtures/proofs/core.pvr",
            "--seconds",
            seconds,
        ]);
    }
    for memory in ["0", "-1", "1048577"] {
        usage_error(&["prove", CORE, "--memory", memory]);
        usage_error(&[
            "bench",
            "bench/corpus.json",
            "--out",
            "/tmp/none",
            "--memory",
            memory,
        ]);
    }
    usage_error(&["prove", CORE, "--evals", "0"]);
    usage_error(&["check", "tests/fixtures/proofs/core.pvr", "--workers", "0"]);
    usage_error(&[
        "check",
        "tests/fixtures/proofs/core.pvr",
        "--workers",
        "1025",
    ]);
    usage_error(&["line", "tests/fixtures/proofs/core.pvr", "--until", "0"]);
    usage_error(&[
        "bench",
        "bench/corpus.json",
        "--out",
        "/tmp/none",
        "--repeats",
        "0",
    ]);
    usage_error(&["prove", CORE, "--seed", "-1"]);
}

#[test]
fn missing_arguments_are_usage_errors() {
    let empty = run(&[]);
    assert_eq!(empty.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&empty.stderr).contains("Usage: 470m1"));
    usage_error(&["prove"]);
    usage_error(&["check"]);
    usage_error(&["bench", "bench/corpus.json", "--out", "/tmp/none"]);
}

#[test]
fn exit_codes_match_the_help_text() {
    let version = run(&["--version"]);
    assert_eq!(version.status.code(), Some(0));

    let valid = run(&["check", "tests/fixtures/proofs/core.pvr"]);
    assert_eq!(valid.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&valid.stdout).contains("\"valid\":true"));

    let invalid = run(&["check", "tests/fixtures/controls/core-truncated.pvr"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("\"valid\":false"));
}
