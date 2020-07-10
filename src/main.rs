use anyhow::{anyhow, Context, Result};
use argh;
use csv;
use serde::Deserialize;
use serde_derive::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{Read, Write, stdin, stdout},
};

#[derive(argh::FromArgs)]
/// Generates alerts to stdout from HTTP logs via stdin.
struct Args {}

#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct RequestRecord {
    #[serde(rename = "remotehost")]
    remote_host: String,
    rfc931: String,
    #[serde(rename = "authuser")]
    auth_user: String,
    date: u64,
    request: String,
    status: u64,
    bytes: u64,
    population: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MonitorConfig {

}

fn main() -> Result<()> {
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

    monitor(&mut stdin(), &mut stdout(), &MonitorConfig{})
}

fn monitor(source: &mut impl Read, sink: &mut impl Write, _config: &MonitorConfig) -> Result<()> {
    let mut reader = csv::Reader::from_reader(source);

    let previous: Option<RequestRecord> = None;

    for result in reader.deserialize() {
        let record: RequestRecord = result?;
        writeln!(sink, "{}", record.remote_host)?;
    }

    Ok(())
}
