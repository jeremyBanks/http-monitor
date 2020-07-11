use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    ops::Range,
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

use crate::{Config, Monitor, RequestRecord};

#[derive(Debug, Clone)]
pub struct ChunkedStatsMonitor {
    /// The number of seconds of requests to include in each stats chunk.
    chunk_seconds: u32,

    /// Requests that are in the chunk currently being aggregated.
    requests: Vec<Rc<RequestRecord>>,
    /// The range of timestamps included in the pending chunk.
    /// Will be None if we haven't had any chunks yet, so we
    /// don't have any timestamp.
    chunk_timestamps: Option<Range<u32>>,

    /// Request counts for the current chunk.
    request_count: u64,
    requests_by_status_code: HashMap<u16, u64>,
    requests_by_section: HashMap<String, u64>,
}

impl ChunkedStatsMonitor {
    fn maybe_flush_before(&mut self, record: &RequestRecord) -> anyhow::Result<Vec<String>> {
        // If this is the first record we're seeing, use it for the starting time
        // of the first chunk.
        let mut chunk_timestamps = self
            .chunk_timestamps
            .clone()
            .unwrap_or_else(|| record.date..(record.date + self.chunk_seconds));

        ensure!(
            record.date >= chunk_timestamps.start,
            "records must be in chronological order but the next record, {:?}, occurs before the start of the current range, {:?}",
            &record,
            &chunk_timestamps
        );

        let mut output = Vec::new();
        while !chunk_timestamps.contains(&record.date) {
            output.append(&mut self.pending()?);
            self.requests.clear();

            chunk_timestamps = chunk_timestamps.end..(chunk_timestamps.end + self.chunk_seconds);
        }

        self.chunk_timestamps = Some(chunk_timestamps);

        Ok(output)
    }
}

impl Monitor for ChunkedStatsMonitor {
    fn from_config(config: &Config) -> Self {
        Self {
            chunk_seconds: config.stats_window,
            requests: Vec::new(),
            chunk_timestamps: None,
            request_count: 0,
            requests_by_status_code: HashMap::new(),
            requests_by_section: HashMap::new(),
        }
    }

    fn push(&mut self, record: &std::rc::Rc<RequestRecord>) -> anyhow::Result<Vec<String>> {
        self.maybe_flush_before(record)?;

        self.requests.push(record.clone());
        self.request_count += 1;
        self.requests_by_status_code
            .entry(record.status)
            .and_modify(|n| *n += 1)
            .or_insert(0);

        Ok(Vec::new())
    }

    fn pending(&mut self) -> anyhow::Result<Vec<String>> {
        Ok(vec![format!("{:#?}", self)])
    }
}
