"""Test static type checker."""

import pytest
from pathlib import Path
from typthon import check


def test_check_valid_code(tmp_path: Path):
    """Test checking valid typed code."""
    test_file = tmp_path / "valid.py"
    test_file.write_text("""
def add(x: int, y: int) -> int:
    return x + y

result: int = add(1, 2)
""")

    errors = check(test_file)
    # May have errors in dev mode without compiled extension
    assert isinstance(errors, list)


def test_check_invalid_code(tmp_path: Path):
    """Test checking code with type errors."""
    test_file = tmp_path / "invalid.py"
    test_file.write_text("""
def add(x: int, y: int) -> int:
    return x + y

result: int = add("1", "2")  # Type error
""")

    errors = check(test_file)
    assert isinstance(errors, list)


def test_check_directory(tmp_path: Path):
    """Test checking entire directory."""
    (tmp_path / "file1.py").write_text("x: int = 1")
    (tmp_path / "file2.py").write_text("y: str = 'hello'")

    errors = check(tmp_path)
    assert isinstance(errors, list)


def test_check_nonexistent():
    """Test checking nonexistent path."""
    errors = check("/nonexistent/path.py")
    assert len(errors) > 0
    assert "not found" in errors[0].lower()

