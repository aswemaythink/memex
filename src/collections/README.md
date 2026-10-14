# collections

The collections of design §4.2. A manifest describes one folder of docs that memex does not write itself, and its kind says how the files are read. `frontmatter.rs` is the flat-YAML reader that logs use now and manifests use later. `manifest.rs` holds a manifest's keys and dispatches to `markdown.rs`, `folder.rs` or `json.rs`, the three kinds, which return no docs until S2.4 and S2.5.
