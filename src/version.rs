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

fn first_letter_is_number(s: &str) -> bool {
  s.chars().next().map_or(false, |x| x.is_ascii_digit())
}

impl Version {
  pub fn parse<S: AsRef<str>>(version_str: S) -> Result<Self, node_semver::SemverError> {
    let lowercased = version_str.as_ref().to_lowercase();
    if lowercased == system_version::display_name() {
      Ok(Self::Bypassed)
    } else if lowercased.contains("alpha")
      || lowercased.contains("nightly")
      || lowercased.contains("dev")
    {
      Ok(Self::Nightly(lowercased))
    } else if first_letter_is_number(lowercased.trim_start_matches('v')) {
      let version_plain = lowercased.trim_start_matches('v');
      // Ensure that the version has 3 parts
      let parts_count = version_plain.split('.').count();
      let version_complete = match parts_count {
        1 => format!("{version_plain}.0.0"),
        2 => format!("{version_plain}.0"),
        _ => version_plain.to_string(),
      };
      let sver = node_semver::Version::parse(&version_complete)?;
      Ok(Self::Semver(sver))
    } else {
      Ok(Self::Alias(lowercased))
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
    format!("{self}")
  }

  pub fn installation_path(&self, config: &config::PactupConfig) -> PathBuf {
    match self {
      Self::Bypassed => system_version::path(),
      v @ (Self::Alias(_) | Self::Latest) => config.aliases_dir().join(v.alias_name().unwrap()),
      Self::Semver(_) => config.installations_dir().join(self.v_str()),
      v @ Self::Nightly(_) => {
        let install_dir = config.installations_dir().join(v.v_str());
        if install_dir.exists() {
          install_dir
        } else {
          config.aliases_dir().join(v.alias_name().unwrap())
        }
      }
    }
  }

  pub fn root_path(&self, config: &config::PactupConfig) -> Option<PathBuf> {
    let path = self.installation_path(config);
    path.canonicalize().ok().and_then(|canon_path| {
      // If the path is a directory and it exists,also not the installations directory
      if canon_path.is_dir() && canon_path.exists() && canon_path != config.installations_dir() {
        Some(canon_path)
      } else {
        None
      }
    })
  }

  pub fn is_nightly(&self) -> bool {
    matches!(self, Self::Nightly(_))
  }

  pub fn digits_only(&self) -> Option<String> {
    match self {
      Self::Semver(v) => Some(v.to_string().trim_start_matches('v').to_string()),
      _ => None,
    }
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
    match self {
      Self::Semver(v) => v == other,
      _ => false,
    }
  }
}
