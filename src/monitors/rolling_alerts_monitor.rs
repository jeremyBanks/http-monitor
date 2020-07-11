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
use thiserror;

use crate::{Config, Monitor, RequestRecord};

#[derive(Debug, Default, Clone)]
pub struct RollingAlertsMonitor {
    /// The number of seconds of requests to include in our rolling window.
    window_seconds: u32,

    /// The average number of requests per second through the window required to trigger an alert.
    alert_rate: u32,

    /// Requests that are in the current alerting window.
    requests: VecDeque<Rc<RequestRecord>>,
}

impl Monitor for RollingAlertsMonitor {
    fn from_config(config: &Config) -> Self {
        Self {
            window_seconds: config.alert_window,
            alert_rate: config.alert_rate,
            ..Self::default()
        }
    }

    fn push(&mut self, record: &std::rc::Rc<RequestRecord>) -> anyhow::Result<Vec<String>> {
        let output = Vec::new();

        self.requests.push_back(record.clone());

        Ok(output)
    }
}
