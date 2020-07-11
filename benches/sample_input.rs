use std::{io::Cursor, str};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use http_monitor::{monitor_stream, Config};

fn bench_monitor_sample_input(c: &mut Criterion) {
    c.bench_function("monitor sample output", |b| {
        b.iter(|| {
            let input = black_box(&include_str!("../samples/input.csv")[..]);
            let expected = black_box(&include_str!("../samples/output.txt")[..]);

            let mut source = Cursor::new(input);
            let mut sink = Cursor::new(Vec::new());
            let config = Config::default();

            monitor_stream(&mut source, &mut sink, &config).unwrap();

            let actual = sink.into_inner();
            let actual = str::from_utf8(&actual).unwrap();
            black_box(actual);
            assert_eq!(actual, expected);
        })
    });
}

criterion_group!(benches, bench_monitor_sample_input);
criterion_main!(benches);
