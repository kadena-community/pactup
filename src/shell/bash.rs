use crate::version_file_strategy::VersionFileStrategy;

use super::shell::Shell;
use indoc::{formatdoc, indoc};
use std::path::Path;

#[derive(Debug)]
pub struct Bash;

impl Shell for Bash {
  fn to_clap_shell(&self) -> clap_complete::Shell {
    clap_complete::Shell::Bash
  }

  fn path(&self, path: &Path) -> anyhow::Result<String> {
    let path = path
      .to_str()
      .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
    let path =
      super::windows_compat::maybe_fix_windows_path(path).unwrap_or_else(|| path.to_string());
    Ok(format!("export PATH={path:?}:$PATH"))
  }

  fn set_env_var(&self, name: &str, value: &str) -> String {
    format!("export {name}={value:?}")
  }

  fn use_on_cd(&self, config: &crate::config::PactupConfig) -> anyhow::Result<String> {
    let autoload_hook = match config.version_file_strategy() {
      VersionFileStrategy::Local => indoc!(
        r"
                    if [[ -f .pact-version || -f .pactrc ]]; then
                        pactup use --silent-if-unchanged
                    fi
                "
      ),
      VersionFileStrategy::Recursive => r"pactup use --silent-if-unchanged",
    };
    Ok(formatdoc!(
      r#"
                __pactup_use_if_file_found() {{
                    {autoload_hook}
                }}

                __pactup_cd() {{
                    \cd "$@" || return $?
                    __pactup_use_if_file_found
                }}

                alias cd=__pactup_cd
                __pactup_use_if_file_found
            "#,
      autoload_hook = autoload_hook
    ))
  }
}
