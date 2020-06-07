#!/bin/bash

set -ex
export CARGO_OPTIONS="--all-features --workspace"

if [ "$TRAVIS_RUST_VERSION" = "nightly" ] && [ -z "$TRAVIS_TAG" ]; then
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Clink-dead-code -Coverflow-checks=off -Copt-level=0 -Cpanic=abort -Zpanic_abort_tests"

  wget https://github.com/mozilla/grcov/releases/download/v0.5.5/grcov-linux-x86_64.tar.bz2
  tar xvf grcov-linux-x86_64.tar.bz2

  wget https://github.com/Kogia-sima/rust-covfix/releases/download/v0.2.1/rust-covfix-linux-x86_64.tar.xz
  tar xvf rust-covfix-linux-x86_64.tar.xz
  mv rust-covfix-linux-x86_64/rust-covfix ./
fi

cargo build $CARGO_OPTIONS
cargo test $CARGO_OPTIONS

if [ "$TRAVIS_RUST_VERSION" = "nightly" ] && [ -z "$TRAVIS_TAG" ]; then
  zip -0 ccov.zip `find . \( -name "sailfish*.gc*" -o -name "test-*.gc*" \) -print`
  ./grcov ccov.zip -s . -t lcov --llvm --ignore "/*" --ignore "integration-tests/*" -o lcov.info
  sed -e 's/^SF:src\//SF:sailfish\/src\//' -i lcov.info
  ./rust-covfix lcov.info -o lcov.info
  bash <(curl -s https://codecov.io/bash) -f lcov.info
fi
