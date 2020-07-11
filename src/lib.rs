//! Package entry point for dd-monitor.

use std::io::{stdin, stdout};

use anyhow::Result;
use atty;

mod monitor;

pub use monitor::{monitor, MonitorConfig};

/// Runs a monitor from stdin to stdout with the default configuration.
pub fn main() -> Result<()> {
    if atty::is(atty::Stream::Stdin) {
        eprintln!("ERROR: stdin must be a stream or file, not a terminal.");
        eprintln!();
        eprintln!("example usage:");
        eprintln!("    cargo run < sample_input.csv");
        eprintln!();
        eprintln!("or with a release binary:");
        eprintln!("    cargo build --release");
        eprintln!("    target/release/dd-monitor < sample_input.csv");

        std::process::exit(1)
    }

    let config = MonitorConfig::default();

    monitor(&mut stdin(), &mut stdout(), &config)
}
