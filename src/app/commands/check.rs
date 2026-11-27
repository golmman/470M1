//! `check PROOF...`: verifies proofs with the independent checker and
//! prints one verdict line each, `config` added.

use atomic470mi::check::{self, CheckOptions};

use crate::app::Status;
use crate::app::cli::CheckArgs;
use crate::app::commands::{config_value, print_line};
use crate::app::config::{self, ConfigFile};
use crate::app::error::Error;

pub(crate) fn run(file: &ConfigFile, args: &CheckArgs) -> Result<Status, Error> {
    let config = config::check(file, args);
    let shown = config_value(&config)?;
    let options = CheckOptions {
        workers: config.workers.0,
        seconds: config.seconds.map(|seconds| seconds.0),
        compact_only: config.compact_only,
    };
    let verdicts = check::check_paths(&args.proofs, options).map_err(Error::Checker)?;
    let mut all_valid = true;
    for mut verdict in verdicts {
        verdict["config"] = shown.clone();
        all_valid &= verdict["valid"] == true;
        print_line(&verdict.to_string())?;
    }
    Ok(if all_valid {
        Status::Ok
    } else {
        Status::Invalid
    })
}
