//! The comment families of design §2.1: the comment markers of each extension or file name.
//! Cases: HDR-SLASH-01, HDR-SLASH-02, HDR-SLASH-03, HDR-SLASH-04, HDR-SLASH-05, HDR-HASH-01, HDR-DOC-01, HDR-DOC-02, HDR-DOC-03, HDR-DOC-04, HDR-MAKE-01, HDR-MAKE-02, HDR-DASH-01, HDR-DASH-02, HDR-DASH-03, HDR-SEMI-01, HDR-PERCENT-01, HDR-MARKUP-01, HDR-PAREN-01, HDR-SKIP-06, HDR-EXT-01

use std::path::Path;

/// How one comment family writes its comments (design §2.1).
pub struct Family {
    /// Line-comment markers, longest first. A line whose text starts with one is a comment line.
    pub lines: &'static [&'static str],
    /// Block comments as (open, close). A block opens on a line that starts with `open` and closes on the line that holds `close`.
    pub blocks: &'static [(&'static str, &'static str)],
    /// Docstring delimiters. A module docstring that opens with one is the header when no `#` block leads (Python).
    pub docstrings: &'static [&'static str],
}

/// `//`, `///` and `//!` line comments, and `/* */` and `/** */` blocks.
static SLASH: Family = Family {
    lines: &["///", "//!", "//"],
    blocks: &[("/*", "*/")],
    docstrings: &[],
};

/// `#` line comments.
static HASH: Family = Family {
    lines: &["#"],
    blocks: &[],
    docstrings: &[],
};

/// `#` line comments, with a module docstring as the fallback when no `#` block leads.
static PYTHON: Family = Family {
    lines: &["#"],
    blocks: &[],
    docstrings: &["\"\"\"", "'''"],
};

/// `--` line comments, and `{- -}` and `--[[ ]]` blocks.
static DASH: Family = Family {
    lines: &["---", "--"],
    blocks: &[("{-", "-}"), ("--[[", "]]")],
    docstrings: &[],
};

/// `;` line comments, as Lisp and assembly write them.
static SEMI: Family = Family {
    lines: &[";;;", ";;", ";"],
    blocks: &[],
    docstrings: &[],
};

/// `%` line comments, as Erlang and TeX write them.
static PERCENT: Family = Family {
    lines: &["%%", "%"],
    blocks: &[],
    docstrings: &[],
};

/// `<!-- -->` blocks, as markup writes them.
static MARKUP: Family = Family {
    lines: &[],
    blocks: &[("<!--", "-->")],
    docstrings: &[],
};

/// `(* *)` blocks, as OCaml writes them.
static OCAML: Family = Family {
    lines: &[],
    blocks: &[("(*", "*)")],
    docstrings: &[],
};

/// The extensions of each family, as design §2.1 lists them.
static BY_EXTENSION: &[(&Family, &[&str])] = &[
    (
        &SLASH,
        &[
            "rs", "c", "h", "cc", "cpp", "hpp", "cs", "java", "kt", "kts", "scala", "swift", "go",
            "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "dart", "php", "css", "scss",
            "less", "zig", "proto", "groovy",
        ],
    ),
    (
        &HASH,
        &[
            "rb", "sh", "bash", "zsh", "fish", "pl", "r", "jl", "ex", "exs", "nix", "tf", "yaml",
            "yml", "toml", "ps1", "cmake", "mk",
        ],
    ),
    (&PYTHON, &["py"]),
    (&DASH, &["hs", "elm", "lua", "sql", "ada"]),
    (&SEMI, &["clj", "cljs", "el", "lisp", "scm", "asm"]),
    (&PERCENT, &["erl", "hrl", "tex", "m"]),
    (&MARKUP, &["html", "vue", "svelte", "xml"]),
    (&OCAML, &["ml", "mli"]),
];

/// The file names of each family, matched exactly (design §2.1).
static BY_NAME: &[(&Family, &[&str])] = &[(&HASH, &["Makefile", "Dockerfile"])];

/// The lines skipped before a header (design §2.1): a shebang, encoding and mode lines, Docker parser directives, Go and
/// Haskell pragmas, a JavaScript directive such as `"use client"`, and a markup prologue (`<?xml`, `<!DOCTYPE`, `<?php`).
/// Each is matched at the start of the trimmed line. The DOCTYPE keyword is matched in both of its usual cases.
pub static PRAGMAS: &[&str] = &[
    "#!",
    "# -*-",
    "# coding:",
    "# coding=",
    "# syntax=",
    "# escape=",
    "//go:",
    "// +build",
    "{-#",
    "\"use ",
    "'use ",
    "<?xml",
    "<!DOCTYPE",
    "<!doctype",
    "<?php",
];

/// The comment family of the file at `path`: by its file name first, then by its extension. None when neither is listed,
/// which means the file is not harvested (design §2.1).
pub fn family_of(path: &Path) -> Option<&'static Family> {
    let by_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| lookup(BY_NAME, name));
    by_name.or_else(|| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .and_then(|extension| lookup(BY_EXTENSION, extension))
    })
}

/// The family whose key list holds `key`.
fn lookup(
    table: &'static [(&'static Family, &'static [&'static str])],
    key: &str,
) -> Option<&'static Family> {
    table
        .iter()
        .find(|(_, keys)| keys.contains(&key))
        .map(|(family, _)| *family)
}
