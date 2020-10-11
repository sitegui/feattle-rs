#!/bin/bash -e

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION=$1

# cargo install cargo-readme
for PACKAGE in feattle-core feattle-sync feattle-ui; do
  cargo readme --project-root $PACKAGE --template ../README.tpl > $PACKAGE/README.md
  sed -i "s/^version = \".*\"$/version = \"$VERSION\"/" $PACKAGE/Cargo.toml
done