# info

The info family covers design §3.5 and §3.6: what `memex info` prints, the atlas of where things are. The header with the root and the log store, the built-in counts, the manifest collections, the log streams with their new entries, the `to` count for an agent, the `here` line of the cwd, the `use` line, the 3,000-byte cap, the warnings of design §4.2 and their JSON, the no-root case and the `--json` document. Its prefix is `INFO`. Each case folder holds a small intent layer as its fixture, and every golden is read with MEMEX_NOW at 2026-10-13T12:00:00Z.

Where design §3.5 is silent, these cases pin the choice: a count of one is singular; the to line appears only with at least one entry; a stream is new within 24 hours inclusive; a stream is ordered by its newest entry, then by name; the collections are alphabetical; a built-in's JSON `kind` is `null`; JSON `newest` is RFC 3339; an agent id that `log` refuses is a usage error for `info` too. INFO-NOTEXT-01 pins that the atlas prints no log title or body text.

INFO-NOW-01 pins that an invalid `$MEMEX_NOW` exits 3 for info. INFO-CAP-02, the exact 3,000-byte boundary, has no case folder: the header carries the sandbox root, whose length differs by OS, so no golden for that boundary is the same on every OS. A unit test in src/render/info.rs pins it instead.
