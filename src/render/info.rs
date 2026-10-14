//! The atlas that `info` prints (design §3.5 and §3.6): its parts as data, with its two renderings.
//! Cases: INFO-CAP-02

mod json;
mod text;

pub use json::document;
pub use text::text;

/// The atlas that `info` prints: where things are, what is in them, and the warnings of the run (design §3.5 and §3.6).
pub struct Atlas {
    /// The root as an absolute path, with `/` separators.
    pub root: String,
    /// The log store as an absolute path, with `/` separators; None when there is no `.memex/`.
    pub store: Option<String>,
    /// The store as the header prints it: root-relative with a trailing `/` inside the root, else absolute, or `none`.
    pub store_label: String,
    /// The built-in docs of the intent layer, counted.
    pub intent: Intent,
    /// The manifest collections, in name order.
    pub collections: Vec<Collection>,
    /// The log streams, newest entry first and then by name.
    pub streams: Vec<Stream>,
    /// The entries addressed to the agent, whenever an agent is known, zero included (design §3.5).
    pub to: Option<ToCount>,
    /// The nearest README at or above the cwd, when the cwd is below the root and such a README exists.
    pub here: Option<HereLine>,
    /// The warnings of the run: the manifests that did not load, the duplicate case ids, then the files that could not be read.
    /// Only JSON lists them.
    pub warnings: Vec<String>,
}

/// The counts of the built-in collections: READMEs, headers and case ids.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Intent {
    /// The READMEs of the root.
    pub readmes: usize,
    /// The files with a header block.
    pub headers: usize,
    /// The case ids of every CASES.md.
    pub cases: usize,
}

/// A manifest collection: its name, its kind, its doc count and what its manifest says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// The collection's name, which is its manifest's file stem.
    pub name: String,
    /// The manifest's kind: `markdown`, `folder` or `json`.
    pub kind: &'static str,
    /// The docs the manifest gives.
    pub count: usize,
    /// The manifest's description, when it has one.
    pub description: Option<String>,
    /// The manifest's owner, when it has one.
    pub owner: Option<String>,
}

/// A log stream: its name, its entry count, how many of its entries are new, and the stamp of its newest entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream {
    /// The stream's folder name.
    pub name: String,
    /// Every entry in the stream.
    pub count: usize,
    /// The entries no more than 24 hours old.
    pub new_24h: usize,
    /// The RFC 3339 time of the newest entry, when its name carries a stamp.
    pub newest: Option<String>,
}

/// The entries addressed to an agent: the agent's id and how many entries name it in their `to` list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToCount {
    /// The agent id, as the atlas was given it.
    pub agent: String,
    /// The entries in the store whose `to` list holds the agent.
    pub count: usize,
}

/// The nearest README of the cwd: its folder's label and its one-liner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HereLine {
    /// The root-relative label of the folder the README is in, `.` for the root.
    pub path: String,
    /// The README's one-liner, in full.
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::text::CAP;
    use super::*;

    /// An atlas whose root is `root` and whose streams line lists forty streams, so it is long enough to be cut.
    fn atlas_with_root(root: String) -> Atlas {
        let stream = |index| Stream {
            name: format!("stream-{index:02}-with-a-long-name"),
            count: 1,
            new_24h: 0,
            newest: None,
        };
        Atlas {
            root,
            store: None,
            store_label: "none (run memex init)".to_string(),
            intent: Intent::default(),
            collections: Vec::new(),
            streams: (0..40).map(stream).collect(),
            to: None,
            here: None,
            warnings: Vec::new(),
        }
    }

    /// INFO-CAP-02: a text of exactly 3,000 bytes is not cut, and one of 3,001 bytes is. The root is padded to hit the boundary,
    /// which no golden can do on every OS, since the sandbox root's length differs.
    #[test]
    fn info_cap_02_a_text_of_exactly_the_cap_is_not_cut_and_one_byte_more_is() {
        let short = text(&atlas_with_root(String::new()));
        assert!(short.len() < CAP && !short.contains("more"), "{short}");
        let exact = text(&atlas_with_root("/".repeat(CAP - short.len())));
        assert!(exact.len() == CAP && !exact.contains("more"), "{exact}");
        let over = text(&atlas_with_root("/".repeat(CAP - short.len() + 1)));
        assert!(over.len() <= CAP && over.contains("+1 more"), "{over}");
    }
}
