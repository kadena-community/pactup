use super::command::Command;
use crate::config::PactupConfig;
use crate::fs::symlink_dir;
use crate::outln;
use crate::path_ext::PathExt;
use crate::shell::{infer_shell, Shell, Shells};
use clap::ValueEnum;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Env {
  /// The shell syntax to use. Infers when missing.
  #[clap(long)]
  shell: Option<Shells>,
  /// Print JSON instead of shell commands.
  #[clap(long, conflicts_with = "shell")]
  json: bool,
  /// Deprecated. This is the default now.
  #[clap(long, hide = true)]
  multi: bool,
  /// Print the script to change Pact versions every directory change
  #[clap(long)]
  use_on_cd: bool,
}

fn generate_symlink_path() -> String {
  format!(
    "{}_{}",
    std::process::id(),
    chrono::Utc::now().timestamp_millis(),
  )
}

pub fn make_symlink(config: &PactupConfig) -> Result<std::path::PathBuf, Error> {
  let base_dir = config.multishell_storage().ensure_exists_silently();
  let mut temp_dir = base_dir.join(generate_symlink_path());

  while temp_dir.exists() {
    temp_dir = base_dir.join(generate_symlink_path());
  }

  match symlink_dir(config.default_version_dir(), &temp_dir) {
    Ok(()) => Ok(temp_dir),
    Err(source) => Err(Error::CantCreateSymlink { source, temp_dir }),
  }
}

impl Command for Env {
  type Error = Error;

  fn apply(self, config: &PactupConfig) -> Result<(), Self::Error> {
    if self.multi {
      outln!(
        config,
        Error,
        "{} {} is deprecated. This is now the default.",
        "warning:".yellow().bold(),
        "--multi".italic()
      );
    }

    let multishell_path = make_symlink(config)?;
    let base_dir = config.base_dir_with_default();

    let env_vars = [
      ("PACTUP_MULTISHELL_PATH", multishell_path.to_str().unwrap()),
      (
        "PACTUP_VERSION_FILE_STRATEGY",
        config.version_file_strategy().as_str(),
      ),
      ("PACTUP_DIR", base_dir.to_str().unwrap()),
      ("PACTUP_LOGLEVEL", config.log_level().as_str()),
      ("PACTUP_PACT4X_REPO", config.pact_4x_repo.as_str()),
      ("PACTUP_PACT5X_REPO", config.pact_5x_repo.as_str()),
      ("PACTUP_ARCH", config.arch.as_str()),
    ];

    if self.json {
      println!(
        "{}",
        serde_json::to_string(&HashMap::from(env_vars)).unwrap()
      );
      return Ok(());
    }

    let shell: Box<dyn Shell> = self
      .shell
      .map(Into::into)
      .or_else(infer_shell)
      .ok_or(Error::CantInferShell)?;

    let binary_path = if cfg!(windows) {
      shell.path(&multishell_path)
    } else {
      shell.path(&multishell_path.join("bin"))
    };

    println!("{}", binary_path?);

    for (name, value) in &env_vars {
      println!("{}", shell.set_env_var(name, value));
    }

    if self.use_on_cd {
      println!("{}", shell.use_on_cd(config)?);
    }
    if let Some(v) = shell.rehash() {
      println!("{v}");
    }

    Ok(())
  }
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(
    "{}\n{}\n{}\n{}",
    "Can't infer shell!",
    "pactup infer your shell based on the process tree.",
    "Maybe it is unsupported? we support the following shells:",
    shells_as_string()
  )]
  CantInferShell,
  #[error("Can't create the symlink for multishells at {temp_dir:?}. Maybe there are some issues with permissions for the directory? {source}")]
  CantCreateSymlink {
    #[source]
    source: std::io::Error,
    temp_dir: std::path::PathBuf,
  },
  #[error(transparent)]
  ShellError {
    #[from]
    source: anyhow::Error,
  },
}

fn shells_as_string() -> String {
  Shells::value_variants()
    .iter()
    .map(|x| format!("* {x}"))
    .collect::<Vec<_>>()
    .join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_smoke() {
    let config = PactupConfig::default();
    Env {
      #[cfg(windows)]
      shell: Some(Shells::Cmd),
      #[cfg(not(windows))]
      shell: Some(Shells::Bash),
      ..Default::default()
    }
    .call(config);
  }
}
