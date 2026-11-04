use criterion::{criterion_group, criterion_main, Criterion};
use luamin::TryFromLuamin;
use script_bench::luamin::RustData;

fn benchmark(c: &mut Criterion) {
    script_bench::luamin::sort_userdata(
        |func| {
            c.bench_function("Sort Rust objects", |b| b.iter(|| func()));
        },
        |array| {
            assert_eq!(array.len(), 10_000);
            let mut prev = None;
            for value in array {
                let next = RustData::try_from_luamin(value).expect("Expected RustData");
                assert!((8..24).contains(&next.0.len()));
                assert!(next.0.bytes().all(|byte| byte.is_ascii_hexdigit()));
                if let Some(prev) = prev {
                    assert!(prev <= next);
                }
                prev = Some(next);
            }
        },
    )
    .unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark,
}

criterion_main!(benches);
