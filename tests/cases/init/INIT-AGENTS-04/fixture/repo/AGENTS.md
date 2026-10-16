# Notes

<!-- memex:begin -->
## memex: read intent before code
This repo keeps an intent layer. The graph: README.md per folder and a header comment atop each file
(first line = purpose). Dimensions: any Markdown file with `memex: <collection>` in its frontmatter
joins that collection, such as CASES.md (`memex: cases`): test ids with their claims; each id is in its test's name or in a comment on it.
- Overview: `memex info`: where everything is (graph, collections, streams, messages for you).
- Orient: `memex show <path>`: header, cites, who cites it, README chain; folders list children.
- Find: `memex search <words>` over READMEs, headers, collections and logs. Use `rg` for symbols.
- Tests for X: `memex search <words> --in cases` (or the collection `memex info` names); `memex show <ID>` gives claim and evidence; `rg <ID>` finds the test.
- Findings: `memex log <stream> [--to <agent>] <<'EOF' … EOF` prints a ref. Hand back the ref, not the report.
- Messages to you: `memex search to:<your-id>`.
- When you change code, update its header/README in the same change, then run `memex check <paths>` and fix what it lists. Cite ids and paths; don't copy.
<!-- memex:end -->

Tail.
