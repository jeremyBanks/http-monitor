use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    convert::TryInto,
    fmt::{Debug, Display},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    ops::Range,
    rc::{Rc, Weak},
    str,
    sync::Arc,
};

use anyhow::{ensure, Context};
use atty;
use chrono::NaiveDateTime;
use csv;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
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
    requests_time_range: Option<Range<u32>>,

    /// Request counts for the current chunk.
    request_count: u64,
    requests_by_status_code: HashMap<u16, u64>,
    requests_by_section: HashMap<String, u64>,
}

impl ChunkedStatsMonitor {
    fn maybe_flush_before(&mut self, record: &RequestRecord) -> anyhow::Result<Vec<String>> {
        // If this is the first record we're seeing, use it for the starting time
        // of the first chunk.
        let mut requests_time_range = self
            .requests_time_range
            .clone()
            .unwrap_or_else(|| record.date..(record.date + self.chunk_seconds));

        let mut output = Vec::new();
        while !requests_time_range.contains(&record.date) {
            output.append(&mut self.pending()?);

            if !self.requests.is_empty() {
                self.requests.clear();
                self.request_count = 0;
                self.requests_by_status_code.clear();
                self.requests_by_section.clear();
            }

            requests_time_range =
                requests_time_range.end..(requests_time_range.end + self.chunk_seconds);
        }

        self.requests_time_range = Some(requests_time_range);

        Ok(output)
    }
}

impl Monitor for ChunkedStatsMonitor {
    fn from_config(config: &Config) -> Self {
        Self {
            chunk_seconds: config.stats_window,
            requests: Vec::new(),
            requests_time_range: None,
            request_count: 0,
            requests_by_status_code: HashMap::new(),
            requests_by_section: HashMap::new(),
        }
    }

    fn push(&mut self, record: &std::rc::Rc<RequestRecord>) -> anyhow::Result<Vec<String>> {
        let output = self.maybe_flush_before(record)?;

        self.requests.push(record.clone());
        self.request_count += 1;
        self.requests_by_status_code
            .entry(record.status)
            .and_modify(|n| *n += 1)
            .or_insert(0);

        Ok(output)
    }

    fn pending(&mut self) -> anyhow::Result<Vec<String>> {
        let range = self.requests_time_range.as_ref().unwrap();

        let start = NaiveDateTime::from_timestamp(range.start.try_into().unwrap(), 0);
        let end = NaiveDateTime::from_timestamp(range.end.try_into().unwrap(), 0);

        Ok(vec![format!(
            "[{} to {}] {} requests (TODO /second), most popular section TODO with TODO% of traffic.",
            start, end, self.request_count
        )])
    }
}
