// Renders the docs pages' shared chrome: sidebar nav, table of contents and mobile drawer.
// The nav is defined once here (NAV, paths relative to docs/), so every page under docs/ gets the same sidebar,
// with the current page marked. Links are resolved against this script's own URL, so they work from docs/,
// docs/guides/, docs/patterns/ and docs/reference/, over http and from file://. Each page loads it last, after its <main>.
// Without JavaScript, the sidebar's <noscript> link to docs/get-started.html, where the docs open, is the fallback.
(function () {
  'use strict';

  const NAV = [
    { title: 'Guides', items: [
      ['Getting started', 'get-started.html'],
      ['Hand-off and messages', 'guides/hand-off.html'],
      ['Checking drift', 'guides/drift.html'],
    ] },
    { title: 'Patterns', items: [
      ['Overview', 'patterns/index.html'],
      ['Test cases', 'patterns/cases.html'],
      ['Doom hunts', 'patterns/doom-hunts.html'],
      ['Performance optimisation', 'patterns/perf-optimisation.html'],
      ['Promote findings', 'patterns/promote-findings.html'],
    ] },
    { title: 'Reference', items: [
      ['Verbs', 'reference/index.html'],
      ['init', 'reference/init.html', 'mono'],
      ['info', 'reference/info.html', 'mono'],
      ['show', 'reference/show.html', 'mono'],
      ['search', 'reference/search.html', 'mono'],
      ['audit', 'reference/audit.html', 'mono'],
      ['log', 'reference/log.html', 'mono'],
      ['Configuration', 'reference/configuration.html'],
    ] },
    { title: 'Concepts', items: [
      ['Concepts', 'concepts.html'],
    ] },
  ];

  const script = document.currentScript;
  const root = new URL('./', script ? script.src : location.href);
  const norm = (u) => { const x = new URL(u, location.href); x.hash = ''; x.search = ''; return x.href.endsWith('/') ? x.href + 'index.html' : x.href; };
  const here = norm(location.href);
  const el = (tag, cls, text) => { const n = document.createElement(tag); if (cls) n.className = cls; if (text) n.textContent = text; return n; };
  const LABEL = 'font-mono text-[0.72rem] font-semibold uppercase tracking-[0.06em] text-muted';

  function sidebar(aside) {
    const nav = el('nav', 'space-y-8');
    nav.setAttribute('aria-label', 'Docs');
    for (const group of NAV) {
      const sec = el('div');
      sec.append(el('p', LABEL, group.title));
      const ul = el('ul', 'mt-2.5 border-l border-rule');
      for (const [label, path, style] of group.items) {
        const li = el('li');
        const font = style === 'mono' ? ' font-mono text-[0.84rem]' : ' text-[0.92rem]';
        const href = new URL(path, root).href;
        const current = norm(href) === here;
        const a = el('a', '-ml-px block border-l py-1 pl-4 leading-snug !no-underline' + font +
          (current ? ' border-accent font-medium !text-fg' : ' border-transparent !text-muted hover:border-trail hover:!text-fg'), label);
        a.href = href;
        if (current) a.setAttribute('aria-current', 'page');
        li.append(a);
        ul.append(li);
      }
      sec.append(ul);
      nav.append(sec);
    }
    aside.replaceChildren(nav);
  }

  function slug(text) { return text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, ''); }

  function toc(aside, main) {
    const heads = [...main.querySelectorAll('h2, h3')];
    if (heads.length < 2) { aside.remove(); return; }
    const box = el('nav', '');
    box.setAttribute('aria-label', 'On this page');
    box.append(el('p', LABEL, 'On this page'));
    const ul = el('ul', 'mt-2.5 space-y-1 text-[0.85rem] leading-snug');
    const links = [];
    for (const h of heads) {
      if (!h.id) h.id = slug(h.textContent);
      const li = el('li', h.tagName === 'H3' ? 'pl-3' : '');
      const a = el('a', 'block py-0.5 !no-underline !text-muted hover:!text-fg', h.textContent.trim());
      a.href = '#' + h.id;
      li.append(a);
      ul.append(li);
      links.push([h, a]);
    }
    box.append(ul);
    aside.replaceChildren(box);
    const spy = () => {
      let cur = links[0];
      for (const pair of links) if (pair[0].getBoundingClientRect().top < 120) cur = pair;
      for (const pair of links) pair[1].classList.toggle('!text-fg', pair === cur);
    };
    addEventListener('scroll', spy, { passive: true });
    spy();
  }

  function drawer(aside, button) {
    const wide = matchMedia('(min-width: 1024px)');
    const backdrop = el('div', 'fixed inset-x-0 bottom-0 top-14 z-30 hidden bg-black/40 lg:hidden');
    document.body.append(backdrop);
    const focusables = () => [button, ...aside.querySelectorAll('a[href]')];
    const set = (open, restore) => {
      button.setAttribute('aria-expanded', String(open));
      button.setAttribute('aria-label', open ? 'Close docs menu' : 'Open docs menu');
      aside.classList.toggle('max-lg:-translate-x-full', !open);
      aside.classList.toggle('max-lg:invisible', !open);
      backdrop.classList.toggle('hidden', !open);
      document.documentElement.classList.toggle('overflow-hidden', open);
      if (open) { (aside.querySelector('[aria-current]') || aside.querySelector('a[href]')).focus(); }
      else if (restore) button.focus();
    };
    const isOpen = () => button.getAttribute('aria-expanded') === 'true';
    button.addEventListener('click', () => set(!isOpen(), true));
    backdrop.addEventListener('click', () => set(false, true));
    aside.addEventListener('click', (e) => { if (e.target.closest('a[href]') && !wide.matches) set(false, false); });
    document.addEventListener('keydown', (e) => {
      if (!isOpen()) return;
      if (e.key === 'Escape') { e.preventDefault(); set(false, true); return; }
      if (e.key !== 'Tab') return;
      // Keep focus in the drawer and its toggle: step through them in order, wrapping at both ends.
      e.preventDefault();
      const f = focusables(), i = f.indexOf(document.activeElement);
      const next = i < 0 ? 0 : (i + (e.shiftKey ? -1 : 1) + f.length) % f.length;
      f[next].focus();
    });
    wide.addEventListener('change', () => { if (wide.matches && isOpen()) set(false, false); });
  }

  const aside = document.getElementById('docs-sidebar');
  const main = document.getElementById('main');
  const tocBox = document.getElementById('docs-toc');
  const button = document.getElementById('docs-menu');
  if (aside) sidebar(aside);
  if (tocBox && main) toc(tocBox, main);
  if (aside && button) drawer(aside, button);
})();
