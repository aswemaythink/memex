//! The help texts and the grammar block of design §3, plus the usage lines that usage errors print.
//! Cases: CLI-*, CLI-HELP-01, CLI-HELP-02, CLI-HELP-03, CLI-HELP-04, CLI-HELP-05, CLI-HELP-06, CLI-HINTB-01, CHK-EXIT-04

use crate::cli::Verb;

/// The grammar block of design §3: one line per verb, then the top-level flags.
/// The first six lines are in `Verb::ALL` order, so `Verb as usize` indexes them.
const GRAMMAR: [&str; 7] = [
    "memex search [<word|field:value>...] [--in <coll>[,<coll>...]] [--limit N] [--json]",
    "memex show   <path|ref>... [--up N] [--comments] [--limit N] [--json]",
    "memex log    <stream> [--to <id>[,...]] [--tag <t>[,...]] [--agent <id>] [--json]  <<'EOF' … EOF",
    "memex info   [--agent <id>] [--json]",
    "memex check  [<path|glob>...] [--limit N] [--json]",
    "memex init",
    "memex --help | --version",
];

const TOP: [&str; 16] = [
    "memex is authored context compression: it indexes a repo's intent layer so agents read intent before code.",
    "",
    GRAMMAR[0],
    GRAMMAR[1],
    GRAMMAR[2],
    GRAMMAR[3],
    GRAMMAR[4],
    GRAMMAR[5],
    GRAMMAR[6],
    "",
    "search  find docs by words and field:value filters",
    "show    print the header, citations and chain of each ref",
    "log     append an entry from stdin and print its ref",
    "info    print where things are, never their contents",
    "check   report where intent is missing or broken, for the root or some of it",
    "init    set up .memex/ and the AGENTS.md snippet",
];

const SEARCH: [&str; 5] = [
    GRAMMAR[0],
    "Find docs by words and field:value filters, grouped by collection.",
    "  --in <coll>[,<coll>...]  search only these collections; with no words, list their roster",
    "  --limit N                hits per group (default 5; 20 with --in)",
    "  --json                   print one JSON document instead of text",
];

const SHOW: [&str; 6] = [
    GRAMMAR[1],
    "Print the header, citations and chain of each path or ref, separated by === lines.",
    "  --up N      depth of the chain; --up 0 prints the file alone",
    "  --comments  also print the file's other comment blocks (a file with no header prints its first 3)",
    "  --limit N   rows in the children and cited-by lists (default 50; comments 20)",
    "  --json      print one JSON document instead of text",
];

const LOG: [&str; 6] = [
    GRAMMAR[2],
    "Append an entry whose body is read from stdin, and print its ref.",
    "  --to <id>[,...]   address the entry to these ids",
    "  --tag <t>[,...]   tag the entry with these names",
    "  --agent <id>      agent id; else the MEMEX_AGENT variable; else omitted",
    "  --json            print {ref, path} as JSON",
];

const INFO: [&str; 4] = [
    GRAMMAR[3],
    "Print where things are, never their contents: the one overview for humans and agents.",
    "  --agent <id>  agent id for the to line; else the MEMEX_AGENT variable",
    "  --json        print one JSON document instead of text",
];

const CHECK: [&str; 5] = [
    GRAMMAR[4],
    "Report where the intent layer is missing, thin or broken: headers, readmes, length, one-liners, thin, cites and ids.",
    "A path names a file or folder, and a glob (with *, ? or [) matches walked paths; with none, the root is checked.",
    "  --limit N   findings listed per kind (default 20)",
    "  --json      print one JSON document instead of text",
];

const INIT: [&str; 2] = [
    GRAMMAR[5],
    "Set up .memex/ and the AGENTS.md snippet; safe to run again, and it never overwrites a file.",
];

/// The grammar line of one verb, as a usage error prints it after `usage: `.
pub fn usage_line(verb: Verb) -> &'static str {
    GRAMMAR[verb as usize]
}

/// The usage block for an error that names no verb: a `usage:` line, then the whole grammar block.
pub fn usage_block() -> Vec<String> {
    let mut lines = vec!["usage:".to_string()];
    lines.extend(GRAMMAR.iter().map(|line| line.to_string()));
    lines
}

/// The help text with a final newline: the top level for None, otherwise one verb's help.
pub fn text(verb: Option<Verb>) -> String {
    let lines: &[&str] = match verb {
        None => &TOP,
        Some(Verb::Search) => &SEARCH,
        Some(Verb::Show) => &SHOW,
        Some(Verb::Log) => &LOG,
        Some(Verb::Info) => &INFO,
        Some(Verb::Check) => &CHECK,
        Some(Verb::Init) => &INIT,
    };
    let mut text = lines.join("\n");
    text.push('\n');
    text
}
