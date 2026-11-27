//! `prove FEN`: searches one position for a win and writes its proof.

use std::time::{Duration, Instant};

use serde_json::{Value, json};

use atomic470mi::{dfpn, format, position};

use crate::app::cli::ProveArgs;
use crate::app::commands::{config_value, print_line};
use crate::app::config::{self, ConfigFile};
use crate::app::error::Error;
use crate::app::values::Guide;
use crate::app::{Status, write};

/// The unbounded deadline an absent `--seconds` stands for.
const UNBOUNDED: Duration = Duration::from_secs(60 * 60 * 24 * 365);

pub(crate) fn run(file: &ConfigFile, args: &ProveArgs) -> Result<Status, Error> {
    let config = config::prove(file, args);
    let shown = config_value(&config)?;
    let root = position(&args.fen).map_err(Error::Position)?;
    if let Some(path) = &config.proofs {
        write::check_target(path)?;
    }
    let started = Instant::now();
    let deadline = match config.seconds {
        Some(seconds) => started
            .checked_add(
                Duration::try_from_secs_f64(seconds.0)
                    .map_err(|_| Error::Usage(format!("--seconds is too large: {}", seconds.0)))?,
            )
            .ok_or_else(|| Error::Usage(format!("--seconds is too large: {}", seconds.0)))?,
        None => started + UNBOUNDED,
    };
    let guide = match config.guide {
        Guide::Hand => dfpn::Guide::Hand,
        Guide::None => dfpn::Guide::None,
    };
    let needed = dfpn::proofs_needed(config.seconds.map(|s| s.0), None);
    let (table, proof_mb) = dfpn::split(config.memory.0, needed);
    let mut search =
        dfpn::Dfpn::try_new(&root, guide, deadline, (table, proof_mb)).map_err(Error::Memory)?;
    search.set_seed(config.seed);
    if let Some(evals) = config.evals {
        search.set_eval_limit(evals.0);
    }
    let moves = search.solve(&root);
    let elapsed = started.elapsed().as_millis() as u64;
    let mut written = None;
    if let (Some(_), Some(path)) = (moves, &config.proofs) {
        let (header, bytes) = format::encode(&root, root.turn(), |pos| {
            let key = (pos.hash(), pos.halfmoves());
            search.winning_move(pos, &key)
        })
        .map_err(Error::Proof)?;
        write::atomic(path, &bytes).map_err(|source| Error::Write {
            path: path.clone(),
            source,
        })?;
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
        "guide": config.guide,
        "seed": config.seed,
        "config": shown,
    });
    if let Some(header) = &written {
        result["proof_moves"] = json!(header.positions);
        result["proof_path"] = json!(config.proofs);
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
    if config.json {
        print_line(&result.to_string())?;
    } else {
        print_line(&format!("config: {}", text_config(&result["config"])))?;
        if moves.is_some() {
            print_line(&format!(
                "win in {} attacker moves, {} nodes, {} ms",
                mate.unwrap_or(0),
                search.nodes,
                elapsed
            ))?;
        } else if result.get("refuted").and_then(Value::as_bool) == Some(true) {
            print_line(&format!(
                "no win: refuted after {} nodes, {} ms",
                search.nodes, elapsed
            ))?;
        } else {
            print_line(&format!(
                "no win found after {} nodes, {} ms",
                search.nodes, elapsed
            ))?;
        }
    }
    Ok(Status::Ok)
}

/// `k=v` pairs in the config's key order: null is `none`, strings raw.
fn text_config(config: &Value) -> String {
    let mut pairs = Vec::new();
    if let Some(map) = config.as_object() {
        for (key, value) in map {
            let shown = match value {
                Value::Null => "none".to_owned(),
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            pairs.push(format!("{key}={shown}"));
        }
    }
    pairs.join(" ")
}
