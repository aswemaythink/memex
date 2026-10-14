# show

The parts of `memex show` (design §3.3), split from `../show.rs`, which runs the verb: it resolves the refs, reports the sections and misses, and sets the exit status. `section.rs` builds what one resolved target prints: a folder's section (its README or `(no README)`, its children, its citations and its chain), a file's (its header, or its whole text for Markdown, and its chain) or a doc's (its text and its fields, with no chain). A folder or file that cannot be read or listed is an io error for its own ref (SHOW-IO-01, SHOW-IO-02), which `../show.rs` reports as a miss.
