#!/bin/bash

DIRECTORY="$(dirname "$0")"

function setup_binary() {
  TEMP_DIR="/tmp/pactup-$(date '+%s')"
  mkdir "$TEMP_DIR"
  cp ./target/release/pactup "$TEMP_DIR/pactup"
  export PATH=$TEMP_DIR:$PATH
  export PACTUPDIR=$TEMP_DIR/.pactup

  # First run of the binary might be slower due to anti-virus software
  echo "Using $(which pactup)"
  echo "  with version $(pactup --version)"
}

setup_binary

RECORDING_PATH=$DIRECTORY/screen_recording

(rm -rf "$RECORDING_PATH" &>/dev/null || true)

asciinema rec \
  --command "$DIRECTORY/recorded_screen_script.sh" \
  --cols 70 \
  --rows 17 \
  "$RECORDING_PATH"

echo "Recording saved to $RECORDING_PATH"
sed "s@$TEMP_DIR@~@g" "$RECORDING_PATH" |
  npx svg-term-cli \
    --window \
    --out "docs/pactup.svg" \
    --height=17 \
    --width=70
