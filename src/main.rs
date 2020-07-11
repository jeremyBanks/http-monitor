//! Binary entry point for dd-monitor.

#[derive(argh::FromArgs, Debug)]
/// Runs monitors against a CSV stream of HTTP request records from stdin.
struct Args {
    /// enables maximum logging for our code and debug logging for dependencies. overrides
    /// RUST_LOG.
    #[argh(switch, short = 'v')]
    verbose: bool,

    /// silence log output except for errors. overrides --verbose and RUST_LOG.
    #[argh(switch, short = 'q')]
    quiet: bool,

    /// the average number of requests per second required to trigger an alert.
    #[argh(option)]
    alert_rate: Option<u64>,

    /// the number of seconds over which the request count is averaged for alerting.
    #[argh(option)]
    alert_window: Option<u64>,

    /// the number of seconds worth of requests to aggregate for each stats output.
    #[argh(option)]
    stats_window: Option<u64>,
}

pub fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    if args.quiet {
        std::env::set_var("RUST_LOG", "error");
    } else if args.verbose {
        std::env::set_var("RUST_LOG", "debug,dd_monitor=trace");
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::try_init()?;

    let mut config = dd_monitor::Config::default();

    // If stdin is a terminal, the user is probably confused. Bail with instructions.
    if atty::is(atty::Stream::Stdin) {
        log::error!(
            "stdin must be a stream or file, not a terminal.
            
            example usage:
                cargo run < sample_input.csv
            
            or with a release binary:
                cargo build --release
                target/release/dd-monitor < sample_input.cs
            "
        );

        std::process::exit(1)
    }

    if let Some(alert_rate) = args.alert_rate {
        config.alert_rate = alert_rate;
    }

    if let Some(alert_window) = args.alert_window {
        config.alert_window = alert_window;
    }

    if let Some(stats_window) = args.stats_window {
        config.stats_window = stats_window;
    }

    log::debug!("{:#?}", &config);

    dd_monitor::monitor_stream(&mut std::io::stdin(), &mut std::io::stdout(), &config)?;

    log::info!("done");

    // To check memory usage, add this sleep so you can ^Z the process
    // in your terminal and check it with:
    //   $ top -p "$(pgrep dd-monitor)"
    // The VIRT and RES columns indicate the number of kilobytes of virtual
    // and real memory allocated for the process.
    //
    // std::thread::sleep(std::time::Duration::from_secs(16 * 60));

    Ok(())
}
