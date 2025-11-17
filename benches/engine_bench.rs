use criterion::{criterion_group, criterion_main, Criterion};
use aegisr_engine::{AegFileSystem, AegCore as EngineCore};
fn benchmark_put_get(c: &mut Criterion) {
    // Ensure clean environment
    AegFileSystem::reset_files();
    AegFileSystem::initialize_config(None, None);

    let key = "test_key";
    let value = "test_value";
    c.bench_function("put_value", |b| {
        b.iter(|| {
            // repeatedly put the same key/value
            EngineCore::put_value(key, value);
        })
    });

    EngineCore::put_value(key, value);
    c.bench_function("get_value", |b| {
        b.iter(|| {
            let _ = EngineCore::get_value(key);
        })
    });
}

criterion_group!(benches, benchmark_put_get);
criterion_main!(benches);
