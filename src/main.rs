use log::error;
use std::process::ExitCode;

use crate::cli::run;

pub(crate) mod cli;
pub(crate) mod commit;
pub(crate) mod error;
pub(crate) mod git;
pub(crate) mod shell;

fn main() -> ExitCode {
    match run() {
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
        Ok(_) => {
            return ExitCode::SUCCESS;
        }
    }
}
