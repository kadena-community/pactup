use super::command::Command;
use crate::alias::create_alias;
use crate::config::PactupConfig;
use crate::downloader::{install_pact_dist, Error as DownloaderError};
use crate::outln;
use crate::progress::ProgressConfig;
use crate::remote_pact_index::{self};
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::version_files::get_user_version_for_directory;
use colored::Colorize;
use log::debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Install {
  /// A version string. Can be a partial semver or a 'development' version.
  pub version: Option<UserVersion>,

  /// Install latest nightly version.
  #[clap(long, conflicts_with_all = &["version", "latest"])]
  pub nightly: bool,

  /// Install latest version.
  #[clap(long, conflicts_with_all = &["version", "nightly"])]
  pub latest: bool,

  /// Show an interactive progress bar for the download status.
  #[clap(long, default_value_t)]
  #[arg(value_enum)]
  pub progress: ProgressConfig,

  /// Force install even if the version is already installed.
  #[clap(long)]
  pub force: bool,
}

impl Install {
  fn version(&self) -> Result<Option<UserVersion>, Error> {
    match (self.version.as_ref(), self.nightly, self.latest) {
      (Some(v), false, false) => Ok(Some(v.clone())),
      (None, true, false) => Ok(Some(UserVersion::Full(Version::Nightly(
        "nightly".to_string(),
      )))),
      (None, false, true) => Ok(Some(UserVersion::Full(Version::Latest))),
      (None, false, false) => Ok(None),
      _ => Err(Error::TooManyVersionsProvided),
    }
  }
}

impl Command for Install {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let current_dir = std::env::current_dir()?;
    let show_progress = self.progress.enabled(config);
    let force_install = self.force;
    let current_version = self
      .version()?
      .or_else(|| get_user_version_for_directory(&current_dir, config))
      .ok_or(Error::CantInferVersion)?;

    let release = match &current_version {
      UserVersion::Full(Version::Semver(actual_version)) => {
        let available_releases = remote_pact_index::list(&config.pact_4x_repo)
          .map_err(|source| Error::CantListRemoteVersions { source })?;

        let picked_release = current_version
          .to_release(&available_releases, config)
          .ok_or_else(|| Error::CantFindPactVersion {
            requested_version: current_version.clone(),
          })?;

        debug!(
          "Resolved {} into Pact version {}",
          Version::Semver(actual_version.clone()).v_str().cyan(),
          picked_release.tag_name.v_str().cyan()
        );
        picked_release.clone()
      }
      UserVersion::Full(v @ (Version::Bypassed | Version::Alias(_))) => {
        return Err(Error::UninstallableVersion { version: v.clone() });
      }
      UserVersion::Full(Version::Nightly(nightly_tag)) => {
        let picked_release = remote_pact_index::get_by_tag(&config.pact_5x_repo, nightly_tag)
          .map_err(|_| Error::CantFindNightly {
            nightly_tag: nightly_tag.clone(),
          })?;

        debug!(
          "Resolved {} into Pact version {}",
          Version::Nightly(nightly_tag.clone()).v_str().cyan(),
          picked_release.tag_name.v_str().cyan()
        );
        picked_release.clone()
      }
      UserVersion::Full(Version::Latest) => {
        let picked_release =
          remote_pact_index::latest(&config.pact_4x_repo).map_err(|_| Error::CantFindLatest)?;

        debug!(
          "Resolved {} into Pact version {}",
          Version::Latest.v_str().cyan(),
          picked_release.tag_name.v_str().cyan()
        );
        picked_release.clone()
      }
      _ => {
        let available_releases = remote_pact_index::list(&config.pact_4x_repo)
          .map_err(|source| Error::CantListRemoteVersions { source })?;

        current_version
          .to_release(&available_releases, config)
          .ok_or_else(|| Error::CantFindPactVersion {
            requested_version: current_version.clone(),
          })?
          .clone()
      }
    };

    // Automatically swap Apple Silicon to x64 arch for appropriate versions.
    let version = &release.tag_name;
    outln!(
      config,
      Info,
      "Installing {} ({})",
      format!("Pact {}", &version).cyan(),
      config.arch.as_str()
    );

    let download_url = release
      .download_url()
      .ok_or_else(|| Error::CantFindReleaseAsset {
        requested_version: current_version.clone(),
      })?;

    match install_pact_dist(
      version,
      &download_url,
      config.installations_dir(),
      config.arch,
      show_progress,
      force_install,
    ) {
      Err(err @ DownloaderError::VersionAlreadyInstalled { .. }) => {
        outln!(config, Error, "{} {}", "warning:".bold().yellow(), err);
      }
      Err(source) => return Err(Error::DownloadError { source }),
      Ok(()) => {}
    };

    if !config.default_version_dir().exists() {
      debug!(
        "Tagging {} as the default version",
        release.tag_name.v_str().cyan()
      );
      create_alias(config, "default", &release.tag_name)?;
    }

    if let Some(tagged_alias) = current_version.inferred_alias() {
      tag_alias(config, &release.tag_name, &tagged_alias)?;
    }

    Ok(())
  }
}

fn tag_alias(
  config: &PactupConfig,
  matched_version: &Version,
  alias: &Version,
) -> Result<(), Error> {
  let alias_name = alias.v_str();
  debug!(
    "Tagging {} as alias for {}",
    alias_name.cyan(),
    matched_version.v_str().cyan()
  );
  create_alias(config, &alias_name, matched_version)?;

  Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
  #[error("Can't download the requested binary: {source}")]
  DownloadError { source: DownloaderError },
  #[error(transparent)]
  IoError {
    #[from]
    source: std::io::Error,
  },
  #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
  CantInferVersion,
  #[error(transparent)]
  CantListRemoteVersions { source: remote_pact_index::Error },
  #[error("Can't find a Pact version that matches {requested_version} in remote")]
  CantFindPactVersion { requested_version: UserVersion },
  #[error("Can't find a release asset for the requested version: {requested_version}")]
  CantFindReleaseAsset { requested_version: UserVersion },
  #[error("Can't find nightly version named {nightly_tag}")]
  CantFindNightly { nightly_tag: String },
  #[error("Can't find any versions in the upstream version index.")]
  CantFindLatest,
  #[error("The requested version is not installable: {}", version.v_str())]
  UninstallableVersion { version: Version },
  #[error("Too many versions provided. Please don't use --nightly with a version string.")]
  TooManyVersionsProvided,
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use std::str::FromStr;

  #[test]
  fn test_set_default_on_new_installation() {
    let base_dir = tempfile::tempdir().unwrap();
    let config = PactupConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
    assert!(!config.default_version_dir().exists());

    Install {
      version: UserVersion::from_str("4.11.0").ok(),
      nightly: false,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    assert!(config.default_version_dir().exists());
    assert_eq!(
      config.default_version_dir().canonicalize().ok(),
      config
        .installations_dir()
        .join("v4.11.0")
        .canonicalize()
        .ok()
    );
  }

  #[test]
  fn test_install_latest() {
    let base_dir = tempfile::tempdir().unwrap();
    let config = PactupConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));

    Install {
      version: None,
      nightly: false,
      latest: true,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    let latest_release =
      remote_pact_index::latest(&config.pact_4x_repo).expect("Can't get pact version list");
    let latest_version = latest_release.tag_name.clone();
    assert!(config.installations_dir().exists());
    assert!(config
      .installations_dir()
      .join(latest_version.to_string())
      .canonicalize()
      .unwrap()
      .exists());
  }

  #[test]
  fn test_install_nightly() {
    let base_dir = tempfile::tempdir().unwrap();
    let config = PactupConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));

    Install {
      version: None,
      nightly: true,
      latest: false,
      force: false,
      progress: ProgressConfig::Never,
    }
    .apply(&config)
    .expect("Can't install");

    let nightly_release =
      remote_pact_index::get_by_tag(&config.pact_5x_repo, &String::from("nightly"))
        .expect("Can't get pact version list");
    let latest_version = nightly_release.tag_name.clone();
    assert!(config.installations_dir().exists());
    assert!(config
      .installations_dir()
      .join(latest_version.to_string())
      .canonicalize()
      .unwrap()
      .exists());
  }
}
