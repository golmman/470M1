//! The binary's one error enum: what failed and its exit code, mapped here
//! and nowhere else.

use std::collections::TryReserveError;
use std::fmt;
use std::io;
use std::path::PathBuf;

use atomic470mi::board::FenError;
use atomic470mi::{check, format, lines};

/// A command's failure; `exit_code` is the process code it maps to.
#[derive(Debug)]
pub(crate) enum Error {
    /// A usage problem the library did not see (exit 2).
    Usage(String),
    /// The `--config` file is missing, unreadable or invalid (exit 2).
    Config { path: PathBuf, reason: String },
    /// A `prove` FEN did not parse (exit 2).
    Position(FenError),
    /// A bench input could not be parsed or holds a bad FEN (exit 2).
    Corpus { path: PathBuf, reason: String },
    /// An output path is a directory or its parent is missing (exit 2).
    Output { path: PathBuf, reason: String },
    /// The MOVES given to `line` are not in the proof (exit 2).
    Moves {
        played: String,
        source: lines::Error,
    },
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
    Checker(check::Error),
    /// A proof could not be encoded or walked after a win (exit 3).
    Proof(format::Error),
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
            | Error::Moves { .. } => 2,
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
            Error::Position(source) => write!(out, "{source}"),
            Error::Corpus { path, reason } => {
                write!(out, "the corpus {}: {reason}", path.display())
            }
            Error::Output { path, reason } => {
                write!(out, "cannot write {}: {reason}", path.display())
            }
            Error::Moves { played, source } => write!(out, "the moves \"{played}\": {source}"),
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
            Error::Checker(source) => write!(out, "{source}"),
            Error::Proof(source) => write!(out, "{source}"),
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
            Error::Position(source) => Some(source),
            Error::Moves { source, .. } => Some(source),
            Error::Checker(source) => Some(source),
            Error::Proof(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use atomic470mi::{format, lines};
    use std::io;
    use std::path::PathBuf;

    /// A typed proof error is forwarded as text, and its I/O source with it.
    #[test]
    fn forwards_a_proof_error_and_its_source() {
        let error = Error::Proof(format::Error::Read {
            path: PathBuf::from("a.pvr"),
            source: io::Error::new(io::ErrorKind::NotFound, "gone"),
        });
        assert_eq!(error.to_string(), "a.pvr: gone");
        assert_eq!(error.exit_code(), 3);
        let source = std::error::Error::source(&error).expect("the proof error");
        assert_eq!(source.to_string(), "a.pvr: gone");
        let inner = std::error::Error::source(source).expect("the I/O error");
        assert_eq!(inner.to_string(), "gone");
    }

    /// The MOVES prefix stays on top of the line error.
    #[test]
    fn keeps_the_moves_prefix() {
        let error = Error::Moves {
            played: "e2e4".to_owned(),
            source: lines::Error::Over,
        };
        assert_eq!(
            error.to_string(),
            "the moves \"e2e4\": the game is over here"
        );
        assert_eq!(error.exit_code(), 2);
    }
}
