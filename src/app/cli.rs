//! The command line, as clap sees it: flags are `Option`s so the resolver
//! can tell absent from given, and every option validates as it parses.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::app::values::{Evals, Guide, Megabytes, Repeats, Seconds, Until, Workers};

const AFTER_HELP: &str = "\
Exit codes:
  0  success (including \"no win found\")
  1  a checked proof is invalid, or `line` on an invalid proof
  2  usage or configuration error
  3  runtime failure: I/O, memory, a failed sub-process or thread,
     a closed stdout";

#[derive(Parser)]
#[command(
    name = "470MI",
    version,
    about = "Proves forced wins in Atomic chess.",
    arg_required_else_help = true,
    after_help = AFTER_HELP
)]
pub(crate) struct Cli {
    /// a TOML file of per-command settings
    #[arg(long, global = true, value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// searches one position for a win
    Prove(ProveArgs),
    /// measures the reference side of a corpus
    Bench(BenchArgs),
    /// verifies proofs, independently of the search
    Check(CheckArgs),
    /// shows a proof's lines
    Line(LineArgs),
    /// prints the version
    Version,
}

#[derive(Args)]
pub(crate) struct ProveArgs {
    /// the position to search, as a FEN
    pub(crate) fen: String,
    /// a search budget
    #[arg(long, value_name = "S", allow_hyphen_values = true)]
    pub(crate) seconds: Option<Seconds>,
    /// the search's memory label
    #[arg(long, value_name = "MB", allow_hyphen_values = true)]
    pub(crate) memory: Option<Megabytes>,
    /// the move-ordering guide
    #[arg(long, value_name = "GUIDE")]
    pub(crate) guide: Option<Guide>,
    /// where the proof is written
    #[arg(long, value_name = "PATH")]
    pub(crate) proofs: Option<PathBuf>,
    /// print one JSON line
    #[arg(long)]
    pub(crate) json: bool,
    /// perturb tie-breaks deterministically
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub(crate) seed: Option<u64>,
    /// stop after N child evaluations
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub(crate) evals: Option<Evals>,
}

#[derive(Args)]
pub(crate) struct BenchArgs {
    /// a corpus or JSONL file of cells
    pub(crate) file: PathBuf,
    /// where results are written
    #[arg(long, value_name = "FILE")]
    pub(crate) out: Option<PathBuf>,
    /// the provari binary, or $PROVARI, or `provari`
    #[arg(long, value_name = "PATH")]
    pub(crate) reference: Option<PathBuf>,
    /// a reference budget
    #[arg(long, value_name = "S", allow_hyphen_values = true)]
    pub(crate) seconds: Option<Seconds>,
    /// the reference's memory label
    #[arg(long, value_name = "MB", allow_hyphen_values = true)]
    pub(crate) memory: Option<Megabytes>,
    /// the guide passed to the reference
    #[arg(long, value_name = "NAME")]
    pub(crate) guide: Option<String>,
    /// run each cell N times
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub(crate) repeats: Option<Repeats>,
}

#[derive(Args)]
pub(crate) struct CheckArgs {
    /// the proofs to verify
    #[arg(required = true)]
    pub(crate) proofs: Vec<PathBuf>,
    /// checks at a time
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub(crate) workers: Option<Workers>,
    /// a per-proof check budget
    #[arg(long, value_name = "S", allow_hyphen_values = true)]
    pub(crate) seconds: Option<Seconds>,
    /// refuse proofs in JSON lines
    #[arg(long)]
    pub(crate) compact_only: bool,
}

#[derive(Args)]
pub(crate) struct LineArgs {
    /// the checked proof to show
    pub(crate) proof: PathBuf,
    /// the moves already played (SAN or UCI, move numbers allowed)
    pub(crate) moves: Vec<String>,
    /// where the proof's tree is written, as PGN
    #[arg(long, value_name = "FILE")]
    pub(crate) pgn: Option<PathBuf>,
    /// where --pgn stops the tree, in attacker moves
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    pub(crate) until: Option<Until>,
}
