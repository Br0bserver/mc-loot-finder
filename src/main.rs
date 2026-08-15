mod catalog;
mod cli;
mod commands;
mod decoration_seed;
mod loot;
mod output;
mod placement;
mod random;
mod worldgen;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(std::env::args().skip(1)) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("Run 'mc-loot-finder help' for usage.");
            ExitCode::from(2)
        }
    }
}
