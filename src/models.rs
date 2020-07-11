use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    net::Ipv4Addr,
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

/// HTTP request record from input.
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RequestRecord {
    /// IP address that the request came from.
    #[serde(rename = "remotehost")]
    pub remote_host: Ipv4Addr,
    /// Unused, included for compatibility.
    #[serde(skip)]
    pub rfc931: (),
    /// Unused, included for compatibility.
    #[serde(skip, rename = "authuser")]
    pub auth_user: (),
    /// Unix timestamp of request.
    pub date: u32,
    /// First line of the http request, with the method and path.
    pub request: String,
    /// HTTP status code of response.
    pub status: u16,
    /// Byte length of response.
    pub bytes: u64,
}

/// Configuration for this log monitoring program.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Config {
    /// Number of seconds of log messages to aggregate for batch stats.
    /// This window is cleared every X seconds, each time stats are logged.
    pub stats_window: u32,
    /// Number of seconds of log messages to aggregate for alerts.
    /// This is a rolling window, with records individually dropping off X seconds after they enter.
    pub alert_window: u32,
    /// Average number of requests per second required to trigger an alert.
    pub alert_rate: u32,
    /// The margin of error on a record's timestamp, in seconds.
    pub maximum_timestamp_error: u32,
}
