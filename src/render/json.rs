//! Builds the `--json` document of `show` (design §3.6): `{"docs":[...],"errors":[...]}`, with each doc's fields, cites, cited_by, children and chain.
//! Cases: SHOW-JSON-01, SHOW-JSON-02, SHOW-JSON-03, SHOW-JSON-04, SHOW-JSON-05

use serde_json::{json, Map, Value as Json};

use super::{Link, Listing, Miss, Ref, Section};
use crate::cite::Cite;
use crate::collections::frontmatter::Value;
use crate::corpus::{Child, Kind};

/// The whole `show` document, as one JSON line with its newline.
pub fn document(sections: &[Section], misses: &[Miss]) -> String {
    let docs: Vec<Json> = sections.iter().map(doc).collect();
    let errors: Vec<Json> = misses.iter().map(miss).collect();
    format!("{}\n", json!({"docs": docs, "errors": errors}))
}

/// One doc: the section's parts under the names of design §3.6. Absent parts are null, and `children` only for a folder.
fn doc(section: &Section) -> Json {
    let mut value = json!({
        "ref": section.label,
        "collection": section.collection,
        "id": section.id,
        "path": section.path,
        "title": section.title,
        "fields": fields(section.fields.as_deref().unwrap_or(&[])),
        "text": section.text,
        "cites": section.cites.iter().map(cite).collect::<Vec<Json>>(),
        "cited_by": cited_by(&section.cited_by),
        "chain": section.chain.as_deref().unwrap_or(&[]).iter().map(link).collect::<Vec<Json>>(),
    });
    if let Some(children) = &section.children {
        value["children"] = children_value(children);
    }
    value
}

/// The frontmatter fields as a JSON object; a list is an array of strings.
fn fields(list: &[(String, Value)]) -> Json {
    let mut map = Map::new();
    for (key, value) in list {
        let item = match value {
            Value::Str(text) => Json::String(text.clone()),
            Value::List(items) => json!(items),
        };
        map.insert(key.clone(), item);
    }
    Json::Object(map)
}

/// The children object: the total and the shown items, each with its path, its kind and its one-liner.
fn children_value(children: &Listing<Child>) -> Json {
    let items: Vec<Json> = children.items.iter().map(child).collect();
    json!({"total": children.total, "items": items})
}

/// One child: `kind` is `dir` or `file`.
fn child(item: &Child) -> Json {
    let kind = match item.kind {
        Kind::Dir => "dir",
        Kind::File => "file",
    };
    json!({"path": item.rel, "kind": kind, "title": item.title})
}

/// The cited-by object: the total and the shown refs.
fn cited_by(list: &Listing<Ref>) -> Json {
    let refs: Vec<Json> = list.items.iter().map(reference).collect();
    json!({"total": list.total, "refs": refs})
}

/// One ref in a cited-by list.
fn reference(item: &Ref) -> Json {
    json!({"ref": item.label, "title": item.title})
}

/// One citation: its target, the target's one-liner and whether it resolves.
fn cite(item: &Cite) -> Json {
    json!({"ref": item.reference, "title": item.title, "resolved": item.resolved})
}

/// One README of a chain.
fn link(item: &Link) -> Json {
    json!({"path": item.path, "text": item.text})
}

/// One error: its ref, its code and its message, with the candidates of an ambiguous ref.
fn miss(item: &Miss) -> Json {
    let mut value = json!({"arg": item.arg, "code": item.code.as_str(), "message": item.message});
    if !item.candidates.is_empty() {
        value["candidates"] = json!(item.candidates);
    }
    value
}
