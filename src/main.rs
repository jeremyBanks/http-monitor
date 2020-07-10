use anyhow::{anyhow, Context, Result};
use argh;
use csv;
use serde::Deserialize;
use serde_derive::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{stdin, stdout, Cursor, Read, Write},
    str,
};

#[derive(argh::FromArgs)]
/// Generates alerts to stdout from HTTP logs via stdin.
struct Args {}

/// HTTP request record from input
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct RequestRecord {
    /// client host that the request came from
    #[serde(rename = "remotehost")]
    remote_host: String,
    rfc931: String,
    #[serde(rename = "authuser")]
    auth_user: String,
    /// unix timestamp of request
    date: u64,
    /// first line of the http request, with the method and path
    request: String,
    /// http status code of response
    status: u64,
    /// bytes length of response
    bytes: u64,
}

/// Configuration for a monitor.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
struct MonitorConfig {
    /// number of seconds of log messages to aggregate for stats.
    stats_window: u64,
    // number of seconds of log messages to aggregate for alerts.
    alert_window: u64,
    // average number of requests per second required to trigger an alert.
    alert_threshold: u64,
}

#[derive(Debug)]
struct MonitorState {
    date: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        // The default stats specified in the assignment description.
        Self {
            stats_window: 10,
            alert_window: 120,
            alert_threshold: 10,
        }
    }
}

/// Application entry point.
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

    let config = MonitorConfig::default();

    monitor(&mut stdin(), &mut stdout(), &config)
}

fn monitor(source: &mut impl Read, sink: &mut impl Write, _config: &MonitorConfig) -> Result<()> {
    let mut reader = csv::Reader::from_reader(source);

    let previous: Option<RequestRecord> = None;

    for result in reader.deserialize() {
        let record: RequestRecord = result?;
        writeln!(sink, "{:?}", record)?;
        break;
    }

    Ok(())
}

#[test]
/// Simplest case: validate that we get the correct output given no input.
fn test_monitor_nothing() -> Result<()> {
    let input = "";
    let expected = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    monitor(&mut source, &mut sink, &config).unwrap();

    let actual = sink.into_inner();
    let actual = str::from_utf8(&actual)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_monitor_sample_input() -> Result<()> {
    let input = &include_str!("../sample_input.csv")[..];
    let expected = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    monitor(&mut source, &mut sink, &config)?;

    let actual = sink.into_inner();
    let actual = str::from_utf8(&actual)?;
    assert_eq!(actual, expected);
    Ok(())
}
