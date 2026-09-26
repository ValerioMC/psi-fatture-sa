use serde::{Deserialize, Serialize};

/// Which Sistema TS installation a transmission goes to. Test accepts the
/// public credentials of the Sogei development kit and has no fiscal effect.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TsEnvironment {
    Test,
    Produzione,
}

impl TsEnvironment {
    pub fn as_str(&self) -> &'static str {
        match self {
            TsEnvironment::Test => "test",
            TsEnvironment::Produzione => "produzione",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "test" => Ok(TsEnvironment::Test),
            "produzione" => Ok(TsEnvironment::Produzione),
            other => Err(format!("Ambiente Sistema TS sconosciuto: {other}")),
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
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_storage_string() {
        for environment in [TsEnvironment::Test, TsEnvironment::Produzione] {
            assert_eq!(TsEnvironment::parse(environment.as_str()), Ok(environment));
        }
        assert!(TsEnvironment::parse("prod").is_err());
    }

    #[test]
    fn test_and_production_use_distinct_hosts() {
        assert!(TsEnvironment::Test.base_url().contains("Test"));
        assert!(!TsEnvironment::Produzione.base_url().contains("Test"));
    }
}
