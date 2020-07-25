## Getting Started

### Dependencies

0. Use a Linux-like operating system, or you may need to slightly adapt these instructions.
1. Install `rustup`, the official Rust toolchain installer, from https://rustup.rs.
   (The default options will be fine, you don't need to customize anything.)
2. In this project's directory, run `cargo build --workspace` to fetch and build all dependencies.

or [![Open in Gitpod.](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/jeremyBanks/http-monitor)

### Running tests

3. `cargo test`

### Running the program

4. `cargo run < ./sample_input.txt`

   Request logs are read from standard input.  
   Monitor outputs are written to standard output.  
   Internal log messages are written to standard error.

   You may specify `run --release` to compile and run an optimized release build instead of the default debug build.

5. `cargo run -- --help` will display usage instructions for configuring the monitors.

### Running benchmarks

6. `cargo bench`

## Assumptions

I have made the following assumptions based on the sample input.

- The input file is valid UTF-8. Rust gives me better assertion messages if I treat the data as text strings instead of byte arrays.
- All rows have the expected format, and all "request" column header lines are valid HTTP 1.x.
- We have enough memory to comfortably store all request logs in our alerting/stats windows at once, without compression.
- No input fields are pathologically large. If an attacker is sending us 100 gigabyte HTTP requests with no newlines, and log messages containing all of that are passed to this program, something will fail.
- Remote hosts are IPv4 addresses, not hostnames or IPv6 addresses.
- Records are in chronological order give-or-take one second, so we can assume that no record will have a timestamp more than two seconds less than the highest timestamp we've seen so far.

## Implementation Notes

I usually do assignments like this in TypeScript or Python, as those are the languages I'm most comfortable with. However, for a task like ingesting and processing HTTP logs, where the volume may be huge and performance is critical, they didn't feel like the best choice. Instead, I went with Rust, where I have much finer-grained control over memory allocation and performance. This took much longer to write, but I'm also much more confident in the quality of the result.

Each row is deserialized from CSV into our `RequestRecord` type, for a more efficient binary representation. Fields that we don't use (`rfc931` and `authuser`) are represented in our model with placeholder zero-size types, but the actual values are discarded during parsing to save memory.

To deal with the records timestamps being up to a two seconds out-of-order, I implemented a `SortedRequestIterator` which wraps an iterator of parsed records with a two second buffer, which uses a heap to sort buffered samples before flushing theme into a deque as the time window moves forward.

Once a record is pulled out of that buffer, we start reference-counting it and hand it off to each monitor (one for alerts, one for stats) separately, so we hold onto it as long as one of them needs it. The alerts monitor use a continuously rolling time window, so it stores references in a deque. The stats monitor processes in chunks, so it stores them in a simple vec that is cleared at the end of each chunk.

On my current machine, it can process the sample file data using 5.5MB of RAM in 6.5ms on my i7-7700HQ, about 1.3 microseconds per row, single-threaded, assuming input and output are already in memory, and **assuming my benchmark is correct, which it may not be. I haven't used this test framework before. I do not stand by these measurements**, but they seem encouraging compared to what I'd have to use to run _any_ JavaScript code.

I wrote some integration tests, but run out of time before writing the unit tests I would like to have.

I depend on the following Rust crates:

- `anyhow`: dynamic error type for application code and prototyping (adds stack traces and other context for developers, not frequently used in library interfaces).
- `atty`: function for detecting which standard I/O streams are attached to terminals, like `isatty` in libc.
- `argh`: command-line argument parsing library.
- `log`: textual logging interface.
- `env_logger`: terminal logging backend.
- `criterion`: benchmarking framework.
- `itertools`: iterator extension methods.
- `chrono`: date and time.

## Potential Improvements

- More unit tests.
- Reduce memory use by reusing string objects if they're duplicated between requests using `string_cache`.
- More precise error handling: most uses of `anyhow` should be replaced with `thiserror`, and the uses that remain should have `.context(...)` information attached. Uses of `.unwrap()` should be eliminated and uses of `.expect(...)` minimized.
- The config object shouldn't be passed around so much, that's smelly.
- If we were running on data in real time, and there was a long wait between events, stats for the events right before that might be delayed. I would consider adopting to using an async executor (such as the one from `async_std`) that would let us check if we need to flush stats every second, instead of only in response to new records or the end of a stream.
- We could make it parallel but it's already pretty fast; I'd benchmark under heavy load to see if it really was CPU-bound before adding that complexity.
