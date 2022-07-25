use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raidprotect_captcha::{generate_captcha, generate_captcha_png};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("captcha with 6 letters", |b| {
        b.iter(|| generate_captcha(black_box("ABCDEF")))
    });

    c.bench_function("captcha with 6 letters as png", |b| {
        b.iter(|| generate_captcha_png(black_box("ABCDEF")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
