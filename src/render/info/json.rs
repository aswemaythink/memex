//! Builds `info`'s atlas as the `--json` document (design §3.6).
//! Cases: INFO-JSON-01, INFO-JSON-02, INFO-JSON-03, INFO-WARN-02

use serde_json::{json, Value as Json};

use super::{Atlas, Stream};

/// The `--json` document (design §3.6): one object with the root, the store, the collections and the warnings, and the `to`
/// and `here` objects when they show, with one line break at the end.
pub fn document(atlas: &Atlas) -> String {
    let mut out = json!({
        "root": atlas.root,
        "log_store": atlas.store,
        "collections": collections_json(atlas),
        "warnings": atlas.warnings,
    });
    if let Some(to) = &atlas.to {
        out["to"] = json!({"agent": to.agent, "count": to.count});
    }
    if let Some(here) = &atlas.here {
        out["here"] = json!({"path": here.path, "title": here.title});
    }
    format!("{out}\n")
}

/// The collections of the JSON document (design §3.6): the two built-ins, then the collections (manifests and frontmatter alike), then
/// the streams. A built-in has no kind, description or owner; a stream has kind `log` and its new count, and its newest time when it
/// has one.
fn collections_json(atlas: &Atlas) -> Vec<Json> {
    let intent = &atlas.intent;
    let mut list = vec![
        builtin(crate::corpus::READMES, intent.readmes),
        builtin(crate::corpus::HEADERS, intent.headers),
    ];
    list.extend(atlas.collections.iter().map(|collection| {
        json!({
            "name": collection.name,
            "kind": collection.kind,
            "builtin": false,
            "count": collection.count,
            "description": collection.description,
            "owner": collection.owner,
        })
    }));
    list.extend(atlas.streams.iter().map(stream_json));
    list
}

/// A built-in collection as JSON: its name and its count, with no kind, description or owner.
fn builtin(name: &str, count: usize) -> Json {
    json!({
        "name": name,
        "kind": null,
        "builtin": true,
        "count": count,
        "description": null,
        "owner": null,
    })
}

/// A log stream as JSON: its kind is `log`, and it carries its new count and, when it has one, its newest time.
fn stream_json(stream: &Stream) -> Json {
    let mut item = json!({
        "name": stream.name,
        "kind": "log",
        "builtin": false,
        "count": stream.count,
        "description": null,
        "owner": null,
        "new_24h": stream.new_24h,
    });
    if let Some(newest) = &stream.newest {
        item["newest"] = json!(newest);
    }
    item
}
