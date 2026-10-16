# hooks

`hooks/` holds the plugin's hook script and the registration that runs it. `hooks.json` runs `start.sh` on SessionStart and on SubagentStart: at session start the script prints `memex info`, and for a sub-agent it adds that agent's memex id and `memex info --agent <id>` as context (design §6). It never sets `MEMEX_AGENT`, and it always exits 0.
