//! Contains all the benchmarks for the two algorithms
mod original_py;
mod util;

use criterion::{criterion_group, criterion_main, Criterion};
use criterion::{BenchmarkId, Throughput};
use graph_generator::comm::{self, comp_graph};
use graph_generator::layered_random::LayeredRandomGraph;
use rust_sugiyama::CrossingMinimization;

const SEED: u128 = 12345;

pub fn benchmark_no_crossings(c: &mut Criterion) {
    pyo3::prepare_freethreaded_python();
    let layers_range = (2..21).skip(1).step_by(2);

    let mut group = c.benchmark_group("layered_random_no_crossings_2_to_20");

    for n_layers in layers_range {
        let (edges, vertices) = util::prepare_graph(
            LayeredRandomGraph::new(n_layers)
                .with_seed(SEED)
                .with_degree(2)
                .build()
        );

        util::bench_algos(
            &mut group, 
            n_layers,
            (vertices.len() + edges.len()) as u64,
            edges, 
            vertices,
            7
        );
    }
    group.finish()
}

pub fn bench_comp_graph_change_layers(c: &mut Criterion) {
    pyo3::prepare_freethreaded_python();
    let mut group = c.benchmark_group("comp_graph_2_to_20");

    for n_layers in 2..20 {
        let (edges, vertices) = util::prepare_graph(comp_graph(10, 5, n_layers));
        util::bench_algos(&mut group, n_layers, n_layers as u64, edges, vertices, 7);
    }
    group.finish();
}

pub fn bench_comp_graph_sugiyama_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("comp_graph_2_to_20_sugiyama_cm_comparison");

    for n_layers in 2..20 {
        let (edges, vertices) = util::prepare_graph(comp_graph(10, 5, n_layers));

        group.throughput(Throughput::Elements(
            vertices.len() as u64 + edges.len() as u64,
        ));
        group.bench_with_input(BenchmarkId::new("median", n_layers), &n_layers, |b, _| {
            b.iter(|| {
                rust_sugiyama::from_edges(&edges)
                    .crossing_minimization(CrossingMinimization::Median)
                    .build()
            })
        });

        group.bench_with_input(
            BenchmarkId::new("Barycenter", n_layers),
            &n_layers,
            |b, _| {
                b.iter(|| {
                    rust_sugiyama::from_edges(&edges)
                        .crossing_minimization(CrossingMinimization::Barycenter)
                        .build()
                })
            },
        );
    }
    group.finish();
}

pub fn bench_comp_graph_change_nodes(c: &mut Criterion) {
    pyo3::prepare_freethreaded_python();
    let mut group = c.benchmark_group("comp_graph_nodes");

    for nodes in (12..63).step_by(10) {
        let (edges, vertices) = util::prepare_graph(comp_graph(nodes, nodes / 10, 6));

        util::bench_algos(
            &mut group, 
            nodes, 
            vertices.len() as u64 + edges.len() as u64, 
            edges, 
            vertices,
            7
        );
    }
    group.finish();
}

pub fn bench_cube_graph_change_dims(c: &mut Criterion) {
    //pyo3::prepare_freethreaded_python();
    let mut group = c.benchmark_group("cube_graph_dims_4_timesteps26");
    let timesteps = 6;

    for dims in 3..9 {
        let (edges, vertices) = util::prepare_graph(comm::CubeGraph::new(dims, dims, dims, timesteps).build());

        util::bench_algos(
            &mut group,
            dims,
            (dims * dims * dims) as u64,
            edges,
            vertices,
            3
        );
    }
}

pub fn bench_cube_graph_change_timesteps(c: &mut Criterion) {

}

criterion_group!(no_crossings_2_21, benchmark_no_crossings);
criterion_group!(comp_graph_2_20, bench_comp_graph_change_layers);
criterion_group!(comp_sugiyama_only, bench_comp_graph_sugiyama_only);
criterion_group!(comp_nodes, bench_comp_graph_change_nodes);
criterion_group!(cube_dims, bench_cube_graph_change_dims);
criterion_group!(cube_dims2, bench_cube_graph_change_dims);
criterion_main!(cube_dims);
