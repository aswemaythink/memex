# Site

`site/` is memex's documentation site: the introduction page and the docs pages. `index.html` is the introduction (what memex is, how to install it and how it works), and `docs/` holds short docs pages. They share one look, stylesheet and Tailwind config, so they read as one site. It is plain HTML styled with Tailwind from its pinned Play CDN (`cdn.tailwindcss.com/3.4.17`), with no build step, so opening a page in a browser is the whole dev loop. It supersedes the story-first page of PLAN.md §6 Phase 9, at the user's direction.

`index.html` runs top to bottom, with a sticky nav (Start, which covers Principles and Get started, then Compression, Trails, API, Story, Docs at `docs/get-started.html`, and GitHub):

1. Introduction: what memex is (README.md, design.md §1).
2. Principles: Markdown first, agent-agnostic, intent not code, conventions, bounded output, logs expire (design.md §1).
3. Get started: one command, `npx @aswemaythink/memex init`, with every other route (toolchains, Releases, a checkout, the plugin) in a native `<details>` (design.md §3.5b, §6.1).
4. Context compression: a short lead on authored context compression; "How an agent explores your codebase" with its paragraph and the walk diagram (two agents in turn on one tree), centred; "The intent layer", a table in compression order (READMEs, headers, collections, logs, code), each row with a CSS-only example popover (hover, keyboard focus or tap); and "The bigger picture", one line and the scale diagram (design.md §1, §2, §4.3, §5).
5. Trailblazing: trails let the next agent start where the last one stopped; "Logs" and "Collections", each opening with its definition in bold, then its paragraphs and diagram (design.md §1, §2.2, §3.4).
6. API at a glance: the six verbs, each in mono accent with one line (each linked to its page in `docs/reference/`), the read-only, `showing N of M`, `--limit` and `--json` rules in the lead, and a "Read the full docs" link to `docs/get-started.html` (design.md §3).
7. Story: STORY.md retold in full, on a dashed trail rail with orange markers, the desk sketch and the pull quote.

`docs/` holds the docs, organised as a very light Diátaxis: short pages, one idea per section, with generous spacing. There is no docs home: the docs open on `docs/get-started.html`, and every Docs link points there. Its own README.md lists the pages. Every docs page shares one layout:

- The intro's header, logo, fonts, colours, `site.css` and `site.js`; the header links to the intro and to the docs (Getting started).
- A left sidebar with the full docs navigation, grouped Guides, Reference and Concepts, the current page marked. Below 1024px it collapses behind a hamburger button in the header and opens as a drawer (`aria-expanded`, Escape closes it, focus moves into it and back).
- A right "On this page" table of contents built from the page's h2 and h3, shown only at 1280px and wider.
- A content column of up to 46rem, like the intro's.

`docs/docs.js` renders the sidebar, the table of contents and the drawer, so the nav is defined once; its links resolve against the script's own URL, so they work from every docs folder and from `file://`. Without JavaScript, the sidebar shows a plain link to `docs/get-started.html`. Log retention (the size cap and age limit, their keys and numbers) is named only in `docs/reference/configuration.html`; the intro names none.

The other files:

- `site.css`: shared by every page, the styles Tailwind utilities cannot express: colour tokens for light and dark, links, inline code, terminals, the details marker and the logo mark.
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

- The install lines name the published packages of design.md §6.1 (`@aswemaythink/memex`, `aswemaythink/tap/memex`, `agent-memex`).
- The Play CDN prints a console warning that it is not meant for production. Swapping it for a prebuilt stylesheet would add a build step, so it is a ruling for the user.
- When a verb's behaviour or the story changes, change design.md or STORY.md first, then these pages.
