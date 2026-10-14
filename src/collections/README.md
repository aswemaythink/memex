# collections

The collections of design §4.2. A manifest describes one folder of docs that memex does not write itself, and its kind says how the files are read. `frontmatter.rs` is the flat-YAML reader that logs use now and manifests use later. `manifest.rs` holds a manifest's keys and dispatches to `markdown.rs`, `folder.rs` or `json.rs`, the three kinds. `folder.rs` and `json.rs` read their docs (S2.5), and `json.rs` also holds the field, one-liner and glob helpers that `folder.rs` shares with it; `markdown.rs` returns no docs until S2.4.
