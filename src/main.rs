#![allow(unused_imports)]

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    rc::Rc,
    str,
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use atty;
use csv;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use thiserror;

/// HTTP request record from input
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct RequestRecord {
    /// client host that the request came from
    #[serde(rename = "remotehost")]
    pub remote_host: String,
    ///
    pub rfc931: String,
    #[serde(rename = "authuser")]
    pub auth_user: String,
    /// unix timestamp of request
    pub date: u64,
    /// first line of the http request, with the method and path
    pub request: String,
    /// http status code of response
    pub status: u64,
    /// bytes length of response
    pub bytes: u64,
}

/// Configuration for a monitor.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
struct MonitorConfig {
    /// number of seconds of log messages to aggregate for stats.
    pub stats_window: u64,
    // number of seconds of log messages to aggregate for alerts.
    pub alert_window: u64,
    // average number of requests per second required to trigger an alert.
    pub alert_rate: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        // The default stats specified in the assignment description.
        Self {
            stats_window: 10,
            alert_window: 120,
            alert_rate: 10,
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
    let mut reader = csv::ReaderBuilder::new().from_reader(source);

    let previous: Option<RequestRecord> = None;

    for result in reader.deserialize() {
        let record: RequestRecord = result?;
        writeln!(sink, "{:?}", record)?;
    }

    Ok(())
}

#[test]
/// Tests with no input.
fn test_monitor_nothing() -> Result<()> {
    let input = "";
    let expected = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    // XXX: shouldn't this be an error?
    monitor(&mut source, &mut sink, &config)?;

    let actual = sink.into_inner();
    let actual = str::from_utf8(&actual)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
/// Tests with a single request record.
fn test_monitor_one_request() -> Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status"
        "10.0.0.2","-","apache",1549573860,"GET /api/user HTTP/1.0",200"#;
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

/// Test the provided sample input.
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

/// Test that entirely invalid non-csv input produces an error.
#[test]
fn test_monitor_invalid_non_csv_input() -> Result<()> {
    let input = "1 2\n3 4\n5";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    let result = monitor(&mut source, &mut sink, &config);

    assert!(result.is_err());
    Ok(())
}

/// Test that entirely invalid csv input produces an error (last column missing).
#[test]
fn test_monitor_invalid_csv_input() -> Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status"
        "10.0.0.2","-","apache",1549573860,"GET /api/user HTTP/1.0",200
        "10.0.0.4","-","apache",1549573860,"GET /api/user HTTP/1.0",200";"#;

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    let result = monitor(&mut source, &mut sink, &config);

    assert!(result.is_err());
    Ok(())
}

/// Test that entirely invalid csv input produces an error (extra column in second record).
#[test]
fn test_monitor_invalid_csv_input_2() -> Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status","bytes"
        "10.0.0.1","-","apache",1549574332,"GET /api/user HTTP/1.0",200,1234
        "10.0.0.4","-","apache",1549574333,"GET /report HTTP/1.0",200,1136,10101,13513
        "10.0.0.1","-","apache",1549574334,"GET /api/user HTTP/1.0",200,1194
        "10.0.0.4","-","apache",1549574334,"POST /report HTTP/1.0",404,1307"#;

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = MonitorConfig::default();

    let result = monitor(&mut source, &mut sink, &config);

    assert!(result.is_err());
    Ok(())
}
