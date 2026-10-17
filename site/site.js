// The Tailwind Play CDN config for the memex site.
// It maps colour names to the custom properties in site.css, and sets the fonts and the page width.
// index.html loads it right after the pinned CDN script, so the CDN picks it up before it styles the page.
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
