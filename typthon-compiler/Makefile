.PHONY: build test clean install

# Build compiler
build:
	@mkdir -p bin
	go build -o bin/typthon ./cmd/typthon

# Build with optimization
release:
	@mkdir -p bin
	go build -ldflags="-s -w" -o bin/typthon ./cmd/typthon

# Run tests
test:
	go test ./...

# Clean build artifacts
clean:
	rm -rf bin/
	go clean

# Install to PATH
install: release
	cp bin/typthon /usr/local/bin/

# Development build with race detector
dev:
	go build -race -o bin/typthon ./cmd/typthon

# Format code
fmt:
	go fmt ./...

# Run linter
lint:
	golangci-lint run

