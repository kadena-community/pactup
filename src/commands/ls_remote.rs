use crate::config::PactupConfig;
use crate::remote_pact_index::{self, Release};
use crate::user_version::UserVersion;
use colored::Colorize;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct LsRemote {
  /// Filter versions by a user-defined version or a semver range.
  #[arg(long)]
  filter: Option<UserVersion>,

  /// Version sorting order.
  #[arg(long, default_value = "asc")]
  sort: SortingMethod,

  /// Only show the latest matching version.
  #[arg(long)]
  latest: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum SortingMethod {
  #[clap(name = "desc")]
  /// Sort versions in descending order (latest to earliest).
  Descending,
  #[clap(name = "asc")]
  /// Sort versions in ascending order (earliest to latest).
  Ascending,
}

/// Represents the version display information
#[derive(Debug)]
struct VersionInfo {
  version: String,
  annotations: Vec<String>,
}

impl VersionInfo {
  fn new(release: &Release) -> Self {
    let mut annotations = Vec::new();

    if release.draft {
      annotations.push("(draft)".cyan().to_string());
    }
    if release.prerelease {
      annotations.push("(prerelease)".cyan().to_string());
    }
    if release.tag.is_nightly() {
      annotations.push("(nightly)".cyan().to_string());
    }
    if !release.has_supported_asset() {
      annotations.push("(can't install)".red().to_string());
    }

    Self {
      version: release.tag.to_string(),
      annotations,
    }
  }

  fn display(&self) -> String {
    if self.annotations.is_empty() {
      self.version.clone()
    } else {
      format!("{} {}", self.version, self.annotations.join(" "))
    }
  }
}

impl super::command::Command for LsRemote {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let mut versions = self.fetch_and_filter_versions(config)?;

    if versions.is_empty() {
      eprintln!("{}", "No versions were found!".red());
      return Ok(());
    }

    self.sort_versions(&mut versions);

    if self.latest {
      let latest = Self::get_latest_version(&versions)?;
      println!("{}", latest.tag);
      return Ok(());
    }

    Self::print_versions(&versions);
    Ok(())
  }
}

impl LsRemote {
  fn fetch_and_filter_versions(&self, config: &PactupConfig) -> Result<Vec<Release>, Error> {
    let mut versions = remote_pact_index::list(config.repo_urls())?;

    if let Some(ref filter) = self.filter {
      versions.retain(|v| filter.matches(&v.tag, config));
    }

    Ok(versions)
  }

  fn get_latest_version(versions: &[Release]) -> Result<&Release, Error> {
    versions.first().ok_or(Error::NoVersionsAvailable)
  }

  fn sort_versions(&self, versions: &mut [Release]) {
    versions.sort_by(|a, b| a.tag.cmp(&b.tag));
    if self.sort == SortingMethod::Descending {
      versions.reverse();
    }
  }

  fn print_versions(versions: &[Release]) {
    for version in versions {
      let info = VersionInfo::new(version);
      println!("{}", info.display());
    }
  }
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  RemoteListing {
    #[from]
    source: remote_pact_index::Error,
  },
  #[error("No versions are available.")]
  NoVersionsAvailable,
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use crate::version::Version;

  use super::*;
  use pretty_assertions::assert_eq;
  use url::Url;

  fn create_test_release(version: &str, prerelease: bool, draft: bool) -> Release {
    Release {
      tag: Version::parse(version).unwrap(),
      assets: vec![],
      prerelease,
      draft,
    }
  }

  fn create_test_asset(name: &str) -> remote_pact_index::Asset {
    remote_pact_index::Asset {
      download_url: Url::parse(&format!("https://example.com/download/{name}")).unwrap(),
    }
  }

  #[test]
  fn test_version_info_display() {
    let mut release = create_test_release("4.13.0", true, true);
    release.assets = vec![create_test_asset("pact-4.13.0-linux-x64.tar.gz")];

    let info = VersionInfo::new(&release);
    assert!(info.display().contains("(draft)"));
    assert!(info.display().contains("(prerelease)"));
  }

  #[test]
  fn test_sorting() {
    let cmd = LsRemote {
      filter: None,
      sort: SortingMethod::Ascending,
      latest: false,
    };

    let mut versions = vec![
      create_test_release("4.13.0", false, false),
      create_test_release("4.10.0", false, false),
      create_test_release("4.12.0", false, false),
    ];

    cmd.sort_versions(&mut versions);

    assert_eq!(versions[0].tag.to_string(), "v4.10.0");
    assert_eq!(versions[1].tag.to_string(), "v4.12.0");
    assert_eq!(versions[2].tag.to_string(), "v4.13.0");
  }

  #[test]
  fn test_sorting_descending() {
    let cmd = LsRemote {
      filter: None,
      sort: SortingMethod::Descending,
      latest: false,
    };

    let mut versions = vec![
      create_test_release("4.10.0", false, false),
      create_test_release("4.13.0", false, false),
      create_test_release("4.12.0", false, false),
    ];

    cmd.sort_versions(&mut versions);

    assert_eq!(versions[0].tag.to_string(), "v4.13.0");
    assert_eq!(versions[1].tag.to_string(), "v4.12.0");
    assert_eq!(versions[2].tag.to_string(), "v4.10.0");
  }

  #[test]
  fn test_version_filtering() {
    let cmd = LsRemote {
      filter: Some(UserVersion::from_str("4.13.0").unwrap()),
      sort: SortingMethod::Ascending,
      latest: false,
    };

    let config = PactupConfig::default();
    let filtered = cmd.fetch_and_filter_versions(&config).unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].tag.to_string(), "v4.13.0");
  }

  #[test]
  fn test_latest_version() {
    let versions = vec![
      create_test_release("4.10.0", false, false),
      create_test_release("4.13.0", false, false),
      create_test_release("4.12.0", false, false),
    ];

    let latest = LsRemote::get_latest_version(&versions).unwrap();
    assert_eq!(latest.tag.to_string(), "v4.10.0");
  }

  #[test]
  fn test_empty_versions() {
    let versions: Vec<Release> = vec![];
    let result = LsRemote::get_latest_version(&versions);
    assert!(matches!(result, Err(Error::NoVersionsAvailable)));
  }

  #[test]
  fn test_version_info_with_no_annotations() {
    let mut release = create_test_release("4.13.0", false, false);
    release.assets = vec![
      create_test_asset("pact-4.13.0-linux-x64.tar.gz"),
      create_test_asset("pact-4.13.0-macos-x64.tar.gz"),
      create_test_asset("pact-4.13.0-windows-x64.tar.gz"),
      create_test_asset("pact-4.13.0-darwin-aarch64.tar.gz"),
    ];
    let info = VersionInfo::new(&release);
    assert_eq!(info.display(), "v4.13.0");
  }

  #[test]
  fn test_version_info_with_supported_asset() {
    let mut release = create_test_release("4.13.0", false, false);
    release.assets = vec![
      create_test_asset("pact-4.13.0-linux-x64.tar.gz"),
      create_test_asset("pact-4.13.0-macos-x64.tar.gz"),
      create_test_asset("pact-4.13.0-windows-x64.tar.gz"),
    ];
    let info = VersionInfo::new(&release);
    assert!(!info.display().contains("can't install"));
  }

  #[test]
  fn test_pact_v5_x_x() {
    let mut release = create_test_release("5.0.0", false, false);
    release.assets = vec![
      create_test_asset("pact-5.0.0-linux-x64.tar.gz"),
      create_test_asset("pact-5.0.0-darwin-aarch64.tar.gz"),
      create_test_asset("pact-5.0.0-windows-x64.tar.gz"),
    ];
    let info = VersionInfo::new(&release);
    assert!(!info.display().contains("can't install"));
  }
}
