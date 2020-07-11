//! Binary entry point for dd-monitor.

use dd_monitor;

/// Runs a monitor from stdin to stdout with the default configuration.
pub fn main() -> anyhow::Result<()> {
    // If stdin is a terminal, the user is probably confused. Bail with a usage message.
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

    let config = dd_monitor::Config::default();

    dd_monitor::monitor_stream(&mut stdin(), &mut stdout(), &config)?;

    Ok(())
}
