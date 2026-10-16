# info

The two renderings of `memex info`'s atlas (design §3.5 and §3.6). `../info.rs` holds the atlas's types and re-exports both renderings. `text.rs` prints the header, the graph, collections, streams, `to`, `here` and `use` lines, each label padded to the longest one printed; a text over 3,000 bytes cuts its longest list line to its first items and a `+N more` ending, until it fits. `json.rs` builds the `--json` document of design §3.6 from the same atlas: the two built-in collections (`readmes` and `headers`), the collections, the streams, the `to` and `here` objects, and the warnings.
