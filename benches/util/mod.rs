use std::{
    collections::HashSet,
    env,
    marker::PhantomData,
    time::{SystemTime, UNIX_EPOCH},
};

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use rs_graph_layout::graph_layout::GraphLayout;
use rust_sugiyama::configure::CrossingMinimization;

use crate::original_py;

pub(super) mod cube_graph_config;
pub(super) mod layered_graph_config;
pub(super) mod comm_graph_config;

static WHICH_ENV: &str = "WHICH";
static DIMS_ENV: &str = "DIMS";
static TYPE_ENV: &str = "TYPE";
static SAMPLE_SIZE_ENV: &str = "SIZE";

/// Trait that specifies funcionality needed in order to run a benchmark with the
/// [self::GraphBenchmark::run] method.
///
/// Can be implemented to add more benchmarks for different graph types.
pub(super) trait GraphBenchmarkConfig<'a>
where
    Self: std::fmt::Display + 'a,
    &'a Self: std::iter::IntoIterator<Item: std::fmt::Display + Copy>,
{
    type Error: std::fmt::Debug;

    /// Try to read in the fields of a Config via environment variables.
    fn try_from_env() -> Result<Self, Self::Error>
    where
        Self: Sized;
    /// Calculate the throughput for a benchmark. Used by [criterion::Throughput].
    fn throughput(&self, other: <&'a Self as IntoIterator>::Item) -> u64;
    /// Prepare the graph for a benchmark.
    fn prepare_graph(&self, size: <&'a Self as IntoIterator>::Item) -> (Vec<u32>, Vec<(u32, u32)>) {
        let edges = Self::prepare_edges(&self.build_graph(size));
        let vertices = Self::prepare_vertices(&edges);

        (vertices, edges)
    }
    /// build the graph used in the benchmark. 
    fn build_graph(&self, size: <&'a Self as IntoIterator>::Item) -> Vec<(usize, usize)>;

    /// prepare the edges for a benchmark (they cannot start with 0)
    fn prepare_edges(edges: &[(usize, usize)]) -> Vec<(u32, u32)> {
        edges
            .into_iter()
            .map(|(t, h)| (*t as u32 + 1, *h as u32 + 1))
            .collect()
    }

    /// calculate the vertices for a benchmark from the edges
    fn prepare_vertices(edges: &[(u32, u32)]) -> Vec<u32> {
        edges
            .iter()
            .flat_map(|(t, h)| vec![*t, *h])
            .collect::<HashSet<u32>>()
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct PythonAlgoConfig;
#[derive(Debug, Clone, Copy)]
struct RustAlgoConfig;

/// Used to configure a benchmark for a graph.
///
/// Can be configured via environment variables:
/// - [self::WHICH_ENV]: which algorithm to run. is a 3-bit number. if the first bit is set, the
/// original python implementation will be benchmarked. if the second bit is set, the rust port of
/// the original pythom implementation will be benchmarked. if the third bit is set, sugiyamas
/// algorithm will be benchmarked. it is possible to benchmark multiple alogrithms at the same
/// time. the number can be in the range from 0-7.
/// - [self::SAMPLE_SIZE_ENV]: how many samples to take for each benchmark. used to configure
/// criterions [criterion::BenchmarkGroup::sample_size] method.
///
/// See the respective graph config implementations for details on how to configure them via
/// environment variables
/// 
pub(super) struct GraphBenchmark<'a, T: GraphBenchmarkConfig<'a> + 'a>
where
    &'a T: IntoIterator<Item: Copy + std::fmt::Display>,
{
    /// Which graph to benchmark for. 
    /// Currently this is implemented for [self::cube_graph_config::CubeConfig],
    /// [self::comm_graph_config::CompGraphConfig] and
    /// [self::layered_graph_config::LayeredGraphConfig]
    graph_config: T,
    //typ: MeasurementType,
    //cube_config: CubeConfig,
    /// Do we benchmark the python version?
    python: Option<PythonAlgoConfig>,
    /// Do we benchmark the rust version?
    rust: Option<RustAlgoConfig>,
    /// Do we benchmark sugiyama?
    sugiyama: Option<rust_sugiyama::configure::Config>,
    /// Sample size for criterion
    sample_size: usize,
    _phd: &'a PhantomData<()>,
}

impl<'a, T> GraphBenchmark<'a, T>
where
    T: GraphBenchmarkConfig<'a> + 'a,
    &'a T: IntoIterator<Item: std::fmt::Display + Copy>,
{
    const WHICH_DEFAULT: usize = 7;
    const SAMPLE_SIZE_DEFAULT: usize = 100;

    pub fn from_env() -> Self {
        let (which, sample_size) = Self::read_envs();
        let graph_config = T::try_from_env().expect("Invalid config");

        let python = match which & 1 != 0 {
            true => Some(PythonAlgoConfig),
            false => None,
        };

        let rust = match which & 2 != 0 {
            true => Some(RustAlgoConfig),
            false => None,
        };

        let sugiyama = match which & 4 != 0 {
            true => Some(rust_sugiyama::configure::Config::new_from_env()),
            false => None,
        };

        Self {
            graph_config,
            python,
            rust,
            sugiyama,
            sample_size,
            _phd: &PhantomData,
        }
    }

    pub fn write_benchmark_name(&self) -> String {
        let p = self.python.map_or("", |_| "p");
        let r = self.rust.map_or("", |_| "r");
        let s = self.sugiyama.map_or("", |_| "s");
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        format!(
            "{}_{}{}{}_{}",
            self.graph_config,
            p,
            r,
            s,
            elapsed.as_secs(),
        )
    }

    pub fn bench_algos(
        &self,
        group: &mut BenchmarkGroup<'_, WallTime>,
        items: <&'a T as IntoIterator>::Item,
        vertices: Vec<u32>,
        edges: Vec<(u32, u32)>,
    ) {
        group.throughput(Throughput::Elements(self.graph_config.throughput(items)));

        if let Some(cfg) = self.sugiyama {
            let cm = match cfg.c_minimization {
                CrossingMinimization::Barycenter => "barycenter",
                CrossingMinimization::Median => "median",
            };

            let rt = match cfg.ranking_type {
                rust_sugiyama::configure::RankingType::Original => "original",
                rust_sugiyama::configure::RankingType::MinimizeEdgeLength => "minimize",
                rust_sugiyama::configure::RankingType::Up => "up",
                rust_sugiyama::configure::RankingType::Down => "down",
            };

            group.bench_with_input(
                BenchmarkId::new(format!("Sugiyama-{}-{}-{}", rt, cm, cfg.transpose), items),
                &items,
                |b, _| b.iter(|| rust_sugiyama::from_edges(&edges).with_config(cfg).build()),
            );
        }

        if let Some(_) = self.rust {
            group.bench_with_input(BenchmarkId::new("Original_rs", items), &items, |b, _| {
                b.iter(|| GraphLayout::create_layers(&vertices, &edges, 40, false))
            });
        }

        if let Some(_) = self.python {
            group.bench_with_input(BenchmarkId::new("Original_py", items), &items, |b, _| {
                b.iter(|| original_py::graph_layout(edges.clone()))
            });
        }
    }

    fn read_envs() -> (usize, usize) {
        // from, to, layers/dims, step_py
        let which = env::var(WHICH_ENV)
            .map_or(Ok(Self::WHICH_DEFAULT), |s| s.parse::<usize>())
            .expect("$WHICH set to non numeric value");
        let sample_size = env::var(SAMPLE_SIZE_ENV)
            .map_or(Ok(Self::SAMPLE_SIZE_DEFAULT), |s| s.parse::<usize>())
            .expect("$WHICH set to non numeric value");
        (which, sample_size) //, typ, cube_config)
    }

    /// Run a benchmark
    pub(crate) fn run(&'a self, c: &mut Criterion) {
        let s = format!("{}", self.write_benchmark_name());
        let mut group = c.benchmark_group(s);
        group.sample_size(self.sample_size);

        for dim in &self.graph_config {
            let (vertices, edges) = self.graph_config.prepare_graph(dim);
            self.bench_algos(&mut group, dim, vertices, edges);
        }

        group.finish();
    }
}
