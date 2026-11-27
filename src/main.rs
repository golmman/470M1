//! `470MI`: Atomic win proofs.
//!
//! The CLI exposes the prover (`prove`), the independent checker (`check`),
//! the proof viewer (`line`) and the reference-side bench. Command names and
//! output shapes follow provari where it matters; flags are validated before
//! any search or file read.
//!
//! Adapted from provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d`
//! (`src/main.rs`): tablebase, NNUE/model, web and UCI commands removed; the
//! search commands are stubs or reference-side only.

mod app;

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = match app::parse() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };
    match app::run(cli) {
        Ok(app::Status::Ok) => ExitCode::from(0),
        Ok(app::Status::Invalid) => ExitCode::from(1),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
