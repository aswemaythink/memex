// The Tailwind Play CDN config for the memex site.
// It maps colour names to the custom properties in site.css, and sets the fonts and the page width.
// Every page (index.html and the pages in docs/) loads it right after the pinned CDN script, so the CDN picks it up before it styles the page.
// It also draws each figure's diagram inline, so the diagrams follow the page's theme on iOS Safari too, and fills the intro page's install prompt from INSTALL.md.
tailwind.config = {
  darkMode: 'media',
  theme: {
    extend: {
      colors: {
        bg: 'var(--bg)', soft: 'var(--bg-soft)', fg: 'var(--fg)', muted: 'var(--muted)',
        accent: 'var(--accent)', 'accent-soft': 'var(--accent-soft)', rule: 'var(--rule)', trail: 'var(--trail)',
        term: 'var(--term-bg)', 'term-bar': 'var(--term-bar)', 'term-line': 'var(--term-line)', 'term-fg': 'var(--term-fg)', 'term-dim': 'var(--term-dim)'
      },
      fontFamily: {
        serif: ['"Iowan Old Style"', '"Palatino Linotype"', 'Palatino', '"URW Palladio L"', 'P052', '"Book Antiqua"', 'Georgia', 'serif'],
        sans: ['system-ui', '-apple-system', '"Segoe UI"', 'Roboto', '"Helvetica Neue"', 'Arial', '"Noto Sans"', 'sans-serif'],
        mono: ['ui-monospace', '"SF Mono"', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', '"Liberation Mono"', 'monospace']
      },
      maxWidth: { page: '48rem' }
    }
  }
}

// Diagrams: each `figure img` of a site SVG is swapped for the same SVG drawn inline, in a shadow root so its class names and keyframes stay its own.
// WebKit does not reliably pass the page's colour scheme into an SVG loaded as an <img>, so on iOS Safari the diagrams drew their light palette on a dark page.
// Inline, the SVG's own prefers-color-scheme block is read in the page's context. Without fetch (file://) or JavaScript the <img> stays as it is.
document.addEventListener('DOMContentLoaded', () => {
  if (!window.fetch || !document.head.attachShadow) return;
  document.querySelectorAll('figure img[src$=".svg"]').forEach(async (img) => {
    try {
      const res = await fetch(img.src);
      if (!res.ok) return;
      const svg = new DOMParser().parseFromString(await res.text(), 'image/svg+xml').documentElement;
      if (svg.localName !== 'svg') return;
      svg.querySelectorAll('title, desc').forEach((n) => n.remove());
      svg.removeAttribute('role');
      svg.setAttribute('aria-hidden', 'true');
      svg.setAttribute('focusable', 'false');
      svg.style.cssText = 'display: block; width: 100%; height: 100%;';
      const host = document.createElement('div');
      host.className = img.className;
      host.style.aspectRatio = img.getAttribute('width') + ' / ' + img.getAttribute('height');
      host.setAttribute('role', 'img');
      host.setAttribute('aria-label', img.alt);
      host.attachShadow({ mode: 'open' }).append(document.importNode(svg, true));
      img.replaceWith(host);
    } catch (e) { /* keep the <img> */ }
  });
});

// The install prompt (index.html #prompt): fetch INSTALL.md from its data-src and show it, headings in bold; Copy copies whatever the block shows.
// If the fetch fails, the block keeps its one-line prompt pointing at INSTALL.md, and Copy copies that.
document.addEventListener('DOMContentLoaded', () => {
  const pre = document.getElementById('prompt');
  const copy = document.getElementById('prompt-copy');
  if (!pre || !copy) return;
  const code = pre.querySelector('code');
  if (window.fetch && pre.dataset.src) {
    fetch(pre.dataset.src).then((res) => (res.ok ? res.text() : Promise.reject())).then((text) => {
      const out = document.createDocumentFragment();
      text.trimEnd().split('\n').forEach((line, i) => {
        if (i) out.append('\n');
        if (!/^#/.test(line)) { out.append(line); return; }
        const b = document.createElement('span');
        b.className = 'g';
        b.textContent = line;
        out.append(b);
      });
      code.replaceChildren(out);
    }).catch(() => { /* keep the one-line prompt */ });
  }
  // Without clipboard access (an old browser, or a frame that denies it), select the text instead so the reader can copy it by hand.
  const select = () => {
    const range = document.createRange();
    range.selectNodeContents(code);
    getSelection().removeAllRanges();
    getSelection().addRange(range);
    const done = document.execCommand && document.execCommand('copy');
    if (done) getSelection().removeAllRanges();
    return done;
  };
  copy.hidden = false;
  copy.addEventListener('click', async () => {
    let done = false;
    try { await navigator.clipboard.writeText(code.textContent); done = true; } catch (e) { done = select(); }
    copy.textContent = done ? 'Copied' : 'Selected';
    setTimeout(() => { copy.textContent = 'Copy'; }, 1800);
  });
});
