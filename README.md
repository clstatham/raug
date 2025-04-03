# raug

**raug** is a library for writing and running digital audio processors and signal flow graphs in Rust.

## Design Goals

- Fast, lightweight, zero-copy where possible
- Stack memory >>> Heap memory
- No allocations on the realtime audio thread
- Do as much work ahead of time as possible

## Features

- Two main APIs:
  - `processor` API for writing high-performance raw audio processors
  - `builder` API for ergonomically building signal flow graphs
- Runtime capable of running signal flow graphs, either in realtime or offline
- Save rendered audio to WAV files
- Uses `f64` audio samples by default (can be set to `f32` with cargo feature `f32_samples`)
- Safe API: Very few `unsafe` blocks (currently only 2)

## Examples

See [examples/processor.rs](https://github.com/clstatham/raug/blob/master/examples/processor.rs) for a simple example of writing a raw audio processor.

See [examples/demo.rs](https://github.com/clstatham/raug/blob/master/examples/demo.rs) for a simple example of building a signal flow graph.

## Optional Cargo Feature Flags

- `f32_samples`: Use `f32` audio samples instead of the default `f64`.
- `serde`: Enable [serde](https://crates.io/crates/serde) v1 support for most relevant structures.
- `expr`: Enable parsing mathematical expressions with [`evalexpr`](https://crates.io/crates/evalexpr).
- `fft`: Enable FFT support for frequency-domain processing using [`realfft`](https://crates.io/crates/realfft).
- `jack`: Enable JACK support for realtime audio processing on Linux.

## Related Projects

- Python bindings: [raug-python](https://github.com/clstatham/raug-python)
- GUI using [iced](https://github.com/iced-rs/iced) (WIP): [raug-iced](https://github.com/clstatham/raug-iced)

## Roadmap

- [ ] More built-in processors
- [ ] More examples
- [ ] More tests
- [ ] More optimizations
- [ ] More bindings (JavaScript?)

## Contributing

This is a personal project, but I'm happy to accept contributions. Please open an issue or PR if you have any ideas or feedback.

## Versioning

This project is in early development and does not yet follow semantic versioning. Breaking changes may occur at any time.

The goal is to reach a somewhat-stable starting point and release version 0.1.0 on crates.io soon(tm).

## License

MIT OR Apache 2.0
