#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiTheme {
    pub name: String,
    pub background: String,
    pub panel: String,
    pub accent: String,
    pub text: String,
}

pub fn default_theme() -> GuiTheme {
    GuiTheme {
        name: "Onyx Dark".to_string(),
        background: "#090b10".to_string(),
        panel: "#121722".to_string(),
        accent: "#6ee7f9".to_string(),
        text: "#e6edf6".to_string(),
    }
}
