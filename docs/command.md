# `pactup`

```
A fast and simple Pact manager

Usage: pactup [OPTIONS] <COMMAND>

Commands:
  list-remote  List all remote Pact versions [aliases: ls-remote]
  list         List all locally installed Pact versions [aliases: ls]
  install      Install a new Pact version
  use          Change Pact version
  env          Print and set up required environment variables for pactup
  completions  Print shell completions to stdout
  alias        Alias a version to a common name
  unalias      Remove an alias definition
  default      Set a version as the default version
  current      Print the current Pact version
  exec         Run a command within pactup context
  uninstall    Uninstall a Pact version
  help         Print this message or the help of the given subcommand(s)

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

# `pactup list-remote`

```
List all remote Pact versions

Usage: pactup list-remote [OPTIONS]

Options:
      --filter <FILTER>
          Filter versions by a user-defined version or a semver range

      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --nightly
          Show nightly versions

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --sort <SORT>
          Version sorting order

          [default: asc]

          Possible values:
          - desc: Sort versions in descending order (latest to earliest)
          - asc:  Sort versions in ascending order (earliest to latest)

      --latest
          Only show the latest matching version

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup list`

```
List all locally installed Pact versions

Usage: pactup list [OPTIONS]

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup install`

```
Install a new Pact version

Usage: pactup install [OPTIONS] [VERSION]

Arguments:
  [VERSION]
          A version string. Can be a partial semver or a 'development' version

Options:
      --nightly
          Install latest nightly version

      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --latest
          Install latest version

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --progress <PROGRESS>
          Show an interactive progress bar for the download status

          [default: auto]
          [possible values: auto, never, always]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup use`

```
Change Pact version

Usage: pactup use [OPTIONS] [VERSION]

Arguments:
  [VERSION]


Options:
      --install-if-missing
          Install the version if it isn't installed yet

      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --silent-if-unchanged
          Don't output a message identifying the version being used if it will not change due to execution of this command

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup env`

```
Print and set up required environment variables for pactup

This command generates a series of shell commands that should be evaluated by your shell to create a pactup-ready environment.

Each shell has its own syntax of evaluating a dynamic expression. For example, evaluating pactup on Bash and Zsh would look like `eval "$(pactup env)"`. In Fish, evaluating would look like `pactup env | source`

Usage: pactup env [OPTIONS]

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --shell <SHELL>
          The shell syntax to use. Infers when missing

          [possible values: bash, zsh, fish, power-shell]

      --json
          Print JSON instead of shell commands

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --use-on-cd
          Print the script to change Node versions every directory change

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup completions`

```
Print shell completions to stdout

Usage: pactup completions [OPTIONS]

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --shell <SHELL>
          The shell syntax to use. Infers when missing

          [possible values: bash, zsh, fish, power-shell]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup alias`

```
Alias a version to a common name

Usage: pactup alias [OPTIONS] <TO_VERSION> <NAME>

Arguments:
  <TO_VERSION>


  <NAME>


Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup unalias`

```
Remove an alias definition

Usage: pactup unalias [OPTIONS] <REQUESTED_ALIAS>

Arguments:
  <REQUESTED_ALIAS>


Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup default`

```
Set a version as the default version

This is a shorthand for `pactup alias VERSION default`

Usage: pactup default [OPTIONS] <VERSION>

Arguments:
  <VERSION>


Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup current`

```
Print the current Pact version

Usage: pactup current [OPTIONS]

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup exec`

```
Run a command within pactup context

Example:
--------
pactup exec --using=v4.11.0 pact --version
=> v4.11.0

Usage: pactup exec [OPTIONS] [ARGUMENTS]...

Arguments:
  [ARGUMENTS]...
          The command to run

Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --using <VERSION>
          Either an explicit version, or a filename with the version written in it

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup uninstall`

```
Uninstall a Pact version

> Warning: when providing an alias, it will remove the Pact version the alias > is pointing to, along with the other aliases that point to the same version.

Usage: pactup uninstall [OPTIONS] [VERSION]

Arguments:
  [VERSION]


Options:
      --pact-4x-repo <PACT_4X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT4X_REPO]
          [default: kadena-io/pact]

      --pact-5x-repo <PACT_5X_REPO>
          <https://github.com/kadena-io/pact>

          [env: PACTUP_PACT5X_REPO]
          [default: kadena-io/pact-5]

      --pact-dir <BASE_DIR>
          The root directory of pact installations

          [env: PACTUP_PACT_DIR]

      --log-level <LOG_LEVEL>
          The log level of pactup commands

          [env: PACTUP_LOGLEVEL]
          [default: info]
          [possible values: quiet, error, info]

      --arch <ARCH>
          Override the architecture of the installed pact binary. Defaults to arch of pactup binary

          [env: PACTUP_ARCH]

      --version-file-strategy <VERSION_FILE_STRATEGY>
          A strategy for how to resolve the Pact version. Used whenever `pactup use` or `pactup install` is called without a version, or when `--use-on-cd` is configured on evaluation

          [env: PACTUP_VERSION_FILE_STRATEGY]
          [default: local]

          Possible values:
          - local:     Use the local version of Node defined within the current directory
          - recursive: Use the version of Node defined within the current directory and all parent directories

  -h, --help
          Print help (see a summary with '-h')
```

# `pactup help`

```

```
