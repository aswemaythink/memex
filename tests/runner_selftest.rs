//! Self-tests for the conformity rules and the runner's `fill` key.
//! Each broken tree under tests/runner-selftest trips exactly one rule, and each message is asserted in full.
//! Cases: all (runner)

#[allow(dead_code)]
// this binary uses only conform::check, the chmod parser and the fill maker; conformance.rs uses the rest
mod harness;

use std::path::{Path, PathBuf};

use harness::conform::check;
use harness::fill::create_files;
use harness::perms::parse_mode;
use harness::FillStep;

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
fn rule2_prefix_clash() {
    assert_eq!(
        check(&tree("rule2-prefix-clash")),
        [
            "rule2: prefix CLI is also used by family cli at other/CASES.md: give each family its own prefix",
        ]
    );
}

#[test]
fn rule2_duplicate_id() {
    assert_eq!(
        check(&tree("rule2-duplicate-id")),
        [
            "rule2: id CLI-VERSION-02 is declared again on line 8 (first on line 7 of cli) at cli/CASES.md: ids are unique repo-wide; delete the duplicate entry",
        ]
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
fn rule4_setup() {
    assert_eq!(
        check(&tree("rule4-setup")),
        [
            "rule4: setup step 1 has an empty run list at cli/CLI-VERSION-01/case.json: give it an argv that starts with git or memex",
            "rule4: setup step 2 runs \"sh\" at cli/CLI-VERSION-01/case.json: setup may run only git or memex",
        ]
    );
}

#[test]
fn rule4_stdin() {
    assert_eq!(
        check(&tree("rule4-stdin")),
        [
            "rule4: step 1 sets both stdin and stdin_file at cli/CLI-VERSION-01/case.json: keep one of them",
            "rule4: step 2 names stdin_file \"../in.txt\", which is not a plain file name at cli/CLI-VERSION-01/case.json: name a file directly inside the case folder",
            "rule4: stdin_file in.txt is missing at cli/CLI-VERSION-01/in.txt: add the file, or fix the name in case.json",
        ]
    );
}

#[test]
fn rule4_path_escape() {
    assert_eq!(
        check(&tree("rule4-path-escape")),
        [
            "rule4: step 1 cwd \"../outside\" is not a path inside the sandbox at cli/CLI-VERSION-01/case.json: write it relative to the sandbox, such as repo/sub, with no .., no leading / and no drive",
            "rule4: snapshot \"/etc\" is not a path inside the sandbox at cli/CLI-VERSION-01/case.json: write it relative to the sandbox, such as repo/sub, with no .., no leading / and no drive",
            "rule4: snapshot \"C:/x\" is not a path inside the sandbox at cli/CLI-VERSION-01/case.json: write it relative to the sandbox, such as repo/sub, with no .., no leading / and no drive",
            "rule4: snapshot \"\" is not a path inside the sandbox at cli/CLI-VERSION-01/case.json: write it relative to the sandbox, such as repo/sub, with no .., no leading / and no drive",
        ]
    );
}

#[test]
fn rule4_chmod() {
    assert_eq!(
        check(&tree("rule4-chmod")),
        ["rule4: chmod needs platforms [\"unix\"] at cli/CLI-VERSION-01/case.json: add \"platforms\": [\"unix\"], since modes exist only on unix"]
    );
}

#[test]
fn chmod_modes_are_one_to_four_octal_digits() {
    for mode in ["0", "000", "555", "7777"] {
        assert!(parse_mode(mode).is_some(), "{mode} is an octal mode");
    }
    assert_eq!(parse_mode("555"), Some(0o555));
    for mode in ["", "8", "0o755", "00000", "rwx", "-1"] {
        assert_eq!(parse_mode(mode), None, "{mode:?} is not an octal mode");
    }
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

#[test]
fn fill_paths_must_stay_inside_the_sandbox() {
    let root = tempfile::tempdir().expect("a temp folder");
    let fill_at = |path: &str| {
        [FillStep {
            path: path.to_string(),
            bytes: 1,
        }]
    };
    assert_eq!(
        create_files(root.path(), &fill_at("../x.md")),
        Err("fill path \"../x.md\" is not a path inside the sandbox: write it relative to the sandbox, such as repo/.memex/logs/impl/x.md, with / between names, no .., no leading / and no drive".to_string())
    );
    for path in [
        "/etc/x.md",
        "repo/../../x.md",
        "C:/x.md",
        "repo\\x.md",
        "repo/x/",
        "",
    ] {
        let message = create_files(root.path(), &fill_at(path))
            .expect_err("a path outside the sandbox is refused");
        assert!(
            message.contains("is not a path inside the sandbox"),
            "{path:?} gives {message}"
        );
    }
}

#[test]
fn fill_makes_nul_bytes_at_the_real_length_in_new_folders() {
    let root = tempfile::tempdir().expect("a temp folder");
    let fills = [FillStep {
        path: "repo/.memex/logs/impl/big.md".to_string(),
        bytes: 1_048_576,
    }];
    create_files(root.path(), &fills).expect("the fill file is made");
    let made = std::fs::read(root.path().join("repo/.memex/logs/impl/big.md"))
        .expect("the fill file can be read");
    assert_eq!(made.len(), 1_048_576);
    assert!(
        made.iter().all(|&byte| byte == 0),
        "a fill file holds only NUL bytes"
    );
}

#[test]
fn fill_never_replaces_an_existing_file() {
    let root = tempfile::tempdir().expect("a temp folder");
    std::fs::create_dir_all(root.path().join("repo")).expect("repo/ is made");
    std::fs::write(root.path().join("repo/x.md"), b"fixture").expect("the fixture file is written");
    let fills = [FillStep {
        path: "repo/x.md".to_string(),
        bytes: 4,
    }];
    assert_eq!(
        create_files(root.path(), &fills),
        Err("fill path \"repo/x.md\" already exists in the sandbox: the fixture or an earlier fill entry holds it; delete one of them".to_string())
    );
    let kept =
        std::fs::read(root.path().join("repo/x.md")).expect("the fixture file is still there");
    assert_eq!(kept, b"fixture");
}
