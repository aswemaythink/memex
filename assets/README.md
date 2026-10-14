# assets

The files that memex embeds in its binary. `agents-snippet.md` is the AGENTS.md block of design §6, byte for byte, and it has one source: `src/verbs/init/agents.rs` includes it with `include_str!`, `memex init` writes it between the memex markers, and `tests/selfhost.rs` checks that this repo's AGENTS.md carries the same bytes. The `SKILL.md` of the plugin (S6.1) will carry it too. Edit the design, then this file, then rerun `memex init` here so the block follows.
