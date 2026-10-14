# init

The init family covers design §3.5b and §6.1: what `memex init` does to a folder, and that a second run changes nothing. It covers the `.memex/` files and their never-overwrite rule, the AGENTS.md block with its append, replace, kept and damaged-marker paths, the CLAUDE.md import, the target folder (the git toplevel, else the cwd), the self-install and its PATH hint, the atlas that ends the report, the re-run, and the exit 3 of an I/O failure (IO, unix only). Its prefix is `INIT`.

Every case that is not about the install sets `memex_on_path` to true, so the binary is not copied into the sandbox home and the golden is the same on every OS. The install cases, INSTALL and the install parts of PATH, are the only ones that run the copy. INSTALL-02 is the Windows twin of INSTALL-01 and has no golden made on unix: its expect.txt is written by hand from the unix golden and must be blessed on Windows.

The CASES.md claim index holds each case's claim and its design section. Where design §3.5b is silent, the claim says so and pins the choice.
