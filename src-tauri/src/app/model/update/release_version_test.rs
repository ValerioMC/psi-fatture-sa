use super::*;

#[test]
fn a_tag_with_a_leading_v_reads_as_its_numbers() {
    let version = ReleaseVersion::parse("v0.9.8").unwrap();

    assert_eq!((version.major, version.minor, version.patch), (0, 9, 8));
    assert_eq!(version.to_string(), "0.9.8");
}

#[test]
fn numbers_compare_as_numbers_not_as_text() {
    assert!(ReleaseVersion::parse("0.10.0").unwrap() > ReleaseVersion::parse("0.9.8").unwrap());
}

#[test]
fn a_pre_release_ranks_below_its_release() {
    let beta = ReleaseVersion::parse("1.0.0-beta.1").unwrap();
    let release = ReleaseVersion::parse("1.0.0").unwrap();

    assert!(beta < release);
    assert!(beta > ReleaseVersion::parse("0.9.9").unwrap());
}

#[test]
fn build_metadata_does_not_count() {
    assert_eq!(
        ReleaseVersion::parse("1.2.3+sha.abc").unwrap(),
        ReleaseVersion::parse("1.2.3").unwrap()
    );
}

#[test]
fn malformed_versions_are_refused() {
    for text in ["", "1.2", "1.2.3.4", "x.y.z", "1.2.3-", "latest"] {
        assert!(
            matches!(ReleaseVersion::parse(text), Err(AppError::Invalid(_))),
            "{text}"
        );
    }
}
