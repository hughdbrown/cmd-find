//! `cmd-find`: like `which`, but the argument is a glob and every match
//! on `PATH` is printed.

use std::env;
use std::process::ExitCode;

use clap::Parser;
use cmd_find::{CommandGlob, find};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Shell glob matched against executable file names, e.g. 'cargo-*'
    pattern: CommandGlob,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = env::var_os("PATH").unwrap_or_default();
    let found = find(&path, &cli.pattern);
    for p in &found {
        println!("{}", p.display());
    }
    if found.is_empty() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
