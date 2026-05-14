#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiState {
    pub active_view: String,
    pub views: Vec<String>,
    pub safety_badge: String,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            active_view: "ChatView".to_string(),
            views: vec![
                "ChatView",
                "AutonomyView",
                "TaskBoardView",
                "ArtifactBrowserView",
                "MemoryBrowserView",
                "SessionTimelineView",
                "CreativeStudioView",
                "SafetyPanelView",
                "SettingsView",
                "SystemStatusView",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            safety_badge: "Sandboxed / allowlisted / network disabled by default".to_string(),
        }
    }
}

impl Default for GuiState {
    fn default() -> Self {
        Self::new()
    }
}
