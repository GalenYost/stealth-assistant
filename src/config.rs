use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub selected_provider: String, // "OpenAI", "DeepSeek", "Gemini", "Claude"
    pub openai_key: String,
    pub openai_model: String,
    pub deepseek_key: String,
    pub deepseek_model: String,
    pub gemini_key: String,
    pub gemini_model: String,
    pub claude_key: String,
    pub claude_model: String,
    
    pub system_prompt: String,
    pub opacity: f32,
    pub is_click_through: bool,
    pub enable_stealth_on_launch: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_provider: "Gemini".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-4o".to_string(),
            deepseek_key: String::new(),
            deepseek_model: "deepseek-chat".to_string(),
            gemini_key: String::new(),
            gemini_model: "gemini-1.5-flash".to_string(),
            claude_key: String::new(),
            claude_model: "claude-3-5-sonnet-20240620".to_string(),
            system_prompt: "You are an expert AI assistant. Keep responses as short as possible while still being complete and informative. No filler, no fluff, no unnecessary greetings. Always give technically correct, precise, and genuinely intelligent answers, not generic or surface-level responses. Think before answering. Never use bullet points, markdown, or any formatting. Respond only in plain conversational text, as if speaking directly to someone in an interview.".to_string(),
            opacity: 0.90,
            is_click_through: false,
            enable_stealth_on_launch: true,
        }
    }
}

impl AppConfig {
    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn load() -> Self {
        let path = config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }

        let legacy = PathBuf::from("stealth_config.json");
        if legacy.exists() && legacy != path {
            if let Ok(data) = std::fs::read_to_string(&legacy) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    let _ = cfg.save();
                    return cfg;
                }
            }
        }

        Self::default()
    }
}

fn config_path() -> PathBuf {
    config_dir()
        .map(|dir| dir.join("stealth-assistant").join("stealth_config.json"))
        .unwrap_or_else(|| PathBuf::from("stealth_config.json"))
}

#[cfg(target_os = "windows")]
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(target_os = "linux")]
fn config_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}
