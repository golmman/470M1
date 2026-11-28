//! `line PROOF [MOVES]`: shows a checked proof's lines, and its PGN tree.

use std::fs;

use atomic470mi::board::{Color, Position};
use atomic470mi::{check, format, lines};

use crate::app::cli::LineArgs;
use crate::app::commands::{config_value, print_line};
use crate::app::config::{self, ConfigFile};
use crate::app::error::Error;
use crate::app::{Status, write};

pub(crate) fn run(file: &ConfigFile, args: &LineArgs) -> Result<Status, Error> {
    let config = config::line(file, args);
    let shown = config_value(&config)?;
    if let Some(path) = &config.pgn {
        write::check_target(path)?;
    }
    let input = &args.proof;
    let bytes = fs::read(input).map_err(|source| Error::Read {
        path: input.clone(),
        source,
    })?;
    let name = input.display().to_string();
    let checked = check::check_proof(&name, &bytes);
    if checked["valid"] != true {
        return Err(Error::Invalid(format!("the proof is not valid: {checked}")));
    }
    let (root, attacker, moves) = read_moves(&bytes)?;
    let played = args.moves.join(" ");
    let (mut result, text) = lines::show(
        root,
        attacker,
        moves,
        &played,
        config.pgn.is_some(),
        config.until.map(|until| until.0),
    )
    .map_err(|source| Error::Moves {
        played: played.clone(),
        source,
    })?;
    if let (Some(path), Some(text)) = (&config.pgn, text) {
        write::atomic(path, text.as_bytes()).map_err(|source| Error::Write {
            path: path.clone(),
            source,
        })?;
    }
    result["config"] = shown;
    print_line(&result.to_string())?;
    Ok(Status::Ok)
}

/// Reads a compact or JSON-lines proof into its root, attacker and moves.
fn read_moves(bytes: &[u8]) -> Result<(Position, Color, atomic470mi::walk::Moves), Error> {
    if let Some((header, indices)) = format::decode(bytes).map_err(Error::Proof)? {
        return format::strategy(&header, &indices).map_err(Error::Proof);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| Error::Proof(format::Error::Mismatch("neither proof format".to_owned())))?;
    let (_, compact) = format::encode_jsonl(text).map_err(Error::Proof)?;
    let (header, indices) = format::decode(&compact)
        .map_err(Error::Proof)?
        .ok_or_else(|| {
            Error::Proof(format::Error::Mismatch(
                "the JSON-lines proof did not convert".to_owned(),
            ))
        })?;
    format::strategy(&header, &indices).map_err(Error::Proof)
}
