use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddsketch_rs::DDSketch;

fn benchmark_add_values(c: &mut Criterion) {
    c.bench_function("add_1000_values", |b| {
        b.iter(|| {
            let mut sketch = DDSketch::new(0.02).unwrap();
            for i in 1..=1000 {
                sketch.add(black_box(i as f64));
            }
        })
    });
}

fn benchmark_quantile_queries(c: &mut Criterion) {
    let mut sketch = DDSketch::new(0.02).unwrap();
    for i in 1..=10000 {
        sketch.add(i as f64);
    }
    
    c.bench_function("quantile_queries", |b| {
        b.iter(|| {
            let _ = sketch.get_quantile_value(black_box(0.5));
            let _ = sketch.get_quantile_value(black_box(0.9));
            let _ = sketch.get_quantile_value(black_box(0.99));
        })
    });
}

fn benchmark_merge(c: &mut Criterion) {
    let mut sketch1 = DDSketch::new(0.02).unwrap();
    let mut sketch2 = DDSketch::new(0.02).unwrap();
    
    for i in 1..=5000 {
        sketch1.add(i as f64);
        sketch2.add((i + 5000) as f64);
    }
    
    c.bench_function("merge_sketches", |b| {
        b.iter(|| {
            let mut s1 = sketch1.clone();
            let s2 = sketch2.clone();
            s1.merge(&s2).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_add_values, benchmark_quantile_queries, benchmark_merge);
criterion_main!(benches);
