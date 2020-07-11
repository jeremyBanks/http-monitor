pub use self::{
  chunked_stats_monitor::ChunkedStatsMonitor, monitor::Monitor,
  rolling_alerts_monitor::RollingAlertsMonitor,
};

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

/// The headers expected in the CSV input data.
pub const CSV_HEADERS: [&str; 7] = [
  "remotehost",
  "rfc931",
  "authuser",
  "date",
  "request",
  "status",
  "bytes",
];

/// HTTP request record from input.
#[derive(Debug, Deserialize, Serialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct RequestRecord {
  /// client host that the request came from
  #[serde(rename = "remotehost")]
  pub remote_host: String,
  ///
  pub rfc931: String,
  #[serde(rename = "authuser")]
  pub auth_user: String,
  /// unix timestamp of request
  pub date: u64,
  /// first line of the http request, with the method and path
  pub request: String,
  /// http status code of response
  pub status: u64,
  /// bytes length of response
  pub bytes: u64,
}

/// A monitor which follows a stream of requests and may produce output in response to events.
trait Monitor: Debug {
  /// Output may be a String or any other type that can be Displayed as a string.
  type Output: Display + Debug = String;

  /// Pushes a new request record into the monitor, returning any new output this produces.
  fn push(&mut self, record: &Rc<RequestRecord>) -> anyhow::Result<Vec<Output>>;

  /// Output for the records that haven't been accounted-for yet.
  ///
  /// Called to ensure that records at the end of a file aren't missed if they fall in
  /// a chunk that hasn't yet been terminated by a subsequent push.
  fn pending(&mut self) -> anyhow::Result<Vec<Output>> {
    // If this doesn't apply for a given monitor, they don't need to implement it.
    Ok(Vec::new())
  }
}
