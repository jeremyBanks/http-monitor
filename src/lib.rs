//! Package entry point for dd-monitor.

#![allow(unused_imports)]

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    io::{stderr, stdin, stdout, Cursor, Read, Write},
    rc::Rc,
    str,
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use atty;
use csv;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use thiserror;

mod monitor;

pub use monitor::{monitor, MonitorConfig};

/// Application entry point.
pub fn main() -> Result<()> {
    if atty::is(atty::Stream::Stdin) {
        eprintln!("ERROR: stdin must be a stream or file, not a terminal.");
        eprintln!();
        eprintln!("example usage:");
        eprintln!("    cargo run < sample_input.csv");
        eprintln!();
        eprintln!("or with a release binary:");
        eprintln!("    cargo build --release");
        eprintln!("    target/release/dd-monitor < sample_input.csv");

        std::process::exit(1)
    }

    let config = MonitorConfig::default();

    monitor(&mut stdin(), &mut stdout(), &config)
}
