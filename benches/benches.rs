//! Contains all the benchmarks for the two algorithms
mod util;

use std::collections::HashSet;
use criterion::{criterion_group, criterion_main, Criterion};
use rs_graph_layout::{SugiyamaConfig, create_layouts_original, create_layouts_sugiyama};
use util::graph_generators::LayeredGraphGenerator;

pub fn benchmark_test(c: &mut Criterion) {
    let edges = LayeredGraphGenerator::new(10)
        .with_seed(123456)
        .with_degree(2)
        .add_random_edges(6)
        .build().iter().map(|(t, h)| (*t as u32, *h as u32)).collect::<Vec<(u32, u32)>>();

    let vertices = edges.iter().flat_map(|(t, h)| vec![*t, *h]).collect::<HashSet<u32>>().into_iter().collect::<Vec<u32>>();
    c.bench_function("original 10", |b| b.iter(|| create_layouts_original(vertices.clone(), edges.clone(), 40, false)));
    c.bench_function("sugiyama 10", |b| b.iter(|| create_layouts_sugiyama(vertices.clone(), edges.clone(), SugiyamaConfig::default())));
}

criterion_group!(bench, benchmark_test);
criterion_main!(bench);
