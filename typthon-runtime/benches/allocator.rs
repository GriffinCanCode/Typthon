use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_allocation(c: &mut Criterion) {
    c.bench_function("alloc_16bytes", |b| {
        b.iter(|| {
            // TODO: Implement once allocator is ready
            black_box(16);
        });
    });
}

criterion_group!(benches, bench_allocation);
criterion_main!(benches);

