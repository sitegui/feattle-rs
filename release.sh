#!/bin/bash -e

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <package> <version>"
    exit 1
fi

PACKAGE=$1
VERSION=$2

sed -i "s/^version = \".*\"$/version = \"$VERSION\"/" "$PACKAGE/Cargo.toml"

cargo readme --project-root "$PACKAGE" --template ../README.tpl > "$PACKAGE/README.md"
if [[ $PACKAGE == feattle ]]; then
  cp feattle-core/README.md .
fi

cargo publish --manifest-path "$PACKAGE/Cargo.toml"