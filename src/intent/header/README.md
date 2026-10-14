# header

The parts of the header reader (design §2.1). `families.rs` is the table of comment families: the markers each extension and file name uses, and the pragma lines that are skipped before a header. `scan.rs` walks a file's first lines past its blank, pragma and license lines, then reads the header block or a module docstring into text. The entry point, `../header.rs`, drops a byte order mark, skips binary files and hands the lines to the scanner.
