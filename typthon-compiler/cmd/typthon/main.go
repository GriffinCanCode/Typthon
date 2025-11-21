// Package main implements the Typthon compiler binary.
//
// Philosophy: Fast, minimal, elegant - inspired by Go's compiler architecture.
package main

import (
	"fmt"
	"os"
)

const version = "0.1.0"

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(1)
	}

	cmd := os.Args[1]
	switch cmd {
	case "compile":
		compile(os.Args[2:])
	case "version":
		fmt.Printf("typthon compiler version %s\n", version)
	case "help":
		usage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", cmd)
		usage()
		os.Exit(1)
	}
}

func usage() {
	fmt.Println(`Typthon Compiler - Compile typed Python to native code

Usage:
    typthon compile <source.py> [-o output]  Compile to native binary
    typthon version                          Show compiler version
    typthon help                             Show this help message

Options:
    -o <file>      Output binary name (default: source name)
    -O <level>     Optimization level (0-3, default: 2)
    -target <arch> Target architecture (amd64, arm64, riscv64)
    -v             Verbose output
    -debug         Enable debug info`)
}

func compile(args []string) {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, "error: no input file")
		os.Exit(1)
	}

	source := args[0]
	fmt.Printf("Compiling %s...\n", source)

	// TODO: Implement compilation pipeline
	// 1. Parse source
	// 2. Type check
	// 3. Generate IR
	// 4. Optimize
	// 5. Generate code
	// 6. Link

	fmt.Println("Compilation successful!")
}
