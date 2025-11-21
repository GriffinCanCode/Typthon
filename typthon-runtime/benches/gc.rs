use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_refcount(c: &mut Criterion) {
    c.bench_function("incref", |b| {
        b.iter(|| {
            // TODO: Implement once GC is ready
            black_box(1);
        });
    });
}

criterion_group!(benches, bench_refcount);
criterion_main!(benches);

