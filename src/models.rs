use std::{fmt::Debug, io::BufRead, net::Ipv4Addr, str};

use serde_derive::{Deserialize, Serialize};

// TODO: intern these strings

/// HTTP request record from input.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Request {
    pub timestamp: u32,
    pub method: String,
    pub section: String,
    pub response_status: u16,
    pub response_length: u64,
}

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

impl RequestRecord {
    pub fn section(&self) -> &str {
        let path = self.request.split(' ').nth(1).unwrap_or("/unknown");
        let section = path.split('/').nth(1).unwrap_or("unknown");
        section
    }

    pub fn read_csv_line(reader: &mut impl BufRead) -> Option<Self> {
        let mut line = String::new();
        if let Err(error) = reader.read_line(&mut line) {
            panic!("failed to read csv line: {:?}", error);
        }
        if line.trim().is_empty() {
            return None;
        }
        let mut line = line.trim().bytes().peekable();
        let mut fields: Vec<String> = Vec::new();

        loop {
            match line.peek() {
                None => break,
                // we don't support any kind of quote-nesting/encoding
                Some(b'"') => {
                    line.next().unwrap(); // skip opening quote
                    let mut field = Vec::new();
                    loop {
                        let next = line.next().unwrap();
                        if next == b'"' {
                            // skip trailing whitespace in fields
                            while line.peek() == Some(b' ').as_ref() {
                                line.next();
                            }
                            // consume comma or newline
                            line.next();
                            break;
                        } else {
                            field.push(next);
                        }
                    }
                    fields.push(String::from_utf8(field).unwrap());
                }
                Some(b' ') => {
                    // skip leading whitespace in fields
                    continue;
                }
                Some(_c) => {
                    let mut field = Vec::new();
                    loop {
                        let next = line.next();
                        if next == Some(b',') || next == None || next == Some(b'\n') {
                            break;
                        } else {
                            field.push(next.unwrap());
                        }
                    }
                    fields.push(String::from_utf8(field).unwrap());
                }
            }
        }

        Some(RequestRecord {
            remote_host: fields[0].parse().unwrap(),
            rfc931: (),
            auth_user: (),
            date: fields[3].parse().unwrap(),
            request: fields[4].parse().unwrap(),
            status: fields[5].parse().unwrap(),
            bytes: fields[6].parse().unwrap(),
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_iter() {
        let input = r#""10.0.0.4","-","apache",1549573859,"GET /api/help HTTP/1.0",200,1234
"10.0.0.5","-","apache",1549573860,"POST /report HTTP/1.0",500,1307
"10.0.0.3","-","apache",1549573860,"POST /report HTTP/1.0",200,1234
"10.0.0.3","-","apache",1549573860,"GET /report HTTP/1.0",200,1194"#;
        let reader = std::io::Cursor::new(input);
        let mut reader = std::io::BufReader::new(reader);

        let request = RequestRecord::read_csv_line(&mut reader).unwrap();
    }
}
