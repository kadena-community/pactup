<h1 align="center">
  Pact version manager (<code>pactup</code>)
</h1>

> 🚀 Fast and simple Pact version manager, built in Rust

<div align="center">
  <img src="./docs/pactup.svg" alt="Blazing fast!">
</div>

## Features

🌎 Cross-platform support (macOS, Windows, Linux)

✨ Single file, easy installation, instant startup

🚀 Built with speed in mind

📂 Works with `.pact-version` and `.pactrc` files

## Installation

### Using a script (macOS/Linux)

For `bash`, `zsh` and `fish` shells, there's an [automatic installation script](./scripts/install.sh).

First ensure that `curl` and `unzip` are already installed on you operating system. Then execute:

```sh
curl -fsSL https://raw.githubusercontent.com/kadena-community/pactup/main/scripts/install.sh | bash
```

#### Upgrade

Upgrading `pactup` is almost the same as installing it. To prevent duplication in your shell config file add `--skip-shell` to install command.

#### Parameters

`--install-dir`

Set a custom directory for pactup to be installed. The default is `$XDG_DATA_HOME/pactup` (if `$XDG_DATA_HOME` is not defined it falls back to `$HOME/.local/share/pactup` on linux and `$HOME/Library/Application Support/pactup` on MacOS).

`--skip-shell`

Skip appending shell specific loader to shell config file, based on the current user shell, defined in `$SHELL`. e.g. for Bash, `$HOME/.bashrc`. `$HOME/.zshrc` for Zsh. For Fish - `$HOME/.config/fish/conf.d/pactup.fish`

`--force-install`

macOS installations using the installation script are deprecated in favor of the Homebrew formula, but this forces the script to install using it anyway.

Example:

```sh
curl -fsSL https://raw.githubusercontent.com/kadena-community/pactup/main/scripts/install.sh | bash -s -- --install-dir "./.pactup" --skip-shell
```

### Manually

#### Using Cargo (Linux/macOS/Windows)

```sh
cargo install pactup
```

Then, [set up your shell for pactup](#shell-setup)

#### Using a release binary (Linux/macOS/Windows)

- Download the [latest release binary](https://github.com/Schniz/pactup/releases) for your system
- Make it available globally on `PATH` environment variable
- [Set up your shell for pactup](#shell-setup)

### Removing

To remove pactup (😢), just delete the `.pactup` folder in your home directory. You should also edit your shell configuration to remove any references to pactup (ie. read [Shell Setup](#shell-setup), and do the opposite).

## Completions

pactup ships its completions with the binary:

```sh
pactup completions --shell <SHELL>
```

Where `<SHELL>` can be one of the supported shells:

- `bash`
- `zsh`
- `fish`
- `power-shell`

Please follow your shell instructions to install them.

### Shell Setup

Environment variables need to be setup before you can start using pactup.
This is done by evaluating the output of `pactup env`.

> [!NOTE]
> Check out the [Configuration](./docs/configuration.md) section to enable highly
> recommended features, like automatic version switching.

Adding a `.pact-version` to your project is as simple as:

```bash
$ pact --version
pact version 4.11.0
$ echo "4.11" > .pact-version
```

Check out the following guides for the shell you use:

#### Bash

Add the following to your `.bashrc` profile:

```bash
eval "$(pactup env --use-on-cd)"
```

#### Zsh

Add the following to your `.zshrc` profile:

```zsh
eval "$(pactup env --use-on-cd)"
```

#### Fish shell

Create `~/.config/fish/conf.d/pactup.fish` add this line to it:

```fish
pactup env --use-on-cd | source
```

#### PowerShell

Add the following to the end of your profile file:

```powershell
pactup env --use-on-cd | Out-String | Invoke-Expression
```

- For macOS/Linux, the profile is located at `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
- On Windows to edit your profile you can run this in a PowerShell
  ```powershell
  notepad $profile
  ```

#### Windows Command Prompt aka Batch aka WinCMD

pactup is also supported but is not entirely covered. [You can set up a startup script](https://superuser.com/a/144348) and append the following lines:

```batch
@echo off
:: for /F will launch a new instance of cmd so we create a guard to prevent an infnite loop
if not defined PACTUP_AUTORUN_GUARD (
    set "PACTUP_AUTORUN_GUARD=AutorunGuard"
    FOR /f "tokens=*" %%z IN ('pactup env --use-on-cd') DO CALL %%z
)
```

#### Usage with Cmder

Usage is very similar to the normal WinCMD install, apart for a few tweaks to allow being called from the cmder startup script. The example **assumes** that the `CMDER_ROOT` environment variable is **set** to the **root directory** of your Cmder installation.
Then you can do something like this:

- Make a .cmd file to invoke it

```batch
:: %CMDER_ROOT%\bin\pactup_init.cmd
@echo off
FOR /f "tokens=*" %%z IN ('pactup env --use-on-cd') DO CALL %%z
```

- Add it to the startup script

```batch
:: %CMDER_ROOT%\config\user_profile.cmd
call "%CMDER_ROOT%\bin\pactup_init.cmd"
```

You can replace `%CMDER_ROOT%` with any other convenient path too.

## [Configuration](./docs/configuration.md)

[See the available configuration options for an extended configuration documentation](./docs/configuration.md)

## [Usage](./docs/commands.md)

[See the available commands for an extended usage documentation](./docs/commands.md)

## Contributing

PRs welcome :tada:

### Developing:

```sh
# Install Rust
git clone https://github.com/kadena-community/pactup
cd crates/pactup
cargo build
```

### Running Binary

```sh
cargo run -- --help # Will behave like `pactup --help`
```

### Running Tests:

```sh
cargo test
```

## Credit

Pact version manager is ported from the amazing [fnm](https://github.com/Schniz/fnm) codebase.
