# bzip2rs

This is an project for a research experiments, which is a Rust implementation of the bzip2 compression/decompression tool.
It provides a command-line interface (CLI) for compressing, decompressing, and testing bzip2 files.

This project has two features: `sys` and `default`.
The `default` feature enables the pure Rust implementation of bzip2 compression and decompression with [banzai](https://crates.io/crates/banzai) and [bzip2-rs](https://crates.io/crates/bzip2-rs).
The `sys` feature enables the use of the `bzip2` crate, which is a wrapper around the `libbz2` C library.

## Compile

### The `default` feature (pure Rust implementation)

```sh
cargo build --release
```

### The `sys` feature (using system libbz2)

```sh
cargo build --release --features sys
```

## See also

- [Go bzip2](https://github.com/pedroalbanese/bzip2)
  - The Go implementation of bzip2 command.
- bzip2 crates for Rust:
  - [bzip2](https://crates.io/crates/bzip2)
    - Bindings to libbzip2 for bzip2 compression and decompression exposed as Reader/Writer streams.
  - [oxiarc-bzip2](https://crates.io/crates/oxiarc-bzip2)
    - Pure Rust implementation of BZip2 compression/decompression algorithm.
  - [banzai](https://crates.io/crates/banzai)
    - A pure Rust bzip2 encoder. ([bnz](https://crates.io/crates/bnz) is a CLI tool)
  - [bzip2-rs](https://crates.io/crates/bzip2-rs)
    - Pure Rust bzip2 decompressor.
