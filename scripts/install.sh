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

# pactup-darwin-arm64.tar.gz
# pactup-darwin-x64.tar.gz
# pactup-linux-arm64-gnu.tar.gz
# pactup-linux-arm64-musl.tar.gz
# pactup-linux-x64-gnu.tar.gz
# pactup-linux-x64-musl.tar.gz
set_filename() {
  if [ "$OS" = "Linux" ]; then
    # Based on https://stackoverflow.com/a/45125525
    case "$(uname -m)" in
    aarch* | armv8*)
      FILENAME="pactup-linux-arm64-gnu.tar.gz"
      ;;
    *)
      FILENAME="pactup-linux-x64-gnu.tar.gz"
      ;;
    esac
  elif [ "$OS" = "Darwin" ]; then
    case "$(uname -m)" in
    aarch* | armv8*)
      FILENAME="pactup-darwin-arm64.tar.gz"
      ;;
    *)
      FILENAME="pactup-darwin-x64.tar.gz"
      ;;
    esac
  # elif [ "$OS" = "Windows" ]; then
  #   FILENAME="pactup-windows"
  #   echo "Downloading the latest pactup binary from GitHub..."
  else
    echo "OS $OS is not supported."
    echo "If you think that's a bug - please file an issue to https://github.com/kadena-community/pactup/issues"
    exit 1
  fi
}

download_pactup() {
  if [ "$RELEASE" = "latest" ]; then
    URL="https://github.com/kadena-community/pactup/releases/latest/download/$FILENAME"
  else
    URL="https://github.com/kadena-community/pactup/releases/download/$RELEASE/$FILENAME"
  fi

  DOWNLOAD_DIR=$(mktemp -d)

  echo "Downloading $URL..."

  mkdir -p "$INSTALL_DIR" &>/dev/null

  if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME.zip"; then
    echo "Download failed.  Check that the release/filename are correct."
    exit 1
  fi

  # if .tar.gz
  if [ "${FILENAME: -7}" == ".tar.gz" ]; then
    tar -xzf "$DOWNLOAD_DIR/$FILENAME.zip" -C "$DOWNLOAD_DIR"
  elif [ "${FILENAME: -4}" == ".zip" ]; then
    unzip -q "$DOWNLOAD_DIR/$FILENAME.zip" -d "$DOWNLOAD_DIR"
  else
    echo "Unknown file extension for $FILENAME"
    exit 1
  fi

  # binary name is same as filename without extension it could be .tar.gz or .zip
  if [ "${FILENAME: -7}" == ".tar.gz" ]; then
    BINARY_NAME="${FILENAME%.tar.gz}"
  elif [ "${FILENAME: -4}" == ".zip" ]; then
    BINARY_NAME="${FILENAME%.zip}"
  fi

  if [ -f "$DOWNLOAD_DIR/$BINARY_NAME" ]; then
    mv "$DOWNLOAD_DIR/$BINARY_NAME" "$INSTALL_DIR/pactup"
  else
    mv "$DOWNLOAD_DIR/$FILENAME/$BINARY_NAME" "$INSTALL_DIR/pactup"
  fi

  chmod u+x "$INSTALL_DIR/pactup"
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

  echo -n "Checking availability of tar... "
  if hash tar 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
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
    echo '  eval "`pactup env --use-on-cd`"'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'export PATH="'$INSTALL_DIR':$PATH"' >>$CONF_FILE
    echo 'eval "`pactup env --use-on-cd`"' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" = "fish" ]; then
    CONF_FILE=$HOME/.config/fish/conf.d/pactup.fish
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Fish. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # pactup'
    echo '  set PATH "'"$INSTALL_DIR"'" $PATH'
    echo '  pactup env --use-on-cd | source'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'set PATH "'"$INSTALL_DIR"'" $PATH' >>$CONF_FILE
    echo 'pactup env --use-on-cd | source' >>$CONF_FILE

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
    echo '  eval "`pactup env --use-on-cd`"'

    echo '' >>$CONF_FILE
    echo '# pactup' >>$CONF_FILE
    echo 'export PATH="'"$INSTALL_DIR"':$PATH"' >>$CONF_FILE
    echo 'eval "`pactup env --use-on-cd`"' >>$CONF_FILE

  else
    echo "Could not infer shell type. Please set up manually. $CURRENT_SHELL"
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
