#!/bin/bash

set -e

DIRECTORY="$(dirname "$0")"
SHELL_TO_RUN="$1"
PROFILE_FILE="$("$DIRECTORY/get_shell_profile.sh" "$SHELL_TO_RUN")"

ls -lah ~
echo "---"
echo "Profile is $PROFILE_FILE"
echo "---"
cat "$PROFILE_FILE"
echo "---"
echo "PATH=$PATH"
echo "---"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  pactup --version
"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  pactup install 4.11
  pactup ls | grep 4.11

  echo 'pactup ls worked.'
"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  pactup use 4.11
  pact --version | grep 4.11

  echo 'pact --version worked.'
"
