use crate::version_file_strategy::VersionFileStrategy;

use super::shell::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct Fish;

impl Shell for Fish {
  fn to_clap_shell(&self) -> clap_complete::Shell {
    clap_complete::Shell::Fish
  }

  fn path(&self, path: &Path) -> anyhow::Result<String> {
    let path = path
      .to_str()
      .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
    let path =
      super::windows_compat::maybe_fix_windows_path(path).unwrap_or_else(|| path.to_string());
    Ok(format!("set -gx PATH {path:?} $PATH;"))
  }

  fn set_env_var(&self, name: &str, value: &str) -> String {
    format!("set -gx {name} {value:?};")
  }

  fn use_on_cd(&self, config: &crate::config::PactupConfig) -> anyhow::Result<String> {
    let version_file_exists_condition = if config.resolve_engines() {
      "test -f .pact-version -o -f .pactrc -o -f package.json"
    } else {
      "test -f .pact-version -o -f .pactrc"
    };
    let autoload_hook = match config.version_file_strategy() {
      VersionFileStrategy::Local => formatdoc!(
        r"
                    if {version_file_exists_condition}
                        pactup use --silent-if-unchanged
                    fi
                ",
        version_file_exists_condition = version_file_exists_condition,
      ),
      VersionFileStrategy::Recursive => String::from(r"pactup use --silent-if-unchanged"),
    };
    Ok(formatdoc!(
      r"
                function __pactup_autoload_hook --on-variable PWD --description 'Change Pact version on directory change'
                    status --is-command-substitution; and return
                    {autoload_hook}
                end

                __pactup_autoload_hook
            ",
      autoload_hook = autoload_hook
    ))
  }
}
