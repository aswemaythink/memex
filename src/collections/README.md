# collections

The collections of design §4.2. A manifest describes one folder of docs that memex does not write itself, and its kind says how the files are read. `frontmatter.rs` is the flat-YAML reader shared by logs and manifests. `manifest.rs` loads and validates the manifests in `.memex/collections/` and dispatches to the reader of each kind: `markdown.rs`, `folder.rs` or `json.rs`. `globs.rs` walks a manifest's roots and matches their files against its globs, for every kind. `json.rs` also holds the field, one-liner and doc helpers that `folder.rs` shares with it.
