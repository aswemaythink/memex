#!/bin/sh
# SessionStart and SubagentStart hook of the memex plugin (design §6). Its input is the hook JSON on stdin.
# SessionStart prints `memex info`. SubagentStart prints the sub-agent's memex id and `memex info --agent <id>` as
# additionalContext. It never sets MEMEX_AGENT, and it always exits 0: a memex error only loses its context.
LC_ALL=C
export LC_ALL
TAB=$(printf '\t')
CR=$(printf '\r')
NL=$(printf '\nx')
NL=${NL%x}

# valid_id ID succeeds for 1 to 64 characters of [A-Za-z0-9._@/+-], the agent types that memex also accepts.
valid_id() {
  case "$1" in
    '' | *[!A-Za-z0-9._@/+-]*) return 1 ;;
  esac
  [ "${#1}" -le 64 ]
}

# field NAME prints the string value of the first JSON field NAME in $input, or nothing.
field() {
  printf '%s\n' "$input" | sed -n "s/.*\"$1\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" | head -n 1
}

# json_escape reads text on stdin and prints it as the body of a JSON string: \ and " escaped, tab and CR as \t and \r,
# and each newline as \n, all on one line.
json_escape() {
  sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e "s/$TAB/\\\\t/g" -e "s/$CR/\\\\r/g" -e 's/$/\\n/' | tr -d '\n'
}

# hint prints the install hint in the form the event takes: JSON context for SubagentStart, plain stdout otherwise.
hint() {
  if [ "$event" = SubagentStart ]; then
    printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"SubagentStart","additionalContext":"memex not found: npx @aswemaythink/memex init\nalternative: cargo install agent-memex, then memex init"}}'
  else
    printf '%s\n' 'memex not found: npx @aswemaythink/memex init' 'alternative: cargo install agent-memex, then memex init'
  fi
}

input=$(cat)
event=$(field hook_event_name)

# The hook input's cwd is the folder the session or sub-agent runs in, so memex reads that folder's atlas: a worktree sub-agent
# sees its own. field does not unescape JSON, so a cwd with a backslash is not used, and neither is one that is not a folder.
cwd=$(field cwd)
case "$cwd" in
  '' | *\\*) ;;
  *) if [ -d "$cwd" ]; then cd "$cwd" 2>/dev/null || :; fi ;;
esac

if ! command -v memex >/dev/null 2>&1; then
  hint
  exit 0
fi

if [ "$event" != SubagentStart ]; then
  memex info
  exit 0
fi

agent=$(field agent_type)
valid_id "$agent" || agent=subagent
info=$(memex info --agent "$agent"; printf x)
info=${info%x}
context=$(printf '%s\n%s' "your memex id is $agent; pass --agent $agent to memex log and memex info" "$info"; printf x)
context=${context%x}
# json_escape writes one \n per line, so the context must end with a newline. A context cut by the cap ends with one too.
case "$context" in
  *"$NL") ;;
  *) context="$context$NL" ;;
esac
if [ "${#context}" -gt 9000 ]; then
  context=$(printf '%s' "$context" | head -c 8999; printf x)
  context="${context%x}$NL"
fi
printf '%s' '{"hookSpecificOutput":{"hookEventName":"SubagentStart","additionalContext":"'
printf '%s' "$context" | json_escape
printf '%s\n' '"}}'
exit 0
