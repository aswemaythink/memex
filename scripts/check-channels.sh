#!/usr/bin/env bash
# Fails unless every install channel's latest memex version equals the given one.
# Channels: crates.io, npm, Homebrew, GitHub Releases. Usage: scripts/check-channels.sh <version|tag>.
# ATTEMPTS (default 1) retries every 30 s while a registry catches up.
# Needs curl, jq and an authenticated gh (GH_TOKEN in CI); it reads only public data and changes nothing.
set -euo pipefail

want="${1:?usage: check-channels.sh <version|tag>}"
want="${want#v}"
repo="aswemaythink/memex"
attempts="${ATTEMPTS:-1}"
ua="memex-release-check (https://github.com/$repo)"

crates() { curl -fsS -A "$ua" https://crates.io/api/v1/crates/agent-memex | jq -r '.crate.max_version'; }
npm_latest() { curl -fsS https://registry.npmjs.org/@aswemaythink%2fmemex | jq -r '."dist-tags".latest'; }
brew() {
  gh api repos/aswemaythink/homebrew-tap/contents/Formula/memex.rb --jq .content | base64 -d |
    sed -n 's/^ *version "\([^"]*\)".*/\1/p' | head -n1
}
github() { gh api "repos/$repo/releases/latest" --jq .tag_name | sed 's/^v//'; }

for attempt in $(seq 1 "$attempts"); do
  fail=0
  for channel in crates npm_latest brew github; do
    got=$("$channel" 2>/dev/null || true)
    if [ "$got" = "$want" ]; then
      printf 'ok    %-10s %s\n' "$channel" "$got"
    else
      printf 'BEHIND %-9s %s (want %s)\n' "$channel" "${got:-<unreadable>}" "$want"
      fail=1
    fi
  done
  [ "$fail" -eq 0 ] && exit 0
  if [ "$attempt" -lt "$attempts" ]; then echo "attempt $attempt/$attempts: retrying in 30 s"; sleep 30; fi
done
echo "Channels disagree with $want: see CONTRIBUTING.md, Releases, to catch a channel up."
exit 1
