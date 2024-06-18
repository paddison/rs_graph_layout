use std::{env, fmt::Display, iter::StepBy, num::ParseIntError, ops::Range};

use graph_generator::layered_random;

use super::{GraphBenchmarkConfig, DIMS_ENV, TYPE_ENV};

const SEED: u128 = 12345;

#[derive(Debug, Clone, Copy)]
enum MeasurmentType {
    Layers,
    RandomVertices,
}

impl Display for MeasurmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MeasurmentType::Layers => "layers",
            MeasurmentType::RandomVertices => "random",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for MeasurmentType {
    type Error = LayeredGraphConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "layers" => Ok(Self::Layers),
            "random" => Ok(Self::RandomVertices),
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

pub(crate) struct LayeredGraphConfig {
    typ: MeasurmentType,
    from: usize,
    to: usize,
    degree: usize,
    step_by: usize,
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
    type Item = <<LayeredGraphConfig as GraphBenchmarkConfig<'a>>::Iter as IntoIterator>::Item;
    type IntoIter = <LayeredGraphConfig as GraphBenchmarkConfig<'a>>::Iter;

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
    type Iter = StepBy<Range<usize>>;

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
            MeasurmentType::RandomVertices => (self.fixed_param, size),
        };

        let mut g = layered_random::LayeredRandomGraph::new(layers).with_seed(SEED).with_degree(self.degree);
        for _ in 0..random_edges {
            g = g.add_random_edge();
        }
        g.build()
    }
}
