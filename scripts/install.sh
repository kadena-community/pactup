#!/bin/bash

set -e

RELEASE="latest"
OS="$(uname -s)"

case "${OS}" in
MINGW* | Win*) OS="Windows" ;;
esac

if [ -d "$HOME/.pactup" ]; then
  INSTALL_DIR="$HOME/.pactup"
elif [ -n "$XDG_DATA_HOME" ]; then
  INSTALL_DIR="$XDG_DATA_HOME/pactup"
elif [ "$OS" = "Darwin" ]; then
  INSTALL_DIR="$HOME/Library/Application Support/pactup"
else
  INSTALL_DIR="$HOME/.local/share/pactup"
fi

# Parse Flags
parse_args() {
  while [[ $# -gt 0 ]]; do
    key="$1"

    case $key in
    -d | --install-dir)
      INSTALL_DIR="$2"
      shift # past argument
      shift # past value
      ;;
    -s | --skip-shell)
      SKIP_SHELL="true"
      shift # past argument
      ;;
    --force-install | --force-no-brew)
      echo "\`--force-install\`: I hope you know what you're doing." >&2
      FORCE_INSTALL="true"
      shift
      ;;
    -r | --release)
      RELEASE="$2"
      shift # past release argument
      shift # past release value
      ;;
    *)
      echo "Unrecognized argument $key"
      exit 1
      ;;
    esac
  done
}

set_filename() {
  if [ "$OS" = "Linux" ]; then
    # Based on https://stackoverflow.com/a/45125525
    case "$(uname -m)" in
    arm | armv7*)
      FILENAME="pactup-arm32"
      ;;
    aarch* | armv8*)
      FILENAME="pactup-arm64"
      ;;
    *)
      FILENAME="pactup-linux"
      ;;
    esac
  elif [ "$OS" = "Darwin" ] && [ "$FORCE_INSTALL" = "true" ]; then
    FILENAME="pactup-macos"
    USE_HOMEBREW="false"
    echo "Downloading the latest pactup binary from GitHub..."
    echo "  Pro tip: it's easier to use Homebrew for managing pactup in macOS."
    echo "           Remove the \`--force-no-brew\` so it will be easy to upgrade."
  elif [ "$OS" = "Darwin" ]; then
    USE_HOMEBREW="true"
    echo "Downloading pactup using Homebrew..."
  elif [ "$OS" = "Windows" ]; then
    FILENAME="pactup-windows"
    echo "Downloading the latest pactup binary from GitHub..."
  else
    echo "OS $OS is not supported."
    echo "If you think that's a bug - please file an issue to https://github.com/kadena-community/pactup/issues"
    exit 1
  fi
}

download_pactup() {
  if [ "$USE_HOMEBREW" = "true" ]; then
    brew install pactup
  else
    if [ "$RELEASE" = "latest" ]; then
      URL="https://github.com/kadena-community/pactup/releases/latest/download/$FILENAME.zip"
    else
      URL="https://github.com/kadena-community/pactup/releases/download/$RELEASE/$FILENAME.zip"
    fi

    DOWNLOAD_DIR=$(mktemp -d)

    echo "Downloading $URL..."

    mkdir -p "$INSTALL_DIR" &>/dev/null

    if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME.zip"; then
      echo "Download failed.  Check that the release/filename are correct."
      exit 1
    fi

    unzip -q "$DOWNLOAD_DIR/$FILENAME.zip" -d "$DOWNLOAD_DIR"

    if [ -f "$DOWNLOAD_DIR/pactup" ]; then
      mv "$DOWNLOAD_DIR/pactup" "$INSTALL_DIR/pactup"
    else
      mv "$DOWNLOAD_DIR/$FILENAME/pactup" "$INSTALL_DIR/pactup"
    fi

    chmod u+x "$INSTALL_DIR/pactup"
  fi
}

check_dependencies() {
  echo "Checking dependencies for the installation script..."

  echo -n "Checking availability of curl... "
  if hash curl 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  echo -n "Checking availability of unzip... "
  if hash unzip 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  if [ "$USE_HOMEBREW" = "true" ]; then
    echo -n "Checking availability of Homebrew (brew)... "
    if hash brew 2>/dev/null; then
      echo "OK!"
    else
      echo "Missing!"
      SHOULD_EXIT="true"
    fi
  fi

  if [ "$SHOULD_EXIT" = "true" ]; then
    echo "Not installing pactup due to missing dependencies."
    exit 1
  fi
}

ensure_containing_dir_exists() {
  local CONTAINING_DIR
  CONTAINING_DIR="$(dirname "$1")"
  if [ ! -d "$CONTAINING_DIR" ]; then
    echo " >> Creating directory $CONTAINING_DIR"
    mkdir -p "$CONTAINING_DIR"
  fi
}

setup_shell() {
  CURRENT_SHELL="$(basename "$SHELL")"

  if [ "$CURRENT_SHELL" = "zsh" ]; then
    CONF_FILE=${ZDOTDIR:-$HOME}/.zshrc
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Zsh. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # pactup'
    echo '  export PATH="'"$INSTALL_DIR"':$PATH"'
    echo '  eval "`pactup env`"'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'export PATH="'$INSTALL_DIR':$PATH"' >>$CONF_FILE
    echo 'eval "`pactup env`"' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" = "fish" ]; then
    CONF_FILE=$HOME/.config/fish/conf.d/pactup.fish
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Fish. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # pactup'
    echo '  set PATH "'"$INSTALL_DIR"'" $PATH'
    echo '  pactup env | source'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'set PATH "'"$INSTALL_DIR"'" $PATH' >>$CONF_FILE
    echo 'pactup env | source' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" = "bash" ]; then
    if [ "$OS" = "Darwin" ]; then
      CONF_FILE=$HOME/.profile
    else
      CONF_FILE=$HOME/.bashrc
    fi
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Bash. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # pactup'
    echo '  export PATH="'"$INSTALL_DIR"':$PATH"'
    echo '  eval "`pactup env`"'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'export PATH="'"$INSTALL_DIR"':$PATH"' >>$CONF_FILE
    echo 'eval "`pactup env`"' >>$CONF_FILE

  else
    echo "Could not infer shell type. Please set up manually."
    exit 1
  fi

  echo ""
  echo "In order to apply the changes, open a new terminal or run the following command:"
  echo ""
  echo "  source $CONF_FILE"
}

parse_args "$@"
set_filename
check_dependencies
download_pactup
if [ "$SKIP_SHELL" != "true" ]; then
  setup_shell
fi
