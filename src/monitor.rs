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

const CSV_HEADERS: [&str; 7] = [
    "remotehost",
    "rfc931",
    "authuser",
    "date",
    "request",
    "status",
    "bytes",
];

/// HTTP request record from input
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RequestRecord {
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
pub struct MonitorConfig {
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

pub fn monitor(
    source: &mut impl Read,
    sink: &mut impl Write,
    _config: &MonitorConfig,
) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new().from_reader(source);

    let _previous: Option<RequestRecord> = None;

    // We need to manually check the headers to cover the edge case that we have a file
    // with headers, but no rows. Serde will implicitly check the headers when deserializing
    // a row into a struct, but if there are no rows the invalid headers would be ignored.
    if reader.headers()? != *&CSV_HEADERS[..] {
        return Err(anyhow!(
            "expected headers {:?}, but got {:?}",
            CSV_HEADERS,
            reader.headers()?
        ));
    }

    for result in reader.deserialize() {
        let record: RequestRecord = result?;
        writeln!(sink, "{:?}", record)?;
    }

    Ok(())
}
