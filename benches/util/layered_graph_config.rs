use std::{env, fmt::Display, iter::StepBy, num::ParseIntError, ops::Range};

use graph_generator::layered_random;

use super::{GraphBenchmarkConfig, DIMS_ENV, TYPE_ENV};

const SEED: u128 = 12345;

/// ## Description
///
/// What to measure for. 
/// Can be set via the [super::DIMS_ENV] environment variable.
/// Valid values are: `'layers'` and `'random'`.
#[derive(Debug, Clone, Copy)]
enum MeasurmentType {
    /// Change the amount of layers for the graph with each benchmark
    Layers,
    /// Change the amount of randomly added edges with each benchmark
    RandomEdges,
}

impl Display for MeasurmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MeasurmentType::Layers => "layers",
            MeasurmentType::RandomEdges => "random",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for MeasurmentType {
    type Error = LayeredGraphConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "layers" => Ok(Self::Layers),
            "random" => Ok(Self::RandomEdges),
            other => Err(LayeredGraphConfigError::InvalidMeasurementType(format!(
                "Unknown measurment type for layered graph: {other}"
            ))),
        }
    }
}

impl TryFrom<String> for MeasurmentType {
    type Error = LayeredGraphConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// ## Description
/// Used to configure a [graph_generator::layered_random] for a benchmark.
///
/// ## Environment Variables
///
/// It can be configured via environment variables when running the benchmark.
/// These are as following: 
/// - [super::DIMS_ENV] has the form of `from-to-step_by-degree-fixed_param`. needs to contain
/// numeric values, used to configure the range of values for the benchmark.
/// - [super::TYPE_ENV] what to benchmark for. See [self::MeasurementType]
///
/// ## Example
///
/// As an example, configuring the config with [super::DIMS_ENV] `2-10-1-3-5` and [super::TYPE_ENV]
/// `layers`, will run a benchmark for graphs with 2 to 10 layers, with outgoing degree of 3,
/// adding 5 random edges every time.
pub(crate) struct LayeredGraphConfig {
    /// What thing to measure for. See [self::MeasurementType]
    typ: MeasurmentType,
    /// start range
    from: usize,
    /// end range
    to: usize,
    /// the degree of a vertices outgoing edge
    degree: usize,
    /// how to advance the range
    step_by: usize,
    /// What no to measure for. If measuring for Layers, this is set to random vertices,
    /// when measuring for random edges this is set to layers.
    fixed_param: usize,
}

#[derive(Debug)]
pub(crate) enum LayeredGraphConfigError {
    InvalidConfigurationString(String),
    InvalidMeasurementType(String),
}

impl Display for LayeredGraphConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            LayeredGraphConfigError::InvalidConfigurationString(s) => s,
            LayeredGraphConfigError::InvalidMeasurementType(s) => s,
        };
        write!(f, "{err_msg}")
    }
}

impl From<ParseIntError> for LayeredGraphConfigError {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidConfigurationString(format!("Invalid configurations string: {}", err))
    }
}

impl<'a> IntoIterator for &'a LayeredGraphConfig {
    type Item = usize;
    type IntoIter = StepBy<Range<Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.from..self.to).step_by(self.step_by)
    }
}

impl Display for LayeredGraphConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}-{}-{}-{}", self.typ, self.from, self.to, self.step_by, self.degree, self.fixed_param)
    }
}

impl<'a> GraphBenchmarkConfig<'a> for LayeredGraphConfig {
    type Error = LayeredGraphConfigError;

    fn try_from_env() -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        const TYPE_DEFAULT: MeasurmentType = MeasurmentType::Layers;

        let typ = env::var(TYPE_ENV).map_or(Ok(TYPE_DEFAULT), MeasurmentType::try_from)?;
        let config = env::var(DIMS_ENV)
            .unwrap_or("5-10-1-2-0".to_string())
            .split('-')
            .map(<str>::parse)
            .collect::<Result<Vec<usize>, ParseIntError>>()?;

        if config.len() != 5 {
            Err(LayeredGraphConfigError::InvalidConfigurationString(
                "configuration string needs to be in the form of: from-to-step_by-dims-fixed_param"
                    .to_string(),
            ))
        } else {
            let config = Self {
                typ,
                from: config[0],
                to: config[1],
                step_by: config[2],
                degree: config[3],
                fixed_param: config[4],
            };

            Ok(config)
        }
    }

    fn throughput(&self, other: <&'_ Self as IntoIterator>::Item) -> u64 {
        self.build_graph(other).len() as u64     
    }

    fn build_graph(&self, size: <&'_ Self as IntoIterator>::Item) -> Vec<(usize, usize)> {
        let (layers, random_edges) = match self.typ {
            MeasurmentType::Layers => (size, self.fixed_param),
            MeasurmentType::RandomEdges => (self.fixed_param, size),
        };

        let mut g = layered_random::LayeredRandomGraph::new(layers).with_seed(SEED).with_degree(self.degree);
        for _ in 0..random_edges {
            g = g.add_random_edge();
        }
        g.build()
    }
}
