# Typthon Development Setup

This document describes the setup for developing the complete Typthon ecosystem: type checker, compiler, and runtime.

## Project Structure

```
Typthon/
├── typthon-core/         # Type checker (Rust)
├── typthon-compiler/     # Native compiler (Go)
└── typthon-runtime/      # Minimal runtime (Rust → staticlib)
```

## Prerequisites

### For Type Checker (typthon-core)
- Rust 1.74+
- Cargo

### For Compiler (typthon-compiler)
- Go 1.22+
- System assembler (`as`, `nasm`, or platform-specific)
- System linker (`ld`, `lld`, or platform-specific)

### For Runtime (typthon-runtime)
- Rust 1.74+
- Cargo

## Building Components

### Build Type Checker

```bash
# From project root
cargo build --release

# Run type checker binary
./target/release/typthon check examples/basic_usage.py

# Build Python package
pip install maturin
maturin develop --release
```

### Build Compiler

```bash
# From typthon-compiler/
cd typthon-compiler

# Build compiler binary
make build
# or: go build -o bin/typthon ./cmd/typthon

# Test compiler
./bin/typthon version
```

### Build Runtime

```bash
# From typthon-runtime/
cd typthon-runtime

# Build static library
cargo build --release

# Output: target/release/libtypthon_runtime.a
# This will be linked into compiled binaries
```

## Development Workflow

### Type Checker Development

```bash
# Run tests
cargo test

# Run specific test
cargo test test_inference

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint
cargo clippy
```

### Compiler Development

```bash
cd typthon-compiler

# Build
make build

# Run tests
make test

# Format
make fmt

# Lint
make lint

# Development build with race detector
make dev
```

### Runtime Development

```bash
cd typthon-runtime

# Build
cargo build --release

# Test
cargo test

# Benchmark
cargo bench

# Run example
cargo run --example minimal
```

## End-to-End Compilation

Once all components are built:

```bash
# 1. Type check Python code
./target/release/typthon check program.py

# 2. Compile to native binary (future)
./typthon-compiler/bin/typthon compile program.py -o program

# 3. Run standalone binary
./program
```

## IDE Setup

### Rust (VS Code)
- Install `rust-analyzer` extension
- Configure: `.vscode/settings.json`
```json
{
  "rust-analyzer.linkedProjects": [
    "Cargo.toml",
    "typthon-runtime/Cargo.toml"
  ]
}
```

### Go (VS Code)
- Install `Go` extension
- Configure: `.vscode/settings.json`
```json
{
  "go.goroot": "/usr/local/go",
  "go.gopath": "~/go"
}
```

## Testing

### Unit Tests

```bash
# Type checker
cargo test

# Compiler
cd typthon-compiler && go test ./...

# Runtime
cd typthon-runtime && cargo test
```

### Integration Tests

```bash
# Full pipeline (once implemented)
./scripts/test_e2e.sh
```

## Cross-Compilation

### Compiler Cross-Compilation

```bash
cd typthon-compiler

# Linux ARM64
GOOS=linux GOARCH=arm64 go build -o bin/typthon-linux-arm64 ./cmd/typthon

# macOS x86-64
GOOS=darwin GOARCH=amd64 go build -o bin/typthon-darwin-amd64 ./cmd/typthon

# Windows
GOOS=windows GOARCH=amd64 go build -o bin/typthon-windows.exe ./cmd/typthon
```

### Runtime Cross-Compilation

```bash
cd typthon-runtime

# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# macOS x86-64
cargo build --release --target x86_64-apple-darwin
```

## Performance Profiling

### Type Checker

```bash
# Profile with cargo
cargo flamegraph --bench simd

# Profile specific benchmark
cargo bench --bench incremental -- --profile-time 10
```

### Compiler

```bash
cd typthon-compiler

# CPU profiling
go test -cpuprofile cpu.prof -bench .
go tool pprof cpu.prof

# Memory profiling
go test -memprofile mem.prof -bench .
go tool pprof mem.prof
```

### Runtime

```bash
cd typthon-runtime

# Benchmark allocator
cargo bench --bench allocator

# Benchmark GC
cargo bench --bench gc
```

## Directory Structure

### typthon-core/
```
compiler/
  analysis/     # Type checking, inference, effects
  ast/          # AST representation
  errors/       # Error handling
  frontend/     # Parsing, config
  types/        # Type system
infrastructure/ # Performance (cache, parallel, incremental)
runtime/        # Python and C++ runtime APIs
bindings/       # FFI layer
cli/            # Command-line binary
```

### typthon-compiler/
```
cmd/typthon/    # Main compiler binary
pkg/
  frontend/     # Parser
  ir/           # Intermediate representation
  ssa/          # SSA construction
  codegen/      # Code generation (amd64, arm64, riscv64)
  linker/       # Linking
  interop/      # FFI
```

### typthon-runtime/
```
src/
  allocator.rs  # Memory allocator
  gc.rs         # Garbage collector
  builtins.rs   # Built-in functions
  interop.rs    # Language interop
  ffi.rs        # C API
```

## Contributing

See `CONTRIBUTING.md` for guidelines.

## Troubleshooting

### "Cannot find typthon-core"
Make sure you're in the project root and `typthon-core/` directory exists.

### Go compiler errors
Ensure Go 1.22+ is installed: `go version`

### Rust compilation errors
Update Rust: `rustup update`

### Linking errors
Ensure system linker is available: `which ld` or `which lld`

## Next Steps

See `ROADMAP.md` for the development plan and current status.

