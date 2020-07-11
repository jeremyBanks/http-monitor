//! Library entry point for dd_monitor.
#![warn(missing_docs, missing_debug_implementations)]

use std::{
    io::{BufRead, BufReader, Read, Write},
    rc::Rc,
    str,
};

use anyhow::ensure;

mod models;
mod monitors;
mod sorted_request_iterator;

pub use self::models::{Config, RequestRecord, RequestRecordTuple};
pub use self::monitors::{ChunkedStatsMonitor, Monitor, RollingAlertsMonitor};
pub use self::sorted_request_iterator::SortedRequestIterator;

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
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(source);

    let mut monitors: Vec<Box<dyn Monitor>> = vec![
        Box::new(ChunkedStatsMonitor::from_config(&config)),
        Box::new(RollingAlertsMonitor::from_config(&config)),
    ];

    log::debug!("monitors (initial state): {:#?}", monitors);

    let rows = reader.deserialize::<RequestRecordTuple>();

    let ordered_records = SortedRequestIterator::new(
        rows.map(|row| row.expect("row should be valid").untuple()),
        config,
    );

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
