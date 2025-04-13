# raug

**raug** is a library for writing and running digital audio processors and signal flow graphs in Rust.

## Features

- Two main APIs:
  - `processor` API for writing high-performance raw audio processors
  - `graph` API for ergonomically building signal flow graphs
- Runtime capable of running signal flow graphs, either in realtime or offline
- Save rendered audio to WAV files

## Examples

See [examples/demo.rs](https://github.com/clstatham/raug/blob/master/examples/demo.rs) for a simple example of building a signal flow graph.

## Optional Cargo Feature Flags

- `jack`: Enable JACK support for realtime audio processing on Linux.

## Contributing

This is a personal project, but I'm happy to accept contributions. Please open an issue or PR if you have any ideas or feedback.

## Versioning

This project is in early development and does not yet follow semantic versioning. Breaking changes may occur at any time.

The goal is to reach a somewhat-stable starting point and release version 0.1.0 on crates.io soon.

## License

MIT OR Apache 2.0
