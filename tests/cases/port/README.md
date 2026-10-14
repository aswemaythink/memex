# port

The port family covers design §4.4, the portability promises that one codebase keeps on Linux, macOS and Windows: CRLF input reads as its LF twin, labels and JSON paths use `/`, a name clash takes the next suffix, bytes that are not UTF-8 decode lossily, an I/O failure names its io::ErrorKind and never the OS text, a BOM is dropped, and case-variant special names are not special. Its prefix is `PORT`. Its cases observe the CLI through its stdout, stderr, `--json` output and store snapshots. The io exit of design §3.6 is CLI-EXIT-05, declared in `cli/` and pinned here.

The live cases are named here one by one, so that the selfhost gate sees each one cited: PORT-CRLF-01, PORT-CRLF-02, PORT-CRLF-03, PORT-CRLF-04, PORT-CRLF-05, PORT-CRLF-06, PORT-SLASH-01, PORT-SLASH-02, PORT-SLASH-03, PORT-WRITE-01, PORT-DECODE-01, PORT-DECODE-02, PORT-ERROR-01, PORT-ERROR-02, PORT-BOM-01, PORT-BOM-02, PORT-NAME-01, PORT-NAME-02, PORT-NAME-03, PORT-EXIT-01, PORT-MODE-01, and CLI-EXIT-05.

Windows CI is the real check for the `/` labels. The goldens are the same on every OS by construction, and only a Windows run shows that the labels and JSON paths really come out `/`-separated there. Each CRLF case runs a CRLF root and its LF twin in one golden, so the two blocks must match apart from their `[cwd ...]` tag.

PORT-DECODE-03 is planned, not pinned. APFS refuses a file name that is not UTF-8, so no fixture or setup step can create one on macOS. The walk.rs unit test `a_path_with_a_part_that_is_not_utf8_has_no_label` pins the label rule until a case can build such a name.
