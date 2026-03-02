# Contributing

## Setup

```bash
git clone https://github.com/MatthewMckee4/cram
cd cram
cargo build
```

## Workflow

- All changes via pull requests, squash merge only
- Branch naming: `feat/`, `fix/`, `docs/`, `ci/`
- Run `just test` before submitting
- Run `uvx prek run -a` to verify all hooks pass

## Pre-commit

Install prek: `pip install prek` or `cargo install prek`
Run hooks: `uvx prek run -a`
