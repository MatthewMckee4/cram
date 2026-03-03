# Cram

A flashcard app with [Typst](https://typst.app/)-powered card rendering.

## Installation

### Shell script (macOS, Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MatthewMckee4/cram/releases/download/0.0.1-alpha.2/cram-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MatthewMckee4/cram/releases/download/0.0.1-alpha.2/cram-installer.ps1 | iex"
```

### Build from source

```sh
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo install --path crates/cram
```

### Self-update

Once installed, cram can update itself:

```sh
cram self update
```

## Usage

Launch the GUI:

```sh
cram
```

List all decks:

```sh
cram list
```

## Documentation

- [Installation](docs/installation.md)
- [Getting Started](docs/getting-started.md)
- [Writing Cards](docs/writing-cards.md)
- [Configuration](docs/configuration.md)

## License

MIT
