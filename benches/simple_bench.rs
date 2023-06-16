use std::collections::{hash_map::RandomState, HashMap};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smolmap::SmolMap;

fn criterion_benchmark(c: &mut Criterion) {
    let mut smol: SmolMap<_, _, 100000> = SmolMap::new(RandomState::new());
    let mut map = HashMap::with_capacity(100000);

    for i in 0..100000 {
        smol.insert(i, i);
        map.insert(i, i);
    }

    c.bench_function("smolmap get 100000", |b| {
        b.iter(|| smol.get(black_box(&50000)))
    });
    c.bench_function("hashmap get 100000", |b| {
        b.iter(|| map.get(black_box(&50000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
