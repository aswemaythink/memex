# manifest

The manifest family covers design §4.2: how the manifests in `.memex/collections/` load, or are skipped with a warning, and how a `markdown` manifest reads its docs (ids, titles, frontmatter fields, `.json` sidecars and include globs). Its prefix is `MAN`. The LOAD and MD topics are built here. The FOLDER and JSON topics are planned entries, which the folder and json kinds fill in when they land. Each case folder holds its own `.memex/collections/` manifests and the files they match, and the expected warnings are printed on stderr exactly as `memex: warning: <path>: <reason>`.
