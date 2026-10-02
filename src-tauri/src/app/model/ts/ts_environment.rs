use crate::app::common::AppError;
use serde::{Deserialize, Serialize};

/// Which Sistema TS installation a transmission goes to. Test accepts the
/// public credentials of the Sogei development kit and has no fiscal effect,
/// and exists only in builds for the developer: see `available`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TsEnvironment {
    Test,
    Produzione,
}

/// Environments of a build for the developer: debug, or release with the
/// `sogei-test` feature.
const DEVELOPER_ENVIRONMENTS: &[TsEnvironment] = &[TsEnvironment::Produzione, TsEnvironment::Test];
const DISTRIBUTED_ENVIRONMENTS: &[TsEnvironment] = &[TsEnvironment::Produzione];

impl TsEnvironment {
    /// The environments this build may talk to. A distributed release knows
    /// production alone, so its users can neither pick nor reach the test host.
    pub fn available() -> &'static [TsEnvironment] {
        if cfg!(any(debug_assertions, feature = "sogei-test")) {
            DEVELOPER_ENVIRONMENTS
        } else {
            DISTRIBUTED_ENVIRONMENTS
        }
    }

    pub fn is_available(self) -> bool {
        Self::available().contains(&self)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TsEnvironment::Test => "test",
            TsEnvironment::Produzione => "produzione",
        }
    }

    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "test" => Ok(TsEnvironment::Test),
            "produzione" => Ok(TsEnvironment::Produzione),
            other => Err(AppError::Invalid(format!(
                "Ambiente Sistema TS sconosciuto: {other}"
            ))),
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            TsEnvironment::Test => "https://invioSS730pTest.sanita.finanze.it",
            TsEnvironment::Produzione => "https://invioSS730p.sanita.finanze.it",
        }
    }
}

#[cfg(test)]
#[path = "ts_environment_test.rs"]
mod tests;
