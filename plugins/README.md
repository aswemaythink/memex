# plugins

`plugins/` holds the Claude Code plugins that memex ships. Today there is one, [memex](memex/README.md), and it is optional: memex works without it, because agents read AGENTS.md. The repo root's `.claude-plugin/marketplace.json` lists the plugins, so this repo is its own marketplace (design §6). Cargo leaves this folder out of the published crate.
