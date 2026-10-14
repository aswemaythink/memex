# resolve

The resolve folder turns a ref into what it names (design §3.1), or into a miss: not found, or ambiguous with its candidates. `mod.rs` runs the resolution order: absolute paths, relative paths against the cwd and the root, then names. `names.rs` resolves a `collection:id` label, a reserved spelling (`readmes:`, `headers:`, `logs:`), a bare id and a unique basename. `outside.rs` resolves a path that leaves the root against the root that holds it, so a linked worktree's files read with that worktree's own intent layer.

- `mod.rs`: the outcome of a ref, the order of the steps, and the relative-path step.
- `names.rs`: the steps for labels, bare ids and basenames, and the candidates of an ambiguous ref.
- `outside.rs`: absolute and out-of-root paths, and the corpus of the root that holds them.
