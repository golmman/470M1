//! The binary's application layer: it owns the CLI, the config file, the
//! error-to-exit-code mapping and every write; the library never sees it.

pub(crate) mod cli;
mod commands;
mod config;
mod error;
mod values;
mod write;

use clap::Parser;

pub(crate) use cli::Cli;
pub(crate) use error::Error;

/// What a command decided: success or "not shown valid" (exit 0 or 1).
pub(crate) enum Status {
    Ok,
    Invalid,
}

/// Parses the process arguments; a clap error is printed by the caller as
/// `e.print()` with `e.exit_code()`.
pub(crate) fn parse() -> Result<Cli, clap::Error> {
    Cli::try_parse()
}

/// Runs the parsed command.
pub(crate) fn run(cli: Cli) -> Result<Status, Error> {
    let file = config::load(cli.config.as_deref())?;
    match cli.command {
        cli::Command::Prove(args) => commands::prove::run(&file, &args),
        cli::Command::Bench(args) => commands::bench::run(&file, &args),
        cli::Command::Check(args) => commands::check::run(&file, &args),
        cli::Command::Line(args) => commands::line::run(&file, &args),
        cli::Command::Version => {
            commands::print_line(&format!("470MI {}", env!("CARGO_PKG_VERSION")))?;
            Ok(Status::Ok)
        }
    }
}
