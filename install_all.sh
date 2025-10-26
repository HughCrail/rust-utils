#!/bin/bash

set -e

members=$(tomlq -r '.workspace.members[]' Cargo.toml)

for member in $members; do
  echo "Installing ${member}..."
  cargo install --path "${member}"
done

echo "All members installed successfully."