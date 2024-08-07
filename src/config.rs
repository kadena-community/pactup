use crate::log_level::LogLevel;
use crate::path_ext::PathExt;
use crate::version_file_strategy::VersionFileStrategy;
use crate::{arch::Arch, directories::Directories};

#[derive(clap::Parser, Debug)]
pub struct PactupConfig {
  /// <https://github.com/kadena-io/pact>
  #[clap(
    long,
    env = "PACTUP_PACT4X_REPO",
    default_value = "kadena-io/pact",
    global = true,
    hide_env_values = true
  )]
  pub pact_4x_repo: String,

  /// <https://github.com/kadena-io/pact>
  #[clap(
    long,
    env = "PACTUP_PACT5X_REPO",
    default_value = "kadena-io/pact-5",
    global = true,
    hide_env_values = true
  )]
  pub pact_5x_repo: String,

  /// The root directory of pact installations.
  #[clap(
    long = "pactup-dir",
    env = "PACTUP_PACT_DIR",
    global = true,
    hide_env_values = true
  )]
  pub base_dir: Option<std::path::PathBuf>,

  /// Where the current pact version link is stored.
  /// This value will be populated automatically by evaluating
  /// `pactup env` in your shell profile. Read more about it using `pactupenv`
  #[clap(
    long,
    env = "PACTUP_MULTISHELL_PATH",
    hide_env_values = true,
    hide = true
  )]
  multishell_path: Option<std::path::PathBuf>,

  /// The log level of pactup commands
  #[clap(
    long,
    env = "PACTUP_LOGLEVEL",
    default_value_t,
    global = true,
    hide_env_values = true
  )]
  log_level: LogLevel,

  /// Override the architecture of the installed pact binary.
  /// Defaults to arch of pactup binary.
  #[clap(
    long,
    env = "PACTUP_ARCH",
    default_value_t,
    global = true,
    hide_env_values = true,
    hide_default_value = true
  )]
  pub arch: Arch,

  /// A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is
  /// called without a version, or when `--use-on-cd` is configured on evaluation.
  #[clap(
    long,
    env = "PACTUP_VERSION_FILE_STRATEGY",
    default_value_t,
    global = true,
    hide_env_values = true
  )]
  version_file_strategy: VersionFileStrategy,

  /// Resolve `engines.pact` field in `package.json` whenever a `.pact-version` or `.pactrc` file is not present.
  /// Experimental: This feature is subject to change.
  /// Note: `engines.pact` can be any semver range, with the latest satisfying version being resolved.
  #[clap(
    long,
    env = "PACTUP_RESOLVE_ENGINES",
    global = true,
    hide_env_values = true,
    verbatim_doc_comment
  )]
  resolve_engines: bool,

  #[clap(skip)]
  directories: Directories,
}

impl Default for PactupConfig {
  fn default() -> Self {
    Self {
      pact_4x_repo: "kadena-io/pact".to_string(),
      pact_5x_repo: "kadena-io/pact-5".to_string(),
      base_dir: None,
      multishell_path: None,
      log_level: LogLevel::Info,
      arch: Arch::default(),
      version_file_strategy: VersionFileStrategy::default(),
      directories: Directories::default(),
      resolve_engines: false,
    }
  }
}

impl PactupConfig {
  pub fn version_file_strategy(&self) -> &VersionFileStrategy {
    &self.version_file_strategy
  }

  pub fn resolve_engines(&self) -> bool {
    self.resolve_engines
  }

  pub fn multishell_path(&self) -> Option<&std::path::Path> {
    match &self.multishell_path {
      None => None,
      Some(v) => Some(v.as_path()),
    }
  }

  pub fn log_level(&self) -> &LogLevel {
    &self.log_level
  }

  pub fn base_dir_with_default(&self) -> std::path::PathBuf {
    let user_pref = self.base_dir.clone();
    if let Some(dir) = user_pref {
      return dir;
    }

    self.directories.default_base_dir()
  }

  pub fn installations_dir(&self) -> std::path::PathBuf {
    self
      .base_dir_with_default()
      .join("pact-versions")
      .ensure_exists_silently()
  }

  pub fn multishell_storage(&self) -> std::path::PathBuf {
    self.directories.multishell_storage()
  }

  pub fn default_version_dir(&self) -> std::path::PathBuf {
    self.aliases_dir().join("default")
  }

  pub fn aliases_dir(&self) -> std::path::PathBuf {
    self
      .base_dir_with_default()
      .join("aliases")
      .ensure_exists_silently()
  }

  #[cfg(test)]
  pub fn with_base_dir(mut self, base_dir: Option<std::path::PathBuf>) -> Self {
    self.base_dir = base_dir;
    self
  }
}
