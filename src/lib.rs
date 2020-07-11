// TODO: trim imports once we're done
#![allow(unused_imports)]
#![warn(missing_docs, missing_debug_implementations)]

//! Library entry point for dd_monitor.
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    rc::{Rc, Weak},
    str,
    sync::Arc,
};

use anyhow::{ensure, Context};
use atty;
use csv;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use thiserror;

mod models;
mod monitors;

pub use crate::models::{Config, RequestRecord};
pub use monitors::{ChunkedStatsMonitor, Monitor, RollingAlertsMonitor};

/// The headers expected in the CSV input data.
const CSV_HEADERS: [&str; 7] = [
    "remotehost",
    "rfc931",
    "authuser",
    "date",
    "request",
    "status",
    "bytes",
];

// TODO: load config from json
impl Default for Config {
    fn default() -> Self {
        // The default config specified in the assignment description.
        Self {
            stats_window: 10,
            alert_window: 120,
            alert_rate: 10,
        }
    }
}

/// Reads CSV request records from source, runs monitors according to config, writing their output to sink.
pub fn monitor_stream(
    source: &mut impl Read,
    sink: &mut impl Write,
    config: &Config,
) -> anyhow::Result<()> {
    let mut reader = csv::Reader::from_reader(source);

    // We need to manually check the headers to cover the edge case that we have a file
    // with headers, but no rows. (Serde will implicitly check the headers when deserializing
    // a row into a struct, but if there are no rows the invalid headers would be ignored.)
    ensure!(
        reader.headers()? == *&CSV_HEADERS[..],
        "expected headers {:?}, but got {:?}",
        CSV_HEADERS,
        reader.headers()?
    );

    let mut monitors: Vec<Box<dyn Monitor>> = vec![
        // TODO: pass config into these as appropriate
        Box::new(ChunkedStatsMonitor::from_config(&config)),
        Box::new(RollingAlertsMonitor::from_config(&config)),
    ];

    for record in reader.deserialize::<RequestRecord>() {
        let record = Rc::new(record?);

        for monitor in monitors.iter_mut() {
            let output = monitor.push(&record)?;
            for line in output {
                writeln!(sink, "{}", &line)?;
            }
        }
    }

    Ok(())
}
