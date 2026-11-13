#!/usr/bin/env bash
# Sets the crate version for a release. Called by semantic-release (prepareCmd).
# Usage: scripts/set-version.sh 1.2.3
set -euo pipefail

version="${1:?usage: set-version.sh <version>}"
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "invalid version: $version" >&2
  exit 1
fi

cd "$(dirname "$0")/.."

# Replace only the first `version = ` line (the [package] one).
sed -i.bak -E "0,/^version = \".*\"/s//version = \"${version}\"/" Cargo.toml
rm -f Cargo.toml.bak

# Refresh the root package's entry in Cargo.lock without touching dependencies.
cargo update --workspace --offline

grep -m1 '^version = ' Cargo.toml
