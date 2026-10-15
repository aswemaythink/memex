# intent

The readers of the intent layer (design §2). `header.rs`, with its `header/` folder, reads a file's leading comment block, `readme.rs` reads a README's one-liner, and `cases.rs` reads a CASES.md into case docs. `files.rs` picks the reader for each walked file and gives its docs; a file with no comment family has no header, so it is never opened and gives no warning (design §4.2). `mod.rs` holds the rule they share: the paragraphs of a block, and the one-liner, which is the first sentence of its first paragraph.
