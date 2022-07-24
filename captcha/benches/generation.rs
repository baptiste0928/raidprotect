use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raidprotect_captcha::generate_captcha;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("captcha with 5 letters", |b| {
        b.iter(|| generate_captcha(black_box("ABCDE")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
