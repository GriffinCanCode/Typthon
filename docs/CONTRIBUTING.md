# Contributing to Typthon

Thank you for your interest in contributing to Typthon!

## Development Setup

### Prerequisites

- Python 3.10+
- Rust 1.74+
- C++17 compiler (gcc/clang/MSVC)
- Make (optional but recommended)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/typthon.git
cd typthon

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin
pip install maturin

# Build in development mode
make dev

# Or manually:
maturin develop

# Install dev dependencies
pip install -e ".[dev]"
```

## Development Workflow

### Making Changes

1. Create a feature branch
2. Make your changes
3. Run tests: `make test`
4. Run lints: `make lint`
5. Format code: `make format`
6. Commit with clear message

### Code Style

**Rust**
- Follow Rust conventions
- Run `cargo fmt` before committing
- Pass `cargo clippy -- -D warnings`
- Document public APIs

**Python**
- Follow PEP 8
- Use `ruff format` for formatting
- Use `ruff check` for linting
- Type hints on all public APIs
- Docstrings in Google style

**C++**
- C++17 standard
- Modern C++ idioms (RAII, smart pointers)
- Document non-obvious code
- Prefer `const` and `constexpr`

### Testing

```bash
# Run all tests
make test

# Run specific test file
pytest tests/test_types.py -v

# Run with coverage
pytest --cov=typthon tests/

# Run benchmarks
pytest tests/ --benchmark-only
```

### Building

```bash
# Development build (fast, with debug symbols)
make dev

# Release build (optimized)
make build

# Install locally
make install
```

## Architecture Guidelines

### Design Principles

1. **Elegance First**: Simple, clear solutions over clever ones
2. **Performance**: Optimize hot paths, profile before optimizing
3. **Type Safety**: Maximum type safety in all layers
4. **Testability**: Every component should be easily testable
5. **Documentation**: Code should be self-documenting

### Adding Features

When adding new features:

1. **Design**: Think from first principles
2. **Layer**: Choose the right layer (Python/Rust/C++)
3. **Interface**: Design elegant API first
4. **Implement**: Start with tests (TDD)
5. **Document**: Update relevant docs
6. **Benchmark**: Measure performance impact

### Layer Selection

- **C++**: Only for performance-critical set operations
- **Rust**: Type system logic, AST analysis, inference
- **Python**: User-facing API, runtime validation

## Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch
3. **Write** tests for your changes
4. **Ensure** all tests pass
5. **Update** documentation
6. **Submit** PR with clear description

### PR Checklist

- [ ] Tests added/updated
- [ ] All tests passing
- [ ] Code formatted (`make format`)
- [ ] Lints passing (`make lint`)
- [ ] Documentation updated
- [ ] ARCHITECTURE.md updated (if needed)
- [ ] Examples added (if applicable)

## Performance Guidelines

### When to Optimize

- Profile first, optimize second
- Focus on algorithmic improvements
- Use SIMD only when proven beneficial
- Benchmark before and after

### Benchmarking

```bash
# Run benchmarks
pytest tests/test_benchmarks.py --benchmark-only

# Compare with baseline
pytest tests/ --benchmark-compare=baseline

# Save benchmark results
pytest tests/ --benchmark-save=baseline
```

## Documentation

### Code Documentation

- Rust: Use `///` for public APIs
- Python: Use Google-style docstrings
- C++: Use `///` for complex algorithms

### User Documentation

- Update README.md for new features
- Add examples to `examples/`
- Update ARCHITECTURE.md for design changes

## Questions?

- Open an issue for bugs
- Discussions for feature proposals
- PRs for code contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

