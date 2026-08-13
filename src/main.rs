mod rust_cli;
mod rust_core;

use std::process::ExitCode;

fn main() -> ExitCode {
    match rust_cli::run(std::env::args().skip(1)) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("Run 'mc-loot-finder help' for usage.");
            ExitCode::from(2)
        }
    }
}
