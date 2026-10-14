# init

The init code, the modules of `memex init` (design §3.5b), split by what each one writes. `../init.rs` picks the target folder (the git toplevel, else the cwd), runs the steps in the order of design §3.5b, and prints the atlas of the target. `files.rs` writes `.memex/` (its `.gitignore`, `config.json` and `collections/.gitkeep`, each created or kept), and holds the helpers the other two share: the listing that matches a special name exactly, the labels that print a path root-relative, the temp-file writes and the I/O error that names the kind and the path. `agents.rs` writes the AGENTS.md block between the markers (created, appended, replaced, kept, or left alone when the markers are damaged) and appends the CLAUDE.md import once; it never creates CLAUDE.md. `install.rs` copies the running binary to `$HOME/.local/bin` (or `%LOCALAPPDATA%\Programs\memex` on Windows) when no memex is on PATH, replaces it only when its bytes differ, and prints the PATH hint when its folder is not on PATH.

- `files.rs`: `store` (the `.memex/` files), `place`, `lists` (exact names from a listing, never `exists()`), `create_new` (a noclobber persist), `replace` (a persist that follows a symlink), `read_text` (refuses a file that is not UTF-8), `write_temp`, `label`, `io_error`.
- `agents.rs`: `agents_md`, `claude_md`, and the pure `merge`, which decides between Same, Changed and Damaged from the text alone. The snippet is `include_str!` of `assets/agents-snippet.md`, the one source.
- `install.rs`: `run`, `install` (created, kept or updated), `memex_on_path` (a listed `memex`, or `memex.exe` on Windows, that can run) and `on_path` (the install folder against PATH). Each platform difference has both arms.

Choices where design §3.5b is silent:

- The report names files, not folders: `.memex/` is implied by its files. A `.memex/` entry that is listed is kept whatever its kind. AGENTS.md and CLAUDE.md are read first, so a folder under one of those names is an I/O error (exit 3), and a symlink is followed when they are written.
- A damaged AGENTS.md is reported `kept`, with the warning on stderr, since nothing changed.
- A block is written with the file's own line ending, CRLF when the file has one; the snippet itself is LF.
- AGENTS.md that is not UTF-8 is not rewritten: init exits 3 and names the file.
- CLAUDE.md that is the same file as AGENTS.md (a symlink) is kept, so the import never lands in AGENTS.md.
- A missing `$HOME` (unix) or `%LOCALAPPDATA%` (Windows) is an environment error, exit 3, because the binary has nowhere to go.
- Only `memex.exe` counts as installed on Windows, as the test harness scrubs PATH; a bare `memex` file is not something the shell runs there.
- The atlas is `info` run from the target, so `init` changes the process's working folder to the target first.
- The `.gitignore` bytes are written here, as `logs/write.rs` writes them there; the two copies are the same design §4.1 text.
