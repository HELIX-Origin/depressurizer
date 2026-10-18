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

impl Default for AppSettings {
    fn default() -> Self {
        // SECURITY: the Steam Web API key (and SteamID64) are NEVER hardcoded, defaulted,
        // or read from the environment / a `.env` file. The end user supplies them in Settings.
        let detected_path =
            crate::steam::paths::detect_steam_path().map(|p| p.to_string_lossy().to_string());

        Self {
            steam_path: detected_path,
            api_key: None,
            active_user: None,
            backup_on_save: true,
            dense_view: false,
        }
    }
}
