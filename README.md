# Scar - a Source Code Analyzer in Rust

## Goal
Scar goals are:

1. Experiment with Rust
2. Build a quite efficient code analyzer, initially for C++ projects, to study
   dependencies between files (and, possibly, classes / structures / types)
3. Follow good architectural guidelines and principles (e.g., clean
   architecture)

## Dependencies

```
regex = "1.10"
tempdir = "0.3"
walkdir = "2.5.0"
lazy_static = "1.4"
clap = { version = "4.4.2", features = ["derive"] }
```

## Build

```
cargo build --release
```

## Usage

```
Usage: scar [OPTIONS] --path <PROJECT_PATH>

Options:
  -t, --topn
  -i, --topnimpact
  -p, --path <PROJECT_PATH>
  -n, --num <OUTPUT_SIZE>    [default: 42]
  -h, --help                 Print help
```

