# Contributing

## Setup

```bash
git clone https://github.com/pyratatui/pyratatui.git
cd pyratatui
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pip install maturin
maturin develop
```

Rust source lives in `src/`, the Python package in `python/pyratatui/`.

## Checks

Run these before opening a pull request:

```bash
pytest
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
ruff check . && ruff format --check .
mypy .
```

After changing Rust code, re-run `maturin develop` before `pytest`.

## Guidelines

* Keep changes focused; pyratatui is a binding library, not a widget
  ecosystem — new third-party integrations belong in separate packages.
* Match the existing style: small functions, direct conversions, comments
  only where something is non-obvious.
* Add tests for new behavior; update `CHANGELOG.md` under `[Unreleased]`.
* Public API changes also need updates to
  `python/pyratatui/_pyratatui.pyi` and `python/pyratatui/__init__.py`.

## Issues

Include steps to reproduce, expected vs. actual behavior, and your OS,
Python, and Rust versions.
