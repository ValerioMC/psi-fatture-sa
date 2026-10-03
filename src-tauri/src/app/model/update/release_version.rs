use std::cmp::Ordering;

use crate::app::common::AppError;

/// A version as releases are tagged: `X.Y.Z`, an optional leading `v` and an optional
/// `-pre` suffix, which ranks below the same `X.Y.Z` without it. Build metadata is ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: Option<String>,
}

impl ReleaseVersion {
    pub fn parse(text: &str) -> Result<Self, AppError> {
        let invalid = || AppError::Invalid(format!("Versione non valida: '{text}'"));
        let trimmed = text.trim();
        let bare = trimmed.strip_prefix('v').unwrap_or(trimmed);
        let without_build = bare.split('+').next().unwrap_or_default();
        let (core, pre_release) = match without_build.split_once('-') {
            Some((core, pre)) if !pre.is_empty() => (core, Some(pre.to_string())),
            Some(_) => return Err(invalid()),
            None => (without_build, None),
        };
        let numbers: Vec<u64> = core
            .split('.')
            .map(|part| part.parse::<u64>().map_err(|_| invalid()))
            .collect::<Result<_, _>>()?;
        let [major, minor, patch] = numbers[..] else {
            return Err(invalid());
        };
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

impl std::fmt::Display for ReleaseVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)?;
        match &self.pre_release {
            Some(pre) => write!(formatter, "-{pre}"),
            None => Ok(()),
        }
    }
}

impl Ord for ReleaseVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.pre_release, &other.pre_release) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(mine), Some(theirs)) => mine.cmp(theirs),
            })
    }
}

impl PartialOrd for ReleaseVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[path = "release_version_test.rs"]
mod tests;
