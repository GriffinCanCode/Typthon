"""Static type checking functionality."""

from pathlib import Path
from typing import Union

try:
    from typthon._core import check_file
except ImportError:
    def check_file(path: str) -> list[str]:
        return [f"[Dev Mode] Would check: {path}"]


def check(target: Union[str, Path], *, recursive: bool = True) -> list[str]:
    """
    Perform static type checking on a file or directory.

    Args:
        target: File or directory path
        recursive: Check subdirectories

    Returns:
        List of error messages

    Example:
        errors = check("my_module.py")
        if errors:
            for error in errors:
                print(error)
    """
    path = Path(target)

    if not path.exists():
        return [f"Path not found: {target}"]

    if path.is_file():
        return check_file(str(path))

    if path.is_dir():
        errors = []
        pattern = "**/*.py" if recursive else "*.py"
        for py_file in path.glob(pattern):
            errors.extend(check_file(str(py_file)))
        return errors

    return [f"Invalid target: {target}"]

