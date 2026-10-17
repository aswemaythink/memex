# Site

`site/` is memex's documentation site, one static page: `index.html`. It says what memex is, how to install it and how to use it. It is plain HTML styled with Tailwind from its pinned Play CDN (`cdn.tailwindcss.com/3.4.17`), with no build step, so opening the page in a browser is the whole dev loop. It supersedes the story-first page of PLAN.md §6 Phase 9, at the user's direction.

`index.html` runs top to bottom, with a sticky nav of anchor links:

1. Introduction: what memex is (README.md, design.md §1).
2. Principles: Markdown first, agent-agnostic, intent not code, conventions, bounded output, logs expire (design.md §1).
3. Get started: one command, `npx @rynwhite/memex init`, marked as coming with v0.1.0, with every other route (toolchains, Releases, a checkout, the plugin) in a native `<details>` (design.md §3.5b, §6.1).
4. Context compression: the intent layer as a table in compression order (READMEs, headers, collections, logs, code), each row with a CSS-only example popover (hover, keyboard focus or tap); then "How an agent explores your codebase" (READMEs, then headers, then code last) with the walk diagram, followed by a paragraph on scale and the scale diagram (a context window holds a few files of code but the whole intent layer), and the citation forms (design.md §1, §2, §4.3, §5).
5. Patterns: cases, doom hunts, performance verdicts, hand-off and messages, checking intent against code (drift), and one anti-pattern, promoting logs into a collection (design.md §2.2, §4.2, §5).
6. The verbs agents use: init, info, show, search, check and log, each a short description followed by its terminal sample (design.md §3, README.md).
7. Story: STORY.md retold in full, on a dashed trail rail with orange markers, the desk sketch and the pull quote.

The other files:

- `site.css`: the styles Tailwind utilities cannot express: colour tokens for light and dark, links, inline code, terminals, the details marker and the logo mark.
- `site.js`: the Tailwind Play CDN config, which maps colour names to the tokens in `site.css` and sets the fonts and the page width.
- `walk.svg`, `scale.svg`, `desk.svg`: the walk diagram (an agent reads a repo's README one-liners and headers top-down, then opens one file's code), the scale diagram (sixty files, each a dim code block under a thin orange intent strip; an agent runs along the strips and opens one file; one context window holds three files of code or all sixty strips) and the story's desk sketch. `walk.svg` and `scale.svg` animate unless the reader prefers reduced motion. Each carries its own light and dark palette in a `prefers-color-scheme` style block, since an `<img>` cannot read the page's custom properties.

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
