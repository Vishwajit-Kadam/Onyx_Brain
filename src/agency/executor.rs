//! Agency executor — tracks individual execution steps with timing, retry logic,
//! and completion reporting.
//!
//! Used by the autonomous worker to record what each step did, how long it took,
//! and whether it succeeded or required retries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a single execution step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A tracked execution step within an autonomous workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub label: String,
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub retries: usize,
    pub max_retries: usize,
    pub error_message: Option<String>,
    pub output_summary: Option<String>,
    /// Legacy field for backward compatibility.
    pub completed: bool,
}

impl ExecutionStep {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: StepStatus::Pending,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            retries: 0,
            max_retries: 2,
            error_message: None,
            output_summary: None,
            completed: false,
        }
    }

    /// Mark the step as started.
    pub fn start(&mut self) {
        self.status = StepStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark the step as completed successfully.
    pub fn complete(&mut self, summary: impl Into<String>) {
        let now = Utc::now();
        self.status = StepStatus::Completed;
        self.completed_at = Some(now);
        self.completed = true;
        self.output_summary = Some(summary.into());
        if let Some(start) = self.started_at {
            self.duration_ms =
                Some(now.signed_duration_since(start).num_milliseconds().max(0) as u64);
        }
    }

    /// Mark the step as failed. Returns true if a retry is available.
    pub fn fail(&mut self, error: impl Into<String>) -> bool {
        self.error_message = Some(error.into());
        if self.retries < self.max_retries {
            self.retries += 1;
            self.status = StepStatus::Retrying;
            true
        } else {
            self.status = StepStatus::Failed;
            self.completed_at = Some(Utc::now());
            false
        }
    }

    /// Skip this step (e.g., precondition not met).
    pub fn skip(&mut self, reason: impl Into<String>) {
        self.status = StepStatus::Skipped;
        self.output_summary = Some(reason.into());
        self.completed_at = Some(Utc::now());
    }

    /// Whether this step is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped
        )
    }

    /// Whether this step succeeded.
    pub fn succeeded(&self) -> bool {
        self.status == StepStatus::Completed
    }

    /// Produce a one-line summary.
    pub fn summarize(&self) -> String {
        let duration = self
            .duration_ms
            .map(|ms| format!(" ({ms}ms)"))
            .unwrap_or_default();
        let retries = if self.retries > 0 {
            format!(" [retries: {}]", self.retries)
        } else {
            String::new()
        };
        format!("{}: {}{}{}", self.label, self.status, duration, retries)
    }
}

/// A batch of execution steps forming a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
}

impl ExecutionPlan {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, label: impl Into<String>) -> usize {
        let idx = self.steps.len();
        self.steps.push(ExecutionStep::new(label));
        idx
    }

    /// Count of completed steps.
    pub fn completed_count(&self) -> usize {
        self.steps.iter().filter(|s| s.succeeded()).count()
    }

    /// Count of failed steps.
    pub fn failed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count()
    }

    /// Overall progress as a fraction.
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let terminal = self.steps.iter().filter(|s| s.is_terminal()).count();
        terminal as f32 / self.steps.len() as f32
    }

    /// Whether all steps have completed (successfully or not).
    pub fn is_done(&self) -> bool {
        !self.steps.is_empty() && self.steps.iter().all(|s| s.is_terminal())
    }

    /// Whether any step failed.
    pub fn has_failures(&self) -> bool {
        self.steps.iter().any(|s| s.status == StepStatus::Failed)
    }

    /// Produce a multi-line execution report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Execution Plan: {}/{} complete, {} failed",
            self.completed_count(),
            self.steps.len(),
            self.failed_count()
        )];
        for step in &self.steps {
            lines.push(format!("  {}", step.summarize()));
        }
        lines.join("\n")
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_lifecycle() {
        let mut step = ExecutionStep::new("cargo check");
        assert_eq!(step.status, StepStatus::Pending);
        step.start();
        assert_eq!(step.status, StepStatus::Running);
        step.complete("all checks passed");
        assert!(step.succeeded());
        assert!(step.is_terminal());
        assert!(step.completed);
    }

    #[test]
    fn step_retry_on_failure() {
        let mut step = ExecutionStep::new("cargo test");
        step.max_retries = 2;
        step.start();
        assert!(step.fail("compilation error")); // retry 1
        assert_eq!(step.status, StepStatus::Retrying);
        assert!(step.fail("compilation error again")); // retry 2
        assert!(!step.fail("still failing")); // no more retries
        assert_eq!(step.status, StepStatus::Failed);
    }

    #[test]
    fn execution_plan_tracks_progress() {
        let mut plan = ExecutionPlan::new();
        plan.add_step("step 1");
        plan.add_step("step 2");
        plan.add_step("step 3");
        assert!((plan.progress() - 0.0).abs() < f32::EPSILON);

        plan.steps[0].start();
        plan.steps[0].complete("done");
        plan.steps[1].skip("not needed");
        assert!((plan.progress() - 2.0 / 3.0).abs() < 0.01);
        assert!(!plan.is_done());
    }

    #[test]
    fn plan_report_is_readable() {
        let mut plan = ExecutionPlan::new();
        plan.add_step("init");
        plan.add_step("build");
        plan.steps[0].start();
        plan.steps[0].complete("ok");
        let report = plan.report();
        assert!(report.contains("1/2 complete"));
        assert!(report.contains("init: Completed"));
    }
}
