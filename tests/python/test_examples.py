"""Every example must at least compile."""

from __future__ import annotations

import py_compile
from pathlib import Path


def _examples_dir() -> Path:
    for parent in Path(__file__).parents:
        candidate = parent / "examples"
        if candidate.is_dir():
            return candidate
    raise AssertionError("examples directory not found")


EXAMPLES = sorted(_examples_dir().glob("*.py"))


def test_examples_exist() -> None:
    assert EXAMPLES, "no examples found"


def test_examples_compile() -> None:
    for path in EXAMPLES:
        py_compile.compile(str(path), doraise=True)
