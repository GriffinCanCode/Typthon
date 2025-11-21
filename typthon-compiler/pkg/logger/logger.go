// Package logger provides standardized logging utilities for the Typthon compiler
package logger

import (
	"io"
	"log/slog"
	"os"
	"path/filepath"
)

// Global logger instance
var defaultLogger *slog.Logger

// LogLevel represents the logging level
type LogLevel int

const (
	LevelDebug LogLevel = iota
	LevelInfo
	LevelWarn
	LevelError
)

// Config holds logger configuration
type Config struct {
	Level     LogLevel
	Format    string // "text" or "json"
	Output    io.Writer
	AddSource bool
	LogFile   string
}

// DefaultConfig returns the default logger configuration
func DefaultConfig() Config {
	return Config{
		Level:     LevelInfo,
		Format:    "text",
		Output:    os.Stderr,
		AddSource: false,
	}
}

// Init initializes the global logger with the given configuration
func Init(cfg Config) error {
	var handler slog.Handler

	output := cfg.Output
	if cfg.LogFile != "" {
		file, err := os.OpenFile(cfg.LogFile, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
		if err != nil {
			return err
		}
		output = file
	}

	opts := &slog.HandlerOptions{
		Level:     toSlogLevel(cfg.Level),
		AddSource: cfg.AddSource,
	}

	if cfg.Format == "json" {
		handler = slog.NewJSONHandler(output, opts)
	} else {
		handler = slog.NewTextHandler(output, opts)
	}

	defaultLogger = slog.New(handler)
	slog.SetDefault(defaultLogger)

	return nil
}

// InitDev initializes logging for development (debug level, text format)
func InitDev() {
	_ = Init(Config{
		Level:     LevelDebug,
		Format:    "text",
		Output:    os.Stderr,
		AddSource: true,
	})
}

// InitProd initializes logging for production (info level, json format)
func InitProd(logDir string) error {
	logPath := filepath.Join(logDir, "typthon-compiler.log")
	return Init(Config{
		Level:     LevelInfo,
		Format:    "json",
		LogFile:   logPath,
		AddSource: false,
	})
}

func toSlogLevel(level LogLevel) slog.Level {
	switch level {
	case LevelDebug:
		return slog.LevelDebug
	case LevelInfo:
		return slog.LevelInfo
	case LevelWarn:
		return slog.LevelWarn
	case LevelError:
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}

// Debug logs a debug message
func Debug(msg string, args ...any) {
	if defaultLogger != nil {
		defaultLogger.Debug(msg, args...)
	}
}

// Info logs an info message
func Info(msg string, args ...any) {
	if defaultLogger != nil {
		defaultLogger.Info(msg, args...)
	}
}

// Warn logs a warning message
func Warn(msg string, args ...any) {
	if defaultLogger != nil {
		defaultLogger.Warn(msg, args...)
	}
}

// Error logs an error message
func Error(msg string, args ...any) {
	if defaultLogger != nil {
		defaultLogger.Error(msg, args...)
	}
}

// With returns a new logger with the given attributes
func With(args ...any) *slog.Logger {
	if defaultLogger != nil {
		return defaultLogger.With(args...)
	}
	return slog.Default().With(args...)
}

// WithGroup returns a new logger with the given group
func WithGroup(name string) *slog.Logger {
	if defaultLogger != nil {
		return defaultLogger.WithGroup(name)
	}
	return slog.Default().WithGroup(name)
}

// Compiler-specific logging helpers

// LogPhase logs the start of a compilation phase
func LogPhase(phase string) {
	Info("Starting compilation phase", "phase", phase)
}

// LogPhaseComplete logs the completion of a compilation phase
func LogPhaseComplete(phase string) {
	Info("Completed compilation phase", "phase", phase)
}

// LogLexing logs lexing activity
func LogLexing(file string, tokenCount int) {
	Debug("Lexing complete", "file", file, "tokens", tokenCount)
}

// LogParsing logs parsing activity
func LogParsing(file string, nodeCount int) {
	Debug("Parsing complete", "file", file, "nodes", nodeCount)
}

// LogSSAGeneration logs SSA generation
func LogSSAGeneration(funcName string, blockCount int) {
	Debug("SSA generation complete", "function", funcName, "blocks", blockCount)
}

// LogCodeGen logs code generation
func LogCodeGen(arch string, funcName string, instructionCount int) {
	Debug("Code generation complete",
		"arch", arch,
		"function", funcName,
		"instructions", instructionCount)
}

// LogOptimization logs optimization passes
func LogOptimization(pass string, changeCount int) {
	Info("Optimization pass complete", "pass", pass, "changes", changeCount)
}

// LogError logs a compilation error
func LogError(phase string, file string, line int, msg string) {
	Error("Compilation error",
		"phase", phase,
		"file", file,
		"line", line,
		"message", msg)
}

// LogWarning logs a compilation warning
func LogWarning(phase string, file string, line int, msg string) {
	Warn("Compilation warning",
		"phase", phase,
		"file", file,
		"line", line,
		"message", msg)
}

// LogCompilerStart logs compiler startup
func LogCompilerStart(args []string) {
	Info("Typthon compiler starting", "args", args)
}

// LogCompilerComplete logs compiler completion
func LogCompilerComplete(success bool, duration string) {
	if success {
		Info("Compilation successful", "duration", duration)
	} else {
		Error("Compilation failed", "duration", duration)
	}
}

// LogFileProcessing logs file processing start
func LogFileProcessing(file string) {
	Info("Processing file", "file", file)
}

// LogLinkingStart logs linker start
func LogLinkingStart(objectCount int) {
	Info("Starting linking", "objects", objectCount)
}

// LogLinkingComplete logs linker completion
func LogLinkingComplete(outputFile string) {
	Info("Linking complete", "output", outputFile)
}
