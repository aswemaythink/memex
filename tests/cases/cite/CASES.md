---
memex: cases
---
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
- `CITE-PATH-06` A bare `Makefile` token in a README cites nothing, since a path token needs a `/` or a `.ext`, so the README's cites and the Makefile's cited by both say `(none)`.
  design §2.3 path token: a maximal run that contains `/` or a `.ext`. Run: memex show README.md, then memex show Makefile.

## LINK

- `CITE-LINK-01` A markdown link to a walked file cites it, resolved from the folder of its README, and an anchor after the path is dropped.
  design §2.3 `[text](path)`: resolved from the folder, then the root; #anchors are not refs. Run: memex show src/README.md.
- `CITE-LINK-02` A link to a URL, an in-page anchor and a bare URL in prose are not refs, even where their path names a walked file.
  design §2.3: URLs and #anchors are not refs. Run: memex show README.md.
- `CITE-LINK-03` A markdown link to a missing file is an unresolved cite, printed with its target as written.
  design §2.3: a link that resolves to nothing is listed as unresolved. Run: memex show README.md.
- `CITE-LINK-04` A markdown link whose destination is in angle brackets, `[t](<path with spaces.rs>)`, cites the walked file at that path, spaces and all.
  design §2.3 `[text](path)`: a file in the walked set, resolved from the citing doc's folder, then the root. Run: memex show README.md.
- `CITE-LINK-05` A markdown link in a README outside src/ that names src/cli.rs cites that file, since the README's own folder holds no such path and the link resolves from the root.
  design §2.3 `[text](path)`: resolved from the citing doc's folder, then the root. Run: memex show docs/README.md, then memex show src/cli.rs.

## MAN

- `CITE-MAN-01` A collection doc is cited like a file: a link to it lists the linking README under its cited by, and `notes:beta` in a README cites that doc, which lists the README under its cited by.
  design §2.3 `[text](path)` and `collection:id` with a known collection, and §2.2 frontmatter collections. Run: memex show notes:alpha, then memex show README.md, then memex show notes:beta. The fixture's notes are `memex: notes` frontmatter docs.

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
- `CITE-BACK-04` `--limit` cuts the cited by list, and its limits line always reads `showing N of M`, adding the `(--limit M for all)` hint only when the list was cut.
  design §3.3 Limits: the cited-by list uses --limit and always prints showing N of M, with the hint only when N < M. Run: memex show src/cli.rs --limit 2, then --limit 3.
- `CITE-BACK-05` A log that cites another log is listed under that log's cited by as `stream:id` with its title, and the citing log's cites lists the log it cites.
  design §2.3 (log docs are scanned for citations) and §3.3 Log or collection doc. Run: memex show impl:20261013-115000-first-finding, then memex show review:20261013-115500-second-finding.

## JSON

- `CITE-JSON-01` `--json` gives each cite as `{ref, title, resolved}`, and an unresolved cite has `resolved` false and an empty title.
  design §3.6 show: cites. Run: memex show README.md --json.
- `CITE-JSON-02` `--json` gives cited_by its total and its refs, and a ref has a `via` key only when it cites by prefix.
  design §3.6 show: cited_by. Run: memex show cases:NAME-001 --json.

## FIELD

- `CITE-FIELD-01` A decision whose frontmatter has `case: MONEY-001` is listed under cited by on MONEY-001, and its own cites list MONEY-001.
  design §2.3: the values of a collection doc's frontmatter fields are scanned, so metadata links both ways. Run: memex show MONEY-001, then memex show decisions:rounding.
- `CITE-FIELD-02` A flow list and a block list of ids in a decision's frontmatter cite every id in the order the fields give them, each list item is scanned on its own, and a `PREFIX-*` value is a backlink on each id it names.
  design §2.3: each item of a list value is scanned on its own, so a `"```"` item does not hide the item after it; a prefix is never a cite. Run: memex show decisions:pricing, then memex show MONEY-004, then memex show MONEY-005.
- `CITE-FIELD-03` A path in a frontmatter field resolves from the decision's folder first and then from the root, as a path in the body does.
  design §2.3: a path token resolves from the citing doc's folder, then the root. Run: memex show decisions:pricing, then memex show decisions:fees, then memex show docs/spec.md, then memex show archive/docs/spec.md.
- `CITE-FIELD-04` An explicit ref in a field is cited when it resolves, a `coll:id` or a link, and is listed as unresolved when it names nothing, while a bare undeclared id and a bare path that names no file are not cited.
  design §2.3: explicit refs that resolve to nothing are unresolved, and bare tokens never are. Run: memex show decisions:pricing.
- `CITE-FIELD-05` A collection doc's `memex` key is not one of its fields, and its field cites come before its body's cites, each id listed once at its first appearance.
  design §2.3: the memex key is never scanned, and field cites come before body cites in cites. Run: memex show decisions:rounding.
- `CITE-FIELD-06` A log's frontmatter is not scanned, so a log whose tags hold a declared id does not cite it, while an id in its body is cited.
  design §2.3: a log's frontmatter is memex's own record of the run and is not scanned. Run: memex show MONEY-001, then memex show MONEY-002, then memex show impl:20261013-120000-pricing.
- `CITE-FIELD-07` A folder doc whose marker is a Markdown file cites through the fields of that file's frontmatter, as a collection doc does.
  design §2.3 and §4.2: a folder doc whose marker is `.md` takes its fields from that frontmatter. Run: memex show MONEY-001, then memex show specs:ledger.
- `CITE-FIELD-08` A json manifest's doc cites from its text alone, as before: its strings are the text, so a fence that opens in one string and closes in a later one hides the ids between them, and its fields are not read again.
  design §2.3: the scanned fields are a collection doc's frontmatter fields, and a json doc's strings are its text. Run: memex show ledger:LEDGER-001.
- `CITE-FIELD-09` A folder doc whose marker is a JSON file is not cited through its fields, as before, since its fields are not frontmatter.
  design §2.3 and PLAN §13 R28: the scanned fields are a collection doc's frontmatter fields. Run: memex show MONEY-001, then memex show ledgers:cash.
