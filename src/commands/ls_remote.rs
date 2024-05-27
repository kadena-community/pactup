use crate::config::PactupConfig;
use crate::remote_pact_index;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct LsRemote {}

impl super::command::Command for LsRemote {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let all_versions = remote_pact_index::list(&config.pact_4x_repo)?;
    let nightly_versions = remote_pact_index::list(&config.pact_5x_repo)?;

    for version in all_versions {
      print!("{}", version.tag_name);
      if version.draft {
        print!(" (draft)");
      }
      if version.prerelease {
        print!(" (prerelease)");
      }

      println!();
    }

    for version in nightly_versions {
      print!("{}", version.tag_name);
      if version.draft {
        print!(" (draft)");
      }
      if version.prerelease {
        print!(" (prerelease)");
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
