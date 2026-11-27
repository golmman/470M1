//! The `--config` file and the resolved per-command configuration:
//! precedence is default < file < flag, and the whole file is validated
//! whatever the command.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::cli::{BenchArgs, CheckArgs, LineArgs, ProveArgs};
use crate::app::error::Error;
use crate::app::values::{Evals, Guide, Megabytes, Repeats, Seconds, Until, Workers};

/// A `--config` file: one optional table per command, keys as the flags
/// with `_` for `-`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigFile {
    prove: Option<ProveTable>,
    check: Option<CheckTable>,
    line: Option<LineTable>,
    bench: Option<BenchTable>,
}

impl ConfigFile {
    /// Reads and validates the whole file, whatever command is running.
    pub(crate) fn load(path: &Path) -> Result<ConfigFile, Error> {
        let text = fs::read_to_string(path).map_err(|error| Error::Config {
            path: path.to_owned(),
            reason: error.to_string(),
        })?;
        toml::from_str(&text).map_err(|error| Error::Config {
            path: path.to_owned(),
            reason: error.to_string(),
        })
    }

    /// The empty file: no tables, so flags and defaults decide.
    fn empty() -> ConfigFile {
        ConfigFile {
            prove: None,
            check: None,
            line: None,
            bench: None,
        }
    }
}

/// The file or the empty file.
pub(crate) fn load(path: Option<&Path>) -> Result<ConfigFile, Error> {
    match path {
        Some(path) => ConfigFile::load(path),
        None => Ok(ConfigFile::empty()),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProveTable {
    seconds: Option<Seconds>,
    memory: Option<Megabytes>,
    guide: Option<Guide>,
    proofs: Option<PathBuf>,
    json: Option<bool>,
    seed: Option<u64>,
    evals: Option<Evals>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckTable {
    workers: Option<Workers>,
    seconds: Option<Seconds>,
    compact_only: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineTable {
    pgn: Option<PathBuf>,
    until: Option<Until>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchTable {
    out: Option<PathBuf>,
    reference: Option<PathBuf>,
    seconds: Option<Seconds>,
    memory: Option<Megabytes>,
    guide: Option<String>,
    repeats: Option<Repeats>,
}

/// `prove`'s effective configuration; the Default impl holds today's
/// defaults.
#[derive(Serialize)]
pub(crate) struct ProveConfig {
    pub(crate) seconds: Option<Seconds>,
    pub(crate) memory: Megabytes,
    pub(crate) guide: Guide,
    pub(crate) proofs: Option<PathBuf>,
    pub(crate) json: bool,
    pub(crate) seed: u64,
    pub(crate) evals: Option<Evals>,
}

impl Default for ProveConfig {
    fn default() -> Self {
        ProveConfig {
            seconds: None,
            memory: Megabytes(192),
            guide: Guide::Hand,
            proofs: None,
            json: false,
            seed: 0,
            evals: None,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CheckConfig {
    pub(crate) workers: Workers,
    pub(crate) seconds: Option<Seconds>,
    pub(crate) compact_only: bool,
}

impl Default for CheckConfig {
    fn default() -> Self {
        CheckConfig {
            workers: Workers(1),
            seconds: None,
            compact_only: false,
        }
    }
}

#[derive(Default, Serialize)]
pub(crate) struct LineConfig {
    pub(crate) pgn: Option<PathBuf>,
    pub(crate) until: Option<Until>,
}

#[derive(Serialize)]
pub(crate) struct BenchConfig {
    pub(crate) out: Option<PathBuf>,
    pub(crate) reference: PathBuf,
    pub(crate) seconds: Seconds,
    pub(crate) memory: Megabytes,
    pub(crate) guide: String,
    pub(crate) repeats: Repeats,
}

impl Default for BenchConfig {
    fn default() -> Self {
        BenchConfig {
            out: None,
            reference: PathBuf::from(std::env::var("PROVARI").unwrap_or_else(|_| "provari".into())),
            seconds: Seconds(10.0),
            memory: Megabytes(192),
            guide: "hand".into(),
            repeats: Repeats(3),
        }
    }
}

/// flag.or(file).unwrap_or(default), per field; booleans switch on.
pub(crate) fn prove(file: &ConfigFile, args: &ProveArgs) -> ProveConfig {
    let table = file.prove.as_ref();
    let default = ProveConfig::default();
    ProveConfig {
        seconds: args.seconds.or(table.and_then(|t| t.seconds)),
        memory: args
            .memory
            .or(table.and_then(|t| t.memory))
            .unwrap_or(default.memory),
        guide: args
            .guide
            .or(table.and_then(|t| t.guide))
            .unwrap_or(default.guide),
        proofs: args.proofs.clone().or(table.and_then(|t| t.proofs.clone())),
        json: args.json || table.and_then(|t| t.json).unwrap_or(default.json),
        seed: args
            .seed
            .or(table.and_then(|t| t.seed))
            .unwrap_or(default.seed),
        evals: args.evals.or(table.and_then(|t| t.evals)),
    }
}

pub(crate) fn check(file: &ConfigFile, args: &CheckArgs) -> CheckConfig {
    let table = file.check.as_ref();
    let default = CheckConfig::default();
    CheckConfig {
        workers: args
            .workers
            .or(table.and_then(|t| t.workers))
            .unwrap_or(default.workers),
        seconds: args.seconds.or(table.and_then(|t| t.seconds)),
        compact_only: args.compact_only
            || table
                .and_then(|t| t.compact_only)
                .unwrap_or(default.compact_only),
    }
}

pub(crate) fn line(file: &ConfigFile, args: &LineArgs) -> LineConfig {
    let table = file.line.as_ref();
    LineConfig {
        pgn: args.pgn.clone().or(table.and_then(|t| t.pgn.clone())),
        until: args.until.or(table.and_then(|t| t.until)),
    }
}

/// `out` may come from the file; with neither it is a usage error.
pub(crate) fn bench(file: &ConfigFile, args: &BenchArgs) -> Result<BenchConfig, Error> {
    let table = file.bench.as_ref();
    let default = BenchConfig::default();
    let config = BenchConfig {
        out: args.out.clone().or(table.and_then(|t| t.out.clone())),
        reference: args
            .reference
            .clone()
            .or(table.and_then(|t| t.reference.clone()))
            .unwrap_or(default.reference),
        seconds: args
            .seconds
            .or(table.and_then(|t| t.seconds))
            .unwrap_or(default.seconds),
        memory: args
            .memory
            .or(table.and_then(|t| t.memory))
            .unwrap_or(default.memory),
        guide: args
            .guide
            .clone()
            .or(table.and_then(|t| t.guide.clone()))
            .unwrap_or(default.guide),
        repeats: args
            .repeats
            .or(table.and_then(|t| t.repeats))
            .unwrap_or(default.repeats),
    };
    if config.out.is_none() {
        return Err(Error::Usage("bench needs --out FILE".into()));
    }
    Ok(config)
}
