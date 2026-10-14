# manifest

The manifest family covers design §4.2: how the manifests in `.memex/collections/` load, or are skipped with a warning, and how a `markdown` manifest reads its docs (ids, titles, frontmatter fields, `.json` sidecars and include globs). Its prefix is `MAN`. The LOAD and MD topics are built here. The folder and json kinds are cased as `MANK-*` in the [manifest-kinds](../manifest-kinds/README.md) family, which reads them through the same loader. Each case folder holds its own `.memex/collections/` manifests and the files they match, and the expected warnings are printed on stderr exactly as `memex: warning: <path>: <reason>`.

Where design §4.2 is silent, these cases pin the choice: a `weights` key that is not `id`, `title` or `body` is accepted, as MAN-LOAD-07 shows with `status`. Only the three fields are weighed, since the index has no other field to weigh; no case ranks with such a key yet.
