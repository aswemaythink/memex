#!/usr/bin/env bash
# Fails when the version files disagree with Cargo.toml, or with the release tag when one is given.
# Usage: scripts/check-versions.sh [repo-root] [tag]. The tag may also come from RELEASE_TAG; a `v` prefix is optional.
# Checked: Cargo.toml, Cargo.lock, the Claude Code plugin manifest and the top CHANGELOG.md heading. The npm package
# and the Homebrew formula are generated from Cargo.toml by cargo-dist, so they follow it.
set -euo pipefail

root="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
tag="${2:-${RELEASE_TAG:-}}"
crate="agent-memex"

# The version line of [package] in Cargo.toml.
cargo_toml=$(awk '/^\[/{p=($0=="[package]")} p && /^version *=/{gsub(/.*= *"|".*/,""); print; exit}' "$root/Cargo.toml")
# The version of this crate's own entry in Cargo.lock.
cargo_lock=$(awk -v n="$crate" '$0=="name = \"" n "\""{f=1; next} f && /^version *=/{gsub(/.*= *"|".*/,""); print; exit}' "$root/Cargo.lock")
plugin=$(sed -n 's/^ *"version": *"\([^"]*\)".*/\1/p' "$root/plugins/memex/.claude-plugin/plugin.json" | head -n1)
changelog=$(sed -n 's/^## \([0-9][^ ]*\).*/\1/p' "$root/CHANGELOG.md" | head -n1)

fail=0
check() { # name value
  if [ "$2" = "$cargo_toml" ]; then
    printf 'ok    %-42s %s\n' "$1" "$2"
  else
    printf 'DRIFT %-42s %s (Cargo.toml has %s)\n' "$1" "${2:-<missing>}" "$cargo_toml"
    fail=1
  fi
}

if [ -z "$cargo_toml" ]; then echo "no [package] version in $root/Cargo.toml"; exit 1; fi
printf 'ok    %-42s %s\n' "Cargo.toml" "$cargo_toml"
check "Cargo.lock ($crate)" "$cargo_lock"
check "plugins/memex/.claude-plugin/plugin.json" "$plugin"
check "CHANGELOG.md top heading" "$changelog"
if [ -n "$tag" ]; then check "release tag $tag" "${tag#v}"; fi

if [ "$fail" -ne 0 ]; then
  echo "Versions disagree: bump every file above to one version (see CONTRIBUTING.md, Releases)."
  exit 1
fi
