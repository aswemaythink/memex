//! Shared runner code for the conformance suite and its self-tests.
//! It holds the case.json schema, and the modules that check, discover, run and bless cases.
//! Cases: all (runner)

pub mod bless;
pub mod conform;
pub mod discover;
pub mod env;
pub mod fill;
pub mod perms;
pub mod sandbox;
pub mod transcript;

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// A case's case.json. Unknown keys are an error.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSpec {
    /// `init` or `commit`: how repo/ is made from the fixture.
    #[serde(default)]
    pub git: Option<GitMode>,
    #[serde(default)]
    pub setup: Vec<SetupStep>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub snapshot: Vec<String>,
    #[serde(default)]
    pub memex_on_path: bool,
    /// Absent means every OS.
    #[serde(default)]
    pub platforms: Option<Vec<Platform>>,
    /// Permission changes made after setup and before the steps. Unix only.
    #[serde(default)]
    pub chmod: Vec<ChmodStep>,
    /// Files made at test time at their real lengths, instead of committed (see fill.rs).
    #[serde(default)]
    pub fill: Vec<FillStep>,
}

/// One `chmod` entry: an octal mode for a sandbox path, such as `{"path": "repo/x", "mode": "000"}`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChmodStep {
    /// Relative to the sandbox, such as repo/x.
    pub path: String,
    /// Octal digits, such as `000` or `555`.
    pub mode: String,
}

/// One `fill` entry: a file of NUL bytes that the runner makes in the sandbox, such as
/// `{"path": "repo/.memex/logs/impl/20261012-120000-big.md", "bytes": 10485000}`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FillStep {
    /// Relative to the sandbox, with real names (`.memex`, not `dot_memex`), as chmod paths are.
    pub path: String,
    /// The file's length in bytes.
    pub bytes: u64,
}

/// How the fixture becomes repo/.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitMode {
    Init,
    Commit,
}

/// An OS a case can run on. `unix` covers every OS that is not Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Unix,
    Windows,
}

/// A setup command: an argv whose first element is git or memex.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupStep {
    pub run: Vec<String>,
}

/// One memex invocation. Its output is recorded in the transcript.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub args: Vec<String>,
    #[serde(default)]
    pub stdin: Option<String>,
    /// A file name inside the case folder.
    #[serde(default)]
    pub stdin_file: Option<String>,
    /// Relative to the sandbox; repo/ when absent.
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl CaseSpec {
    /// True when the case changes permissions, which only unix can do.
    pub fn uses_chmod(&self) -> bool {
        !self.chmod.is_empty()
    }

    /// True when the case runs on the OS this test binary was built for.
    pub fn runs_on_this_os(&self) -> bool {
        let here = if cfg!(windows) {
            Platform::Windows
        } else {
            Platform::Unix
        };
        match &self.platforms {
            None => true,
            Some(list) => list.contains(&here),
        }
    }
}

/// Reads and parses a case.json. The error is serde's message, on one line.
pub fn load_spec(path: &Path) -> Result<CaseSpec, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read it ({:?})", e.kind()))?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
