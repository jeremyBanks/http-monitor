use std::{
    borrow::Cow,
    io::{stdin, stdout, stderr}
};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_derive::Deserialize;
use csv;

#[derive(Debug, Deserialize)]
struct LogEntry {
    #[serde(rename="remotehost")]
    remote_host: String,
    rfc931: String,
    #[serde(rename="authuser")]
    auth_user: String,
    date: u64,
    request: String,
    status: u64,
    bytes: u64,
    population: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Record {
    city: String,
    region: String,
    country: String,
    population: Option<u64>,
}

fn main() -> Result<()>  {
    let mut reader = csv::Reader::from_reader(stdin());
    for result in reader.deserialize() {
        let record: LogEntry = result?;
        println!("{:?}", record);
    }
    Ok(())
}
