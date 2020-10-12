#!/bin/bash -e

for PACKAGE in feattle-core feattle-sync feattle-ui feattle; do
  cargo readme --project-root "$PACKAGE" --template ../README.tpl > "$PACKAGE/README.md"
  if [[ $PACKAGE == feattle ]]; then
    cp feattle-core/README.md .
  fi
done