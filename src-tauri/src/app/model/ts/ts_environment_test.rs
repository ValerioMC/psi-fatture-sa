use super::*;

#[test]
fn round_trips_through_storage_string() {
    for environment in [TsEnvironment::Test, TsEnvironment::Produzione] {
        assert_eq!(
            TsEnvironment::parse(environment.as_str()).ok(),
            Some(environment)
        );
    }
    assert!(TsEnvironment::parse("prod").is_err());
}

#[test]
fn production_is_available_in_every_build() {
    assert!(TsEnvironment::Produzione.is_available());
    assert!(DISTRIBUTED_ENVIRONMENTS.contains(&TsEnvironment::Produzione));
    assert!(!DISTRIBUTED_ENVIRONMENTS.contains(&TsEnvironment::Test));
}

#[test]
#[cfg(any(debug_assertions, feature = "sogei-test"))]
fn test_is_available_in_developer_builds() {
    assert!(TsEnvironment::Test.is_available());
}

#[test]
fn test_and_production_use_distinct_hosts() {
    assert!(TsEnvironment::Test.base_url().contains("Test"));
    assert!(!TsEnvironment::Produzione.base_url().contains("Test"));
}
