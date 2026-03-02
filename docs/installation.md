# Installation

## Prerequisites

- Rust 1.91 or later (install from [rustup.rs](https://rustup.rs))
- System libraries for the GUI framework:
  - **Linux (Debian/Ubuntu):** `sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`
  - **macOS:** No extra dependencies needed
  - **Windows:** No extra dependencies needed

## Pre-built binaries

### Shell script (macOS, Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MatthewMckee4/cram/releases/download/0.0.1-alpha.0/cram-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MatthewMckee4/cram/releases/download/0.0.1-alpha.0/cram-installer.ps1 | iex"
```

## From source

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo install --path crates/cram
```

This builds and installs the `cram` binary to `~/.cargo/bin/`.

## Self-update

If you installed via a pre-built binary, cram can update itself:

```bash
cram self update
```

Pass `--prerelease` to include alpha/beta/rc versions:

```bash
cram self update --prerelease
```

If you hit GitHub API rate limits, provide a token:

```bash
cram self update --token <GITHUB_TOKEN>
```

## Development build

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo build
cargo run
```

## Running tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```
