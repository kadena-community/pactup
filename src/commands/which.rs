use super::command::Command;
use crate::config::PactupConfig;
use crate::installed_versions;
use crate::user_version_reader::UserVersionReader;
use crate::version_file_strategy::VersionFileStrategy;
use colored::Colorize;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Which {
  version: Option<UserVersionReader>,
}

impl Command for Which {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let all_versions = installed_versions::list(config.installations_dir())
      .map_err(|source| Error::VersionListingError { source })?;
    let requested_version = self
      .version
      .unwrap_or_else(|| {
        let current_dir = std::env::current_dir().unwrap();
        UserVersionReader::Path(current_dir)
      })
      .into_user_version(config)
      .ok_or_else(|| match config.version_file_strategy() {
        VersionFileStrategy::Local => InferVersionError::Local,
        VersionFileStrategy::Recursive => InferVersionError::Recursive,
      })
      .map_err(|source| Error::CantInferVersion { source })?;

    let current_version = requested_version.to_version(&all_versions, config);
    if let Some(version) = current_version {
      println!("{}", version.installation_path(config).display());
    } else {
      let error_message = format!(
        "Can't find an installed Pact version matching {}.",
        requested_version.to_string().italic()
      );
      eprintln!("{}", error_message.red());
    }
    Ok(())
  }
}

#[derive(Debug, Error)]
pub enum Error {
  #[error("Can't get locally installed versions: {}", source)]
  VersionListingError { source: installed_versions::Error },

  #[error(transparent)]
  CantInferVersion {
    #[from]
    source: InferVersionError,
  },
}

#[derive(Debug, Error)]
pub enum InferVersionError {
  #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
  Local,
  #[error("Could not find any version to use. Maybe you don't have a default version set?\nTry running `pactup default <VERSION>` to set one,\nor create a .pact-version file inside your project to declare a Pact version.")]
  Recursive,
}
