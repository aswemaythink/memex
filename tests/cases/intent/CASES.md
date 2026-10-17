---
memex: cases
---
# Intent cases

## LINE

- `INT-LINE-01` A README's one-liner is the first sentence of the first paragraph after its H1, so the child line of its folder shows that sentence and not the next one, exit 0.
  design §2 One-liners. Run: memex show .
- `INT-LINE-02` When no paragraph follows a README's H1, the H1 text is its one-liner, so an H2 with nothing under it leaves the H1 text, exit 0.
  design §2 One-liners: the H1 text if there is no paragraph. Run: memex show .
- `INT-LINE-03` A heading is not a paragraph, so the first prose after the H1 gives the one-liner whether or not a blank line separates a heading from it, exit 0.
  design §2 One-liners (paragraphs after the H1). Run: memex show .
- `INT-LINE-04` A sentence ends at `!`, `.` or `?` followed by whitespace, so `4.1` and `v1.2` do not end one, and a line break inside a paragraph reads as a single space, exit 0.
  design §2 One-liners (the sentence rule). Run: memex show .
- `INT-LINE-05` A one-liner longer than 120 characters is cut to 119 characters and an ellipsis in text output, while `--json` keeps the whole one-liner, exit 0.
  design §2 One-liners (text truncates at 120, JSON keeps the full text) and design §3.6. Run: memex show . and memex show . --json.

## NOHEAD

- `INT-NOHEAD-01` A README whose first line is not an H1 takes its one-liner from the first sentence of its first paragraph, exit 0.
  design §2 is silent on a README with no H1. This reads the file's first prose paragraph where INT-LINE-01 reads the one after the H1. Run: memex show .
- `INT-NOHEAD-02` With no H1, a heading block is skipped as it is after an H1, so the first prose paragraph after the heading gives the one-liner, exit 0.
  design §2 One-liners (the first paragraph); the no-H1 reading is as INT-NOHEAD-01 and headings are not paragraphs, as INT-LINE-03. Run: memex show .
- `INT-NOHEAD-03` A subfolder whose README has no H1 and no prose lists with its name and nothing after it, not `(no README)`, while a subfolder with no README still shows `(no README)`, exit 0.
  design §3.3 Folder: a README's one-liner comes from §2, and `(no README)` is for a folder with no README. Run: memex show .

## IGNORE

- `INT-IGNORE-01` A file and a folder named in `.gitignore` are absent from the children of `show .`, exit 0.
  design §4.1 item 2: `.gitignore` is honoured, and git-ignored entries are skipped, with or without a git repository. Run: memex show .
- `INT-IGNORE-02` Entries named in `.ignore` are absent from the children of `show .`, exit 0.
  design §4.1 item 2: `.ignore` is honoured, so it excludes paths from memex and rg together. Run: memex show .
- `INT-IGNORE-03` A README that `.gitignore` names is not read, so its folder shows `(no README)` and lists its other files, exit 0.
  design §4.1 item 2 (git-ignored entries are skipped) and design §3.3 Folder (no README gives `(no README)`). Run: memex show . and memex show kept.

## SKIP

- `INT-SKIP-01` Hidden entries, files and folders alike (`.memex/` among them), are absent from the children of `show .`, exit 0.
  design §4.1 item 2: the built-in skip list starts with hidden entries. Run: memex show .
- `INT-SKIP-02` `node_modules` and `target` folders are absent from the children of `show .` and of a folder below the root, at any depth, exit 0.
  design §4.1 item 2: the built-in skip list names `node_modules` and `target`. Run: memex show . and memex show sub.

## NAME

- `INT-NAME-01` A file named `readme.md` in lowercase is not a folder's README, so the folder shows `(no README)` and lists the file as a child, exit 0.
  design §4.4 Name matching: a README is matched by its listed name, `README.md`, exactly. Run: memex show . and memex show lower --json.
- `INT-NAME-02` A file named `README.MD` in capitals is not a folder's README, so the folder shows `(no README)` and lists the file as a child, exit 0.
  design §4.4 Name matching: the match is exact on every OS. Run: memex show . and memex show upper.
- `INT-NAME-03` A folder named `README.md` is a folder, not a README: `show docs` lists it as `README.md/` with `(no README)`, and `show docs/README.md` gives that folder with `(no README)`, exit 0.
  design §4.2 and §4.4: a README is a file matched by its listed name, so a folder of that name is no README. Run: memex show docs and memex show docs/README.md.

## CHAIN

- `INT-CHAIN-01` The chain of a folder with no README of its own lists the nearest ancestor README first and the root README last, exit 0.
  design §3.3 Chain: ancestor READMEs in full, nearest first, root last. Run: memex show a/b.

## BOM

- `INT-BOM-01` A README that starts with a UTF-8 byte order mark has its one-liner and its text read without the mark, exit 0.
  design §4.4 Text: a leading UTF-8 BOM is dropped on every read, READMEs included. Run: memex show . and memex show bom.

## READ

- `INT-READ-01` A README that cannot be read is one stderr warning, `memex: warning: cannot read <path> (PermissionDenied)`, its folder lists as `(no README)`, and `show .`, `search` and `info` still exit 0.
  design §4.2 (an unreadable README is a warning that is skipped, and never fails a read verb) and §4.4 Errors. Run: memex show ., memex search demo and memex info, with sub/README.md at mode 000.
