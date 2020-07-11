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

use anyhow::{anyhow, Context};
use atty;
use csv;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use thiserror;

use crate::{Config, Monitor, RequestRecord};

#[derive(Debug, Default, Clone)]
pub struct ChunkedStatsMonitor {
    /// The number of seconds of requests to include in each stats chunk.
    chunk_seconds: u64,

    /// Requests that are in the chunk currently being aggregated.
    requests: Vec<Rc<RequestRecord>>,

    /// Request counts for the current chunk.
    request_count: u64,
    requests_by_status_code: HashMap<u16, u64>,
    requests_by_section: HashMap<String, u64>,
}

impl ChunkedStatsMonitor {
    fn maybe_flush_before(&mut self, record: &RequestRecord) -> anyhow::Result<()> {
        self.requests.clear();

        Ok(())
    }
}

impl Monitor for ChunkedStatsMonitor {
    fn from_config(config: &Config) -> Self {
        Self {
            chunk_seconds: config.stats_window,
            ..Self::default()
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
}
