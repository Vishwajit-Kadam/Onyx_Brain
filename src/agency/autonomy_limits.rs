use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyLimits {
    pub max_session_minutes: u64,
    pub max_tasks: usize,
    pub max_phases: usize,
    pub max_retries_per_task: usize,
    pub max_tool_actions: usize,
    pub max_generated_file_bytes: usize,
    pub max_artifacts: usize,
    pub max_context_files_read: usize,
    pub network_allowed: bool,
    pub unrestricted_shell_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyPolicyReport {
    pub limits: AutonomyLimits,
    pub summary: String,
    pub safety_rules: Vec<String>,
}

impl Default for AutonomyLimits {
    fn default() -> Self {
        Self {
            max_session_minutes: 30,
            max_tasks: 40,
            max_phases: 8,
            max_retries_per_task: 2,
            max_tool_actions: 80,
            max_generated_file_bytes: 1_048_576,
            max_artifacts: 20,
            max_context_files_read: 40,
            network_allowed: false,
            unrestricted_shell_allowed: false,
        }
    }
}

pub fn autonomy_policy() -> AutonomyPolicyReport {
    AutonomyPolicyReport {
        limits: AutonomyLimits::default(),
        summary:
            "bounded autonomy inside sandbox, allowlisted commands, finite task/retry/time limits"
                .to_string(),
        safety_rules: vec![
            "no background execution".to_string(),
            "no network by default".to_string(),
            "no unrestricted shell".to_string(),
            "no writes outside sandbox".to_string(),
            "no AGI or consciousness claims".to_string(),
        ],
    }
}
