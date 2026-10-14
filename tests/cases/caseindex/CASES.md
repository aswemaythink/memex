# Caseindex cases

The caseindex family pins design §2.2, how a CASES.md becomes case docs, and the case section of §3.3 and §3.6. Every case folder is a small fixture repo whose CASES.md holds the entries under test.

## ITEM

- `CASE-ITEM-01` A column-0 dash item whose first element is a case id is a case, shown as `cases:ID`.
  Scenario: a fixture CASES.md with a dash item for NAME-001; show cases:NAME-001 prints it, exit 0. design §2.2 column 0.
- `CASE-ITEM-02` A column-0 star item whose first element is a case id is a case too.
  Scenario: a star item for STAR-001; show cases:STAR-001 prints it, exit 0. design §2.2 column 0.
- `CASE-ITEM-03` An indented item is not a case, so its id is not found, and its line stays detail of the item above it.
  Scenario: an indented item for INDENT-001 under PARENT-001; show cases:INDENT-001 exits 1, and show cases:PARENT-001 prints the line as detail. design §2.2 column 0.
- `CASE-ITEM-04` A paragraph that opens with a backticked case id, but is not a list item, is not a case.
  Scenario: a column-0 paragraph that opens with PROSE-001; show cases:PROSE-001 exits 1. design §2.2 list item.

## CLAIM

- `CASE-CLAIM-01` The claim is the rest of the item's first line after the id, and it is the first line of the body.
  Scenario: a dash item for CLAIM-001 with one sentence; show cases:CLAIM-001 prints that sentence first. design §2.2 Claim.
- `CASE-CLAIM-02` An item with no text after its id is still a case, and its claim is empty.
  Scenario: a bare item for EMPTY-001 under a heading; show cases:EMPTY-001 prints the section with no claim line. design §2.2 Case id and claim.

## DETAIL

- `CASE-DETAIL-01` Indented lines under an item are its detail, printed after its claim without their indent.
  Scenario: an item with two indented lines; show cases:DETAIL-001 prints the claim, then both lines unindented. design §2.2 Detail.
- `CASE-DETAIL-02` A column-0 line ends an item, so a later column-0 line that is not an item is not its detail.
  Scenario: an item, a column-0 sentence, then the next item; show cases:ENDS-001 omits the sentence. design §2.2 Detail.
- `CASE-DETAIL-03` A blank line does not end an item: an indented line after it is detail, a blank line between detail lines is kept, and trailing blank lines are dropped.
  Scenario: an item with a blank line between two indented paragraphs, then a blank line and the next item; show cases:BLANK-001 keeps one blank line in the middle and none at its end. design §2.2 Detail.

## GROUP

- `CASE-GROUP-01` The group is the nearest heading above an item, at any heading level, and it is recorded as the group field.
  Scenario: items under a level-two, a level-three and a later level-one heading; each shows its nearest heading as its group. design §2.2 Group.
- `CASE-GROUP-02` An item above every heading has no group, so its group block shows (none).
  Scenario: an item before any heading and one after a heading; show cases:TOP-001 shows (none) and show cases:TOP-002 shows its heading. design §2.2 Group.

## FENCE

- `CASE-FENCE-01` An item inside a column-0 code fence is code, not a list item, so it is not a case.
  Scenario: a fenced dash item for FENCED-001 and a real item after the fence for FENCE-002; show cases:FENCED-001 exits 1 and show cases:FENCE-002 prints it. design §2.3, fenced code is never read as intent, applied to CASES.md.
- `CASE-FENCE-02` A heading inside a code fence does not set the group of the items after the fence.
  Scenario: a tilde fence holding a level-two heading, then an item; show cases:FENCE-003 shows the heading above the fence as its group. design §2.3.

## GRAMMAR

- `CASE-GRAMMAR-01` A lowercase id is not a case id, so its item is not a case.
  Scenario: a dash item for name-001; show cases:name-001 exits 1. design §2.2 Case id.
- `CASE-GRAMMAR-02` An id with several dash-separated parts and a short digit suffix is a case id.
  Scenario: items for STT-WRAP-06, LIB-MEASURE-001 and A-1; show prints all three. design §2.2 Case id.
- `CASE-GRAMMAR-03` An id with no dash, with a trailing dash or with a double dash is not a case id.
  Scenario: items for NODASH, TRAILING- and DOUBLE--1; show of each exits 1 with not found. design §2.2 Case id.

## CRLF

- `CASE-CRLF-01` A CASES.md with CRLF line ends gives the same case, with no carriage return in its text.
  Scenario: a CRLF fixture with an item and one indented line; show cases:CRLF-001 prints both lines with no carriage return. design §4.4 Text.
- `CASE-CRLF-02` A leading UTF-8 byte order mark is dropped, so an item on the first line is still a case.
  Scenario: a fixture whose first bytes are a byte order mark, then an item for BOM-001; show cases:BOM-001 prints it. design §4.4 Text.

## PLAN

- `CASE-PLAN-01` A claim that marks its case as planned is a case like any other, and its claim prints as written.
  Scenario: an item for PLAN-001 whose claim begins with the planned marker; show cases:PLAN-001 prints the claim unchanged. design §2.2 Planned cases.

## EVID

- `CASE-EVID-01` A backticked path in a case that names a file of the walk is its evidence, listed by its root-relative path.
  Scenario: an item for EVID-001 whose detail cites src/hash.rs, which exists; show prints it under evidence. design §2.2 Evidence.
- `CASE-EVID-02` A path resolves from the folder of its CASES.md first, so a file beside it wins over a root file of the same name.
  Scenario: tests/CASES.md cites hash.rs while tests/hash.rs and hash.rs both exist; the evidence is tests/hash.rs only. design §2.2 Evidence.
- `CASE-EVID-03` A path that the CASES.md folder does not hold resolves from the root.
  Scenario: tests/CASES.md cites src/cli.rs, which exists only at the root; the evidence is src/cli.rs. design §2.2 Evidence.
- `CASE-EVID-04` A path that names no file of the walk is not evidence, and the case still shows, exit 0.
  Scenario: an item citing missing/gone.rs, which does not exist; the evidence shows (none), exit 0. design §2.2 Evidence.
- `CASE-EVID-05` A backticked span with no slash and no extension is not a path, even when a file has that name.
  Scenario: an item whose spans are a bare word, a bare case id and a command, with files named like the bare word; the evidence shows (none). design §2.2 Evidence and §2.3 path token.
- `CASE-EVID-06` A path with a leading slash names a file from the root only.
  Scenario: tests/CASES.md cites a path with a leading slash to src/cli.rs, and tests/src/cli.rs also exists; the evidence is src/cli.rs. design §2.3 leading slash.
- `CASE-EVID-07` Each evidence file is listed once, in the order its first mention is written.
  Scenario: an item citing src/hash.rs, then src/cli.rs, then src/hash.rs again; the evidence lists hash.rs then cli.rs. design §2.2 Evidence.
- `CASE-EVID-08` Trailing punctuation inside the backticks is dropped before a path is resolved.
  Scenario: an item citing src/hash.rs. and src/cli.rs, with the dot and the comma inside the backticks; both are evidence. design §2.3 path token.

## JSON

- `CASE-JSON-01` `--json` on a case gives one doc with ref cases:ID, collection cases, its CASES.md as path, its claim as title, and its group and evidence as fields, exit 0.
  Scenario: a grouped item citing src/hash.rs; show cases:JSON-001 --json prints that doc with empty cites and an empty chain. design §3.6 Doc.

## SECT

- `CASE-SECT-01` A case section prints its claim and detail, then its group, its defining CASES.md, its evidence and its cited by list, with no cites block and no chain.
  Scenario: a grouped item with detail and evidence; show cases:SECT-001 prints the blocks in that order. design §3.3 Case.

## DUP

- `CASE-DUP-01` Two entries with the same id make show cases:ID ambiguous, listing both CASES.md paths, with exit 1.
  Scenario: two CASES.md files that each declare DUP-001; show cases:DUP-001 prints nothing on stdout and names both files on stderr. design §4.2 and §3.1 step 6.
- `CASE-DUP-02` (planned) A duplicate case id is an info warning, listed by memex info.
  Scenario: two entries with the same id; memex info lists the warning. design §4.2, built with INFO in S4.1.

## BARE

- `CASE-BARE-01` A bare case id, as in memex show NAME-001, shows the same section as cases:NAME-001.
  Scenario: memex show NAME-001 with the item for NAME-001 in the fixture. design §3.1 step 4, built with REF in S3.2.
