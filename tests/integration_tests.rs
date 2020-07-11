//! Integration tests for dd-monitor, asserting the expected outputs for given inputs.

use std::{io::Cursor, panic::catch_unwind, str};

use anyhow;

use dd_monitor::{monitor_stream, Config};

#[test]
/// Tests with no input.
fn test_monitor_nothing() -> anyhow::Result<()> {
    let input = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    let result = monitor_stream(&mut source, &mut sink, &config);

    assert!(result.is_err(), "missing header");
    Ok(())
}

#[test]
#[ignore = "not implemented"]
fn test_monitor_one_request() -> anyhow::Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status","bytes"
"10.0.0.2","-","apache",1549573860,"GET /api/user HTTP/1.0",200,200"#;
    let expected = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    monitor_stream(&mut source, &mut sink, &config)?;

    let actual = sink.into_inner();
    let actual = str::from_utf8(&actual)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
#[ignore = "not implemented"]
fn test_monitor_sample_input() -> anyhow::Result<()> {
    let input = &include_str!("../sample_input.csv")[..];
    let expected = "";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    monitor_stream(&mut source, &mut sink, &config)?;

    let actual = sink.into_inner();
    let actual = str::from_utf8(&actual)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_monitor_invalid_csv_input() -> anyhow::Result<()> {
    let input = "1 2\n3 4\n5";

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    let result = monitor_stream(&mut source, &mut sink, &config);

    assert!(result.is_err(), "invalid header");
    Ok(())
}

#[test]
fn test_monitor_invalid_csv_input_2() -> anyhow::Result<()> {
    let input = r#"241"#;

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    let result = monitor_stream(&mut source, &mut sink, &config);

    assert!(result.is_err(), "invalid header");
    Ok(())
}
#[test]
fn test_monitor_invalid_csv_input_3() -> anyhow::Result<()> {
    let input = r#"241
        123
        456"#;

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    let result = monitor_stream(&mut source, &mut sink, &config);

    assert!(result.is_err(), "this isn't csv it's just some numbers");
    Ok(())
}

#[test]
fn test_monitor_invalid_csv_input_4() -> anyhow::Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status"
"10.0.0.2","-","apache",1549573860,"GET /api/user HTTP/1.0",200
"10.0.0.4","-","apache",1549573860,"GET /api/user HTTP/1.0",200";"#;

    let mut source = Cursor::new(input);
    let mut sink = Cursor::new(Vec::new());
    let config = Config::default();

    let result = monitor_stream(&mut source, &mut sink, &config);

    assert!(result.is_err(), "size column missing");
    Ok(())
}

#[test]
fn test_monitor_invalid_csv_input_extra_column() -> anyhow::Result<()> {
    let input = r#""remotehost","rfc931","authuser","date","request","status","bytes"
"10.0.0.1","-","apache",1549574332,"GET /api/user HTTP/1.0",200,1234
"10.0.0.4","-","apache",1549574333,"GET /report HTTP/1.0",200,1136,10101,13513
"10.0.0.1","-","apache",1549574334,"GET /api/user HTTP/1.0",200,1194
"10.0.0.4","-","apache",1549574334,"POST /report HTTP/1.0",404,1307"#;

    let result = catch_unwind(|| {
        let mut source = Cursor::new(input);
        let mut sink = Cursor::new(Vec::new());
        let config = Config::default();

        monitor_stream(&mut source, &mut sink, &config)
    });

    assert!(result.is_err(), "extra column in record two");

    Ok(())
}
