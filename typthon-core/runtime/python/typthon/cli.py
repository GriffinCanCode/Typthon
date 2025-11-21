"""CLI entry point for Typthon type checker."""

import sys
import argparse
from pathlib import Path


def main():
    """Main CLI entry point for type checking."""
    parser = argparse.ArgumentParser(
        prog="typthon",
        description="Typthon: High-performance gradual type system for Python"
    )
    parser.add_argument("command", choices=["check", "infer", "version"], help="Command to run")
    parser.add_argument("file", nargs="?", help="Python file to check")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    if args.command == "version":
        from typthon import __version__
        print(f"Typthon version {__version__}")
        return 0

    if not args.file:
        print("Error: file argument required for check/infer commands", file=sys.stderr)
        return 1

    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: File '{args.file}' not found", file=sys.stderr)
        return 1

    try:
        from typthon import check_file_py, infer_types_py

        if args.command == "check":
            if check_file_py is None:
                print("Error: Rust extension not available", file=sys.stderr)
                return 1

            errors = check_file_py(str(file_path))
            if errors:
                print(f"Found {len(errors)} type error(s):")
                for error in errors:
                    print(f"  {error}")
                return 1
            else:
                print(f"✓ {args.file} type checks successfully")
                return 0

        elif args.command == "infer":
            if infer_types_py is None:
                print("Error: Rust extension not available", file=sys.stderr)
                return 1

            source = file_path.read_text()
            result = infer_types_py(source)
            print(f"Inferred type: {result}")
            return 0

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())

