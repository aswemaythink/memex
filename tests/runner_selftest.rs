//! Self-tests for the conformity rules: each broken tree under tests/runner-selftest trips exactly one rule, and each message is asserted in full.
//! Cases: all (runner)

#[allow(dead_code)] // this binary uses only conform::check; conformance.rs uses the rest
mod harness;

use std::path::{Path, PathBuf};

use harness::conform::check;

fn tree(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("runner-selftest")
        .join(name)
}

#[test]
fn ok_tree_passes_every_rule() {
    assert_eq!(check(&tree("ok")), Vec::<String>::new());
}

#[test]
fn rule1_stray_entry() {
    assert_eq!(
        check(&tree("rule1-stray-entry")),
        ["rule1: stray entry at stray.txt: only README.md and family folders belong at this level; move it into a family folder or delete it"]
    );
}

#[test]
fn rule2_bad_id() {
    assert_eq!(
        check(&tree("rule2-bad-id")),
        ["rule2: malformed entry on line 7 (\"- `CLI-version-02` lowercase id, rejected.\") at cli/CASES.md: write each entry as \"- `PREFIX-TOPIC-NN` claim\", with capitals and two digits, e.g. CLI-VERSION-01"]
    );
}

#[test]
fn rule2_topic_mismatch() {
    assert_eq!(
        check(&tree("rule2-topic-mismatch")),
        ["rule2: entry CLI-VERSION-01 on line 5 sits under \"## HELP\" but its topic is VERSION at cli/CASES.md: move it under \"## VERSION\" or fix the id"]
    );
}

#[test]
fn rule3_missing_dir() {
    assert_eq!(
        check(&tree("rule3-missing-dir")),
        ["rule3: entry CLI-HELP-01 has no case dir at cli/CLI-HELP-01: create the folder with case.json and expect.txt, or mark the claim (planned)"]
    );
}

#[test]
fn rule3_planned_has_dir() {
    assert_eq!(
        check(&tree("rule3-planned-has-dir")),
        ["rule3: planned entry CLI-HELP-01 has a case dir at cli/CLI-HELP-01: delete the folder, or remove (planned) from the claim"]
    );
}

#[test]
fn rule4_stray_file() {
    assert_eq!(
        check(&tree("rule4-stray-file")),
        ["rule4: stray entry at cli/CLI-VERSION-01/junk: a case folder holds only case.json, expect.txt, fixture/ and named stdin files; delete it, or name it as a stdin_file"]
    );
}

#[test]
fn rule4_bad_json() {
    assert_eq!(
        check(&tree("rule4-bad-json")),
        ["rule4: case.json does not parse against the schema (EOF while parsing an object at line 1 column 1) at cli/CLI-VERSION-01/case.json: fix it; the schema is in tests/cases/README.md"]
    );
}

#[test]
fn rule5_placeholder() {
    assert_eq!(
        check(&tree("rule5-placeholder")),
        ["rule5: placeholder \"$HOME\" on line 4 at cli/CLI-VERSION-01/expect.txt: goldens may use only $SANDBOX and $VERSION; change the case so its output has no other $NAME token"]
    );
}

#[test]
fn rule6_empty() {
    assert_eq!(
        check(&tree("rule6-empty")),
        ["rule6: no case exists at .: add a family folder with at least one case folder; see tests/cases/README.md"]
    );
}
