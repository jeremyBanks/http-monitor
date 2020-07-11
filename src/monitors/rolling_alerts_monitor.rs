use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    convert::TryInto,
    fmt::{Debug, Display},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    rc::{Rc, Weak},
    str,
    sync::Arc,
};

use anyhow::{anyhow, Context};
use atty;
use chrono::NaiveDateTime;
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

    /// Whether the alert is currently triggered.
    alert_triggered: bool,

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
        let mut output = Vec::new();

        self.requests.push_back(record.clone());

        let min_time_exclusive = record.date - self.window_seconds;

        while self.requests.front().unwrap().date <= min_time_exclusive {
            self.requests.pop_front();
        }

        let average = self.requests.len() as f64 / self.window_seconds as f64;

        let alert_triggered = average >= self.alert_rate as f64;

        let date = NaiveDateTime::from_timestamp(record.date.try_into().unwrap(), 0);

        if alert_triggered != self.alert_triggered {
            self.alert_triggered = alert_triggered;
            if self.alert_triggered {
                output.push(format!("{} ALERT-----+------> average of {:5.1}rps over last {:3} seconds exceeds threshold of  {:5.1}rps <-------ALERT", date, average, self.window_seconds, self.alert_rate as f64));
            } else {
                output.push(format!("{} RECOVERY--+------> average of {:5.1}rps over last {:3} seconds is below threshold of {:5.1}rps <----RECOVERY", date, average, self.window_seconds, self.alert_rate as f64));
            }
        }

        Ok(output)
    }
}
