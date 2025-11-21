.PHONY: build install test clean dev lint format cli check-cli cicd-dryrun cicd-test

# Build the package
build:
	maturin build --release

# Build CLI binary
cli:
	cargo build --release --bin typthon

# Install in development mode
dev:
	maturin develop

# Install the built wheel
install: build
	pip install --force-reinstall target/wheels/*.whl

# Run CLI on project (requires building)
check-cli: cli
	./target/release/typthon python/typthon/

# Run tests
test:
	pytest tests/ -v

# Run benchmarks
bench:
	pytest tests/ --benchmark-only

# Clean build artifacts
clean:
	rm -rf target/
	rm -rf build/
	rm -rf dist/
	rm -rf python/typthon.egg-info/
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.so" -delete

# Lint code
lint:
	ruff check python/
	cargo clippy -- -D warnings

# Format code
format:
	ruff format python/
	cargo fmt

# Type check Python code
typecheck:
	mypy python/typthon/

# All checks before commit
check: lint typecheck test

# Build documentation
docs:
	@echo "Documentation generation not yet configured"

# Publish to PyPI
publish: build
	maturin publish

# CI/CD dry run (requires Docker)
cicd-dryrun:
	act --container-architecture linux/amd64 -j test --matrix os:ubuntu-latest --matrix python-version:3.10 --dryrun

# CI/CD local test (requires Docker)
cicd-test:
	act --container-architecture linux/amd64 -j test --matrix os:ubuntu-latest --matrix python-version:3.10

