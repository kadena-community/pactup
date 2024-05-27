#!/bin/bash

set -e

export PATH=$PATH_ADDITION:$PATH

GAL_PROMPT_PREFIX="\e[34m✡\e[m  "

function type() {
  printf $GAL_PROMPT_PREFIX
  echo -n " "
  echo $* | npx tsx scripts/type-letters.ts
}

type 'eval "$(pactup env)"'
eval "$(pactup env)"

type 'pactup --version'
pactup --version

type 'cat .pact-version'
cat .pact-version

type 'pactup install'
pactup install --progress=never

type 'pactup use'
pactup use

type 'pact -v'
pact -v

sleep 2
echo ""
