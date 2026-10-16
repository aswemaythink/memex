//! Reads a Markdown file's `memex:` frontmatter into docs (design §2.2) and its title (design §3.3).
//! A head is read once: when it holds the whole file, the text comes from it, so no Markdown file is opened twice for its head.
//! Cases: COLL-FRONT-01, COLL-FRONT-02, COLL-FRONT-03, COLL-FRONT-04, COLL-FRONT-05, COLL-FRONT-06, COLL-FRONT-07, COLL-FRONT-08, COLL-NAME-01, COLL-NAME-02, COLL-ONE-01, COLL-ONE-02, COLL-ONE-03, COLL-ONE-04, COLL-README-01, COLL-UNREAD-01, COLL-PLAIN-01, COLL-INFO-02, SHOW-MDCHILD-01, SHOW-MDCHILD-02, SHOW-MDCHILD-03, SHOW-MDCHILD-04

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use super::entries;
use super::files::Found;
use super::Origin;
use crate::collections::frontmatter::{after_frontmatter, split, Document, Value};
use crate::collections::manifest::{Kind, Loaded, Manifest};
use crate::collections::title;
use crate::corpus::Doc;
use crate::error::Error;
use crate::root::slash;
use crate::text;

/// The most bytes of a file that frontmatter reads from its head: the limit that header detection reads too (design §2.1).
const HEAD_BYTES: u64 = 8 * 1024;

/// The line that opens and closes a frontmatter block.
const FENCE: &str = "---";

/// The start of the frontmatter line that names the collection (design §2.2).
const KEY_LINE: &str = "memex:";

/// The frontmatter key that names the collection, which is left out of a doc's fields.
const KEY: &str = "memex";

/// The UTF-8 byte order mark, which a file may start with before its frontmatter.
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The first bytes of a file: up to 8 KiB, and whether they are all of it, so that a file no longer than the limit is read once.
pub struct Head {
    /// The bytes read, at most `HEAD_BYTES` of them.
    bytes: Vec<u8>,
    /// True when the file is no longer than the limit, so that `bytes` holds all of it.
    whole: bool,
}

impl Head {
    /// Reads the head of the file at `path` in one open. One byte past the limit is read too, so a file of exactly the limit
    /// counts as whole. A file that cannot be read is the error `cannot read`, which the corpus warns about (design §4.2).
    pub fn read(path: &Path) -> Result<Head, Error> {
        let mut bytes = Vec::new();
        File::open(path)
            .and_then(|file| file.take(HEAD_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|failure| Error::io_at("cannot read", path, &failure))?;
        Ok(Head::of(bytes))
    }

    /// Reads the whole file at `path` in one open, for a reader that always needs all of it, such as a README: its head and its
    /// text, read as `text::read` reads it (design §4.4). A file that cannot be read is the error `cannot read`.
    pub fn read_all(path: &Path) -> Result<(Head, String), Error> {
        let bytes =
            fs::read(path).map_err(|failure| Error::io_at("cannot read", path, &failure))?;
        let text = text::decode(&bytes);
        Ok((Head::of(bytes), text))
    }

    /// The head of a file whose first bytes are `bytes`: at most `HEAD_BYTES` of them, and whole when there are no more.
    fn of(mut bytes: Vec<u8>) -> Head {
        let whole = bytes.len() as u64 <= HEAD_BYTES;
        bytes.truncate(HEAD_BYTES as usize);
        Head { bytes, whole }
    }

    /// The text of the whole file, read as `text::read` reads it (design §4.4): the head itself when it holds the whole file, else
    /// the file again.
    pub fn text(&self, path: &Path) -> Result<String, Error> {
        if self.whole {
            return Ok(text::decode(&self.bytes));
        }
        text::read(path).map_err(|failure| Error::io_at("cannot read", path, &failure))
    }

    /// The title of the file's text, which its folder's child line shows (design §3.3): the default line of the text after any
    /// frontmatter. It is read from the head alone, so a title that starts past the limit is not seen (design §2.1), and a head
    /// cut by the limit keeps only its complete lines.
    pub fn markdown_title(&self) -> String {
        let mut head_text = text::decode(&self.bytes);
        if !self.whole {
            head_text.truncate(head_text.rfind('\n').map_or(0, |end| end + 1));
        }
        title::default_title(after_frontmatter(&head_text))
    }

    /// True when the frontmatter of the head has a `memex:` line before its closing fence, and that line is complete within the
    /// head, so a line cut by the limit is not seen (design §2.2). The key must lie in the first 8 KiB of the file.
    pub fn has_key(&self) -> bool {
        let body = self
            .bytes
            .strip_prefix(BOM)
            .unwrap_or(self.bytes.as_slice());
        if !body.starts_with(b"---") {
            return false;
        }
        let text = text::decode(&self.bytes);
        let mut lines = text
            .split_inclusive('\n')
            .filter(|piece| self.whole || piece.ends_with('\n'))
            .map(text::line_of);
        lines.next() == Some(FENCE)
            && lines
                .take_while(|line| *line != FENCE)
                .any(|line| line.starts_with(KEY_LINE))
    }
}

/// True for a README whose frontmatter holds a `memex:` key, which the graph ignores (design §2.2). The key is in the head, and
/// the fence closes in the whole text `text`.
pub fn readme_has_key(head: &Head, text: &str) -> bool {
    head.has_key() && closed(text).is_some()
}

/// The docs of the Markdown file at `path`, which the walk labels `rel` (design §2.2), and its title (design §3.3). A file whose head
/// has no `memex:` key is no doc and is not read again. A file with the key gives its entries, or else its one doc; a bad name is
/// a warning.
pub fn markdown(rel: &str, path: &Path) -> Result<Found, Error> {
    let head = Head::read(path)?;
    let title = Some(head.markdown_title());
    let found = if head.has_key() {
        from_text(rel, path, &head.text(path)?)
    } else {
        Found::default()
    };
    Ok(Found { title, ..found })
}

/// The docs of a Markdown text whose frontmatter names a collection (design §2.2). A file whose fence never closes is no doc.
fn from_text(rel: &str, path: &Path, text: &str) -> Found {
    let Some(document) = closed(text) else {
        return Found::default();
    };
    let Some(value) = memex_value(&document.fields).cloned() else {
        return Found::default();
    };
    match value {
        Value::Str(name) if valid_name(&name) => Found {
            docs: docs_of(&name, rel, document),
            ..Found::default()
        },
        value => Found {
            warnings: vec![name_warning(path, &value)],
            ..Found::default()
        },
    }
}

/// The frontmatter of a text whose fence closes (design §2.2): its fields and body, as `split` gives them. A block that never closes
/// is no frontmatter, so None.
fn closed(text: &str) -> Option<Document> {
    let document = split(text);
    // `split` keeps the whole text as the body when the fence never closes, and a closed block always drops its opening line.
    (document.body.len() < text.len()).then_some(document)
}

/// The value of the `memex` key: the first one, as `split` keeps it.
fn memex_value(fields: &[(String, Value)]) -> Option<&Value> {
    fields
        .iter()
        .find(|(key, _)| key == KEY)
        .map(|(_, value)| value)
}

/// True for a collection name: `[a-z0-9][a-z0-9-]{0,39}`, as a log stream name is (design §2.2 and §3.4).
fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase() || ch.is_ascii_digit())
        && name.len() <= 40
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// The warning for a `memex:` value that is not a collection name, which skips the file (design §2.2).
fn name_warning(path: &Path, value: &Value) -> String {
    format!(
        "{}: 'memex: {}' is not a collection name; use a-z, 0-9 and -",
        slash(path),
        shown(value)
    )
}

/// A frontmatter value as the warning shows it: its text, or a flow list in brackets.
fn shown(value: &Value) -> String {
    match value {
        Value::Str(text) => text.clone(),
        Value::List(items) => format!("[{}]", items.join(", ")),
    }
}

/// The docs of a file whose `memex:` names the collection `name`: its entries when it has any (design §2.2), else its one doc.
fn docs_of(name: &str, rel: &str, document: Document) -> Vec<Doc> {
    let entries = entries::docs(name, rel, &document.body);
    if entries.is_empty() {
        vec![whole_doc(name, rel, document)]
    } else {
        entries
    }
}

/// The one doc of a collection file (design §2.2): its id is the `id` field, else the file stem; its title is its first heading;
/// its fields are its frontmatter keys other than `memex`; its text is its body. Its label is `name:id`.
fn whole_doc(name: &str, rel: &str, document: Document) -> Doc {
    let id = text_field(&document.fields, "id").unwrap_or_else(|| stem_of(rel));
    let title = title::default_title(&document.body);
    let fields = document
        .fields
        .into_iter()
        .filter(|(key, _)| key != KEY)
        .collect();
    Doc {
        label: format!("{name}:{id}"),
        collection: name.to_string(),
        id,
        path: Some(rel.to_string()),
        title,
        text: document.body,
        fields,
        origin: Origin::Whole,
    }
}

/// The text of a frontmatter field whose value is a scalar with some text.
fn text_field(fields: &[(String, Value)], key: &str) -> Option<String> {
    fields.iter().find_map(|(name, value)| match value {
        Value::Str(text) if name == key && !text.is_empty() => Some(text.clone()),
        _ => None,
    })
}

/// The file name of a root-relative path without its last extension.
fn stem_of(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())
}

/// The number of docs that each collection named by frontmatter holds (design §4.2, one collection from two sources). A file's
/// entries and whole-file docs count; the graph's docs and the manifests' docs are counted elsewhere.
pub fn counts<'a>(docs: impl Iterator<Item = &'a Doc>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for doc in docs.filter(|doc| matches!(doc.origin, Origin::Entry | Origin::Whole)) {
        *counts.entry(doc.collection.clone()).or_insert(0) += 1;
    }
    counts
}

/// Adds each counted collection to the loaded manifests (design §3.5 and §4.2): a manifest of the same name takes the count, and a
/// name with no manifest becomes a Markdown collection with no description or owner.
pub fn join(counts: BTreeMap<String, usize>, loaded: &mut Vec<Loaded>) {
    for (name, count) in counts {
        match loaded.iter_mut().find(|found| found.manifest.name == name) {
            Some(found) => found.count += count,
            None => loaded.push(Loaded {
                manifest: markdown_manifest(name),
                count,
                warnings: Vec::new(),
            }),
        }
    }
}

/// The stand-in manifest of a collection that only frontmatter names: it has no file, and its kind is `markdown` (PLAN §13 R7), with no roots and the default ranking (design §4.2).
fn markdown_manifest(name: String) -> Manifest {
    Manifest {
        name,
        description: None,
        owner: None,
        kind: Kind::Markdown,
        roots: Vec::new(),
        include: Vec::new(),
        id: None,
        title: None,
        weights: BTreeMap::new(),
        marker: None,
        merge: Vec::new(),
        at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_collection_name_is_lowercase_letters_digits_and_dashes_up_to_40() {
        assert!(valid_name("decisions"));
        assert!(valid_name("0-notes"));
        assert!(valid_name(&"a".repeat(40)));
        assert!(!valid_name(&"a".repeat(41)));
        assert!(!valid_name("-lead"));
        assert!(!valid_name("Decisions"));
        assert!(!valid_name("two words"));
        assert!(!valid_name(""));
    }

    #[test]
    fn a_key_before_the_closing_fence_is_seen_and_a_key_after_it_is_not() {
        let head = |text: &str| Head {
            bytes: text.as_bytes().to_vec(),
            whole: true,
        };
        assert!(head("---\nmemex: x\n---\n# T\n").has_key());
        assert!(head("\u{feff}---\r\nmemex: x\r\n---\r\n").has_key());
        assert!(!head("---\nstatus: open\n---\nmemex: x\n").has_key());
        assert!(!head("# Title\n---\nmemex: x\n").has_key());
    }

    #[test]
    fn a_key_line_cut_by_the_limit_is_not_seen() {
        let whole = Head {
            bytes: b"---\nmemex: decis".to_vec(),
            whole: true,
        };
        let cut = Head {
            bytes: b"---\nmemex: decis".to_vec(),
            whole: false,
        };
        assert!(whole.has_key());
        assert!(!cut.has_key());
    }

    #[test]
    fn an_unclosed_fence_is_no_frontmatter() {
        assert!(closed("---\nmemex: x\n").is_none());
        assert!(closed("---\nmemex: x\n---\nbody\n").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_markdown_file_is_an_error_naming_its_path() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("a temp folder");
        let path = dir.path().join("locked.md");
        std::fs::write(&path, "---\nmemex: x\n---\n").expect("a file");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("a mode");
        if std::fs::read(&path).is_ok() {
            // Running as root reads any file, so there is nothing to check.
            return;
        }
        let error = markdown("locked.md", &path).err();
        assert!(
            error.is_some_and(|error| error.message.ends_with("locked.md (PermissionDenied)")),
            "an unreadable .md is an error"
        );
    }
}
