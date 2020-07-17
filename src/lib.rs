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
    let mut monitors: Vec<Box<dyn Monitor>> = vec![
        Box::new(ChunkedStatsMonitor::from_config(&config)),
        Box::new(RollingAlertsMonitor::from_config(&config)),
    ];

    log::debug!("monitors (initial state): {:#?}", monitors);

    let ordered_records =
        SortedRequestIterator::new(rows.map(|row| row.expect("row should be valid")), config);

    loop {
        let record = RequestRecord::read_csv_line(&mut source);
        if record == None {
            break;
        }
        let record = Rc::new(record.unwrap());

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
