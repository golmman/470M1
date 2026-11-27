//! The CLI's bad flags fail with a usage error, not a panic, and its exit
//! codes match the help text. Every flag's good value shows in the printed
//! effective `config`, and garbage input never panics.

use std::fs;
use std::process::{Command, Output, Stdio};

const CORE: &str = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1";
const M2: &str = "tests/fixtures/proofs/m2.pvr";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_470MI"))
        .args(args)
        .output()
        .unwrap()
}

fn tmp(name: &str) -> String {
    format!("{}/{name}", env!("CARGO_TARGET_TMPDIR"))
}

fn usage_error(args: &[&str]) {
    let output = run(args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
    assert!(stderr.starts_with("error: "), "{args:?}: {stderr}");
    assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
}

fn json_line(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap_or_else(|| panic!("no JSON line in {stdout}"))
}

fn config_of(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    json_line(&output)["config"].clone()
}

fn bench_corpus(name: &str) -> String {
    let path = tmp(&format!("cli-corpus-{name}.jsonl"));
    fs::write(
        &path,
        format!(
            "{{\"id\":\"core\",\"fen\":\"{CORE}\"}}\n{{\"id\":\"core2\",\"fen\":\"{CORE}\"}}\n"
        ),
    )
    .unwrap();
    path
}

fn bench_out(name: &str) -> String {
    tmp(&format!("cli-bench-{name}.out"))
}

fn bench_config(args: &[&str]) -> serde_json::Value {
    static TAG: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let tag = TAG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = tmp(&format!("cli-bench-config-{tag}.out"));
    let corpus = bench_corpus(&format!("cfg-{tag}"));
    let mut full: Vec<String> = vec!["bench".into(), corpus, "--out".into(), out.clone()];
    full.extend(args.iter().map(|arg| arg.to_string()));
    let strs: Vec<&str> = full.iter().map(String::as_str).collect();
    let output = run(&strs);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{strs:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = fs::read_to_string(&out)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_owned();
    serde_json::from_str::<serde_json::Value>(&first).unwrap()["config"].clone()
}

#[test]
fn prove_flags_land_in_config() {
    let cases: &[(&[&str], &str, serde_json::Value)] = &[
        (&["--seconds", "5"], "seconds", serde_json::json!(5.0)),
        (&["--memory", "64"], "memory", serde_json::json!(64)),
        (&["--guide", "none"], "guide", serde_json::json!("none")),
        (
            &["--proofs", &tmp("cli.pvr")],
            "proofs",
            serde_json::json!(tmp("cli.pvr")),
        ),
        (&["--json"], "json", serde_json::json!(true)),
        (&["--seed", "7"], "seed", serde_json::json!(7)),
        (&["--evals", "1000"], "evals", serde_json::json!(1000)),
    ];
    for (flags, key, want) in cases {
        let mut args = vec!["prove", CORE];
        if !flags.contains(&"--evals") {
            args.extend(["--evals", "1000"]);
        }
        if !flags.contains(&"--json") {
            args.push("--json");
        }
        args.extend(flags.iter());
        let config = config_of(&args);
        assert_eq!(&config[key], want, "{flags:?}");
    }
}

#[test]
fn check_flags_land_in_config() {
    let cases: &[(&[&str], &str, serde_json::Value)] = &[
        (&["--workers", "2"], "workers", serde_json::json!(2)),
        (&["--seconds", "5"], "seconds", serde_json::json!(5.0)),
        (&["--compact-only"], "compact_only", serde_json::json!(true)),
    ];
    for (flags, key, want) in cases {
        let mut args = vec!["check", M2];
        args.extend(flags.iter());
        let config = config_of(&args);
        assert_eq!(&config[key], want, "{flags:?}");
    }
}

#[test]
fn line_flags_land_in_config() {
    let pgn = tmp("cli.pgn");
    let cases: &[(&[&str], &str, serde_json::Value)] = &[
        (&["--pgn", pgn.as_str()], "pgn", serde_json::json!(pgn)),
        (&["--until", "2"], "until", serde_json::json!(2)),
    ];
    for (flags, key, want) in cases {
        let mut args = vec!["line", M2];
        args.extend(flags.iter());
        let config = config_of(&args);
        assert_eq!(&config[key], want, "{flags:?}");
    }
}

#[test]
fn bench_flags_land_in_config() {
    let cases: &[(&[&str], &str, serde_json::Value)] = &[
        (&["--seconds", "5"], "seconds", serde_json::json!(5.0)),
        (&["--memory", "64"], "memory", serde_json::json!(64)),
        (&["--guide", "model"], "guide", serde_json::json!("model")),
        (&["--repeats", "1"], "repeats", serde_json::json!(1)),
        (
            &["--reference", "/bin/true"],
            "reference",
            serde_json::json!("/bin/true"),
        ),
    ];
    for (flags, key, want) in cases {
        let mut args: Vec<&str> = vec![];
        if !flags.contains(&"--reference") {
            args.extend(["--reference", "/bin/true"]);
        }
        if !flags.contains(&"--repeats") {
            args.extend(["--repeats", "1"]);
        }
        args.extend(flags.iter());
        let config = bench_config(&args);
        assert_eq!(&config[key], want, "{flags:?}");
    }
}

#[test]
fn bad_flag_values_are_usage_errors() {
    let bad: &[(&[&str], &str)] = &[
        (&["prove", CORE, "--seconds", "NaN"], "--seconds"),
        (&["prove", CORE, "--seconds", "0"], "--seconds"),
        (&["prove", CORE, "--seconds", "-1"], "--seconds"),
        (&["prove", CORE, "--seconds", "1e300"], "--seconds"),
        (&["prove", CORE, "--memory", "0"], "--memory"),
        (&["prove", CORE, "--memory", "1048577"], "--memory"),
        (&["prove", CORE, "--guide", "model"], "--guide"),
        (&["prove", CORE, "--seed", "x"], "--seed"),
        (&["prove", CORE, "--evals", "0"], "--evals"),
        (&["check", M2, "--workers", "0"], "--workers"),
        (&["check", M2, "--workers", "1025"], "--workers"),
        (&["check", M2, "--seconds", "NaN"], "--seconds"),
        (&["check", M2, "--tablebase", "tables"], "--tablebase"),
        (&["line", M2, "--until", "0"], "--until"),
        (
            &["bench", "bench/corpus.json", "--repeats", "0"],
            "--repeats",
        ),
        (&["bench", "bench/corpus.json", "--memory", "0"], "--memory"),
        (
            &["bench", "bench/corpus.json", "--seconds", "inf"],
            "--seconds",
        ),
        (&["prove", CORE, "--json=yes"], "--json"),
        (&["check", M2, "--compact-only=yes"], "--compact-only"),
        (&["prove", CORE, "--config"], "--config"),
        (
            &["bench", "bench/corpus.json", "--out", "x", "--guide"],
            "--guide",
        ),
    ];
    for (args, flag) in bad {
        let output = run(args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(stderr.contains(flag), "{args:?}: {stderr}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }

    // An output path that is a directory is refused before any work.
    let dir = env!("CARGO_TARGET_TMPDIR");
    let corpus = bench_corpus("outdir");
    let outdir_cases: &[&[&str]] = &[
        &["prove", CORE, "--proofs", dir],
        &["line", M2, "--pgn", dir],
        &["bench", &corpus, "--out", dir, "--reference", "/bin/true"],
    ];
    for args in outdir_cases {
        let output = run(args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }
}

#[test]
fn missing_arguments_are_usage_errors() {
    let empty = run(&[]);
    assert_eq!(empty.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&empty.stderr).contains("Usage: 470MI"));
    usage_error(&["prove"]);
    usage_error(&["check"]);
    usage_error(&["bench"]);
}

#[test]
fn help_and_version() {
    let help = run(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--config"), "{text}");
    assert!(text.contains("3"), "{text}");

    for flag in ["--version", "version"] {
        let version = run(&[flag]);
        assert_eq!(version.status.code(), Some(0));
        assert_eq!(
            String::from_utf8_lossy(&version.stdout).trim(),
            "470MI 0.1.0"
        );
    }
}

#[test]
fn exit_codes_0_to_3() {
    // 0: a search that does not find a win still succeeds.
    let no_win = run(&["prove", CORE, "--evals", "1000", "--json"]);
    assert_eq!(no_win.status.code(), Some(0));

    // 0 and 1: valid and invalid proofs.
    let valid = run(&["check", "tests/fixtures/proofs/core.pvr"]);
    assert_eq!(valid.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&valid.stdout).contains("\"valid\":true"));
    let invalid = run(&["check", "tests/fixtures/controls/core-truncated.pvr"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("\"valid\":false"));

    // 1: `line` on an invalid proof.
    let line = run(&["line", "tests/fixtures/controls/core-truncated.pvr"]);
    assert_eq!(line.status.code(), Some(1));

    // 2: an output path whose parent is missing fails before the search.
    let missing_parent = run(&[
        "prove",
        CORE,
        "--proofs",
        &format!("{}/missing-dir/x.pvr", env!("CARGO_TARGET_TMPDIR")),
    ]);
    assert_eq!(missing_parent.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&missing_parent.stdout);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(!stdout.contains("nodes"), "{stdout}");

    // 3: the bench reference cannot be spawned.
    let corpus = bench_corpus("spawn");
    let bench = run(&[
        "bench",
        &corpus,
        "--out",
        &bench_out("spawn"),
        "--reference",
        "/nonexistent",
    ]);
    assert_eq!(bench.status.code(), Some(3));
}

#[test]
fn a_closed_stdout_is_an_error_not_a_panic() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_470MI"))
        .args(["prove", CORE, "--evals", "200000", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn garbage_never_panics() {
    let mut cases: Vec<Vec<String>> = Vec::new();
    for fen in [
        String::new(),
        "x".into(),
        "8/8/8/8/8/8/8/8 w - - 0 1".into(),
        "x".repeat(10_000),
        format!("{CORE} x").replace(" w ", " x "),
    ] {
        cases.push(vec!["prove".into(), fen, "--evals".into(), "10".into()]);
    }
    let empty = tmp("cli-empty");
    fs::write(&empty, b"").unwrap();
    let random = tmp("cli-random");
    fs::write(&random, vec![0x9fu8; 4096]).unwrap();
    let text = tmp("cli-text");
    fs::write(&text, "not a proof at all\n").unwrap();
    for path in [
        empty.clone(),
        random,
        text,
        "tests/fixtures/controls/core-truncated.pvr".into(),
    ] {
        cases.push(vec!["check".into(), path.clone()]);
        cases.push(vec!["line".into(), path]);
    }
    let tomls = tmp("cli-bad.toml");
    for contents in [
        "\u{0}\u{1}\u{2}\u{3}",
        "[prove]\nseconds = \"x\"\n",
        "[[prove]]\n",
        "[prove]\nmemory = -1\n",
        "seconds = 1\n",
    ] {
        fs::write(&tomls, contents).unwrap();
        cases.push(vec![
            "prove".into(),
            CORE.into(),
            "--config".into(),
            tomls.clone(),
        ]);
    }
    for case in &cases {
        let args: Vec<&str> = case.iter().map(String::as_str).collect();
        let output = run(&args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        assert!((1..=3).contains(&code), "{args:?}: {code} {stderr}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }
}
