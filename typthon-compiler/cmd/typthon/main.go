// Package main implements the Typthon compiler binary.
//
// Philosophy: Fast, minimal, elegant - inspired by Go's compiler architecture.
package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"time"

	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/amd64"
	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/arm64"
	"github.com/GriffinCanCode/typthon-compiler/pkg/frontend"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

const version = "0.1.0"

func main() {
	// Initialize logging early
	logger.InitDev()
	logger.LogCompilerStart(os.Args)

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
		logger.Error("Unknown command", "command", cmd)
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
	start := time.Now()

	if len(args) == 0 {
		logger.Error("No input file provided")
		fmt.Fprintln(os.Stderr, "error: no input file")
		os.Exit(1)
	}

	sourceFile := args[0]
	outputFile := getOutputFile(args, sourceFile)

	logger.LogFileProcessing(sourceFile)
	fmt.Printf("Compiling %s...\n", sourceFile)

	// Read source file
	source, err := os.ReadFile(sourceFile)
	if err != nil {
		logger.Error("Failed to read source file", "file", sourceFile, "error", err)
		fmt.Fprintf(os.Stderr, "error reading file: %v\n", err)
		os.Exit(1)
	}

	if err := compileProgram(string(source), outputFile); err != nil {
		duration := time.Since(start).String()
		logger.LogCompilerComplete(false, duration)
		fmt.Fprintf(os.Stderr, "compilation failed: %v\n", err)
		os.Exit(1)
	}

	duration := time.Since(start).String()
	logger.LogCompilerComplete(true, duration)
	fmt.Println("Compilation successful!")
}

func getOutputFile(args []string, sourceFile string) string {
	for i, arg := range args {
		if arg == "-o" && i+1 < len(args) {
			return args[i+1]
		}
	}
	// Default: strip .py extension
	if len(sourceFile) > 3 && sourceFile[len(sourceFile)-3:] == ".py" {
		return sourceFile[:len(sourceFile)-3]
	}
	return "a.out"
}

func compileProgram(source string, output string) error {
	// 1. Parse source
	logger.LogPhase("parsing")
	parser := frontend.NewParser(source)
	ast, err := parser.Parse()
	if err != nil {
		logger.LogError("parsing", "", 0, err.Error())
		return fmt.Errorf("parse error: %w", err)
	}
	logger.LogPhaseComplete("parsing")

	// 2. Generate IR
	logger.LogPhase("IR generation")
	builder := ir.NewBuilder()
	irProg, err := builder.Build(ast)
	if err != nil {
		logger.LogError("IR generation", "", 0, err.Error())
		return fmt.Errorf("IR generation error: %w", err)
	}
	logger.Info("IR generation complete", "functions", len(irProg.Functions))
	logger.LogPhaseComplete("IR generation")

	// 3. Convert to SSA
	logger.LogPhase("SSA conversion")
	ssaProg := ssa.Convert(irProg)
	logger.Info("SSA conversion complete", "functions", len(ssaProg.Functions))
	logger.LogPhaseComplete("SSA conversion")

	// 4. Generate assembly
	logger.LogPhase("code generation")
	asmFile := output + ".s"
	f, err := os.Create(asmFile)
	if err != nil {
		logger.Error("Failed to create assembly file", "file", asmFile, "error", err)
		return fmt.Errorf("failed to create assembly file: %w", err)
	}
	defer f.Close()

	// Select code generator based on architecture
	arch := runtime.GOARCH
	logger.Info("Generating assembly", "arch", arch, "output", asmFile)
	switch arch {
	case "arm64":
		gen := arm64.NewGenerator(f)
		if err := gen.Generate(ssaProg); err != nil {
			logger.LogError("code generation", "", 0, err.Error())
			return fmt.Errorf("code generation error: %w", err)
		}
	case "amd64":
		gen := amd64.NewGenerator(f)
		if err := gen.Generate(ssaProg); err != nil {
			logger.LogError("code generation", "", 0, err.Error())
			return fmt.Errorf("code generation error: %w", err)
		}
	default:
		logger.Error("Unsupported architecture", "arch", arch)
		return fmt.Errorf("unsupported architecture: %s", arch)
	}
	f.Close()
	logger.LogPhaseComplete("code generation")

	// 5. Assemble
	logger.LogPhase("assembly")
	objFile := output + ".o"
	cmd := exec.Command("as", "-o", objFile, asmFile)
	if out, err := cmd.CombinedOutput(); err != nil {
		logger.Error("Assembly failed", "error", err, "output", string(out))
		return fmt.Errorf("assembly failed: %w\n%s", err, out)
	}
	logger.LogPhaseComplete("assembly")

	// 6. Link with runtime
	logger.LogLinkingStart(2)

	// Find runtime.c - check multiple locations
	var runtimeC string
	possiblePaths := []string{
		filepath.Join(filepath.Dir(os.Args[0]), "..", "runtime", "runtime.c"),
		filepath.Join(filepath.Dir(os.Args[0]), "runtime", "runtime.c"),
		"runtime/runtime.c",
	}

	for _, path := range possiblePaths {
		if _, err := os.Stat(path); err == nil {
			runtimeC = path
			break
		}
	}

	if runtimeC == "" {
		logger.Error("Could not find runtime.c")
		return fmt.Errorf("could not find runtime.c in any expected location")
	}

	// Compile runtime
	runtimeObj := output + "_runtime.o"
	cmd = exec.Command("cc", "-c", "-o", runtimeObj, runtimeC)
	if out, err := cmd.CombinedOutput(); err != nil {
		logger.Error("Runtime compilation failed", "error", err, "output", string(out))
		return fmt.Errorf("runtime compilation failed: %w\n%s", err, out)
	}

	// Link everything
	cmd = exec.Command("cc", "-o", output, objFile, runtimeObj)
	if out, err := cmd.CombinedOutput(); err != nil {
		logger.Error("Linking failed", "error", err, "output", string(out))
		return fmt.Errorf("linking failed: %w\n%s", err, out)
	}
	logger.LogLinkingComplete(output)

	// Cleanup temporary files
	logger.Debug("Cleaning up temporary files")
	os.Remove(asmFile)
	os.Remove(objFile)
	os.Remove(runtimeObj)

	return nil
}
