//! The binary's one error enum: what failed and its exit code, mapped here
//! and nowhere else.

use std::collections::TryReserveError;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// A command's failure; `exit_code` is the process code it maps to.
#[derive(Debug)]
pub(crate) enum Error {
    /// A usage problem the library did not see (exit 2).
    Usage(String),
    /// The `--config` file is missing, unreadable or invalid (exit 2).
    Config { path: PathBuf, reason: String },
    /// A `prove` FEN did not parse (exit 2).
    Position(String),
    /// A bench input could not be parsed or holds a bad FEN (exit 2).
    Corpus { path: PathBuf, reason: String },
    /// An output path is a directory or its parent is missing (exit 2).
    Output { path: PathBuf, reason: String },
    /// The MOVES given to `line` are not in the proof (exit 2).
    Moves(String),
    /// `line` on a proof not shown valid (exit 1).
    Invalid(String),
    /// An input file could not be read (exit 3).
    Read { path: PathBuf, source: io::Error },
    /// An output file could not be written (exit 3).
    Write { path: PathBuf, source: io::Error },
    /// The search's memory could not be reserved (exit 3).
    Memory(TryReserveError),
    /// The bench reference could not be started (exit 3).
    Reference { program: PathBuf, source: io::Error },
    /// The checker's threads failed, or it refused its options (exit 3).
    Checker(String),
    /// A proof could not be encoded or walked after a win (exit 3).
    Proof(String),
    /// stdout could not be written, e.g. a closed pipe (exit 3).
    Stdout(io::Error),
}

impl Error {
    pub(crate) fn exit_code(&self) -> u8 {
        match self {
            Error::Usage(_)
            | Error::Config { .. }
            | Error::Position(_)
            | Error::Corpus { .. }
            | Error::Output { .. }
            | Error::Moves(_) => 2,
            Error::Invalid(_) => 1,
            Error::Read { .. }
            | Error::Write { .. }
            | Error::Memory(_)
            | Error::Reference { .. }
            | Error::Checker(_)
            | Error::Proof(_)
            | Error::Stdout(_) => 3,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Usage(message) => write!(out, "{message}"),
            Error::Config { path, reason } => {
                write!(out, "the config file {}: {reason}", path.display())
            }
            Error::Position(message) => write!(out, "{message}"),
            Error::Corpus { path, reason } => {
                write!(out, "the corpus {}: {reason}", path.display())
            }
            Error::Output { path, reason } => {
                write!(out, "cannot write {}: {reason}", path.display())
            }
            Error::Moves(message) => write!(out, "{message}"),
            Error::Invalid(message) => write!(out, "{message}"),
            Error::Read { path, source } => {
                write!(out, "cannot read {}: {source}", path.display())
            }
            Error::Write { path, source } => {
                write!(out, "cannot write {}: {source}", path.display())
            }
            Error::Memory(source) => write!(out, "the search's memory: {source}"),
            Error::Reference { program, source } => {
                write!(out, "cannot run {}: {source}", program.display())
            }
            Error::Checker(message) => write!(out, "{message}"),
            Error::Proof(message) => write!(out, "{message}"),
            Error::Stdout(source) => write!(out, "cannot write to stdout: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Read { source, .. }
            | Error::Write { source, .. }
            | Error::Reference { source, .. }
            | Error::Stdout(source) => Some(source),
            Error::Memory(source) => Some(source),
            _ => None,
        }
    }
}
