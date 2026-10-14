# Site

`site/` is memex's project site: one static page that tells the story first, then shows the tool. It is plain HTML and CSS with no build step, no framework, no script and no external font, so opening `index.html` in a browser is the whole dev loop. PLAN.md §6 Phase 9 specifies it.

- `index.html`: the page. The story is STORY.md retold (1945, the camera and the trail, now, memex); the terminal samples are adapted from design.md §3; the install lines are design.md §6.1.
- `style.css`: the styles. Light and dark themes are custom properties on `:root`, switched by `prefers-color-scheme`. The layout holds at 375px, and terminal blocks scroll inside themselves.

## Preview

From the repo root, serve the folder and open http://localhost:8000:

```sh
python3 -m http.server --directory site 8000
```

Opening `site/index.html` from disk works too.

## Deploy

`.github/workflows/pages.yml` publishes this folder to GitHub Pages on every push to main that touches `site/**`. Turn it on once under Settings, Pages, Source: GitHub Actions.

## Before it goes live

- The copy is a draft, to be judged against the bar in STORY.md: it should make people feel what we felt.
- The install lines name the packages of design.md §6.1 (`@rynwhite/memex`, `rynwhite/tap/memex`, `agent-memex`). They work once the v0.1.0 release of PLAN.md Phase 8 exists, so hold the first deploy until then.
- When the story or a verb's behaviour changes, change STORY.md or design.md first, then this page.
