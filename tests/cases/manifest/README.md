# manifest

The manifest family covers design §4.2: how the manifests in `.memex/collections/` load. A manifest that cannot load is skipped with a warning. Its prefix is `MAN`. The LOAD and IO topics are built here. The folder and json kinds are cased as `MANK-*` in the [manifest-kinds](../manifest-kinds/README.md) family, which reads them through the same loader; the Markdown kind is gone, and a manifest that still says `kind: markdown` is skipped with the warning that names the fix (MAN-LOAD-14). Each case folder holds its own `.memex/collections/` manifests and the files they match, and the expected warnings are printed on stderr exactly as `memex: warning: <path>: <reason>`, and a file that cannot be read as `memex: warning: cannot read <path> (<ErrorKind>)`.

Where design §4.2 is silent, these cases pin the choice: a `weights` key that is not `id`, `title` or `body` is accepted, as MAN-LOAD-07 shows with `status`. Only the three fields are weighed, since the index has no other field to weigh; no case ranks with such a key yet.

Retired: MAN-MD-01..15 (PLAN §15): the markdown kind is gone; frontmatter collections are pinned by COLL-*.
