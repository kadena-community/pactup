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

  /// Include nightly versions.
  #[arg(long)]
  nightly: bool,

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

/// Prints the version information with annotations.
fn print_version_info(version: &Release) {
  let mut annotations = vec![];

  if version.draft {
    annotations.push("(draft)".cyan().to_string());
  }
  if version.prerelease {
    annotations.push("(prerelease)".cyan().to_string());
  }
  if version.tag_name.is_nightly() {
    annotations.push("(nightly)".cyan().to_string());
  }
  if !version.has_supported_asset() {
    annotations.push("(can't install)".red().to_string());
  }

  if annotations.is_empty() {
    println!("{}", version.tag_name);
  } else {
    println!("{} {}", version.tag_name, annotations.join(" "));
  }
}

impl super::command::Command for LsRemote {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    // Fetch all versions based on the `nightly` flag.
    let mut all_versions = self.fetch_versions(config)?;

    // Filter versions if a filter is provided.
    if let Some(ref filter) = self.filter {
      all_versions.retain(|v| filter.matches(&v.tag_name, config));
    }

    // Check if no versions are left after filtering.
    if all_versions.is_empty() {
      eprintln!("{}", "No versions were found!".red());
      return Ok(());
    }

    // Sort the versions.
    self.sort_versions(&mut all_versions);

    // If `latest` flag is set, print the latest version and exit.
    if self.latest {
      let latest_version = self.get_latest_version(&all_versions)?;
      println!("{}", latest_version.tag_name);
      return Ok(());
    }

    // Print the versions with their annotations.
    for version in &all_versions {
      print_version_info(version);
    }

    Ok(())
  }
}

impl LsRemote {
  /// Fetches versions from the remote repositories.
  fn fetch_versions(&self, config: &PactupConfig) -> Result<Vec<Release>, Error> {
    let mut versions = remote_pact_index::list(&config.pact_4x_repo)?;

    if self.nightly {
      let nightly_versions = remote_pact_index::list(&config.pact_5x_repo)?;
      versions.extend(nightly_versions);
    }

    Ok(versions)
  }

  /// Retrieves the latest version from the list of versions.
  fn get_latest_version<'a>(&self, versions: &'a [Release]) -> Result<&'a Release, Error> {
    // Versions should already be sorted at this point.
    if self.nightly {
      versions
        .iter()
        .find(|v| v.tag_name.is_nightly())
        .ok_or(Error::NoNightlyVersions)
    } else {
      versions
        .iter()
        .find(|v| !v.tag_name.is_nightly())
        .ok_or(Error::NoVersionsAvailable)
    }
  }

  /// Sorts the versions based on the specified sorting method.
  fn sort_versions(&self, versions: &mut [Release]) {
    // versions.sort_by(|a, b| a.tag_name.cmp(&b.tag_name));
    if self.sort == SortingMethod::Descending {
      versions.reverse();
    }
  }
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  HttpError {
    #[from]
    source: crate::http::Error,
  },
  #[error("No nightly versions were found.")]
  NoNightlyVersions,
  #[error("No versions are available.")]
  NoVersionsAvailable,
}
