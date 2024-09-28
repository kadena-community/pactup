use crate::{remote_pact_index::Release, version::Version};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum UserVersion {
  OnlyMajor(u64),
  MajorMinor(u64, u64),
  #[allow(dead_code)]
  SemverRange(node_semver::Range),
  Full(Version),
}

impl UserVersion {
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
      .filter(|x| self.matches(&x.tag_name, config))
      .max_by_key(|x| &x.tag_name)
  }

  pub fn alias_name(&self) -> Option<&str> {
    match self {
      Self::Full(version) => version.alias_name(),
      _ => None,
    }
  }

  pub fn matches(&self, version: &Version, config: &crate::config::PactupConfig) -> bool {
    match (self, version) {
      (Self::Full(a), b) if a == b => true,
      (Self::Full(user_version), maybe_alias) => {
        if let (Some(user_alias), Ok(aliases)) =
          (user_version.alias_name(), maybe_alias.find_aliases(config))
        {
          aliases.iter().any(|alias| alias.name() == user_alias)
        } else {
          false
        }
      }
      (Self::SemverRange(range), Version::Semver(semver)) => semver.satisfies(range),
      (_, Version::Bypassed | Version::Nightly(_) | Version::Alias(_) | Version::Latest) => false,
      (Self::OnlyMajor(major), Version::Semver(other)) => *major == other.major,
      (Self::MajorMinor(major, minor), Version::Semver(other)) => {
        *major == other.major && *minor == other.minor
      }
    }
  }

  /// The inferred alias for the user version, if it exists.
  pub fn inferred_alias(&self) -> Option<Version> {
    match self {
      UserVersion::Full(Version::Latest) => Some(Version::Latest),
      UserVersion::Full(Version::Nightly(nightly_type)) => {
        Some(Version::Nightly(nightly_type.clone()))
      }
      _ => None,
    }
  }
}

impl std::fmt::Display for UserVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Full(x) => x.fmt(f),
      Self::SemverRange(x) => x.fmt(f),
      Self::OnlyMajor(major) => write!(f, "v{major}.x.x"),
      Self::MajorMinor(major, minor) => write!(f, "v{major}.{minor}.x"),
    }
  }
}

fn skip_first_v(s: &str) -> &str {
  s.strip_prefix('v').unwrap_or(s)
}

impl FromStr for UserVersion {
  type Err = node_semver::SemverError;
  fn from_str(s: &str) -> Result<UserVersion, Self::Err> {
    let s = s.trim();

    // Trim leading 'v'
    let s_no_v = skip_first_v(s);
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

    // Try to parse as Semver Range
    if let Ok(range) = node_semver::Range::parse(s) {
      return Ok(UserVersion::SemverRange(range));
    }

    // Finally, attempt to parse as a Version (could be an alias or special version)
    Version::parse(s).map(UserVersion::Full)
  }
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
  use crate::config::PactupConfig;

  use super::*;

  #[test]
  fn test_parsing_only_major() {
    let version = UserVersion::from_str("10").ok();
    assert_eq!(version, Some(UserVersion::OnlyMajor(10)));
  }

  #[test]
  fn test_parsing_major_minor() {
    let version = UserVersion::from_str("10.20").ok();
    assert_eq!(version, Some(UserVersion::MajorMinor(10, 20)));
  }

  #[test]
  fn test_parsing_only_major_with_v() {
    let version = UserVersion::from_str("v10").ok();
    assert_eq!(version, Some(UserVersion::OnlyMajor(10)));
  }

  #[test]
  fn test_major_to_version() {
    let expected = Version::parse("6.1.0").unwrap();
    let versions = vec![
      Version::parse("6.0.0").unwrap(),
      Version::parse("6.0.1").unwrap(),
      expected.clone(),
      Version::parse("7.0.1").unwrap(),
    ];
    let result = UserVersion::OnlyMajor(6).to_version(&versions, &PactupConfig::default());

    assert_eq!(result, Some(&expected));
  }

  #[test]
  fn test_major_minor_to_version() {
    let expected = Version::parse("6.0.1").unwrap();
    let versions = vec![
      Version::parse("6.0.0").unwrap(),
      Version::parse("6.1.0").unwrap(),
      expected.clone(),
      Version::parse("7.0.1").unwrap(),
    ];
    let result = UserVersion::MajorMinor(6, 0).to_version(&versions, &PactupConfig::default());

    assert_eq!(result, Some(&expected));
  }

  #[test]
  fn test_semver_to_version() {
    let expected = Version::parse("6.0.0").unwrap();
    let versions = vec![
      expected.clone(),
      Version::parse("6.1.0").unwrap(),
      Version::parse("6.0.1").unwrap(),
      Version::parse("7.0.1").unwrap(),
    ];
    let result =
      UserVersion::Full(expected.clone()).to_version(&versions, &PactupConfig::default());

    assert_eq!(result, Some(&expected));
  }
}
