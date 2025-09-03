use criterion::{criterion_group, criterion_main, Criterion};
use rust_cindent::rs_compute_indent;

fn bench_compute_indent(c: &mut Criterion) {
    c.bench_function("compute_indent", |b| b.iter(|| rs_compute_indent(10)));
}

criterion_group!(benches, bench_compute_indent);
criterion_main!(benches);
