//! Library entry point for http_monitor.
#![warn(missing_docs, missing_debug_implementations)]

use std::{
    io::{Read, Write},
    rc::Rc,
    str,
};

use anyhow::ensure;

mod models;
mod monitors;
mod sorted_request_iterator;

pub use self::models::{Config, RequestRecord};
pub use self::monitors::{ChunkedStatsMonitor, Monitor, RollingAlertsMonitor};
pub use self::sorted_request_iterator::SortedRequestIterator;

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
            maximum_timestamp_error: 1,
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

    log::debug!("validated headers");

    let mut monitors: Vec<Box<dyn Monitor>> = vec![
        Box::new(ChunkedStatsMonitor::from_config(&config)),
        Box::new(RollingAlertsMonitor::from_config(&config)),
    ];

    log::debug!("monitors (initial state): {:#?}", monitors);

    let rows = reader.deserialize::<RequestRecord>();

    let ordered_records =
        SortedRequestIterator::new(rows.map(|row| row.expect("row should be valid")), config);

    for record in ordered_records {
        let record = Rc::new(record);

        for monitor in monitors.iter_mut() {
            let output = monitor.push(&record)?;
            for line in output {
                writeln!(sink, "{}", &line)?;
            }
        }
    }

    for monitor in monitors.iter_mut() {
        let output = monitor.pending()?;
        for line in output {
            writeln!(sink, "{}", &line)?;
        }
    }

    log::debug!("monitors (final state): {:#?}", monitors);

    Ok(())
}
