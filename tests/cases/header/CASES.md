# Header cases

## SLASH

- `HDR-SLASH-01` A `.rs` file whose first block is `//!` lines has its header text with each marker and the space after it removed, and a bare `//!` line kept as a blank line.
  design §2.1, the `//`, `///`, `//!` family. Run: memex show purpose.rs --up 0.
- `HDR-SLASH-02` A `.cs` file whose first block is `///` lines has its header text with the `///` markers removed.
  design §2.1, the `//`, `///`, `//!` family. Run: memex show Widget.cs --up 0.
- `HDR-SLASH-03` A `.java` file whose first block is a `/** */` block has its header text with the opening and closing markers and each line's leading `*` removed.
  design §2.1, the `/* */` and `/** */` family. Run: memex show Widget.java --up 0.
- `HDR-SLASH-04` A `.css` file whose first block is a `/* */` block keeps the blank line inside the block, so the header is the whole block.
  design §2.1, the `/* */` family and the block end rule. Run: memex show theme.css --up 0.
- `HDR-SLASH-05` A `.ts` file whose first block is `//` lines has its header text with the markers removed.
  design §2.1, the `//` family. Run: memex show client.ts --up 0.

## HASH

- `HDR-HASH-01` A `.yaml` file whose first block is `#` lines has its header text with the `#` markers removed.
  design §2.1, the `#` family. Run: memex show config.yaml --up 0.

## DOC

- `HDR-DOC-01` A `.py` file that opens with a multi-line `"""` docstring has the text between its quotes as its header.
  design §2.1, the module docstring of py. Run: memex show tool.py --up 0.
- `HDR-DOC-02` A `.py` file that opens with a one-line `'''` docstring has the text between its quotes as its header.
  design §2.1, the module docstring of py. Run: memex show shim.py --up 0.
- `HDR-DOC-03` A `.py` file with a `#` block before its docstring has the `#` block as its header, and the docstring is not used.
  design §2.1, the docstring is used only when there is no leading `#` block. Run: memex show both.py --up 0.
- `HDR-DOC-04` A `.py` file with code before its docstring has `(no header)`, because the docstring is not the first thing in the file.
  design §2.1, the header is the next comment block, and a module docstring is the first statement. Run: memex show late.py --up 0.

## MAKE

- `HDR-MAKE-01` A file named `Makefile` with a `#` block has that block as its header, found by its name since it has no extension.
  design §2.1, the `#` family lists the names `Makefile` and `Dockerfile`. Run: memex show Makefile --up 0.
- `HDR-MAKE-02` A file named `Dockerfile` with a `#` block has that block as its header.
  design §2.1, the `#` family lists the names `Makefile` and `Dockerfile`. Run: memex show Dockerfile --up 0.

## DASH

- `HDR-DASH-01` A `.sql` file whose first block is `--` lines has its header text with the `--` markers removed.
  design §2.1, the `--` family. Run: memex show schema.sql --up 0.
- `HDR-DASH-02` A `.hs` file whose first block is a `{- -}` block has its header text between the braces.
  design §2.1, the `{- -}` family. Run: memex show Demo.hs --up 0.
- `HDR-DASH-03` A `.lua` file whose first block is a `--[[ ]]` block has its header text between the markers.
  design §2.1, the `--[[ ]]` family. Run: memex show demo.lua --up 0.

## SEMI

- `HDR-SEMI-01` A `.clj` file whose first block is `;;` lines has its header text with the `;` markers removed.
  design §2.1, the `;` family. Run: memex show core.clj --up 0.

## PERCENT

- `HDR-PERCENT-01` An `.erl` file whose first block is `%%` lines has its header text with the `%` markers removed.
  design §2.1, the `%` family. Run: memex show demo.erl --up 0.

## MARKUP

- `HDR-MARKUP-01` A `.vue` file whose first block is an `<!-- -->` block has its header text between the markers.
  design §2.1, the `<!-- -->` family. Run: memex show Card.vue --up 0.

## PAREN

- `HDR-PAREN-01` An `.ml` file whose first block is a `(* *)` block has its header text between the markers.
  design §2.1, the `(* *)` family. Run: memex show demo.ml --up 0.

## SKIP

- `HDR-SKIP-01` A shebang line before the first block is skipped, so the block after it is the header.
  design §2.1, the header rule skips a shebang. Run: memex show run.sh --up 0.
- `HDR-SKIP-02` A Go `//go:build` line and the blank line after it are skipped, so the package comment is the header.
  design §2.1, the header rule skips pragma lines such as `//go:build`. Run: memex show demo.go --up 0.
- `HDR-SKIP-03` A JavaScript `"use client";` directive before the first block is skipped, so the block after it is the header.
  design §2.1, the header rule skips pragma lines such as `"use client"`. Run: memex show Button.tsx --up 0.
- `HDR-SKIP-04` A leading license block, one with `SPDX-License-Identifier`, and the blank line after it are skipped, and the next comment block is the header.
  design §2.1, the header rule skips one leading license block. Run: memex show lib.rs --up 0.
- `HDR-SKIP-05` A file whose only comment block is a license has no header, so it shows `(no header)`.
  design §2.1, the header is the next comment block, and none follows the license. Run: memex show only.rs --up 0.
- `HDR-SKIP-06` An XML declaration, a `<!DOCTYPE` or `<!doctype` line, and a `<?php` open tag before the first block are skipped as pragmas, so the block after each is the header.
  design §2.1, the header rule skips the markup prologues `<?xml …?>`, `<!DOCTYPE …>` and `<?php`. Run: memex show feed.xml --up 0, page.html, lower.html and open.php.
- `HDR-SKIP-07` A license block ends at its first blank line: a blank line inside a `/* */` license ends it, and a header with no blank line after its license is part of that license, so neither file has a header.
  design §2.1, a license block ends at its first blank line, so a header must be separated from it by one. Run: memex show block.rs --up 0, and joined.rs.
- `HDR-SKIP-08` A `# -*- coding: utf-8 -*-` line before the block is skipped, so the block after it is the header.
  design §2.1, the header rule skips encoding and pragma lines such as `# -*-`. Run: memex show coded.py --up 0.

## BLOCK

- `HDR-BLOCK-01` A blank line inside a comment block does not end it, so the comment after the blank line is part of the header.
  design §2.1, the block ends at the first line that is neither comment nor blank. Run: memex show notes.rs --up 0.
- `HDR-BLOCK-02` The first code line ends the header, so a comment after it is not part of the header.
  design §2.1, the block ends at the first line that is neither comment nor blank. Run: memex show after.rs --up 0.

## LINE

- `HDR-LINE-01` The one-liner in a folder listing is the first sentence of the header, with its lines joined and the rest of the header left out.
  design §2 one-liners, and design §2.1 first sentence. Run: memex show lib --up 0.
- `HDR-LINE-02` A first paragraph with no sentence mark is one sentence, so the one-liner is the whole paragraph.
  design §2 one-liners: a paragraph with no mark is one sentence. Run: memex show lib --up 0.
- `HDR-LINE-03` A dot inside a version such as `v1.2` does not end a sentence, because a sentence ends at a mark followed by whitespace.
  design §2 one-liners: a sentence ends at `.`, `!` or `?` followed by whitespace or the end of the paragraph. Run: memex show lib --up 0.

## LIMIT

- `HDR-LIMIT-01` `show` prints the whole header block, all 70 of its lines, and not only the first 60 that the index keeps.
  design §2.1 limits: the index keeps the first 60 lines and `show` prints the whole block. Run: memex show long.rs --up 0.

## CAP

- `HDR-CAP-01` A header that starts after the first 8 KiB of a file is not read, so the file shows `(no header)`.
  design §2.1 files read: only the first 8 KiB of each file is read. Run: memex show late.rs --up 0.
- `HDR-CAP-02` A header that runs past 8 KiB shows the lines inside the first 8 KiB, then a blank line and the final line `… (header cut at 8 KiB)`, and the line that starts at byte 8192 is not shown.
  design §2.1 files read: a header still open at the 8 KiB cut is shown with the cut line, never silently. Run: memex show cut.rs --up 0.
- `HDR-CAP-03` The cut line is shown by `show` only: the index leaves it out, so `kib` finds no header, and a header's JSON one-liner is its first paragraph's first sentence without the line.
  design §2.1 files read and the one-liner rule of design §2 (the index and one-liners never carry the cut line). Run: memex search kib, then memex search --in headers --json.
- `HDR-CAP-04` A header that ends inside the read limit has no cut line, even in a file longer than 8 KiB, so the line marks only a header still open at the cut.
  design §2.1 files read (the cut line marks a header still open at the cut). Run: memex show ends.rs --up 0.
- `HDR-CAP-05` A file of exactly 8 KiB whose header runs to its last byte has no cut line, since nothing lies past the read limit.
  design §2.1 files read (only the first 8 KiB is read, and the cut line is for a header that goes on past it). Run: memex show exact.rs --up 0.

## BIN

- `HDR-BIN-01` A binary file, one with a NUL byte, has no header even when its first line is a comment.
  design §2.1: binary files are skipped. Run: memex show blob.rs --up 0.

## EXT

- `HDR-EXT-01` A file whose extension is not in the table has no header, even when it opens with a `//` comment.
  design §2.1: files with unknown extensions are not harvested. Run: memex show notes.foo --up 0.

## NONE

- `HDR-NONE-01` A `.rs` file with code and no comment block shows `(no header)` in its folder's listing.
  design §2.1, a file with no header shows as `(no header)` in listings. Run: memex show lib --up 0.

## CRLF

- `HDR-CRLF-01` A `.rs` file with CRLF line endings has its header lines without the carriage returns.
  design §4.4 Text: `\r\n` is accepted everywhere, headers included. Run: memex show crlf.rs --up 0.

## BOM

- `HDR-BOM-01` A UTF-8 byte order mark at the start of a file is dropped, so the header after it is found.
  design §4.4 Text: a leading UTF-8 BOM is dropped on read. Run: memex show bom.rs --up 0.
- `HDR-BOM-02` A UTF-8 byte order mark before a shebang is dropped, so the shebang is skipped and the `#` block after it is the header.
  design §4.4 Text: the BOM is dropped before the header is read, and design §2.1 skips the shebang. Run: memex show bom-run.sh --up 0.

## READ

- `HDR-READ-01` A header file that cannot be read is one stderr warning, and its folder lists it as `(no header)`, while `show .`, `show sub`, `search` and `info` still exit 0.
  design §4.2 (an unreadable header is a warning that is skipped) and §2.1 Header detection. Run: memex show ., memex show sub, memex search demo and memex info, with sub/locked.rs at mode 000.
## TABLE
- `HDR-TABLE-01` Every extension in the comment table of design §2.1 reads its header with its own comment syntax, so each folder listing shows one sample's one-liner per extension.
  design §2.1 table: one sample per extension, one folder per row family; the docstring row is HDR-DOC-01 to HDR-DOC-04. Run: memex show slash, then hash, dash, semi, percent, markup and paren.
