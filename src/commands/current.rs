use super::command::Command;
use crate::config::PactupConfig;
use crate::current_version::{current_version, Error};

#[derive(clap::Parser, Debug)]
pub struct Current {
  #[clap(short, long)]
  path: bool,
}

impl Command for Current {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    let version = current_version(config)?;
    if let Some(ver) = version {
      if self.path {
        println!("{}", ver.installation_path(config).display());
        return Ok(());
      }
      println!("{}", ver.v_str());
    } else {
      println!("none");
    }

    Ok(())
  }
}
