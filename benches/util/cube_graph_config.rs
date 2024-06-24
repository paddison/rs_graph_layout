/*
 AYUDAME/TEMANEJO toolset
--------------------------

 (C) 2024, HLRS, University of Stuttgart
 All rights reserved.
 This software is published under the terms of the BSD license:

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the <organization> nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use std::{env, error::Error, fmt::Display, iter::StepBy, num::ParseIntError, ops::Range};

use graph_generator::comm::CubeGraph;

use super::GraphBenchmarkConfig;

/// ## Description
/// Used to configure a [graph_generator::comm::CubeGraph] for a benchmark.
///
/// ## Environment Variables
///
/// It can be configured via environment variables when running the benchmark.
/// These are as following: 
/// - [super::DIMS_ENV] has the form of `from-to-step_by-fixed_param`. needs to contain
/// numeric values, used to configure the range of values for the benchmark.
/// - [super::TYPE_ENV] what to benchmark for. See [self::MeasurementType]
///
/// ## Example
///
/// As an example, configuring the config with [super::DIMS_ENV] `2-10-1-5` and [super::TYPE_ENV]
/// `dims`, will run a benchmark for cubes with a sidelength of 2 to 10, for 5 timesteps each time
pub(crate) struct CubeConfig {
    typ: MeasurementType,
    from: usize,
    to: usize,
    step_by: usize,
    /// Set to timesteps, if measuring Dims, and vice versa
    fixed_param: usize,
}

impl Display for CubeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}-{}",
            self.typ, self.from, self.to, self.step_by, self.fixed_param
        )
    }
}

impl<'a> IntoIterator for &'a CubeConfig {
    type Item = usize;
    type IntoIter = StepBy<Range<Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.from..self.to).step_by(self.step_by)
    }
}

impl<'a> GraphBenchmarkConfig<'a> for CubeConfig {
    type Error = CubeConfigError;

    fn try_from_env() -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        const CUBE_CONFIG_STRING_DEFAULT: &'static str = "3-5-1-4";
        const TYPE_DEFAULT: MeasurementType = MeasurementType::Dims;

        let typ = env::var(super::TYPE_ENV).map_or(Ok(TYPE_DEFAULT), MeasurementType::try_from)?;

        let cube_config = env::var(super::DIMS_ENV)
            .unwrap_or(CUBE_CONFIG_STRING_DEFAULT.to_string())
            .split("-")
            .map(|n| n.parse::<usize>())
            .collect::<Result<Vec<_>, ParseIntError>>()?;

        if cube_config.len() != 4 {
            Err(CubeConfigError::InvalidConfigurationString(
                "Cube Configuration string needs to be: from-to-step_by-fixed_param".to_string(),
            ))
        } else {
            let config = Self {
                typ,
                from: cube_config[0],
                to: cube_config[1],
                step_by: cube_config[2],
                fixed_param: cube_config[3],
            };
            Ok(config)
        }
    }

    fn throughput(&self, other: usize) -> u64 {
        (match self.typ {
            MeasurementType::Dims => other * other * other * self.fixed_param,
            MeasurementType::Timesteps => self.fixed_param * self.fixed_param * self.fixed_param * other,
        }) as u64
        //other as u64 * 3 * self.fixed_param as u64
    }

    fn build_graph(&self, size: usize) -> Vec<(usize, usize)> {
        let fixed = self.fixed_param;
        match self.typ {
            MeasurementType::Dims => CubeGraph::new(size, size, size, fixed),
            MeasurementType::Timesteps => CubeGraph::new(fixed, fixed, fixed, size),
        }
        .build()
    }
}

/// What to measure for.
/// Can be configured by setting the [super::TYPE_ENV] environment variable.
/// Permitted values are: `'dims'` and `'timesteps'`.
enum MeasurementType {
    /// Measure for a change in dimensionality for the cube
    Dims,
    /// Measure for a change in timesteps for the cube
    Timesteps,
}

impl std::fmt::Display for MeasurementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Dims => "dim",
            Self::Timesteps => "timesteps",
        };

        write!(f, "{name}")
    }
}

impl TryFrom<&str> for MeasurementType {
    type Error = CubeConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dims" => Ok(Self::Dims),
            "timesteps" => Ok(Self::Timesteps),
            other => Err(CubeConfigError::UnknownMeasurementType(format!(
                "Invalid value for Measurement Type: {other}"
            ))),
        }
    }
}

impl TryFrom<String> for MeasurementType {
    type Error = CubeConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CubeConfigError {
    UnknownMeasurementType(String),
    InvalidConfigurationString(String),
}

impl std::fmt::Display for CubeConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            CubeConfigError::UnknownMeasurementType(s) => s,
            CubeConfigError::InvalidConfigurationString(s) => s,
        };
        write!(f, "{err_msg}")
    }
}

impl From<ParseIntError> for CubeConfigError {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidConfigurationString(err.to_string())
    }
}

impl Error for CubeConfigError {}
