# cite

`src/cite/` holds the parts of `cite.rs` that find citations and backlinks (design §2.3): the tokenizer, the path resolver and the index that the corpus builds the first time a verb asks for citations.

- `tokens.rs`: reads a doc's text as tokens, skipping fenced code: links, `collection:id` labels, `X-*` prefixes and words, and tells a case id from other text. A fence line is three or more backticks or tildes indented at most three spaces, so a four-space indent is text, and a fence left open runs to the end of the doc. A link destination in angle brackets is read to its `>`, so it may hold spaces.
- `paths.rs`: resolves a path token or a markdown link from the citing doc's folder and then from the root, never above the root, and gives a target's one-liner.
- `index.rs`: resolves each doc's tokens by the precedence of design §2.3, drops self-citations, and records each doc's cites and the backlinks it gives. A prefix citation is a backlink marked with its prefix, and never a cite. `direct_count` counts a target's backlinks without the prefix ones, which is the `<-N` of a `search` roster. A link or path token that names a manifest doc's file cites the doc by its `collection:id` label, unless an intent doc holds that file, and `manifest_label` lets `show <path>` read that doc's citations under the same label (design §2.3).
