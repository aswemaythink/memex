# header

The header family covers design §2.1: which leading comment block `show` prints as a file's header, for each comment family, the skip rules for shebang, pragma and license lines, where the block ends, the 8 KiB read limit, binary files, unknown extensions, `(no header)`, CRLF and the byte order mark, and the one-liner that a folder listing shows. Its prefix is `HDR`.

Each case runs `show <file> --up 0` to print a header in full, or `show lib --up 0` where the one-liner in a folder listing is the point. The cases share no fixture, and each fixture holds only the files its case reads. HDR-READ-01 is the header file that cannot be read: one warning, and the read verbs still succeed.
