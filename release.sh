#!/bin/bash -e

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <package> <version>"
    exit 1
fi

PACKAGE=$1
VERSION=$2

cargo readme --project-root "$PACKAGE" --template ../README.tpl > "$PACKAGE/README.md"
sed -i "s/^version = \".*\"$/version = \"$VERSION\"/" "$PACKAGE/Cargo.toml"
cargo publish --manifest-path "$PACKAGE/Cargo.toml"
