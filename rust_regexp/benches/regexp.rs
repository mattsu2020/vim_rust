use criterion::{black_box, criterion_group, criterion_main, Criterion};
use regex::Regex;
use rust_regexp::search;

fn bench_search(c: &mut Criterion) {
    let pat = b"foo.*bar";
    let text = b"foo something bar";
    c.bench_function("rust_regexp", |b| {
        b.iter(|| search(black_box(pat), black_box(text), false))
    });

    let re = Regex::new("foo.*bar").unwrap();
    c.bench_function("regex_crate", |b| {
        b.iter(|| re.is_match(black_box("foo something bar")))
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
