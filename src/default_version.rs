use log::info;

use crate::config::PactupConfig;
use crate::version::Version;
use std::str::FromStr;

pub fn find_default_version(config: &PactupConfig) -> Option<Version> {
  if let Ok(version_path) = config.default_version_dir().canonicalize() {
    let file_name = version_path.file_name()?;
    info!("Found default version: {:?}", file_name);
    Version::from_str(file_name.to_str()?).ok()?.into()
  } else {
    info!("No default version found");
    None
  }
}
