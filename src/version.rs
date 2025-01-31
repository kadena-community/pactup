use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

use crate::alias;
use crate::config;
use crate::system_version;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Serialize)]
pub enum Version {
  Semver(node_semver::Version),
  Alias(String),
  Nightly(String),
  Latest,
  Bypassed,
}

impl Version {
  pub fn parse<S: AsRef<str>>(version_str: S) -> Result<Self, node_semver::SemverError> {
    let s = version_str.as_ref().trim();
    let lowercased = s.to_lowercase();

    // Check for special versions first
    if lowercased == system_version::display_name() {
      return Ok(Self::Bypassed);
    }

    if lowercased == "latest" {
      return Ok(Self::Latest);
    }

    // Check for nightly/development versions
    if Self::is_development_version(&lowercased) {
      return Ok(Self::Nightly(lowercased));
    }

    // Try to parse as semver (with or without 'v' prefix)
    let version_plain = s.trim_start_matches('v');
    if Self::is_numeric_version(version_plain) {
      let version_complete = Self::ensure_complete_version(version_plain);
      return node_semver::Version::parse(&version_complete).map(Self::Semver);
    }

    // If nothing else matches, treat as alias
    Ok(Self::Alias(lowercased))
  }

  fn is_development_version(s: &str) -> bool {
    s.contains("alpha") || s.contains("nightly") || s.contains("dev")
  }

  fn is_numeric_version(s: &str) -> bool {
    s.chars().next().is_some_and(|x| x.is_ascii_digit())
  }

  fn ensure_complete_version(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    match parts.len() {
      1 => format!("{s}.0.0"),
      2 => format!("{s}.0"),
      _ => s.to_string(),
    }
  }

  pub fn alias_name(&self) -> Option<&str> {
    match self {
      Self::Nightly(name) | Self::Alias(name) => Some(name),
      _ => None,
    }
  }

  pub fn find_aliases(
    &self,
    config: &config::PactupConfig,
  ) -> std::io::Result<Vec<alias::StoredAlias>> {
    let aliases = alias::list_aliases(config)?
      .into_iter()
      .filter(|alias| alias.s_ver() == self.v_str())
      .collect();
    Ok(aliases)
  }

  pub fn v_str(&self) -> String {
    self.to_string()
  }

  pub fn installation_path(&self, config: &config::PactupConfig) -> PathBuf {
    match self {
      Self::Bypassed => system_version::path(),
      v @ (Self::Alias(_) | Self::Latest) => config.aliases_dir().join(v.alias_name().unwrap()),
      Self::Semver(_) => config.installations_dir().join(self.v_str()),
      v @ Self::Nightly(_) => Self::get_nightly_path(config, v),
    }
  }

  fn get_nightly_path(config: &config::PactupConfig, v: &Version) -> PathBuf {
    let install_dir = config.installations_dir().join(v.v_str());
    if install_dir.exists() {
      install_dir
    } else {
      config.aliases_dir().join(v.alias_name().unwrap())
    }
  }

  pub fn root_path(&self, config: &config::PactupConfig) -> Option<PathBuf> {
    let path = self.installation_path(config);
    path.canonicalize().ok().and_then(|canon_path| {
      if Self::is_valid_installation_dir(&canon_path, config) {
        Some(canon_path)
      } else {
        None
      }
    })
  }

  fn is_valid_installation_dir(path: &PathBuf, config: &config::PactupConfig) -> bool {
    path.is_dir() && path.exists() && path != &config.installations_dir()
  }

  pub fn is_nightly(&self) -> bool {
    matches!(self, Self::Nightly(_))
  }
}

impl<'de> serde::Deserialize<'de> for Version {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let version_str = String::deserialize(deserializer)?;
    Version::parse(version_str).map_err(serde::de::Error::custom)
  }
}

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Bypassed => write!(f, "{}", system_version::display_name()),
      Self::Nightly(nightly) => write!(f, "{nightly}"),
      Self::Semver(semver) => write!(f, "v{semver}"),
      Self::Alias(alias) => write!(f, "{alias}"),
      Self::Latest => write!(f, "latest"),
    }
  }
}

impl FromStr for Version {
  type Err = node_semver::SemverError;
  fn from_str(s: &str) -> Result<Version, Self::Err> {
    Self::parse(s)
  }
}

impl PartialEq<node_semver::Version> for Version {
  fn eq(&self, other: &node_semver::Version) -> bool {
    matches!(self, Self::Semver(v) if v == other)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_version_parsing() {
    let test_cases = vec![
      (
        "4.11.0",
        Ok(Version::Semver(
          node_semver::Version::parse("4.11.0").unwrap(),
        )),
      ),
      (
        "v4.11.0",
        Ok(Version::Semver(
          node_semver::Version::parse("4.11.0").unwrap(),
        )),
      ),
      (
        "4.11",
        Ok(Version::Semver(
          node_semver::Version::parse("4.11.0").unwrap(),
        )),
      ),
      (
        "4",
        Ok(Version::Semver(
          node_semver::Version::parse("4.0.0").unwrap(),
        )),
      ),
      ("nightly", Ok(Version::Nightly("nightly".to_string()))),
      ("latest", Ok(Version::Latest)),
      ("system", Ok(Version::Bypassed)),
      ("dev", Ok(Version::Nightly("dev".to_string()))),
    ];

    for (input, expected) in test_cases {
      assert_eq!(
        Version::parse(input),
        expected,
        "Failed to parse version: {}",
        input
      );
    }
  }

  #[test]
  fn test_version_display() {
    let test_cases = vec![
      (
        Version::Semver(node_semver::Version::parse("4.11.0").unwrap()),
        "v4.11.0",
      ),
      (Version::Nightly("nightly".to_string()), "nightly"),
      (Version::Latest, "latest"),
      (Version::Bypassed, "system"),
      (Version::Alias("custom".to_string()), "custom"),
    ];

    for (version, expected) in test_cases {
      assert_eq!(version.to_string(), expected);
    }
  }
}
