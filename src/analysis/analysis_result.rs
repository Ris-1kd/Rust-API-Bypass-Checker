use failure::Error;
use failure::Fail;
use std::fmt;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Debug for AnalysisInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalysisInfo",)
    }
}

impl fmt::Debug for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnalysisError",)
    }
}

#[allow(non_local_definitions)]
#[derive(Fail)]
pub enum AnalysisError {
    #[fail(display = "Analysis timeout")]
    TimeOut,
    #[fail(display = "The fixed-point algorithm reached the maximum iteration, abort")]
    MaxIteration,
}

pub struct AnalysisInfo {
    pub analysis_time: Duration,
    pub total_diagnostics: usize,
    pub supported_diagnostics: usize,
    pub unsupported_diagnostics: usize,
    pub call_boundary_diagnostics: usize,
    pub supported_special_calls: usize,
    pub unsupported_special_calls: usize,
    pub opaque_call_boundaries: usize,
}
