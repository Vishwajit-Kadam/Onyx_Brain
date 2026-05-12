use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    agency::{load_action_journal_index, load_project_registry},
    storage::{doctor, DiskStore},
    tools::FilesystemTool,
    ONYX_VERSION_NUMBER,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegressionCheckReport {
    pub checks_passed: u64,
    pub checks_failed: u64,
    pub status: String,
    pub failures: Vec<String>,
}

pub fn regression_check(store: &DiskStore) -> Result<RegressionCheckReport> {
    store.ensure_layout()?;
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();
    macro_rules! check {
        ($condition:expr, $message:expr) => {
            if $condition {
                passed += 1;
            } else {
                failed += 1;
                failures.push($message.to_string());
            }
        };
    }
    check!(store.paths.data.exists(), "data folder missing");
    check!(store.paths.sandbox.exists(), "sandbox folder missing");
    check!(ONYX_VERSION_NUMBER == "0.0.1", "version is not v0.0.1");
    let fs = FilesystemTool::new(&store.paths.sandbox)?;
    check!(
        fs.safe_path("../escape").is_err(),
        "sandbox traversal allowed"
    );
    check!(
        load_project_registry(store).is_ok(),
        "project registry failed to load"
    );
    check!(
        load_action_journal_index(store).is_ok(),
        "journal index failed to load"
    );
    let doctor = doctor(store, false)?;
    check!(doctor.critical == 0, "doctor has critical issues");
    Ok(RegressionCheckReport {
        checks_passed: passed,
        checks_failed: failed,
        status: if failed == 0 { "pass" } else { "fail" }.to_string(),
        failures,
    })
}
