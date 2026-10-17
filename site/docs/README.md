# Docs

`site/docs/` is memex's documentation: short, separate pages of plain HTML, organised as a very light Diátaxis (guides, reference, concepts), with no build step. The layout every page shares, and `docs.js`, are described in `site/README.md`.

There is no docs home: the docs open on `get-started.html`.

- `get-started.html`: setting memex up for agents. Install and `memex init`, what init wires up (the `AGENTS.md` block, `CLAUDE.md`, the Claude Code plugin), what agents do from then on, the orchestrator case, and what the human still does.
- `patterns/`: the how-to guides. `index.html` introduces them; one page per pattern, each with when to use it, the steps and the samples: `cases.html`, `doom-hunts.html`, `perf-verdicts.html`, `hand-off.html`, `drift.html`, and the anti-pattern `distill-logs.html` (copying logs into a collection adds noise; write down what is still true).
- `reference/`: `index.html` lists the verbs, the conventions they share and the exit codes; one page per verb (`init`, `info`, `show`, `search`, `check`, `log`), each with a short description, the flags that matter and a terminal sample; and `configuration.html`, the log store settings in `.memex/config.json` (design.md §4.1, §4.3), the one page that names those keys and numbers.
- `concepts.html`: the intent layer, compression and trails in short form, linking back to the intro's sections.
- `docs.js`: renders the sidebar nav (defined once, in `NAV`), the "On this page" table of contents and the mobile drawer on every page.

Behaviour comes from design.md (§2, §3, §5, §6) and README.md. When a verb's behaviour changes, change design.md first, then these pages. Add a page by writing its HTML (copy a sibling's shell) and adding it to `NAV` in `docs.js`.
