use std::{error::Error, fmt::Display};

use super::GraphBenchmarkConfig;

#[derive(Debug)]
enum CommGraphConfigError {
    InvalidConfigurationString(String),
}

impl Display for CommGraphConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CommGraphConfigError::InvalidConfigurationString(s) => s,
        };
        write!(f, "{s}")
    }
}

impl Error for CommGraphConfigError {}

enum MeasurementType {
    Layers(usize, usize),
    Blues(usize, usize),
    Reds(usize, usize),
}

struct CommGraphConfig {
    typ: MeasurementType,
    from: usize,
    to: usize,
    step_by: usize,
}

impl<'a> Iterator for &'a CommGraphConfig {

}

impl<'a> GraphBenchmarkConfig<'a> for CommGraphConfig {
    type Error = CommGraphConfigError; 
}
