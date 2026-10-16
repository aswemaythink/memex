# Site

`site/` is memex's documentation site: one static page, `index.html`, that says what memex is, how to install it and how to use it. It is plain HTML styled with Tailwind from its pinned Play CDN (`cdn.tailwindcss.com/3.4.17`), with no build step, so opening the page in a browser is the whole dev loop. It supersedes the story-first page of PLAN.md §6 Phase 9, at the user's direction.

`index.html` runs top to bottom, with a sticky nav of anchor links:

1. Introduction: what memex is (README.md, design.md §1).
2. Install and quick start: the working `cargo install --path . --locked` from a checkout, the planned v0.1.0 commands (design.md §6.1, marked as not yet published), then the verbs with terminal samples adapted from design.md §3 and README.md.
3. The intent layer: the layer table, the citation chips and the citation forms (design.md §2).
4. Context compression: three inline SVG diagrams, the compression ladder, two-way citations and hand-off by reference (design.md §1, §5).
5. Patterns: cases, doom hunts, performance verdicts, hand-off and messages, checking intent against code (drift), promoting a stream (design.md §2.2, §4.2, §5).
6. Principles (design.md §1).
7. Story: STORY.md retold, kept compact.

Colours are CSS custom properties on `:root`, switched by `prefers-color-scheme`, and the Tailwind config maps its colour names to them. The reading column is 90ch wide, the layout holds at 375px, and terminal blocks scroll inside themselves.

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
