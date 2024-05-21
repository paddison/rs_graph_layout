use std::collections::HashSet;

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId, Throughput};
use rs_graph_layout::graph_layout::GraphLayout;

use crate::original_py;

pub(super) mod graph_generators;
pub(super) mod lcg;


pub(crate) fn bench_algos(
    group: &mut BenchmarkGroup<'_, WallTime>,
    items: usize,
    throughput: u64,
    edges: Vec<(u32, u32)>,
    vertices: Vec<u32>,
    which: usize,
) {
    group.throughput(Throughput::Elements(throughput));

    if which & 4 != 0 {
        group.bench_with_input(BenchmarkId::new("Sugiyama", items), &items, |b, _| {
            b.iter(|| rust_sugiyama::from_edges(&edges).build())
        });
    }

    if which & 2  != 0 {
        group.bench_with_input(BenchmarkId::new("Original_rs", items), &items, |b, _| {
            b.iter(|| GraphLayout::create_layers(&vertices, &edges, 40, false))
        });
    }

    if which & 1 != 0 {
        group.bench_with_input(BenchmarkId::new("Original_py", items), &items, |b, _| {
            b.iter(|| original_py::graph_layout(edges.clone()))
        });
    }
}

fn get_vertices(edges: &[(u32, u32)]) -> Vec<u32> {
    edges
        .iter()
        .flat_map(|(t, h)| vec![*t, *h])
        .collect::<HashSet<u32>>()
        .into_iter()
        .collect::<Vec<u32>>()
}

fn increment_edges(edges: Vec<(usize, usize)>) -> Vec<(u32, u32)> {
    edges
        .into_iter()
        .map(|(t, h)| (t as u32 + 1, h as u32 + 1))
        .collect::<Vec<(u32, u32)>>()
}

pub(crate) fn prepare_graph(edges: Vec<(usize, usize)>) -> (Vec<(u32, u32)>, Vec<u32>) {
    let edges = increment_edges(edges);
    let vertices = get_vertices(&edges);
    (edges, vertices)
}
