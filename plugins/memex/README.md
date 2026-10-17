# memex plugin

The memex plugin is an optional Claude Code plugin for memex: one skill and one hook script. Nothing else is in it. It adds no MCP server, no other hooks and no per-agent code. The design is in DESIGN.md §6, and the install lines are in §6.1.

## Contents

- `.claude-plugin/plugin.json`: the plugin manifest.
- `skills/memex/SKILL.md`: the AGENTS.md snippet, verbatim, plus the usage patterns (design §5) and the conventions (design §2). `tests/selfhost.rs` checks that the snippet is in it.
- `hooks/hooks.json` and `hooks/start.sh`: the SessionStart and SubagentStart hooks.

## Install

Inside Claude Code:

```
/plugin marketplace add aswemaythink/memex
/plugin install memex
```

If `/plugin install memex` does not find the plugin, use its full id: `/plugin install memex@memex`.

The plugin needs `memex` on PATH. Install it with `npx @aswemaythink/memex init`, or with `cargo install agent-memex` and then `memex init`. The hook is a POSIX `sh` script, so it also needs `sh` on PATH. On Windows that is Git Bash.

## What the hook does

- **SessionStart** prints `memex info`, which becomes context for the session.
- **SubagentStart** adds, as `additionalContext`, the line `your memex id is <agent_type>; pass --agent <agent_type> to memex log and memex info`, followed by the output of `memex info --agent <agent_type>`. The context is capped at 9,000 bytes.
- Both events run memex in the `cwd` of the hook input, so a sub-agent that works in its own git worktree reads that worktree's atlas. A `cwd` that is not a folder, or that holds a backslash (the hook does not unescape JSON), is ignored.
- The hook never sets `MEMEX_AGENT`. A session-wide value would leak the parent's id into every sub-agent.
- Without `memex` on PATH, both events print the install hint instead.
- Errors from memex never fail the hook: it always exits 0.

## Check it

From the repo root, this runs the SessionStart hook and prints the overview:

```sh
printf '{"hook_event_name":"SessionStart","cwd":"%s"}' "$PWD" | sh plugins/memex/hooks/start.sh
```

`cargo test --locked --test plugin_hook` runs the hook's tests. They use a fixture repo and a fake memex, and they run on Unix only.

## Remove it

Uninstall the plugin in `/plugin`. That leaves the repo alone: `.memex/` and the marked block in AGENTS.md stay until you delete them (design §6.1).
