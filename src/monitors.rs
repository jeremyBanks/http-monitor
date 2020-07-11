mod chunked_stats_monitor;
mod monitor;
mod rolling_alerts_monitor;

/// Configuration for a monitor.
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct MonitorConfig {
    /// Number of seconds of log messages to aggregate for batch stats.
    /// This window is cleared every X seconds, each time stats are logged.
    pub stats_window: u64,
    /// Number of seconds of log messages to aggregate for alerts.
    /// This is a rolling window, with records individually dropping off X seconds after they enter.
    pub alert_window: u64,
    /// Average number of requests per second required to trigger an alert.
    pub alert_rate: u64,
}

#[derive(Debug, Default)]
struct ChunkedStatsMonitor {}

impl Monitor for ChunkedStatsMonitor {
    fn push(&mut self, record: &Rc<RequestRecord>) -> anyhow::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
struct RollingAlertsMonitor {}

impl Monitor for RollingAlertsMonitor {
    fn push(&mut self, record: &Rc<RequestRecord>) -> anyhow::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Default)]
struct StatsState {
    /// Total number of requests.
    total_requests: u64,
    /// Number of requests by "section" path name.
    requests_by_section: HashMap<String, u64>,
    /// Number of requests by HTTP status code of our response.
    requests_by_status: HashMap<u64, u64>,
}

#[derive(Debug, Clone, Default)]
struct AlertState {
    /// Total number of requests.
    total_requests: u64,
    /// The request at which the alert was triggered, iff an alert is currently triggered.
    triggered_at: Option<Rc<RequestRecord>>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        // The default config specified in the assignment description.
        Self {
            stats_window: 10,
            alert_window: 120,
            alert_rate: 10,
        }
    }
}

pub fn monitor_stream(
    source: &mut impl Read,
    sink: &mut impl Write,
    _config: &MonitorConfig,
) -> anyhow::Result<()> {
    let mut reader = csv::Reader::from_reader(source);

    // We need to manually check the headers to cover the edge case that we have a file
    // with headers, but no rows. Serde will implicitly check the headers when deserializing
    // a row into a struct, but if there are no rows the invalid headers would be ignored.
    if reader.headers()? != *&CSV_HEADERS[..] {
        return Err(anyhow!(
            "expected headers {:?}, but got {:?}",
            CSV_HEADERS,
            reader.headers()?
        ));
    }

    let mut monitors: Vec<Box<dyn Monitor>> = vec![
        Box::new(ChunkedStatsMonitor::default()),
        Box::new(AlertsRollingMonitor::default()),
    ];

    let mut alert_window = VecDeque::new();
    let mut alert_state = AlertState::default();

    let mut stats_window = VecDeque::new();
    let mut stats_events = StatsState::default();

    for record in reader.deserialize::<RequestRecord>() {
        let record = Rc::new(record?);

        for monitor in monitors {
            monitor.push(&record)?
        }
    }

    Ok(())
}
