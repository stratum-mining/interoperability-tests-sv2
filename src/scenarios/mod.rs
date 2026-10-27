pub mod common;
pub mod mining;

use std::{fmt, future::Future, pin::Pin};

use crate::endpoint::Endpoint;

/// Why a scenario did not pass.
#[derive(Debug)]
pub enum ScenarioError {
    /// The endpoint violated the scenario's assertion.
    Failed(String),
    /// A precondition the test client cannot provoke never occurred (see
    /// `COVERAGE.md`, Testability Tiers), so nothing was asserted. Never
    /// counted as a pass.
    NotExercised(String),
}

impl ScenarioError {
    pub fn not_exercised(reason: impl Into<String>) -> Self {
        Self::NotExercised(reason.into())
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(s) => f.write_str(s),
            Self::NotExercised(s) => write!(f, "not exercised: {s}"),
        }
    }
}

impl std::error::Error for ScenarioError {}

impl From<String> for ScenarioError {
    fn from(s: String) -> Self {
        Self::Failed(s)
    }
}

impl From<&str> for ScenarioError {
    fn from(s: &str) -> Self {
        Self::Failed(s.to_string())
    }
}

/// The outcome of one scenario run.
///
/// `Ok(Some(detail))` carries a human-readable success detail (e.g. negotiated
/// version/flags) for CLI reports.
#[derive(Debug)]
pub struct ScenarioReport {
    pub id: &'static str,
    pub result: ScenarioResult,
}

impl ScenarioReport {
    /// Whether the endpoint violated the scenario's assertion. A scenario
    /// that was not exercised is neither a pass nor a failure.
    pub fn failed(&self) -> bool {
        matches!(self.result, Err(ScenarioError::Failed(_)))
    }
}

/// The result of running a single scenario.
///
/// `Ok(None)` — passed with no extra detail.
/// `Ok(Some(detail))` — passed; the string is a human-readable summary (e.g.
/// negotiated version/flags) for CLI reports.
/// `Err(ScenarioError)` — failed or not exercised; the error carries the reason.
pub type ScenarioResult = Result<Option<String>, ScenarioError>;

/// A registered scenario: takes an [`Endpoint`] (consumed so it can be spawned
/// in its own task) and returns a boxed future resolving to
/// [`ScenarioResult`].
pub type ScenarioFn = fn(Endpoint) -> Pin<Box<dyn Future<Output = ScenarioResult> + Send>>;

// ---------------------------------------------------------------------------
// Suites — named groups of scenarios applicable to a given app type.
//
// Each suite composes entry groups from submodules.  The submodule
// structure mirrors the scenario catalog in `COVERAGE.md`:
//
//   common/    — Common Protocol Messages (§3.6)
//   mining/    — Mining Protocol Messages (§5.3)
//   general    — cross-cutting invariants (GEN-*)
//   extensions — Protocol Extensions (§3.4, X*-*)
//
// Modules appear when their first scenario lands (no empty scaffolding).
// ---------------------------------------------------------------------------

/// Mining Protocol Server suite: all Common + Mining Protocol scenarios.
pub fn mining_server() -> Vec<(&'static str, ScenarioFn)> {
    let mut v = common::setup_connection::entries();
    v.extend(mining::open_standard_mining_channel::entries());
    v.extend(mining::open_extended_mining_channel::entries());
    v.extend(mining::new_mining_job::entries());
    v.extend(mining::new_extended_mining_job::entries());
    v
}
