#!/usr/bin/env -S just --justfile

_default:
  @just --list -u

alias r := ready

# Make sure you have cargo-binstall installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  cargo binstall cargo-watch cargo-insta typos-cli taplo-cli wasm-pack cargo-llvm-cov cargo-shear -y

# When ready, run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  just fmt
  just check
  just test
  just lint
  just doc
  just ast
  cargo shear
  git status

# Clone or update submodules
# submodules:
#   just clone-submodule tasks/coverage/test262 git@github.com:tc39/test262.git 17ba9aea47e496f5b2bc6ce7405b3f32e3cfbf7a
#   just clone-submodule tasks/coverage/babel git@github.com:babel/babel.git 4bd1b2c2f1bb3f702cfcb50448736e33c7000128
#   just clone-submodule tasks/coverage/typescript git@github.com:microsoft/TypeScript.git 64d2eeea7b9c7f1a79edf42cb99f302535136a2e
#   just clone-submodule tasks/prettier_conformance/prettier git@github.com:prettier/prettier.git 7142cf354cce2558f41574f44b967baf11d5b603

install-hook:
  echo "#!/bin/sh\njust fmt" > .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

# --no-vcs-ignores: cargo-watch has a bug loading all .gitignores, including the ones listed in .gitignore
# use .ignore file getting the ignore list
# Run `cargo watch`
watch command:
  cargo watch --no-vcs-ignores -i '*snap*' -x '{{command}}'

# Run the example in `cli`, `installer`, `transform`
example tool *args='':
  just watch 'run -p pact_toolbox_{{tool}} --example {{tool}} -- {{args}}'

# Format all files
fmt:
  cargo fmt
  taplo format

# Run cargo check
check:
  cargo ck

# Run all the tests
test:
  cargo test

# Lint the whole project
lint:
  cargo lint -- --deny warnings

doc:
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items



# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Removed Unused Dependencies
shear:
  cargo shear --fix

# Automatically DRY up Cargo.toml manifests in a workspace.
autoinherit:
  cargo binstall cargo-autoinherit
  cargo autoinherit

# # Test Transform
# test-transform *args='':
#   cargo run -p oxc_transform_conformance -- {{args}}
#   cargo run -p oxc_transform_conformance -- --exec  {{args}}

# Build oxlint in release build
pactup:
  cargo build --release -p pactup --bin pactup

# # Generate the JavaScript global variables. See `tasks/javascript_globals`
# javascript-globals:
#   cargo run -p javascript_globals

# Upgrade all Rust dependencies
upgrade:
  cargo upgrade --incompatible

clone-submodule dir url sha:
  git clone --depth=1 {{url}} {{dir}} || true
  cd {{dir}} && git fetch origin {{sha}} && git reset --hard {{sha}}
