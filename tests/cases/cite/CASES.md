# Cite cases

## BARE

- `CITE-BARE-01` A declared bare id in a README is cited, and the case it names lists that README under cited by.
  design §2.3 bare id. Run: memex show README.md, then memex show cases:NAME-001.
- `CITE-BARE-02` `UTF-8` and an id that no CASES.md declares are not cited, so the README's cites block says `(none)`.
  design §2.3: prose like UTF-8 never links. Run: memex show README.md.
- `CITE-BARE-03` `show NAME-001` resolves the bare id to its case, and that case lists the README that cites the id under cited by.
  design §2.3 bare id, and §3.1 step 4: a bare id matches a doc's id exactly. Run: memex show NAME-001.

## PREFIX

- `CITE-PREFIX-01` `NAME-*` in a README is a backlink on each declared NAME id, marked `(via NAME-*)`, and it is never a cites line.
  design §2.3 prefix citation, and the cites block has no limit, so a family is not expanded there. Run: memex show README.md, then memex show cases:NAME-002.
- `CITE-PREFIX-02` A doc that cites an id both directly and by prefix is listed once under cited by, with no via marker.
  design §2.3: the direct citation is the one that counts. Run: memex show cases:NAME-001, then memex show cases:NAME-002.

## COLL

- `CITE-COLL-01` `cases:NAME-001` in a README is cited under its label, with the case's claim as its one-liner.
  design §2.3 `collection:id` with a known collection, and §3.1 labels. Run: memex show README.md.
- `CITE-COLL-02` `cases:NOPE-01` with a known collection and no such doc is an unresolved cite, printed as `unresolved`.
  design §2.3: explicit refs that resolve to nothing are listed as unresolved. Run: memex show README.md.
- `CITE-COLL-03` `foo:NAME-001` with an unknown collection is not an explicit ref: its bare id is cited, and nothing is unresolved.
  design §2.3: `collection:id` counts only when the collection is known. Run: memex show README.md.
- `CITE-COLL-04` `impl:ID` with the log stream as a known collection cites that log entry under its label, with its title.
  design §2.3 and §3.1: a stream is a collection. Run: memex show README.md.

## PATH

- `CITE-PATH-01` A relative path token resolves from the folder of the citing README first, so `cli.rs` in src/README.md cites src/cli.rs.
  design §2.3 path token: the citing doc's folder, then the root. Run: memex show src/README.md.
- `CITE-PATH-02` A path token that is not in the citing folder resolves from the root, so `src/cli.rs` in src/deep/README.md cites src/cli.rs.
  design §2.3: then the root. Run: memex show src/deep/README.md.
- `CITE-PATH-03` A leading slash names a root-relative path, so `/cli.rs` cites the root's cli.rs while a bare `cli.rs` cites src/deep/cli.rs.
  design §2.3: a leading / is root-relative. Run: memex show src/deep/README.md.
- `CITE-PATH-04` Trailing punctuation and wrapping characters are stripped, so every spelling of one path is one cite.
  design §2.3: trailing .,:;) is stripped. Run: memex show src/README.md.
- `CITE-PATH-05` A path that climbs above the root, or names no walked file, is neither a cite nor unresolved.
  design §2.3: a `..` that leaves the root is not a citation, and bare tokens never print as unresolved. Run: memex show README.md.

## LINK

- `CITE-LINK-01` A markdown link to a walked file cites it, resolved from the folder of its README, and an anchor after the path is dropped.
  design §2.3 `[text](path)`: resolved from the folder, then the root; #anchors are not refs. Run: memex show src/README.md.
- `CITE-LINK-02` A link to a URL, an in-page anchor and a bare URL in prose are not refs, even where their path names a walked file.
  design §2.3: URLs and #anchors are not refs. Run: memex show README.md.
- `CITE-LINK-03` A markdown link to a missing file is an unresolved cite, printed with its target as written.
  design §2.3: a link that resolves to nothing is listed as unresolved. Run: memex show README.md.
- `CITE-LINK-04` A markdown link whose destination is in angle brackets, `[t](<path with spaces.rs>)`, cites the walked file at that path, spaces and all.
  design §2.3 `[text](path)`: a file in the walked set, resolved from the citing doc's folder, then the root. Run: memex show README.md.

## UNRES

- `CITE-UNRES-01` Each unresolved ref is listed once, among the resolved cites, in the order it first appears.
  design §2.3 and §3.3: cites are in first-appearance order and deduplicated. Run: memex show README.md.

## FOLDER

- `CITE-FOLDER-01` A citation of a folder path, with or without its trailing slash, cites the folder, labelled without the slash and titled by its README.
  design §2.3 and §3.3 Folder: a citation of src/logs or src/logs/ cites the folder. Run: memex show README.md, then memex show src/logs.
- `CITE-FOLDER-02` A folder's cites block lists the citations of its README, and its children list leaves out the README.
  design §3.3 Folder: the folder's cites and cited by use the folder path. Run: memex show src/logs.

## FENCE

- `CITE-FENCE-01` Text inside a fence of backticks or of tildes is not read, while the same ids outside the fence are cited.
  design §2.3: citations are never extracted from fenced code blocks. Run: memex show README.md.
- `CITE-FENCE-02` A line indented four spaces before ``` is not a fence, so the lines after it are read, and the id in the indented line is cited.
  design §2.3: a fence line is indented at most three spaces. Run: memex show README.md.
- `CITE-FENCE-03` A fence indented up to three spaces opens and closes like any fence, and a closing line indented four spaces does not close it, so the text after that line stays hidden until the real closer.
  design §2.3: a fence line is indented at most three spaces, for the opener and the closer alike. Run: memex show README.md.
- `CITE-FENCE-04` A fence that is never closed runs to the end of the doc, so the ids and paths after its opening line are not cited, while the id before it is.
  design §2.3: an unclosed fence runs to the end of the doc. Run: memex show README.md.

## SELF

- `CITE-SELF-01` A case does not cite its own id in its claim, and a README does not cite its own folder, so the cited by lists stay `(none)`.
  design §2.3: self-citations are dropped. Run: memex show cases:NAME-001, then memex show src/README.md, then memex show src.

## BACK

- `CITE-BACK-01` Cited by lists each citing README, header and case, sorted by label, with its one-liner.
  design §2.3 derived backlinks, and §3.3 File. Run: memex show src/cli.rs.
- `CITE-BACK-02` A log entry that cites a file is listed under cited by as stream:id with its title, and the log's own section lists what it cites.
  design §2.3: log docs are scanned; §3.3 Log or collection doc. Run: memex show src/cli.rs, then memex show impl:20261013-120000-x.
- `CITE-BACK-03` A case that cites another case is a backlink on it, and the citing case's own section shows cited by with no cites block.
  design §2.3 and §3.3 Case. Run: memex show cases:NAME-001, then memex show cases:NAME-002.
- `CITE-BACK-04` `--limit` cuts the cited by list, and its limits line gives the total only when the list was cut.
  design §3.3 Limits: the cited-by list uses --limit and prints showing N of M. Run: memex show src/cli.rs --limit 2, then --limit 3.

## JSON

- `CITE-JSON-01` `--json` gives each cite as `{ref, title, resolved}`, and an unresolved cite has `resolved` false and an empty title.
  design §3.6 show: cites. Run: memex show README.md --json.
- `CITE-JSON-02` `--json` gives cited_by its total and its refs, and a ref has a `via` key only when it cites by prefix.
  design §3.6 show: cited_by. Run: memex show cases:NAME-001 --json.
