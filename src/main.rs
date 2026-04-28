use std::process::ExitCode;

use gut_cli::run_cli;

fn main() -> ExitCode {
    run_cli(std::env::args())
}
