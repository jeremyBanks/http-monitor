mod chunked_stats_monitor;
mod rolling_alerts_monitor;

pub use self::chunked_stats_monitor::ChunkedStatsMonitor;
pub use self::rolling_alerts_monitor::RollingAlertsMonitor;

use crate::Config;

/// A monitor which follows a stream of requests and may produce output in response to events.
pub trait Monitor: std::fmt::Debug {
    /// Creates a new instance of this monitor for a given program configuration.
    fn from_config(config: &Config) -> Self
    where
        // We need to apply this constraint or Rust won't let dynamically dispatch
        // to instances of this trait.
        Self: Sized;

    /// Pushes a new request record into the monitor, returning any new output this produces.
    fn push(
        &mut self,
        record: &std::rc::Rc<crate::models::RequestRecord>,
    ) -> anyhow::Result<Vec<String>>;

    /// Output for the records that haven't been accounted-for yet.
    ///
    /// Called to ensure that records at the end of a stream aren't missed if they fall in
    /// a chunk that hasn't yet been terminated by a subsequent push. May also be used to
    /// provide a ~live view of the current incomplete chunk.
    fn pending(&mut self) -> anyhow::Result<Vec<String>> {
        // If this doesn't apply for a given monitor, they don't need to implement it.
        // For example, RollingAlertsMonitor's output immediately reflects all of the events
        // it's been given so it never has any pending.
        Ok(Vec::new())
    }
}
