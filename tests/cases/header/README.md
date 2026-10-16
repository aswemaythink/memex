# header

The header family covers design §2.1: which leading comment block `show` prints as a file's header. For each comment family it covers the skip rules for shebang, pragma and license lines, where the block ends, the 8 KiB read limit, binary files, unknown extensions, `(no header)`, CRLF and the byte order mark, and the one-liner that a folder listing shows. Its prefix is `HDR`.

Each case runs `show <file> --up 0` to print a header in full, or `show lib --up 0` where the one-liner in a folder listing is the point. The cases share no fixture, and each fixture holds only the files its case reads. HDR-SKIP-08 pins that an encoding line is skipped, and HDR-TABLE-01 holds one sample per extension of the design §2.1 table, one folder per comment family.
