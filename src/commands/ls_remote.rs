use crate::config::PactupConfig;
use crate::remote_pact_index;
use thiserror::Error;

use crate::user_version::UserVersion;

use colored::Colorize;

#[derive(clap::Parser, Debug)]
pub struct LsRemote {
  /// Filter versions by a user-defined version or a semver range
  #[arg(long)]
  filter: Option<UserVersion>,

  /// Show nightly versions
  #[arg(long)]
  nightly: bool,

  /// Version sorting order
  #[arg(long, default_value = "asc")]
  sort: SortingMethod,

  /// Only show the latest matching version
  #[arg(long)]
  latest: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum SortingMethod {
  #[clap(name = "desc")]
  /// Sort versions in descending order (latest to earliest)
  Descending,
  #[clap(name = "asc")]
  /// Sort versions in ascending order (earliest to latest)
  Ascending,
}

impl super::command::Command for LsRemote {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let mut all_versions = if self.nightly {
      let mut stable_versions = remote_pact_index::list(&config.pact_4x_repo)?;
      let nightly_versions = remote_pact_index::list(&config.pact_5x_repo)?;
      stable_versions.extend(nightly_versions);
      stable_versions
    } else {
      let stable_versions = remote_pact_index::list(&config.pact_4x_repo)?;
      stable_versions
    };

    if let Some(filter) = &self.filter {
      all_versions.retain(|v| filter.matches(&v.tag_name, config));
    }

    if self.latest {
      all_versions.truncate(1);
    }

    all_versions.sort_by_key(|v| v.tag_name.clone());
    if let SortingMethod::Descending = self.sort {
      all_versions.reverse();
    }

    if all_versions.is_empty() {
      eprintln!("{}", "No versions were found!".red());
      return Ok(());
    }

    for version in &all_versions {
      print!("{}", version.tag_name);
      if version.draft {
        print!("{}", format!(" (draft)").cyan());
      }
      if version.prerelease {
        print!("{}", format!(" (prerelease)").cyan());
      }

      if version.tag_name.is_nightly() {
        print!("{}", format!(" (nightly)").cyan());
      }

      println!();
    }

    Ok(())
  }
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  HttpError {
    #[from]
    source: crate::http::Error,
  },
}
