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

use std::{env, error::Error, fmt::Display, iter::StepBy, num::{ParseFloatError, ParseIntError}, ops::Range};

use graph_generator::comm::comp_graph;

use crate::util::TYPE_ENV;

use super::{GraphBenchmarkConfig, DIMS_ENV};

#[derive(Debug)]
pub(crate) enum CommGraphConfigError {
    InvalidConfigurationString(String),
    InvalidMeasurementType(String),
    ParseError(String),
}

impl Display for CommGraphConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CommGraphConfigError::InvalidConfigurationString(s) => s,
            CommGraphConfigError::InvalidMeasurementType(s) => s,
            CommGraphConfigError::ParseError(s) => s,
        };
        write!(f, "{s}")
    }
}

impl Error for CommGraphConfigError {}

impl From<ParseIntError> for CommGraphConfigError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(format!("{}", value))
    }
}

impl From<ParseFloatError> for CommGraphConfigError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseError(format!("{}", value))
    }
}

/// What to measure for.
///
/// Can be configured by setting the environment variable [super::TYPE_ENV].
///
/// Permitted values are: `'timesteps-n-m'`, `'inside-n-m'`, `'outside-n-m'` and `'ratio-n-m'`.
/// where n and m are numbers
#[derive(Debug)]
enum MeasurementType {
    /// Measure for a change in timesteps. The first field is the number of inside nodes, the second
    /// the number of outside nodes
    Timesteps(usize, usize),
    /// Measure for a change in inside nodes. The first field is the number of outside nodes, the
    /// second the number of timesteps
    Inside(usize, usize),
    /// Measure for a change in outside nodes. The first field is the number of inside nodes, the
    /// second the number of timesteps
    Outside(usize, usize),
    /// Measure for a change in notes in general. The first field is the ratio inside/outside
    /// nodes, the second the number of timesteps
    Ratio(f64, usize),
}

impl Display for MeasurementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MeasurementType::Timesteps(inside, outside) => format!("timesteps-r{}-b{}", inside, outside),
            MeasurementType::Inside(outside, layers) => format!("inside-r{}-l{}", outside, layers),
            MeasurementType::Outside(inside, layers) => format!("outside-b{}-l{}", inside, layers),
            MeasurementType::Ratio(ratio, layers) => format!("ratio-r{}-l{}", ratio, layers),
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for MeasurementType {
    type Error = CommGraphConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts = value.split('-').collect::<Vec<_>>();
        if parts.len() != 3 {
            Err(CommGraphConfigError::InvalidMeasurementType("Format for measurement type: type-n-m".into()))
        } else {
            match parts[0] {
                "ratio" => {
                    let ratio = parts[1].parse::<f64>()?;
                    let layers = parts[2].parse::<usize>()?;
                    Ok(Self::Ratio(ratio, layers))
                },
                other => {
                    let params = parts[1..]
                        .iter()
                        .map(|s| s.parse::<usize>())
                        .collect::<Result<Vec<_>, _>>()?;
                    match other {
                        "layers" => Ok(Self::Timesteps(params[1], params[2])),
                        "inside" => Ok(Self::Inside(params[1], params[2])),
                        "outside" => Ok(Self::Outside(params[1], params[2])),
                        invalid => Err(CommGraphConfigError::InvalidMeasurementType(format!("Invalid name for measurement type: {}", invalid)))
                    }
                }
            }
        }
    }
}

impl TryFrom<String> for MeasurementType {
    type Error = CommGraphConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// ## Description
/// Used to configure a [graph_generator::comm::comp_graph] for a benchmark.
///
/// ## Environment Variables
///
/// It can be configured via environment variables when running the benchmark.
/// These are as following: 
/// - [super::DIMS_ENV] has the form of `from-to-step_by`. needs to contain
/// numeric values, used to configure the range of values for the benchmark.
/// - [super::TYPE_ENV] what to benchmark for. See [self::MeasurementType]
///
/// ## Example
///
/// As an example, configuring the config with [super::DIMS_ENV] `2-10-1` and [super::TYPE_ENV]
/// `timesteps-10-5`, will run a benchmark for 2 to 10 timesteps with 10 inside nodes and 5
/// outside 
/// each step.
pub(crate) struct CompGraphConfig {
    typ: MeasurementType,
    from: usize,
    to: usize,
    step_by: usize,
}

impl Display for CompGraphConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}-{}", self.typ, self.from, self.to, self.step_by)
    }
}

impl<'a> IntoIterator for &'a CompGraphConfig {
    type Item = usize;
    type IntoIter = StepBy<Range<Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.from..self.to).step_by(self.step_by)
    }
}

impl<'a> GraphBenchmarkConfig<'a> for CompGraphConfig {
    type Error = CommGraphConfigError; 

    fn try_from_env() -> Result<Self, Self::Error>
    where
        Self: Sized {
        let comm_config = env::var(DIMS_ENV)
            .unwrap_or("2-10-1".to_string())
            .split('-')
            .map(str::parse::<usize>)
            .collect::<Result<Vec<_>, ParseIntError>>()?;
        
        if comm_config.len() != 3 {
            Err(CommGraphConfigError::InvalidConfigurationString("Configuration string format: from-to-step_by".into()))
        } else {
            let typ: MeasurementType = env::var(TYPE_ENV).unwrap_or("layers-2-10".into()).try_into()?;
            let cfg = Self {
                typ,
                from: comm_config[0],
                to: comm_config[1],
                step_by: comm_config[2],
            };
            Ok(cfg)
        }

    }

    fn throughput(&self, other: <&'a Self as IntoIterator>::Item) -> u64 {
        let x = match self.typ {
            MeasurementType::Timesteps(a, b) => a * b,
            MeasurementType::Inside(a, b) => a * b,
            MeasurementType::Outside(a, b) => a * b,
            MeasurementType::Ratio(_, b) => b,
        };
        (x * other) as u64
    }

    fn build_graph(&self, size: <&'a Self as IntoIterator>::Item) -> Vec<(usize, usize)> {
        match self.typ {
            MeasurementType::Timesteps(reds, blues) => comp_graph(blues, reds, size),
            MeasurementType::Inside(reds, layers) => comp_graph(size, reds, layers),
            MeasurementType::Outside(blues, layers) => comp_graph(blues, size, layers),
            MeasurementType::Ratio(ratio, layers) => comp_graph((size as f64 * ratio) as usize, (size as f64 * (1. - ratio)) as usize, layers),
        }
    }
}
