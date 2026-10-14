# render

The output of `memex show` (design §3.3 and §3.6). `../render.rs` holds the section model that a verb builds: a label, a body, the fields, the children, the citations, the cited-by list and the chain, each with its limit already applied. `text.rs` prints the sections and misses as text, with the blocks in their fixed order, the limits lines and the 120-character cut of one-liners. `json.rs` builds the `{docs, errors}` document of the design's JSON shape.
