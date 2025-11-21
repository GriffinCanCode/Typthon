"""LSP server integration for Typthon."""

import sys
import subprocess
import shutil
from pathlib import Path


def main():
    """Main entry point for Typthon LSP server."""
    # Look for the typthon-lsp binary
    lsp_binary = shutil.which("typthon-lsp")

    if not lsp_binary:
        # Try to find it in common locations
        possible_paths = [
            Path(__file__).parent.parent.parent.parent.parent / "typthon-lsp" / "target" / "release" / "typthon-lsp",
            Path(__file__).parent / "bin" / "typthon-lsp",
            Path.home() / ".local" / "bin" / "typthon-lsp",
            Path("/usr/local/bin/typthon-lsp"),
        ]
        for path in possible_paths:
            if path.exists():
                lsp_binary = str(path)
                break

    if not lsp_binary:
        print(
            "Error: typthon-lsp not found.\n"
            "The LSP server is not yet built or installed.\n"
            "To build it:\n"
            "  cd typthon-lsp && cargo build --release\n"
            "See: https://github.com/griffinstrier/Typthon/blob/main/typthon-lsp/README.md",
            file=sys.stderr
        )
        return 1

    try:
        # Run the LSP server, passing through all arguments
        result = subprocess.run([lsp_binary] + sys.argv[1:])
        return result.returncode
    except Exception as e:
        print(f"Error running LSP server: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

