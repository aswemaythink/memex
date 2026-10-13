//! Shared runner code for the conformance suite and its self-tests: the case.json schema, and the modules that check, discover, run and bless cases.
//! Cases: all (runner)

pub mod bless;
pub mod conform;
pub mod discover;
pub mod env;
pub mod sandbox;
pub mod transcript;

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// A case's case.json (PLAN.md §2). Unknown keys are an error.
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
