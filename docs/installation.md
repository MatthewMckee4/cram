# Installation

## Prerequisites

- Rust 1.80 or later (install from [rustup.rs](https://rustup.rs))
- System libraries for the GUI framework:
    - **Linux (Debian/Ubuntu):** `sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`
    - **macOS:** No extra dependencies needed
    - **Windows:** No extra dependencies needed

## From source

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo install --path crates/cram
```

This builds and installs the `cram` binary to `~/.cargo/bin/`.

## Development build

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo build
cargo run
```

## Running tests

```bash
just test          # uses cargo-nextest if available, else cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Benchmarks

```bash
cargo bench --bench render_throughput
```
