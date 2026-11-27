//! `--config FILE`: every key lands in the effective config, precedence is
//! default < file < flag, file errors are exit 2, and the defaults equal the
//! documented table. Logging and config never move the search.

use std::fs;
use std::process::{Command, Output};

use serde_json::{Value, json};

const CORE: &str = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1";
const M2: &str = "tests/fixtures/proofs/m2.pvr";
const DEFAULTS: &str = "tests/fixtures/config/defaults.toml";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_470MI"))
        .args(args)
        .output()
        .unwrap()
}

fn tmp(name: &str) -> String {
    format!("{}/{name}", env!("CARGO_TARGET_TMPDIR"))
}

fn toml(name: &str, contents: &str) -> String {
    let path = tmp(&format!("config-{name}.toml"));
    fs::write(&path, contents).unwrap();
    path
}

fn last_json(output: &Output) -> Value {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap_or(Value::Null)
}

/// Runs `prove` (bounded), `check`, `line` or `bench` with a config file
/// and returns its effective `config` object.
fn effective(command: &str, file: Option<&str>, extra: &[&str]) -> Value {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let tag = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = tmp(&format!("config-bench-{tag}.out"));
    let corpus = {
        let path = tmp(&format!("config-corpus-{tag}.jsonl"));
        fs::write(
            &path,
            format!(
                "{{\"id\":\"core\",\"fen\":\"{CORE}\"}}\n{{\"id\":\"core2\",\"fen\":\"{CORE}\"}}\n"
            ),
        )
        .unwrap();
        path
    };
    let mut args: Vec<&str> = match command {
        "prove" => vec!["prove", CORE, "--json"],
        "check" => vec!["check", M2],
        "line" => vec!["line", M2],
        "bench" => vec![
            "bench",
            &corpus,
            "--reference",
            "/bin/true",
            "--repeats",
            "1",
        ],
        _ => unreachable!(),
    };
    if command == "bench" && !extra.contains(&"--out") {
        args.extend(["--out", &out]);
    }
    if command == "prove" && !extra.contains(&"--evals") && !extra.contains(&"--seconds") {
        args.extend(["--evals", "1000"]);
    }
    args.extend(extra);
    if let Some(file) = file {
        args.extend(["--config", file]);
    }
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_470MI"));
    cmd.args(&args).env_remove("PROVARI");
    let output = cmd.output().unwrap();
    if command == "bench" {
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let first = fs::read_to_string(&out)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_owned();
        return serde_json::from_str::<Value>(&first).unwrap()["config"].clone();
    }
    assert_eq!(
        output.status.code(),
        Some(0),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    last_json(&output)["config"].clone()
}

#[test]
fn every_toml_key_lands_in_config() {
    let cases: &[(&str, &str, &str, Value)] = &[
        ("prove", "seconds", "seconds = 5.0", json!(5.0)),
        ("prove", "memory", "memory = 64", json!(64)),
        ("prove", "guide", "guide = \"none\"", json!("none")),
        (
            "prove",
            "proofs",
            &format!("proofs = \"{}\"", tmp("k.pvr")),
            json!(tmp("k.pvr")),
        ),
        ("prove", "json", "json = true", json!(true)),
        ("prove", "seed", "seed = 7", json!(7)),
        ("prove", "evals", "evals = 500", json!(500)),
        ("check", "workers", "workers = 2", json!(2)),
        ("check", "seconds", "seconds = 5.0", json!(5.0)),
        ("check", "compact_only", "compact_only = true", json!(true)),
        (
            "line",
            "pgn",
            &format!("pgn = \"{}\"", tmp("k.pgn")),
            json!(tmp("k.pgn")),
        ),
        ("line", "until", "until = 2", json!(2)),
        ("bench", "seconds", "seconds = 5.0", json!(5.0)),
        ("bench", "memory", "memory = 64", json!(64)),
        ("bench", "guide", "guide = \"model\"", json!("model")),
        ("bench", "repeats", "repeats = 1", json!(1)),
        (
            "bench",
            "reference",
            "reference = \"/bin/true\"",
            json!("/bin/true"),
        ),
        (
            "bench",
            "out",
            &format!("out = \"{}\"", tmp("k.out")),
            json!(tmp("k.out")),
        ),
    ];
    for (command, key, line, want) in cases {
        let file = toml(
            &format!("{command}-{key}"),
            &format!("[{command}]\n{line}\n"),
        );
        let mut extra: Vec<&str> = vec![];
        if *command == "prove" {
            // Bound the search by flag unless the file's `evals` does it.
            extra = if *key == "evals" {
                vec!["--seconds", "600"]
            } else {
                vec!["--evals", "1000"]
            };
        }
        if *command == "bench" && *key == "out" {
            // The file gives `out` itself; the flag stays absent.
            let corpus = tmp(&format!("key-out-{key}.jsonl"));
            fs::write(
                &corpus,
                format!(
                    "{{\"id\":\"c\",\"fen\":\"{CORE}\"}}\n{{\"id\":\"c2\",\"fen\":\"{CORE}\"}}\n"
                ),
            )
            .unwrap();
            let output = run(&[
                "bench",
                &corpus,
                "--reference",
                "/bin/true",
                "--repeats",
                "1",
                "--config",
                &file,
            ]);
            assert_eq!(output.status.code(), Some(0));
            let first = fs::read_to_string(tmp("k.out")).unwrap();
            let config =
                serde_json::from_str::<Value>(first.lines().next().unwrap()).unwrap()["config"]
                    .clone();
            assert_eq!(&config[key], want, "{line}");
            continue;
        }
        let config = effective(command, Some(&file), &extra);
        assert_eq!(&config[key], want, "{line}");
    }
}

#[test]
fn bad_toml_values_are_usage_errors() {
    let cases: &[(&str, &str, &str)] = &[
        ("prove", "seconds", "seconds = \"x\""),
        ("prove", "seconds", "seconds = -1.0"),
        ("prove", "memory", "memory = -1"),
        ("prove", "memory", "memory = 0"),
        ("prove", "guide", "guide = \"model\""),
        ("prove", "json", "json = \"yes\""),
        ("prove", "seed", "seed = -1"),
        ("prove", "evals", "evals = 0"),
        ("check", "workers", "workers = 0"),
        ("check", "seconds", "seconds = 0"),
        ("check", "compact_only", "compact_only = \"yes\""),
        ("line", "until", "until = 0"),
        ("bench", "seconds", "seconds = -1"),
        ("bench", "memory", "memory = 0"),
        ("bench", "repeats", "repeats = 0"),
        ("prove", "proofs", "proofs = 5"),
        ("line", "pgn", "pgn = 5"),
        ("bench", "out", "out = 5"),
        ("bench", "reference", "reference = 5"),
        ("bench", "guide", "guide = 1"),
    ];
    for (command, key, line) in cases {
        let file = toml(
            &format!("bad-{command}-{key}"),
            &format!("[{command}]\n{line}\n"),
        );
        let args: Vec<&str> = match *command {
            "prove" => vec!["prove", CORE, "--evals", "1000", "--config", &file],
            "check" => vec!["check", M2, "--config", &file],
            "line" => vec!["line", M2, "--config", &file],
            _ => vec!["bench", "bench/corpus.json", "--config", &file],
        };
        let output = run(&args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{line}: {stderr}");
        assert!(stderr.contains(key), "{line}: {stderr}");
        assert!(!stderr.contains("panicked"), "{line}: {stderr}");
    }
}

#[test]
fn flag_beats_file_beats_default() {
    let file64 = toml("prove-64", "[prove]\nmemory = 64\n");
    let config = effective("prove", Some(&file64), &["--memory", "128"]);
    assert_eq!(config["memory"], json!(128));
    let config = effective("prove", Some(&file64), &[]);
    assert_eq!(config["memory"], json!(64));
    let config = effective("prove", None, &[]);
    assert_eq!(config["memory"], json!(192));

    let file64 = toml("check-64", "[check]\nseconds = 64\n");
    let config = effective("check", Some(&file64), &["--seconds", "128"]);
    assert_eq!(config["seconds"], json!(128.0));
    let config = effective("check", Some(&file64), &[]);
    assert_eq!(config["seconds"], json!(64.0));
    let config = effective("check", None, &[]);
    assert_eq!(config["seconds"], Value::Null);

    let file64 = toml("line-64", "[line]\nuntil = 64\n");
    let config = effective("line", Some(&file64), &["--until", "128"]);
    assert_eq!(config["until"], json!(128));
    let config = effective("line", Some(&file64), &[]);
    assert_eq!(config["until"], json!(64));
    let config = effective("line", None, &[]);
    assert_eq!(config["until"], Value::Null);

    let file64 = toml("bench-64", "[bench]\nseconds = 64\n");
    let config = effective("bench", Some(&file64), &["--seconds", "128"]);
    assert_eq!(config["seconds"], json!(128.0));
    let config = effective("bench", Some(&file64), &[]);
    assert_eq!(config["seconds"], json!(64.0));
    let config = effective("bench", None, &[]);
    assert_eq!(config["seconds"], json!(10.0));
}

#[test]
fn config_file_errors_are_usage_errors() {
    let unknown_key = toml("unknown-key", "[prove]\nmemori = 64\n");
    let unknown_table = toml("unknown-table", "[prvoe]\nmemory = 64\n");
    let bad_syntax = toml("bad-syntax", "[prove\nmemory = 64\n");
    for file in [unknown_key, unknown_table, bad_syntax] {
        let output = run(&["prove", CORE, "--evals", "1000", "--config", &file]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{file}: {stderr}");
    }
    let missing = run(&[
        "prove",
        CORE,
        "--evals",
        "1000",
        "--config",
        "/nonexistent.toml",
    ]);
    assert_eq!(missing.status.code(), Some(2));
    // A bad-syntax error names the file's line.
    let file = toml("bad-syntax2", "[prove]\nmemory = = 64\n");
    let output = run(&["prove", CORE, "--evals", "1000", "--config", &file]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("line"), "{stderr}");
}

/// A directory holding a fake `provari` that exits 0 and prints nothing, so
/// the `reference` default can be observed through PATH.
fn fake_provari_dir() -> String {
    let dir = tmp("fake-bin");
    fs::create_dir_all(&dir).unwrap();
    let path = format!("{dir}/provari");
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir
}

#[test]
fn prove_defaults_with_no_flags_at_all() {
    // A mate in 1 finishes at once in text mode; the `config:` line is the
    // defaults table, spelled out.
    const MATE1: &str = "6rk/8/8/8/8/8/8/6QK w - - 0 1";
    const WANT: &str =
        "config: evals=none guide=hand json=false memory=192 proofs=none seconds=none seed=0";
    for file in [None, Some(DEFAULTS)] {
        let mut args = vec!["prove", MATE1];
        if let Some(file) = file {
            args.extend(["--config", file]);
        }
        let output = run(&args);
        assert_eq!(output.status.code(), Some(0), "{file:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first = stdout.lines().next().unwrap_or("");
        assert_eq!(first, WANT, "{file:?}");
    }
}

#[test]
fn defaults_match_the_table() {
    // `prove` needs `--evals --json` to finish and to print; both are flags,
    // so only the remaining keys are pinned to the defaults table.
    for file in [None, Some(DEFAULTS)] {
        let prove = effective("prove", file, &[]);
        for (key, want) in [
            ("seconds", Value::Null),
            ("memory", json!(192)),
            ("guide", json!("hand")),
            ("proofs", Value::Null),
            ("seed", json!(0)),
        ] {
            assert_eq!(prove[key], want, "prove {key} {file:?}");
        }

        let check = effective("check", file, &[]);
        assert_eq!(
            check,
            json!({"workers": 1, "seconds": null, "compact_only": false}),
            "{file:?}"
        );
        let line = effective("line", file, &[]);
        assert_eq!(line, json!({"pgn": null, "until": null}), "{file:?}");

        // Bench with no flags beyond --out: `provari` comes from PATH.
        let out = tmp(&format!("defaults-{}.out", file.is_some() as usize));
        let corpus = tmp(&format!("defaults-{}.jsonl", file.is_some() as usize));
        fs::write(
            &corpus,
            format!("{{\"id\":\"c\",\"fen\":\"{CORE}\"}}\n{{\"id\":\"c2\",\"fen\":\"{CORE}\"}}\n"),
        )
        .unwrap();
        let bin = fake_provari_dir();
        let path = format!("{bin}:{}", std::env::var("PATH").unwrap());
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_470MI"));
        cmd.args(["bench", &corpus, "--out", &out])
            .env_remove("PROVARI")
            .env("PATH", path);
        if let Some(file) = file {
            cmd.args(["--config", file]);
        }
        let output = cmd.output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let first = fs::read_to_string(&out).unwrap();
        let bench =
            serde_json::from_str::<Value>(first.lines().next().unwrap()).unwrap()["config"].clone();
        assert_eq!(
            bench,
            json!({
                "out": out, "reference": "provari", "seconds": 10.0,
                "memory": 192, "guide": "hand", "repeats": 3,
            }),
            "{file:?}"
        );
    }
}

#[test]
fn config_never_moves_the_search() {
    let fields = [
        "nodes",
        "child_evals",
        "expansions",
        "root",
        "found",
        "mate",
    ];
    let mut runs: Vec<Value> = Vec::new();
    for extra in [
        vec![],
        vec!["--config", DEFAULTS],
        vec!["--memory", "192", "--guide", "hand", "--seed", "0"],
    ] {
        let mut args = vec!["prove", CORE, "--evals", "200000", "--json"];
        args.extend(extra);
        let output = run(&args);
        assert_eq!(output.status.code(), Some(0));
        runs.push(last_json(&output));
    }
    for field in fields {
        assert_eq!(
            runs[0][field], runs[1][field],
            "{field}: file moved the search"
        );
        assert_eq!(
            runs[0][field], runs[2][field],
            "{field}: flags moved the search"
        );
    }
}
