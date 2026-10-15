//! Generators of the text that memex parses, one per grammar, for the fuzz tests.
//! Each writes its design grammar with the hostile variants that memex refuses, so a test knows the exit each input gets.
//! Cases: none (stress)

use std::collections::BTreeMap;

use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;
use proptest::sample::select;
use serde_json::Value;

// The word tables are one row per line, so a reader can scan them: rustfmt would give each word a line of its own.
#[rustfmt::skip]
const KEYS: &[&str] = &["status", "mood", "phase", "owner", "notes", "scope", "risk", "wave", "Mood2", "x-ray"];
#[rustfmt::skip]
const FORGED: &[&str] = &["To", "TAGS", "Stream", "time", "Agent", "branch", "WORKTREE", "Commit"];
#[rustfmt::skip]
const PREFIXES: &[&str] = &["GEN", "ALPHA-BETA", "X", "TOK2", "CASE-7"];
const SCALAR: &str = "[a-z0-9]([a-z0-9 ._/-]{0,14}[a-z0-9])?";
const ITEM: &str = "[A-Za-z0-9]([A-Za-z0-9 ._/-]{0,8}[A-Za-z0-9])?";

/// A log body, and the fields a reader must see when memex accepts it.
#[derive(Debug, Clone)]
pub struct LogBody {
    pub text: String,
    pub fields: Option<BTreeMap<String, Value>>,
}

/// One list item: its text, and its quote if any.
type Item = (String, Option<char>);

/// One frontmatter field: its key, the lines that write it, and the value a reader must see.
#[derive(Debug, Clone)]
struct Field {
    key: &'static str,
    lines: Vec<String>,
    value: Value,
}

/// How a body is broken. `Clean` is accepted; each other variant is refused with exit 2 (design §3.4).
#[derive(Debug, Clone)]
enum Hostile {
    Clean,
    Forged(&'static str),
    Repeat,
    LoneCr,
    Unclosed,
    NoTitle,
}

/// Log bodies: frontmatter in the shapes of design §4.2, with a BOM and CRLF as choices, and the refusals of design §3.4.
pub fn log_body() -> impl Strategy<Value = LogBody> {
    let fields = vec(select(KEYS.to_vec()).prop_flat_map(field), 0..=5);
    (fields, any::<(bool, bool)>(), hostile())
        .prop_map(|(fields, (crlf, bom), hostile)| build_log(fields, crlf, bom, &hostile))
}

/// One field under `key`: a scalar, quoted or bare, an empty value, a flow list or a block list.
fn field(key: &'static str) -> impl Strategy<Value = Field> {
    prop_oneof![
        3 => (SCALAR, quote()).prop_map(move |(text, quote)| Field { key, lines: vec![format!("{key}: {}", wrap(&text, quote))], value: Value::String(text) }),
        1 => Just(Field { key, lines: vec![format!("{key}:")], value: Value::String(String::new()) }),
        2 => vec(item(), 0..=3).prop_map(move |items| list(key, &items, true)),
        2 => vec(item(), 1..=3).prop_map(move |items| list(key, &items, false)),
    ]
}

/// A list field: a flow list `key: [a, b]` when `flow`, otherwise `key:` and one `  - item` line per item.
fn list(key: &'static str, items: &[Item], flow: bool) -> Field {
    let texts: Vec<String> = items
        .iter()
        .map(|(text, quote)| wrap(text, *quote))
        .collect();
    let lines = if flow {
        vec![format!("{key}: [{}]", texts.join(", "))]
    } else {
        std::iter::once(format!("{key}:"))
            .chain(texts.iter().map(|text| format!("  - {text}")))
            .collect()
    };
    let value = Value::Array(
        items
            .iter()
            .map(|(text, _)| Value::String(text.clone()))
            .collect(),
    );
    Field { key, lines, value }
}

/// One list item, bare or quoted.
fn item() -> impl Strategy<Value = Item> {
    (ITEM, quote())
}

/// No quote, a double quote or a single quote.
fn quote() -> impl Strategy<Value = Option<char>> {
    prop_oneof![2 => Just(None), 1 => Just(Some('"')), 1 => Just(Some('\''))]
}

/// The text inside its quote, when it has one.
fn wrap(text: &str, quote: Option<char>) -> String {
    quote.map_or_else(|| text.to_string(), |mark| format!("{mark}{text}{mark}"))
}

/// A body's breakage: mostly none, otherwise one refusal.
fn hostile() -> impl Strategy<Value = Hostile> {
    prop_oneof![
        4 => Just(Hostile::Clean),
        1 => select(FORGED.to_vec()).prop_map(Hostile::Forged),
        1 => Just(Hostile::Repeat),
        1 => Just(Hostile::LoneCr),
        1 => Just(Hostile::Unclosed),
        1 => Just(Hostile::NoTitle),
    ]
}

/// Writes the body from its fields and its breakage. Fields are keyed, so a repeated key keeps its last field.
fn build_log(fields: Vec<Field>, crlf: bool, bom: bool, hostile: &Hostile) -> LogBody {
    let fields: BTreeMap<&str, Field> =
        fields.into_iter().map(|field| (field.key, field)).collect();
    let mut lines = vec!["---".to_string()];
    lines.extend(fields.values().flat_map(|field| field.lines.clone()));
    lines.extend(breakage(hostile));
    if !matches!(hostile, Hostile::Unclosed) {
        lines.push("---".to_string());
    }
    let title = if matches!(hostile, Hostile::NoTitle) {
        "#"
    } else {
        "# Generated title"
    };
    lines.extend([title.to_string(), String::new(), "Body text.".to_string()]);
    let eol = if crlf { "\r\n" } else { "\n" };
    let bom = if bom { "\u{feff}" } else { "" };
    let expected: BTreeMap<String, Value> = fields
        .values()
        .map(|field| (field.key.to_string(), field.value.clone()))
        .collect();
    let accepted = matches!(hostile, Hostile::Clean).then_some(expected);
    LogBody {
        text: format!("{bom}{}{eol}", lines.join(eol)),
        fields: accepted,
    }
}

/// The lines that a breakage adds to the block.
fn breakage(hostile: &Hostile) -> Vec<String> {
    match hostile {
        Hostile::Forged(key) => vec![format!("{key}: 1")],
        Hostile::Repeat => vec!["dup: 1".to_string(), "DUP: 2".to_string()],
        Hostile::LoneCr => vec!["note: a\rb".to_string()],
        Hostile::Clean | Hostile::Unclosed | Hostile::NoTitle => Vec::new(),
    }
}

/// A CASES.md and the ids it declares: each item outside a fence, in the order written (design §2.2).
#[derive(Debug, Clone)]
pub struct CasesDoc {
    pub text: String,
    pub declared: Vec<String>,
}

/// One block of a generated CASES.md.
#[derive(Debug, Clone)]
enum Block {
    Heading(String),
    Item(&'static str, bool, String),
    Paragraph(String),
    Fence(bool, usize, Vec<String>),
}

/// CASES.md text: headings, items, paragraphs, closed fences of fake items, and an optional unclosed fence at the end.
pub fn cases_doc() -> impl Strategy<Value = CasesDoc> {
    let tail = option::of(vec(fence_line(), 0..=2));
    (vec(block(), 1..=12), tail, any::<(bool, bool)>())
        .prop_map(|(blocks, tail, (crlf, bom))| build_cases(&blocks, tail.as_deref(), crlf, bom))
}

/// One block of a CASES.md, weighted towards items. A paragraph of empty text is a blank line.
fn block() -> impl Strategy<Value = Block> {
    let claim = "[A-Za-z][A-Za-z0-9 ,.()`-]{0,40}";
    prop_oneof![
        2 => "[A-Za-z][A-Za-z ]{0,20}".prop_map(Block::Heading),
        6 => (select(PREFIXES.to_vec()), any::<bool>(), claim).prop_map(|(p, star, claim)| Block::Item(p, star, claim)),
        2 => "[a-z][a-z ,.]{0,30}".prop_map(Block::Paragraph),
        1 => Just(Block::Paragraph(String::new())),
        2 => (any::<bool>(), 3usize..=4, vec(fence_line(), 0..=3)).prop_map(|(t, w, b)| Block::Fence(t, w, b)),
    ]
}

/// One line inside a fence: words, or a fake item that the fence must hide. Backticks and tildes are left out, so the
/// line never closes the fence early.
fn fence_line() -> impl Strategy<Value = String> {
    prop_oneof![
        2 => "[a-z#*\\- ]{0,20}",
        1 => "[0-9]{4}".prop_map(|number| format!("- `FAKE-{number}` fake claim")),
    ]
}

/// Writes the blocks as CASES.md lines behind a `memex: cases` frontmatter block, and collects the ids that the items declare.
fn build_cases(blocks: &[Block], tail: Option<&[String]>, crlf: bool, bom: bool) -> CasesDoc {
    let mut lines = vec![
        "---".to_string(),
        "memex: cases".to_string(),
        "---".to_string(),
        "# Generated cases".to_string(),
        String::new(),
    ];
    let mut declared = Vec::new();
    for block in blocks {
        lines.extend(block_lines(block, declared.len() + 1, &mut declared));
    }
    if let Some(body) = tail {
        lines.extend(std::iter::once("```".to_string()).chain(body.iter().cloned()));
    }
    let eol = if crlf { "\r\n" } else { "\n" };
    let bom = if bom { "\u{feff}" } else { "" };
    CasesDoc {
        text: format!("{bom}{}{eol}", lines.join(eol)),
        declared,
    }
}

/// The lines of one block. An item takes the id that `number` gives it, and is pushed onto `declared`.
fn block_lines(block: &Block, number: usize, declared: &mut Vec<String>) -> Vec<String> {
    match block {
        Block::Heading(text) => vec![format!("## {text}")],
        Block::Item(prefix, star, claim) => {
            let id = format!("{prefix}-{number:04}");
            let marker = if *star { "*" } else { "-" };
            let line = format!("{marker} `{id}` {claim}").trim_end().to_string();
            declared.push(id);
            vec![line]
        }
        Block::Paragraph(text) => vec![text.clone()],
        Block::Fence(tilde, width, body) => {
            let fence = (if *tilde { "~" } else { "`" }).repeat(*width);
            let info = if *tilde { "" } else { "rust" };
            [vec![format!("{fence}{info}")], body.clone(), vec![fence]].concat()
        }
    }
}

/// A source file: its path under the sandbox root, and its bytes.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// One generated file before it is rendered: its extension (`rs`, `py` or `html`), its header style, its options, and
/// two lines of words for the header.
#[derive(Debug, Clone)]
struct Header {
    ext: &'static str,
    style: u8,
    license: bool,
    bom: bool,
    crlf: bool,
    words: (String, String),
}

// The header templates of each comment syntax (design §2.1): `%P` is the purpose and `%E` the second line of words.
#[rustfmt::skip]
const RS_HEADS: [&str; 4] = ["//! %P\n//! %E\n", "/// %P\n", "// %P\n// %E\n", "/* %P\n * %E\n */\n"];
#[rustfmt::skip]
const PY_HEADS: [&str; 2] = ["# %P\n# %E\n", "\"\"\"%P\n\n%E\n\"\"\"\n"];
const HTML_HEAD: &str = "<!-- %P\n     %E -->\n";
const RS_LICENSE: &str = "// SPDX-License-Identifier: MIT\n\n";
const PY_LICENSE: &str = "# SPDX-License-Identifier: MIT\n\n";
const HTML_LICENSE: &str = "<!-- SPDX-License-Identifier: MIT -->\n\n";
const RS_CODE: &str = "pub fn value() -> u8 {\n    1\n}\n";
const PY_CODE: &str = "def value():\n    return 1\n";
const HTML_CODE: &str = "<p>value</p>\n";

/// One to four source files whose headers use random comment syntax, BOMs, CRLF and license blocks.
pub fn source_files() -> impl Strategy<Value = Vec<SourceFile>> {
    vec(header(), 1..=4).prop_map(|headers| {
        let named = |(index, header): (usize, &Header)| SourceFile {
            name: format!("src/f{index}.{}", header.ext),
            bytes: header.bytes(),
        };
        headers.iter().enumerate().map(named).collect()
    })
}

/// One header, with a purpose sentence and a second line of text.
fn header() -> impl Strategy<Value = Header> {
    let words = ("[A-Za-zé][A-Za-zé ,]{3,40}[A-Za-z]\\.", "[a-z][a-z ]{0,20}");
    let options = any::<(bool, bool, bool)>();
    (select(vec!["rs", "py", "html"]), 0u8..6, options, words).prop_map(
        |(ext, style, (license, bom, crlf), words)| Header {
            ext,
            style,
            license,
            bom,
            crlf,
            words,
        },
    )
}

impl Header {
    /// The bytes: the text with CRLF line ends when chosen, and a BOM in front when chosen.
    fn bytes(&self) -> Vec<u8> {
        let eol = if self.crlf { "\r\n" } else { "\n" };
        let bom: &[u8] = if self.bom { &[0xEF, 0xBB, 0xBF] } else { &[] };
        [bom, self.text().replace('\n', eol).as_bytes()].concat()
    }

    /// The text: an optional license block, the header in the extension's comment syntax, then code.
    fn text(&self) -> String {
        let (license, head, code) = match self.ext {
            "rs" => (RS_LICENSE, RS_HEADS[usize::from(self.style % 4)], RS_CODE),
            "py" => (PY_LICENSE, PY_HEADS[usize::from(self.style % 2)], PY_CODE),
            _ => (HTML_LICENSE, HTML_HEAD, HTML_CODE),
        };
        let license = if self.license { license } else { "" };
        let head = head
            .replace("%P", &self.words.0)
            .replace("%E", &self.words.1);
        format!("{license}{head}\n{code}")
    }
}

/// The keys memex writes itself (design §3.4 and §4.3), lower-cased: a reader strips them from a doc's fields.
const MEMEX_KEYS: &[&str] = &[
    "stream", "time", "agent", "to", "tags", "branch", "worktree", "commit",
];

/// The case ids of a roster that `search --in cases` printed: the label of each `cases:` line, in print order.
pub fn roster_ids(stdout: &str) -> Vec<String> {
    let labels = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next());
    labels
        .filter_map(|label| label.strip_prefix("cases:"))
        .map(str::to_string)
        .collect()
}

/// The agent's fields in the one doc that `show --json` printed: its `fields`, without memex's own keys.
pub fn agent_fields(stdout: &[u8]) -> Result<BTreeMap<String, Value>, String> {
    let doc: Value =
        serde_json::from_slice(stdout).map_err(|error| format!("show --json: {error}"))?;
    let fields = doc["docs"][0]["fields"]
        .as_object()
        .ok_or("show --json has no docs[0].fields")?;
    let agent = fields
        .iter()
        .filter(|(key, _)| !MEMEX_KEYS.contains(&key.to_ascii_lowercase().as_str()));
    Ok(agent
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}
