use crate::conversation::{ConversationMode, PersonalityProfile};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    pub theme_mode: ThemeMode,
    pub default_personality: PersonalityProfile,
    pub default_conversation_mode: ConversationMode,
    pub default_autonomy_level: crate::agency::AutonomyLevel,
    pub sidebar_collapsed: bool,
    pub last_active_view: String,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            default_personality: PersonalityProfile::Balanced,
            default_conversation_mode: ConversationMode::Standard,
            default_autonomy_level: crate::agency::AutonomyLevel::Standard,
            sidebar_collapsed: false,
            last_active_view: "Home".to_string(),
        }
    }
}

pub fn settings_path(root: &Path) -> PathBuf {
    root.join("data").join("config").join("gui_settings.json")
}

pub fn load_or_default(root: &Path) -> Result<GuiSettings> {
    let path = settings_path(root);
    if !path.exists() {
        let settings = GuiSettings::default();
        save(root, &settings)?;
        return Ok(settings);
    }
    match fs::read_to_string(&path)
        .with_context(|| format!("reading GUI settings at {}", path.display()))
        .and_then(|text| serde_json::from_str::<GuiSettings>(&text).context("parsing GUI settings"))
    {
        Ok(settings) => Ok(settings),
        Err(_) => {
            let archive = path.with_extension(format!(
                "corrupt-{}.json",
                chrono::Utc::now().format("%Y%m%d%H%M%S")
            ));
            let _ = fs::rename(&path, archive);
            let settings = GuiSettings::default();
            save(root, &settings)?;
            Ok(settings)
        }
    }
}

pub fn save(root: &Path, settings: &GuiSettings) -> Result<()> {
    let path = settings_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating GUI config dir {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(settings)?;
    fs::write(&path, text).with_context(|| format!("writing GUI settings at {}", path.display()))
}
