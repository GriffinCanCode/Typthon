# Typthon Performance Scripts

## Profile-Guided Optimization (PGO)

Profile-Guided Optimization improves performance by using runtime profiling data to guide compiler optimizations.

### Usage

```bash
./scripts/profile.sh
```

This script will:
1. Build Typthon with instrumentation
2. Run representative workloads to collect profiles
3. Merge profile data
4. Rebuild with profile-guided optimizations

### Expected Improvements

- 10-20% improvement on hot paths
- Better branch prediction
- Improved inlining decisions
- Optimized code layout

### Requirements

- Rust toolchain (rustc, cargo)
- llvm-profdata (for merging profiles)
  - macOS: `brew install llvm`
  - Linux: `apt install llvm`

## Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

Available benchmark suites:
- `simd`: SIMD type operation benchmarks
- `incremental`: Incremental checking benchmarks

## Performance Testing

Run performance tests:

```bash
python3 -m pytest tests/test_performance.py -v
```

## Monitoring

View performance metrics during operation:

```rust
use typthon::PerformanceMetrics;

let metrics = PerformanceMetrics::new();
// ... perform operations ...
let summary = metrics.summary();
println!("{}", summary.report());
```

