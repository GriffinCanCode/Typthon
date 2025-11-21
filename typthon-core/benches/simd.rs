//! SIMD optimization benchmarks
//!
//! Measures performance of SIMD vs scalar type operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use typthon::Type;

fn generate_types(n: usize) -> Vec<Type> {
    (0..n).map(|i| match i % 10 {
        0 => Type::Int,
        1 => Type::Float,
        2 => Type::Str,
        3 => Type::Bool,
        4 => Type::List(Box::new(Type::Int)),
        5 => Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        6 => Type::Set(Box::new(Type::Float)),
        7 => Type::Tuple(vec![Type::Int, Type::Str]),
        8 => Type::Function(vec![Type::Int], Box::new(Type::Str)),
        _ => Type::Class(format!("Class{}", i)),
    }).collect()
}

fn generate_distinct_types(n: usize) -> Vec<Type> {
    (0..n).map(|i| Type::Class(format!("Type{}", i))).collect()
}

fn bench_union_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("union");

    // Test small unions (Rust path)
    for size in [2, 5, 9].iter() {
        group.bench_with_input(
            BenchmarkId::new("small", size),
            size,
            |b, &size| {
                let types = generate_types(size);
                b.iter(|| {
                    Type::union(black_box(types.clone()))
                });
            }
        );
    }

    // Test large unions (SIMD path)
    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large", size),
            size,
            |b, &size| {
                let types = generate_distinct_types(size);
                b.iter(|| {
                    Type::union(black_box(types.clone()))
                });
            }
        );
    }

    group.finish();
}

fn bench_intersection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("intersection");

    // Test small intersections (Rust path)
    for size in [2, 5, 9].iter() {
        group.bench_with_input(
            BenchmarkId::new("small", size),
            size,
            |b, &size| {
                let types = generate_types(size);
                b.iter(|| {
                    Type::intersection(black_box(types.clone()))
                });
            }
        );
    }

    // Test large intersections (SIMD path)
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("large", size),
            size,
            |b, &size| {
                let types = generate_distinct_types(size);
                b.iter(|| {
                    Type::intersection(black_box(types.clone()))
                });
            }
        );
    }

    group.finish();
}

fn bench_subtype_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("subtype");

    let types = vec![
        (Type::Int, Type::Any, "int_any"),
        (Type::List(Box::new(Type::Int)), Type::List(Box::new(Type::Any)), "list_covariant"),
        (
            Type::Function(vec![Type::Any], Box::new(Type::Int)),
            Type::Function(vec![Type::Int], Box::new(Type::Any)),
            "function_variance"
        ),
    ];

    for (sub, sup, name) in types {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(sub.is_subtype(&sup))
            });
        });
    }

    group.finish();
}

fn bench_complex_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex");

    // Nested generic types
    let nested = Type::Dict(
        Box::new(Type::Str),
        Box::new(Type::List(Box::new(Type::Tuple(vec![
            Type::Int,
            Type::Float,
            Type::Str,
        ]))))
    );

    group.bench_function("nested_display", |b| {
        b.iter(|| {
            black_box(nested.to_string())
        });
    });

    group.bench_function("nested_clone", |b| {
        b.iter(|| {
            black_box(nested.clone())
        });
    });

    group.finish();
}

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_comparison");

    // Direct comparison at SIMD threshold
    let threshold_types = generate_distinct_types(10);

    group.bench_function("threshold_10_types", |b| {
        b.iter(|| {
            Type::union(black_box(threshold_types.clone()))
        });
    });

    // Massive union to showcase SIMD advantage
    let massive = generate_distinct_types(1000);
    group.bench_function("massive_1000_types", |b| {
        b.iter(|| {
            Type::union(black_box(massive.clone()))
        });
    });

    group.finish();
}

fn bench_type_interning(c: &mut Criterion) {
    use typthon::core::intern;

    let mut group = c.benchmark_group("interning");

    let types = generate_distinct_types(100);

    group.bench_function("intern_100_types", |b| {
        b.iter(|| {
            for ty in &types {
                black_box(intern::intern(ty.clone()));
            }
        });
    });

    // Benchmark repeated interning (should hit cache)
    group.bench_function("intern_cached", |b| {
        // Pre-intern
        for ty in &types {
            intern::intern(ty.clone());
        }

        b.iter(|| {
            for ty in &types {
                black_box(intern::intern(ty.clone()));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_union_operations,
    bench_intersection_operations,
    bench_subtype_checks,
    bench_complex_types,
    bench_simd_vs_scalar,
    bench_type_interning,
);
criterion_main!(benches);

