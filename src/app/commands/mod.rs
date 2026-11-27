//! One module per subcommand; all of stdout goes through `print_line`.

pub(crate) mod bench;
pub(crate) mod check;
pub(crate) mod line;
pub(crate) mod prove;

use std::io::{self, Write};

use serde_json::Value;

use crate::app::error::Error;

/// One line of stdout, locked and flushed; a closed pipe is a `Stdout`
/// error, never a panic (Rust ignores SIGPIPE).
pub(crate) fn print_line(line: &str) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(line.as_bytes())
        .and_then(|()| stdout.write_all(b"\n"))
        .and_then(|()| stdout.flush())
        .map_err(Error::Stdout)
}

/// A resolved config serialized once, for the `config` key of a result.
pub(crate) fn config_value(config: &impl serde::Serialize) -> Result<Value, Error> {
    serde_json::to_value(config)
        .map_err(|error| Error::Usage(format!("cannot show the config: {error}")))
}
