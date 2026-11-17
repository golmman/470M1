//! `470m1`: Atomic win proofs.
//!
//! Plan 1 implements `check`, `line`, a reference-side `bench`, and a `prove`
//! stub; `prove`'s search arrives in plan 2. The command names and output
//! shapes follow provari where it matters.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! (`src/main.rs`): tablebase, NNUE/model, web and UCI commands removed; the
//! search commands are stubs or reference-side only.

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shakmaty::{Color, Position as _};

use atomic470m1::{check, dfpn, format, lines, position};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print!("{}", help());
        return;
    }
    let (command, rest) = args
        .split_first()
        .map(|(c, r)| (c.as_str(), r))
        .unwrap_or(("", &[]));
    let result = match command {
        "check" => match check::run(rest) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(error) => Err(error),
        },
        "line" => line(rest),
        "bench" => bench(rest),
        "prove" => prove(rest),
        "version" | "--version" => {
            println!("470m1 {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "help" => {
            print!("{}", help());
            Ok(())
        }
        "" => {
            eprint!("{}", help());
            std::process::exit(2);
        }
        _ => Err(format!("unknown command {command}: see 470m1 --help")),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn help() -> String {
    format!(
        "470m1 {}\nProves forced wins in Atomic chess.\n\n{}",
        env!("CARGO_PKG_VERSION"),
        HELP
    )
}

const HELP: &str = "\
Usage: 470m1 COMMAND [OPTIONS]

Commands:
  prove FEN                searches one position for a win
  bench FILE --out FILE    measures the reference side of a corpus
  check PROOF...           verifies proofs, independently of the search
  line PROOF ['MOVES']     shows a proof's lines (--pgn FILE writes its tree)
  version                  prints the version

Options:
  --seconds S              a search or check budget
  --memory MB              a search's memory label
  --guide hand|none        the guide for `prove` and `bench`
  --proofs PATH            where `prove` writes its proof
  --json                   `prove` prints one JSON line
  --seed N                 `prove` perturbs tie-breaks deterministically
  --evals N                `prove` stops after N child evaluations
  --workers N              checks at a time
  --compact-only           `check` refuses JSON-lines proofs
  --pgn FILE               where `line` writes the proof's tree
  --until N                where `line --pgn` stops the tree
  --out FILE               where `bench` writes results
  --reference PATH         the provari binary for `bench`
";

/// Searches one position for a win, and writes its proof when asked.
fn prove(args: &[String]) -> Result<(), String> {
    let mut free = Vec::new();
    let mut seconds = None;
    let mut memory = 192usize;
    let mut guide = "hand".to_owned();
    let mut proofs = None;
    let mut json = false;
    let mut seed = 0u64;
    let mut evals = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--seconds" => seconds = Some(value()?.parse().map_err(|_| "bad --seconds")?),
            "--memory" => memory = value()?.parse().map_err(|_| "bad --memory")?,
            "--guide" => guide = value()?.to_owned(),
            "--proofs" => proofs = Some(value()?.to_owned()),
            "--seed" => seed = value()?.parse().map_err(|_| "bad --seed")?,
            "--evals" => evals = Some(value()?.parse().map_err(|_| "bad --evals")?),
            "--json" => json = true,
            unknown if unknown.starts_with("--") => {
                return Err(format!("unknown option {unknown}: see 470m1 --help"));
            }
            _ => free.push(arg.clone()),
        }
    }
    let fen = free
        .first()
        .ok_or("usage: 470m1 prove FEN [--seconds S] [--memory MB] [--guide hand|none] [--proofs PATH] [--json] [--seed N] [--evals N]")?;
    let guide = match guide.as_str() {
        "hand" => dfpn::Guide::Hand,
        "none" => dfpn::Guide::None,
        other => return Err(format!("--guide is hand or none, not {other}")),
    };
    let root = position(fen)?;
    let started = Instant::now();
    let deadline = match seconds {
        Some(seconds) => started + Duration::from_secs_f64(seconds),
        None => started + Duration::from_secs(60 * 60 * 24 * 365),
    };
    let needed = dfpn::proofs_needed(seconds, None);
    let (table, proof_mb) = dfpn::split(memory, needed);
    let mut search = dfpn::Dfpn::new(&root, guide, deadline, (table, proof_mb));
    search.set_seed(seed);
    if let Some(evals) = evals {
        search.set_eval_limit(evals);
    }
    let moves = search.solve(&root);
    let elapsed = started.elapsed().as_millis() as u64;
    let mut written = None;
    if let (Some(_), Some(path)) = (moves, &proofs) {
        let (header, bytes) = format::encode(&root, root.turn(), |pos| {
            let key = (pos.hash(), pos.halfmoves());
            search
                .winning_move(&pos.to_atomic(), &key)
                .and_then(|m| format::board_move(pos, &m))
        })?;
        format::save(std::path::Path::new(path), &bytes)?;
        written = Some(header);
    }
    let mate = written.as_ref().map_or(moves, |header| Some(header.moves));
    let mut result = json!({
        "found": moves.is_some(),
        "mate": mate,
        "nodes": search.nodes,
        "child_evals": search.evals,
        "expansions": search.expansions,
        "ms": elapsed,
        "guide": match guide {
            dfpn::Guide::Hand => "hand",
            dfpn::Guide::None => "none",
        },
        "seed": seed,
    });
    if let Some(header) = &written {
        result["proof_moves"] = json!(header.positions);
        result["proof_path"] = json!(proofs);
    }
    if let Some((proof, disproof)) = search.root_numbers
        && moves.is_none()
    {
        result["root"] = json!([proof, disproof]);
        if disproof == 0 {
            result["refuted"] = json!(true);
        }
    }
    if search.proofs_full() && moves.is_none() {
        result["error"] = json!("the proofs filled their memory: give more --memory");
    }
    if json {
        println!("{result}");
    } else if moves.is_some() {
        println!(
            "win in {} attacker moves, {} nodes, {} ms",
            mate.unwrap_or(0),
            search.nodes,
            elapsed
        );
    } else if result.get("refuted").and_then(Value::as_bool) == Some(true) {
        println!(
            "no win: refuted after {} nodes, {} ms",
            search.nodes, elapsed
        );
    } else {
        println!("no win found after {} nodes, {} ms", search.nodes, elapsed);
    }
    Ok(())
}

/// Shows a checked proof's lines.
fn line(args: &[String]) -> Result<(), String> {
    let mut free = Vec::new();
    let mut pgn = None;
    let mut until = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--pgn" => pgn = Some(value()?.to_owned()),
            "--until" => until = Some(value()?.parse().map_err(|_| "bad --until")?),
            unknown if unknown.starts_with("--") => {
                return Err(format!("unknown option {unknown}: see 470m1 --help"));
            }
            _ => free.push(arg.clone()),
        }
    }
    let (input, played) = free
        .split_first()
        .ok_or("usage: 470m1 line PROOF [MOVES] [--pgn FILE] [--until N]")?;
    let bytes = fs::read(input).map_err(|error| format!("{input}: {error}"))?;
    let checked = check::check_proof(input, &bytes);
    if checked["valid"] != true {
        return Err(format!("the proof is not valid: {checked}"));
    }
    let (root, attacker, moves) = read_moves(&bytes)?;
    let (result, text) = lines::show(
        root,
        attacker,
        moves,
        &played.join(" "),
        pgn.is_some(),
        until,
    )?;
    if let (Some(path), Some(text)) = (pgn, text) {
        fs::write(&path, text).map_err(|error| format!("{path}: {error}"))?;
    }
    println!("{result}");
    Ok(())
}

/// Reads a compact or JSON-lines proof into its root, attacker and moves.
fn read_moves(
    bytes: &[u8],
) -> Result<(shakmaty::variant::Atomic, Color, atomic470m1::walk::Moves), String> {
    if let Some((header, indices)) = format::decode(bytes)? {
        return format::strategy(&header, &indices);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "neither proof format")?;
    let (_, compact) = format::encode_jsonl(text)?;
    let (header, indices) =
        format::decode(&compact)?.ok_or("the JSON-lines proof did not convert")?;
    format::strategy(&header, &indices)
}

/// Measures the reference side of a corpus or JSONL file.
fn bench(args: &[String]) -> Result<(), String> {
    let mut free = Vec::new();
    let mut out = None;
    let mut reference = None;
    let mut seconds = 10.0;
    let mut memory = 192;
    let mut guide = "hand".to_owned();
    let mut repeats = 3;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--out" => out = Some(value()?.to_owned()),
            "--reference" => reference = Some(value()?.to_owned()),
            "--seconds" => seconds = value()?.parse().map_err(|_| "bad --seconds")?,
            "--memory" => memory = value()?.parse().map_err(|_| "bad --memory")?,
            "--guide" => guide = value()?.to_owned(),
            "--repeats" => repeats = value()?.parse().map_err(|_| "bad --repeats")?,
            unknown if unknown.starts_with("--") => {
                return Err(format!("unknown option {unknown}: see 470m1 --help"));
            }
            _ => free.push(arg.clone()),
        }
    }
    let input = free
        .first()
        .ok_or("usage: 470m1 bench FILE --out FILE --reference PATH")?;
    let out = out.ok_or("bench needs --out FILE")?;
    let reference = reference
        .or_else(|| std::env::var("PROVARI").ok())
        .unwrap_or_else(|| "provari".to_owned());
    let cases = load_cases(input)?;
    let mut file = fs::File::create(&out).map_err(|error| format!("{out}: {error}"))?;
    for (id, fen) in cases {
        for run in 1..=repeats {
            let started = Instant::now();
            let output = Command::new(&reference)
                .args([
                    "prove",
                    &fen,
                    "--seconds",
                    &format!("{seconds}"),
                    "--memory",
                    &format!("{memory}"),
                    "--guide",
                    &guide,
                ])
                .stdin(Stdio::null())
                .output()
                .map_err(|error| format!("cannot run {reference}: {error}"))?;
            let wall = started.elapsed().as_secs_f64();
            let text = String::from_utf8_lossy(&output.stdout);
            let payload = text
                .lines()
                .rev()
                .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
                .unwrap_or(Value::Null);
            let row = json!({
                "id": id,
                "side": format!("provari-{guide}"),
                "run": run,
                "wall_s": (wall * 1000.0).round() / 1000.0,
                "returncode": output.status.code(),
                "win": payload.get("found").and_then(Value::as_bool).unwrap_or(false),
                "mate": payload.get("mate"),
                "nodes": payload.get("nodes"),
                "ms": payload.get("ms"),
            });
            writeln!(file, "{row}").map_err(|error| format!("{out}: {error}"))?;
        }
    }
    Ok(())
}

fn load_cases(path: &str) -> Result<Vec<(String, String)>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let mut cases = Vec::new();
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        if let Some(core) = value.get("core") {
            cases.push((id(core), fen(core)?));
        }
        if let Some(sample) = value.get("sample20").and_then(Value::as_array) {
            for cell in sample {
                cases.push((id(cell), fen(cell)?));
            }
        }
        return Ok(cases);
    }
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let cell: Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
        cases.push((id(&cell), fen(&cell)?));
    }
    Ok(cases)
}

fn id(value: &Value) -> String {
    value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn fen(value: &Value) -> Result<String, String> {
    let fen = value
        .get("fen")
        .and_then(Value::as_str)
        .ok_or("a case without a FEN")?;
    let _ = position(fen)?;
    Ok(fen.to_owned())
}
