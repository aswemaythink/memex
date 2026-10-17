# Site

`site/` is memex's documentation site, one static page: `index.html`. It says what memex is, how to install it and how to use it. It is plain HTML styled with Tailwind from its pinned Play CDN (`cdn.tailwindcss.com/3.4.17`), with no build step, so opening the page in a browser is the whole dev loop. It supersedes the story-first page of PLAN.md §6 Phase 9, at the user's direction.

`index.html` runs top to bottom, with a sticky nav of anchor links:

1. Introduction: what memex is (README.md, design.md §1).
2. Principles: Markdown first, agent-agnostic, intent not code, conventions, bounded output, logs expire (design.md §1).
3. Get started: one command, `npx @rynwhite/memex init`, marked as coming with v0.1.0, with every other route (toolchains, Releases, a checkout, the plugin) in a native `<details>` (design.md §3.5b, §6.1).
4. Context compression: a short lead on authored context compression; "How an agent explores your codebase" with its paragraph and the walk diagram (two agents in turn on one tree), centred; "The intent layer", a table in compression order (READMEs, headers, collections, logs, code), each row with a CSS-only example popover (hover, keyboard focus or tap); and "The bigger picture", one line and the scale diagram (design.md §1, §2, §4.3, §5).
5. Trailblazing: trails let the next agent start where the last one stopped; "Logs" and "Collections", each opening with its definition in bold, then its paragraphs and diagram. The page names no log size, day count or config key; retention belongs in a later docs section (design.md §1, §2.2, §3.4).
6. Patterns: cases, doom hunts, performance verdicts, hand-off and messages, checking intent against code (drift), and one anti-pattern, promoting logs into a collection (design.md §2.2, §4.2, §5).
7. The verbs agents use: init, info, show, search, check and log, each a short description followed by its terminal sample (design.md §3, README.md).
8. Story: STORY.md retold in full, on a dashed trail rail with orange markers, the desk sketch and the pull quote.

The other files:

- `site.css`: the styles Tailwind utilities cannot express: colour tokens for light and dark, links, inline code, terminals, the details marker and the logo mark.
- `site.js`: the Tailwind Play CDN config, which maps colour names to the tokens in `site.css` and sets the fonts and the page width.
- `walk.svg`, `scale.svg`, `logs.svg`, `collections.svg`, `desk.svg`: the walk diagram (a still repo tree in grey, a dotted rail beside it and a README or header chip per folder and file, dim until read; two agents take turns on the rail, with the current task named at the top: one, to understand the API, reads the READMEs down to `api` and the headers of its three files and opens no code; the other, to change the schema, reads the same READMEs, opens `api/model.ts`, then reads db's README and opens the migration `db/001_orders.sql`), the scale diagram (sixty files, each a dim code block under a thin orange intent strip; an agent zigzags down the gaps between them, lighting the one strip per row it reads, then the code of three of those files lights up, the files the headers pointed to; it spans the full column), the logs diagram (worker-1 and worker-2 each add an entry to the top of an `impl` stream drawn as a stack of ref-labelled cards and send the orchestrator one line, `report: impl:…`; each new entry pushes the stack down and the oldest fades out, pruned; the orchestrator opens one ref, that entry lights, and one line from it moves into a README chip), the collections diagram (a `performance-optimize` collection of four records, PERF-01 to -04, each with an id, a title and a KEPT or CUT verdict; an agent tasked to make checkout faster runs `memex search --in performance-optimize` down a rail beside them, each record lights as it is read, the cut ones settle dim and struck through, the kept ones stay lit, then a dashed PERF-05 joins the column, trying) and the story's desk sketch. The four diagrams animate unless the reader prefers reduced motion, which shows the walk's first run at its end, the scale route with its lit strips and three highlighted files, the log stack with the opened entry lit and the README written, and the collection with its verdicts settled and PERF-05 in place. Each carries its own light and dark palette in a `prefers-color-scheme` style block, since an `<img>` cannot read the page's custom properties.

Colours are CSS custom properties on `:root`, switched by `prefers-color-scheme`, and the Tailwind config maps its colour names to them. Everything (header, prose, terminals, table, diagrams, footer) shares one 48rem column, so every edge lines up; on phones it is full width with a 16px gutter, and terminal blocks scroll inside themselves.

## Preview

From the repo root, serve the folder and open http://localhost:8000:

```sh
python3 -m http.server --directory site 8000
```

Opening `site/index.html` from disk works too (it needs the network for the Tailwind CDN).

## Deploy

`.github/workflows/pages.yml` publishes this folder to GitHub Pages on every push to main that touches `site/**`. Turn it on once under Settings, Pages, Source: GitHub Actions.

## Before it goes live

- The planned install lines name the packages of design.md §6.1 (`@rynwhite/memex`, `rynwhite/tap/memex`, `agent-memex`). They work once the v0.1.0 release of PLAN.md Phase 8 exists; then drop the "Coming with v0.1.0" box and hold the first deploy until then.
- The Play CDN prints a console warning that it is not meant for production. Swapping it for a prebuilt stylesheet would add a build step, so it is a ruling for the user.
- When a verb's behaviour or the story changes, change design.md or STORY.md first, then this page.
