# Logger Package

Standardized logging utilities for the Typthon compiler using Go's `slog` package.

## Quick Start

```go
import "github.com/GriffinCanCode/typthon-compiler/pkg/logger"

func main() {
    // Initialize for development
    logger.InitDev()

    // Or initialize for production
    // logger.InitProd("/var/log/typthon")

    logger.Info("Compiler starting")
}
```

## API Reference

### Initialization

- `InitDev()` - Initialize with debug level, text format, stderr output
- `InitProd(logDir string) error` - Initialize with info level, JSON format, file output
- `Init(cfg Config) error` - Initialize with custom configuration

### Basic Logging

- `Debug(msg string, args ...any)` - Log debug message
- `Info(msg string, args ...any)` - Log info message
- `Warn(msg string, args ...any)` - Log warning message
- `Error(msg string, args ...any)` - Log error message

### Compiler-Specific Helpers

- `LogPhase(phase string)` - Log compilation phase start
- `LogPhaseComplete(phase string)` - Log compilation phase completion
- `LogLexing(file string, tokenCount int)` - Log lexing completion
- `LogParsing(file string, nodeCount int)` - Log parsing completion
- `LogSSAGeneration(funcName string, blockCount int)` - Log SSA generation
- `LogCodeGen(arch, funcName string, instructionCount int)` - Log code generation
- `LogOptimization(pass string, changeCount int)` - Log optimization pass
- `LogError(phase, file string, line int, msg string)` - Log compilation error
- `LogWarning(phase, file string, line int, msg string)` - Log compilation warning
- `LogCompilerStart(args []string)` - Log compiler startup
- `LogCompilerComplete(success bool, duration string)` - Log compiler completion
- `LogFileProcessing(file string)` - Log file processing start
- `LogLinkingStart(objectCount int)` - Log linker start
- `LogLinkingComplete(outputFile string)` - Log linker completion

### Contextual Logging

- `With(args ...any) *slog.Logger` - Create logger with persistent attributes
- `WithGroup(name string) *slog.Logger` - Create logger with group context

## Examples

### Basic Usage

```go
logger.Info("Compiling file", "path", inputFile)
logger.Debug("Token count", "count", tokens)
logger.Error("Failed to parse", "file", file, "error", err)
```

### Phase Logging

```go
logger.LogPhase("parsing")
ast, err := parser.Parse()
if err != nil {
    logger.LogError("parsing", filename, line, err.Error())
    return err
}
logger.LogPhaseComplete("parsing")
```

### Contextual Logging

```go
funcLogger := logger.With("function", funcName, "arch", "amd64")
funcLogger.Debug("Allocating registers")
funcLogger.Info("Code generation complete", "instructions", count)
```

## Configuration

```go
config := logger.Config{
    Level:     logger.LevelDebug,
    Format:    "json",          // "text" or "json"
    Output:    os.Stderr,       // Any io.Writer
    AddSource: true,            // Include source location
    LogFile:   "/path/to/log",  // Optional file output
}

if err := logger.Init(config); err != nil {
    log.Fatal(err)
}
```

## See Also

- [Main Logging Documentation](../../../LOGGING.md) - Complete logging guide
- [slog package](https://pkg.go.dev/log/slog) - Go standard library documentation

