use criterion::{criterion_group, criterion_main, Criterion};
use norad::Font;
use tempfile::tempdir;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read & parse Roboto-Regular.ufo", |b| {
        b.iter(|| Font::load("testdata/Roboto-Regular.ufo").expect("font should load"));
    });
    let roboto_regular = Font::load("testdata/Roboto-Regular.ufo").unwrap();
    c.bench_function("write Roboto-Regular.ufo", |b| {
        b.iter_with_large_drop(|| {
            let write_dir = tempdir().unwrap();
            roboto_regular.save(write_dir.path()).expect("font should save");
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
