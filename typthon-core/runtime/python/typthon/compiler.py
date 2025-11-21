"""Compiler integration for Typthon."""

import sys
import subprocess
import shutil
import argparse
from pathlib import Path


def main():
    """Main entry point for Typthon compiler."""
    parser = argparse.ArgumentParser(
        prog="typthon-compile",
        description="Compile Python to native binary using Typthon compiler"
    )
    parser.add_argument("source", help="Python source file to compile")
    parser.add_argument("-o", "--output", help="Output binary path")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")
    parser.add_argument("--version", action="store_true", help="Show version")

    args = parser.parse_args()

    if args.version:
        from typthon import __version__
        print(f"Typthon Compiler version {__version__}")
        return 0

    # Look for the typthon compiler binary
    compiler = shutil.which("typthon")
    if not compiler:
        # Try to find it in common locations
        possible_paths = [
            Path(__file__).parent.parent.parent.parent.parent / "typthon-compiler" / "bin" / "typthon",
            Path.home() / ".local" / "bin" / "typthon",
            Path("/usr/local/bin/typthon"),
        ]
        for path in possible_paths:
            if path.exists():
                compiler = str(path)
                break

    if not compiler:
        print(
            "Error: typthon-compiler not found in PATH.\n"
            "Please install the Typthon compiler or add it to your PATH.\n"
            "See: https://github.com/griffinstrier/Typthon/blob/main/typthon-compiler/README.md",
            file=sys.stderr
        )
        return 1

    source_path = Path(args.source)
    if not source_path.exists():
        print(f"Error: Source file '{args.source}' not found", file=sys.stderr)
        return 1

    # Build the compiler command
    cmd = [compiler, "compile", str(source_path)]
    if args.output:
        cmd.extend(["-o", args.output])

    if args.verbose:
        print(f"Running: {' '.join(cmd)}")

    try:
        result = subprocess.run(cmd, capture_output=not args.verbose, text=True)
        if result.returncode == 0:
            output = args.output or source_path.stem
            print(f"✓ Successfully compiled to '{output}'")
        else:
            if result.stderr:
                print(result.stderr, file=sys.stderr)
            return result.returncode
    except Exception as e:
        print(f"Error running compiler: {e}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())

