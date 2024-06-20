//! Contains all the benchmarks for the two algorithms
mod original_py;
mod util;

use criterion::{criterion_group, criterion_main, Criterion};
use util::{comm_graph_config::CompGraphConfig, cube_graph_config::CubeConfig, GraphBenchmark};

use crate::util::layered_graph_config::LayeredGraphConfig;

pub fn bench_comm_graph(c: &mut Criterion) {
    let benchmark = GraphBenchmark::<CompGraphConfig>::from_env(); 
    benchmark.run(c);
}

pub fn bench_cube_graph(c: &mut Criterion) {
    let benchmark = GraphBenchmark::<CubeConfig>::from_env();
    benchmark.run(c);
}

pub fn bench_layered_graph(c: &mut Criterion) {
    let benchmark = GraphBenchmark::<LayeredGraphConfig>::from_env();
    benchmark.run(c);
}

criterion_group!(layered, bench_layered_graph);
criterion_group!(cube, bench_cube_graph);
criterion_group!(comm, bench_comm_graph);
criterion_main!(layered);
