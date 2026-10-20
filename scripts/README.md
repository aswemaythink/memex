# scripts

`scripts/` holds the release checks: plain bash, for maintainers and CI. See CONTRIBUTING.md, Releases. `check-versions.sh` fails when `Cargo.toml`, `Cargo.lock`, the plugin manifest, the top CHANGELOG heading and an optional tag disagree; check-versions.yml runs it on every pull request and tag. `check-channels.sh` fails unless crates.io, npm, Homebrew and GitHub Releases all show a given version; verify-channels.yml runs it after every release.
