//! `bench FILE --out FILE`: measures the reference side of a corpus,
//! buffering every row for one atomic write at the end.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use serde_json::{Value, json};

use atomic470mi::position;

use crate::app::cli::BenchArgs;
use crate::app::commands::config_value;
use crate::app::config::{self, ConfigFile};
use crate::app::error::Error;
use crate::app::{Status, write};

pub(crate) fn run(file: &ConfigFile, args: &BenchArgs) -> Result<Status, Error> {
    let config = config::bench(file, args)?;
    let out = config
        .out
        .clone()
        .expect("invariant: config::bench requires out");
    write::check_target(&out)?;
    let cases = load_cases(&args.file)?;
    let mut rows = json!({ "config": config_value(&config)? }).to_string();
    for (id, fen) in cases {
        for run in 1..=config.repeats.0 {
            let started = Instant::now();
            let output = Command::new(&config.reference)
                .args([
                    "prove",
                    &fen,
                    "--seconds",
                    &format!("{}", config.seconds.0),
                    "--memory",
                    &format!("{}", config.memory.0),
                    "--guide",
                    &config.guide,
                ])
                .stdin(Stdio::null())
                .output()
                .map_err(|source| Error::Reference {
                    program: config.reference.clone(),
                    source,
                })?;
            let wall = started.elapsed().as_secs_f64();
            let text = String::from_utf8_lossy(&output.stdout);
            let payload = text
                .lines()
                .rev()
                .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
                .unwrap_or(Value::Null);
            let row = json!({
                "id": id,
                "side": format!("provari-{}", config.guide),
                "run": run,
                "wall_s": (wall * 1000.0).round() / 1000.0,
                "returncode": output.status.code(),
                "win": payload.get("found").and_then(Value::as_bool).unwrap_or(false),
                "mate": payload.get("mate"),
                "nodes": payload.get("nodes"),
                "ms": payload.get("ms"),
            });
            rows.push('\n');
            rows.push_str(&row.to_string());
        }
    }
    rows.push('\n');
    write::atomic(&out, rows.as_bytes()).map_err(|source| Error::Write {
        path: out.clone(),
        source,
    })?;
    Ok(Status::Ok)
}

fn load_cases(path: &Path) -> Result<Vec<(String, String)>, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_owned(),
        source,
    })?;
    let mut cases = Vec::new();
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        if let Some(core) = value.get("core") {
            cases.push((id(core), fen(path, core)?));
        }
        if let Some(sample) = value.get("sample20").and_then(Value::as_array) {
            for cell in sample {
                cases.push((id(cell), fen(path, cell)?));
            }
        }
        return Ok(cases);
    }
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let cell: Value =
            serde_json::from_str(line).map_err(|error| corpus(path, error.to_string()))?;
        cases.push((id(&cell), fen(path, &cell)?));
    }
    Ok(cases)
}

fn corpus(path: &Path, reason: String) -> Error {
    Error::Corpus {
        path: path.to_owned(),
        reason,
    }
}

fn id(value: &Value) -> String {
    value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn fen(path: &Path, value: &Value) -> Result<String, Error> {
    let fen = value
        .get("fen")
        .and_then(Value::as_str)
        .ok_or_else(|| corpus(path, "a case without a FEN".into()))?;
    let _ = position(fen).map_err(|reason| corpus(path, reason.to_string()))?;
    Ok(fen.to_owned())
}
