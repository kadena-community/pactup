use crate::{remote_pact_index::Release, version::Version};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum UserVersion {
  OnlyMajor(u64),
  MajorMinor(u64, u64),
  SemverRange(node_semver::Range),
  Full(Version),
}

impl UserVersion {
  /// Convert a `UserVersion` to a concrete Version by matching against available versions
  pub fn to_version<'a, T>(
    &self,
    available_versions: T,
    config: &crate::config::PactupConfig,
  ) -> Option<&'a Version>
  where
    T: IntoIterator<Item = &'a Version>,
  {
    available_versions
      .into_iter()
      .filter(|x| self.matches(x, config))
      .max()
  }

  /// Convert a `UserVersion` to a Release by matching against available releases
  pub fn to_release<'a, T>(
    &self,
    available_versions: T,
    config: &crate::config::PactupConfig,
  ) -> Option<&'a Release>
  where
    T: IntoIterator<Item = &'a Release>,
  {
    available_versions
      .into_iter()
      .filter(|x| self.matches(&x.tag, config))
      .max_by_key(|x| &x.tag)
  }

  /// Get the alias name if this version represents an alias
  pub fn alias_name(&self) -> Option<&str> {
    match self {
      Self::Full(version) => version.alias_name(),
      _ => None,
    }
  }

  /// Check if this `UserVersion` matches a concrete Version
  pub fn matches(&self, version: &Version, config: &crate::config::PactupConfig) -> bool {
    use Version::{Alias, Bypassed, Latest, Nightly, Semver};
    match (self, version) {
      // Direct equality match
      (Self::Full(a), b) if a == b => true,

      // Alias matching
      (Self::Full(user_version), maybe_alias) => {
        Self::matches_alias(user_version, maybe_alias, config)
      }

      // Semver range matching
      (Self::SemverRange(range), Semver(semver)) => semver.satisfies(range),

      // Major version matching
      (Self::OnlyMajor(major), Semver(other)) => *major == other.major,

      // Major.Minor version matching
      (Self::MajorMinor(major, minor), Semver(other)) => {
        *major == other.major && *minor == other.minor
      }

      // Special versions don't match partial versions
      (_, Bypassed | Nightly(_) | Alias(_) | Latest) => false,
    }
  }

  fn matches_alias(
    user_version: &Version,
    maybe_alias: &Version,
    config: &crate::config::PactupConfig,
  ) -> bool {
    match (user_version.alias_name(), maybe_alias.find_aliases(config)) {
      (Some(user_alias), Ok(aliases)) => aliases.iter().any(|alias| alias.name() == user_alias),
      _ => false,
    }
  }

  /// Get the inferred alias for special versions (latest, nightly)
  pub fn inferred_alias(&self) -> Option<Version> {
    match self {
      Self::Full(Version::Latest) => Some(Version::Latest),
      Self::Full(Version::Nightly(tag)) => Some(Version::Nightly(tag.clone())),
      _ => None,
    }
  }
}

impl std::fmt::Display for UserVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Full(version) => version.fmt(f),
      Self::SemverRange(range) => range.fmt(f),
      Self::OnlyMajor(major) => write!(f, "v{major}.x.x"),
      Self::MajorMinor(major, minor) => write!(f, "v{major}.{minor}.x"),
    }
  }
}

impl PartialEq for UserVersion {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::OnlyMajor(a), Self::OnlyMajor(b)) => a == b,
      (Self::MajorMinor(a1, a2), Self::MajorMinor(b1, b2)) => a1 == b1 && a2 == b2,
      (Self::Full(v1), Self::Full(v2)) => v1 == v2,
      (Self::SemverRange(r1), Self::SemverRange(r2)) => r1 == r2,
      _ => false,
    }
  }
}

impl FromStr for UserVersion {
  type Err = node_semver::SemverError;

  fn from_str(s: &str) -> Result<UserVersion, Self::Err> {
    let s = s.trim();

    // Trim leading 'v'
    let s_no_v = s.strip_prefix('v').unwrap_or(s);

    // check if the string contains only digits and dots
    if !s_no_v.chars().all(|c| c.is_ascii_digit() || c == '.') {
      // Try to parse as Semver Range
      if let Ok(range) = node_semver::Range::parse(s) {
        return Ok(UserVersion::SemverRange(range));
      }
    }

    let parts: Vec<&str> = s_no_v.split('.').collect();

    // Attempt to parse as OnlyMajor
    if parts.len() == 1 {
      if let Ok(major) = parts[0].parse::<u64>() {
        return Ok(UserVersion::OnlyMajor(major));
      }
    }
    // Attempt to parse as MajorMinor
    else if parts.len() == 2 {
      if let (Ok(major), Ok(minor)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
        return Ok(UserVersion::MajorMinor(major, minor));
      }
    }
    // Attempt to parse as Full Version
    else if parts.len() == 3 {
      if let Ok(version) = Version::parse(s) {
        return Ok(UserVersion::Full(version));
      }
    }

    // Finally, attempt to parse as a Version (could be an alias or special version)
    Version::parse(s).map(UserVersion::Full)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::PactupConfig;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_parsing_versions() {
    let test_cases = vec![
      ("10", Some(UserVersion::OnlyMajor(10))),
      ("10.20", Some(UserVersion::MajorMinor(10, 20))),
      ("v10", Some(UserVersion::OnlyMajor(10))),
      ("v10.20", Some(UserVersion::MajorMinor(10, 20))),
      (
        "10.20.30",
        Some(UserVersion::Full(Version::parse("10.20.30").unwrap())),
      ),
      (
        "^10.20.0",
        Some(UserVersion::SemverRange(
          node_semver::Range::parse("^10.20.0").unwrap(),
        )),
      ),
      (
        "nightly",
        Some(UserVersion::Full(Version::parse("nightly").unwrap())),
      ),
      (
        "latest",
        Some(UserVersion::Full(Version::parse("latest").unwrap())),
      ),
    ];

    for (input, expected) in test_cases {
      let result = UserVersion::from_str(input).ok();
      assert_eq!(
        result, expected,
        "Failed parsing '{}': expected {:?}, got {:?}",
        input, expected, result
      );
    }
  }

  fn create_test_versions() -> Vec<Version> {
    vec![
      Version::parse("6.0.0").unwrap(),
      Version::parse("6.0.1").unwrap(),
      Version::parse("6.0.2").unwrap(),
      Version::parse("6.1.0").unwrap(),
      Version::parse("6.2.0").unwrap(),
      Version::parse("7.0.0").unwrap(),
    ]
  }

  #[test]
  fn test_version_matching() {
    let config = PactupConfig::default();
    let versions = create_test_versions();

    let test_cases = vec![
      (
        UserVersion::OnlyMajor(6),
        "v6.2.0",
        "should match highest version with major 6",
      ),
      (
        UserVersion::MajorMinor(6, 0),
        "v6.0.2",
        "should match highest version with major.minor 6.0",
      ),
      (
        UserVersion::Full(Version::parse("6.0.0").unwrap()),
        "v6.0.0",
        "should match exact version",
      ),
      (
        UserVersion::SemverRange(node_semver::Range::parse("^6.0.0").unwrap()),
        "v6.2.0",
        "should match highest version in range",
      ),
    ];

    for (user_version, expected, message) in test_cases {
      let result = user_version
        .to_version(&versions, &config)
        .map(std::string::ToString::to_string);
      self::assert_eq!(
        result,
        Some(expected.to_string()),
        "{}: {:?}",
        message,
        user_version
      );
    }
  }

  #[test]
  fn test_inferred_aliases() {
    let test_cases = vec![
      (
        UserVersion::Full(Version::Latest),
        Some(Version::Latest),
        "Latest version should infer latest alias",
      ),
      (
        UserVersion::Full(Version::Nightly("nightly".to_string())),
        Some(Version::Nightly("nightly".to_string())),
        "Nightly version should infer nightly alias",
      ),
      (
        UserVersion::OnlyMajor(6),
        None,
        "Partial versions should not infer aliases",
      ),
      (
        UserVersion::Full(Version::Semver(
          node_semver::Version::parse("6.0.0").unwrap(),
        )),
        None,
        "Regular versions should not infer aliases",
      ),
    ];

    for (version, expected, message) in test_cases {
      assert_eq!(version.inferred_alias(), expected, "{}", message);
    }
  }
}
