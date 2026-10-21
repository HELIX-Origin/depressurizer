use serde::{Deserialize, Serialize};

/// Represents an individual game in the user's Steam library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
    pub categories: Vec<String>,
    pub is_hidden: bool,
    pub is_favorite: bool,
    pub playtime_minutes: u32,
    pub last_played: i64,
    pub icon_hash: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<u32>,
    pub review_score: Option<u32>,
    pub platforms: Vec<String>,
    pub vr_support: Option<String>,
    pub hltb_main_story: Option<f32>,
    pub hltb_main_extras: Option<f32>,
    pub hltb_completionist: Option<f32>,
}

/// Represents a detected local Steam account profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamAccount {
    pub steam_id3: String,
    pub steam_id64: Option<u64>,
    pub account_name: String,
    pub persona_name: String,
    pub most_recent: bool,
}

/// HowLongToBeat playtime data (hours).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HltbTimes {
    pub main_story: Option<f32>,
    pub main_extras: Option<f32>,
    pub completionist: Option<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamCategory {
    pub name: String,
    pub count: usize,
    pub is_favorite: bool,
}

/// Supported AutoCat filter types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutoCatFilterType {
    Genre,
    Tags,
    DevPub,
    Year,
    Playtime,
    Score,
    Platform,
    VrSupport,
    Manual,
    Hltb,
    Regex,
}

/// Individual rule within an AutoCat configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCatRule {
    pub id: String,
    pub name: String,
    pub filter_type: AutoCatFilterType,
    pub target_category: String,
    pub params: serde_json::Value,
    pub enabled: bool,
}

/// Overall AutoCat execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCatConfig {
    pub rules: Vec<AutoCatRule>,
    pub clear_existing: bool,
    pub prefix: Option<String>,
}

/// Progress frame streamed to the frontend for each game during an AutoCat run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCatProgress {
    pub current: usize,
    pub total: usize,
    pub done: bool,
    pub game: SteamGame,
}

/// Depressurizer application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub steam_path: Option<String>,
    pub api_key: Option<String>,
    pub active_user: Option<String>,
    pub backup_on_save: bool,
    pub dense_view: bool,
}

impl AppSettings {
    /// Loads settings overrides from an optional `.env` file or environment variables if present.
    /// This enables seamless developer and automated testing without re-inputting keys.
    pub fn load_from_env_if_present(&mut self) {
        // Read from process environment first
        if let Ok(key) = std::env::var("STEAM_API_KEY")
            .or_else(|_| std::env::var("STEAM_WEB_API_KEY"))
            .or_else(|_| std::env::var("API_KEY"))
        {
            if !key.trim().is_empty() {
                self.api_key = Some(key.trim().to_string());
            }
        }

        if let Ok(user) = std::env::var("STEAMID64")
            .or_else(|_| std::env::var("STEAM_ID64"))
            .or_else(|_| std::env::var("STEAM_USER"))
            .or_else(|_| std::env::var("ACTIVE_USER"))
            .or_else(|_| std::env::var("STEAM_ID3"))
        {
            if !user.trim().is_empty() {
                self.active_user = Some(user.trim().to_string());
            }
        }

        if let Ok(path) = std::env::var("STEAM_PATH") {
            if !path.trim().is_empty() {
                self.steam_path = Some(path.trim().to_string());
            }
        }

        // Check common .env file candidate paths (workspace root, project root)
        let candidates = [
            std::path::PathBuf::from(".env"),
            std::path::PathBuf::from("../.env"),
            std::path::PathBuf::from("../../.env"),
        ];

        for path in &candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        if let Some((k, v)) = trimmed.split_once('=') {
                            let key = k.trim();
                            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                            if val.is_empty() {
                                continue;
                            }
                            match key {
                                "STEAM_API_KEY" | "STEAM_WEB_API_KEY" | "API_KEY" => {
                                    if self.api_key.as_deref().unwrap_or("").trim().is_empty() {
                                        self.api_key = Some(val.to_string());
                                    }
                                }
                                "STEAMID64" | "STEAM_ID64" | "STEAM_USER" | "ACTIVE_USER" | "STEAM_ID3" => {
                                    if self.active_user.as_deref().unwrap_or("").trim().is_empty() {
                                        self.active_user = Some(val.to_string());
                                    }
                                }
                                "STEAM_PATH" => {
                                    if self.steam_path.as_deref().unwrap_or("").trim().is_empty() {
                                        self.steam_path = Some(val.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let detected_path =
            crate::steam::paths::detect_steam_path().map(|p| p.to_string_lossy().to_string());

        let mut settings = Self {
            steam_path: detected_path,
            api_key: None,
            active_user: None,
            backup_on_save: true,
            dense_view: false,
        };

        // Auto-read from local .env if present for testing/dev environments
        settings.load_from_env_if_present();
        settings
    }
}
