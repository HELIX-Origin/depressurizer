use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::steam::models::{AppSettings, AutoCatRule, SteamGame};

static DB_MUTEX: Mutex<()> = Mutex::new(());

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("depressurizer")
}

pub fn db_path() -> PathBuf {
    config_dir().join("depressurizer.db")
}

fn open_connection() -> Result<Connection, String> {
    let dir = config_dir();
    if !dir.is_dir() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }
    let conn =
        Connection::open(db_path()).map_err(|e| format!("Failed to open SQLite database: {e}"))?;

    // Performance and integrity pragmas
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(|e| format!("Failed to configure SQLite pragmas: {e}"))?;

    Ok(conn)
}

pub fn init_db() -> Result<(), String> {
    let _lock = DB_MUTEX.lock().unwrap();
    let conn = open_connection()?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS owned_games (
            app_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            playtime_minutes INTEGER NOT NULL DEFAULT 0,
            icon_hash TEXT,
            categories_json TEXT NOT NULL DEFAULT '[]',
            is_hidden INTEGER NOT NULL DEFAULT 0,
            last_played INTEGER NOT NULL DEFAULT 0,
            genres_json TEXT NOT NULL DEFAULT '[]',
            tags_json TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS autocat_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            filter_type TEXT NOT NULL,
            target_category TEXT NOT NULL,
            params_json TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        );",
    )
    .map_err(|e| format!("Failed to initialize database tables: {e}"))?;

    // Add HLTB columns for existing databases (ignore "duplicate column" errors).
    let _ = conn.execute(
        "ALTER TABLE owned_games ADD COLUMN hltb_main_story REAL",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE owned_games ADD COLUMN hltb_main_extras REAL",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE owned_games ADD COLUMN hltb_completionist REAL",
        [],
    );

    Ok(())
}

pub fn load_settings() -> AppSettings {
    let _lock = DB_MUTEX.lock().unwrap();
    let Ok(conn) = open_connection() else {
        return AppSettings::default();
    };

    let mut steam_path = None;
    let mut api_key = None;
    let mut active_user = None;
    let mut backup_on_save = true;
    let mut dense_view = false;

    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM settings") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                match row.0.as_str() {
                    "steam_path" => steam_path = Some(row.1),
                    "api_key" => api_key = Some(row.1),
                    "active_user" => active_user = Some(row.1),
                    "backup_on_save" => backup_on_save = row.1 == "1" || row.1 == "true",
                    "dense_view" => dense_view = row.1 == "1" || row.1 == "true",
                    _ => {}
                }
            }
        }
    }

    if api_key.is_none() || api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
        let default_set = AppSettings::default();
        api_key = default_set.api_key;
        if steam_path.is_none() {
            steam_path = default_set.steam_path;
        }
    }

    AppSettings {
        steam_path,
        api_key,
        active_user,
        backup_on_save,
        dense_view,
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let _lock = DB_MUTEX.lock().unwrap();
    let mut conn = open_connection()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction error: {e}"))?;

    let pairs = [
        (
            "steam_path",
            settings.steam_path.clone().unwrap_or_default(),
        ),
        ("api_key", settings.api_key.clone().unwrap_or_default()),
        (
            "active_user",
            settings.active_user.clone().unwrap_or_default(),
        ),
        (
            "backup_on_save",
            if settings.backup_on_save {
                "1".to_string()
            } else {
                "0".to_string()
            },
        ),
        (
            "dense_view",
            if settings.dense_view {
                "1".to_string()
            } else {
                "0".to_string()
            },
        ),
    ];

    for (k, v) in pairs {
        tx.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )
        .map_err(|e| format!("Failed to save setting {k}: {e}"))?;
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit settings: {e}"))?;
    Ok(())
}

pub fn load_cached_games() -> Vec<SteamGame> {
    let _lock = DB_MUTEX.lock().unwrap();
    let Ok(conn) = open_connection() else {
        return Vec::new();
    };

    let mut stmt = match conn.prepare(
        "SELECT app_id, name, playtime_minutes, icon_hash, categories_json, is_hidden, last_played, genres_json, tags_json,
                hltb_main_story, hltb_main_extras, hltb_completionist
         FROM owned_games ORDER BY name COLLATE NOCASE ASC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        let app_id: u32 = row.get(0)?;
        let name: String = row.get(1)?;
        let playtime_minutes: u32 = row.get(2)?;
        let icon_hash: Option<String> = row.get(3)?;
        let cats_raw: String = row.get(4)?;
        let is_hidden_num: i32 = row.get(5)?;
        let last_played: i64 = row.get(6)?;
        let genres_raw: String = row.get(7)?;
        let tags_raw: String = row.get(8)?;
        let hltb_main_story: Option<f64> = row.get(9).ok();
        let hltb_main_extras: Option<f64> = row.get(10).ok();
        let hltb_completionist: Option<f64> = row.get(11).ok();

        let categories: Vec<String> = serde_json::from_str(&cats_raw).unwrap_or_default();
        let genres: Vec<String> = serde_json::from_str(&genres_raw).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();

        Ok(SteamGame {
            app_id,
            name,
            categories,
            is_hidden: is_hidden_num != 0,
            is_favorite: false,
            playtime_minutes,
            last_played,
            icon_hash,
            genres,
            tags,
            developer: None,
            publisher: None,
            release_year: None,
            review_score: None,
            platforms: vec!["Windows".to_string()],
            vr_support: None,
            hltb_main_story: hltb_main_story.map(|v| v as f32),
            hltb_main_extras: hltb_main_extras.map(|v| v as f32),
            hltb_completionist: hltb_completionist.map(|v| v as f32),
        })
    });

    match rows {
        Ok(r) => r.flatten().collect(),
        Err(_) => Vec::new(),
    }
}

pub fn save_cached_games(games: &[SteamGame]) -> Result<(), String> {
    let _lock = DB_MUTEX.lock().unwrap();
    let mut conn = open_connection()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction error: {e}"))?;

    for g in games {
        let cats_json = serde_json::to_string(&g.categories).unwrap_or_else(|_| "[]".to_string());
        let genres_json = serde_json::to_string(&g.genres).unwrap_or_else(|_| "[]".to_string());
        let tags_json = serde_json::to_string(&g.tags).unwrap_or_else(|_| "[]".to_string());

        tx.execute(
            "INSERT INTO owned_games (
                app_id, name, playtime_minutes, icon_hash, categories_json, is_hidden, last_played, genres_json, tags_json,
                hltb_main_story, hltb_main_extras, hltb_completionist
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(app_id) DO UPDATE SET
                name = excluded.name,
                playtime_minutes = excluded.playtime_minutes,
                icon_hash = excluded.icon_hash,
                categories_json = excluded.categories_json,
                is_hidden = excluded.is_hidden,
                last_played = excluded.last_played,
                genres_json = excluded.genres_json,
                tags_json = excluded.tags_json,
                hltb_main_story = excluded.hltb_main_story,
                hltb_main_extras = excluded.hltb_main_extras,
                hltb_completionist = excluded.hltb_completionist",
            params![
                g.app_id,
                g.name,
                g.playtime_minutes,
                g.icon_hash,
                cats_json,
                if g.is_hidden { 1 } else { 0 },
                g.last_played,
                genres_json,
                tags_json,
                g.hltb_main_story.map(|v| v as f64),
                g.hltb_main_extras.map(|v| v as f64),
                g.hltb_completionist.map(|v| v as f64),
            ],
        ).map_err(|e| format!("Failed to insert game {}: {e}", g.app_id))?;
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit games cache: {e}"))?;
    Ok(())
}

pub fn save_hltb_times(app_id: u32, main: Option<f32>, extras: Option<f32>, completionist: Option<f32>) -> Result<(), String> {
    let _lock = DB_MUTEX.lock().unwrap();
    let conn = open_connection()?;
    conn.execute(
        "UPDATE owned_games SET hltb_main_story = ?1, hltb_main_extras = ?2, hltb_completionist = ?3 WHERE app_id = ?4",
        rusqlite::params![main.map(|v| v as f64), extras.map(|v| v as f64), completionist.map(|v| v as f64), app_id],
    ).map_err(|e| format!("Failed to save HLTB times: {e}"))?;
    Ok(())
}

pub fn load_autocat_rules() -> Vec<AutoCatRule> {
    let _lock = DB_MUTEX.lock().unwrap();
    let Ok(conn) = open_connection() else {
        return default_rules();
    };

    let mut stmt = match conn.prepare(
        "SELECT id, name, filter_type, target_category, params_json, enabled
         FROM autocat_rules ORDER BY sort_order ASC",
    ) {
        Ok(s) => s,
        Err(_) => return default_rules(),
    };

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let filter_type_str: String = row.get(2)?;
        let target_category: String = row.get(3)?;
        let params_json: String = row.get(4)?;
        let enabled: i32 = row.get(5)?;

        let filter_type = match filter_type_str.as_str() {
            "Genre" => crate::steam::models::AutoCatFilterType::Genre,
            "Tags" => crate::steam::models::AutoCatFilterType::Tags,
            "Developer" | "Publisher" | "DevPub" => crate::steam::models::AutoCatFilterType::DevPub,
            "Year" => crate::steam::models::AutoCatFilterType::Year,
            "Score" => crate::steam::models::AutoCatFilterType::Score,
            "Playtime" => crate::steam::models::AutoCatFilterType::Playtime,
            "Platform" => crate::steam::models::AutoCatFilterType::Platform,
            "VrSupport" => crate::steam::models::AutoCatFilterType::VrSupport,
            "Hltb" => crate::steam::models::AutoCatFilterType::Hltb,
            _ => crate::steam::models::AutoCatFilterType::Manual,
        };

        let params = serde_json::from_str(&params_json).unwrap_or_default();

        Ok(AutoCatRule {
            id,
            name,
            filter_type,
            target_category,
            params,
            enabled: enabled != 0,
        })
    });

    let rules: Vec<AutoCatRule> = rows.map(|r| r.flatten().collect()).unwrap_or_default();
    if rules.is_empty() {
        default_rules()
    } else {
        rules
    }
}

pub fn save_autocat_rules(rules: &[AutoCatRule]) -> Result<(), String> {
    let _lock = DB_MUTEX.lock().unwrap();
    let mut conn = open_connection()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction error: {e}"))?;

    tx.execute("DELETE FROM autocat_rules", [])
        .map_err(|e| format!("Clear rules error: {e}"))?;

    for (idx, r) in rules.iter().enumerate() {
        let ft_str = match r.filter_type {
            crate::steam::models::AutoCatFilterType::Genre => "Genre",
            crate::steam::models::AutoCatFilterType::Tags => "Tags",
            crate::steam::models::AutoCatFilterType::DevPub => "DevPub",
            crate::steam::models::AutoCatFilterType::Year => "Year",
            crate::steam::models::AutoCatFilterType::Score => "Score",
            crate::steam::models::AutoCatFilterType::Playtime => "Playtime",
            crate::steam::models::AutoCatFilterType::Platform => "Platform",
            crate::steam::models::AutoCatFilterType::VrSupport => "VrSupport",
            crate::steam::models::AutoCatFilterType::Manual => "Manual",
            crate::steam::models::AutoCatFilterType::Hltb => "Hltb",
        };
        let params_json = serde_json::to_string(&r.params).unwrap_or_else(|_| "{}".to_string());

        tx.execute(
            "INSERT INTO autocat_rules (id, name, filter_type, target_category, params_json, enabled, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![r.id, r.name, ft_str, r.target_category, params_json, if r.enabled { 1 } else { 0 }, idx as i32],
        ).map_err(|e| format!("Insert rule error: {e}"))?;
    }

    tx.commit()
        .map_err(|e| format!("Commit rules error: {e}"))?;
    Ok(())
}

fn default_rules() -> Vec<AutoCatRule> {
    vec![
        AutoCatRule {
            id: "1".to_string(),
            name: "RPGs".to_string(),
            filter_type: crate::steam::models::AutoCatFilterType::Genre,
            target_category: "RPG".to_string(),
            params: [("genre".to_string(), "RPG".to_string())]
                .into_iter()
                .collect(),
            enabled: true,
        },
        AutoCatRule {
            id: "2".to_string(),
            name: "Action".to_string(),
            filter_type: crate::steam::models::AutoCatFilterType::Genre,
            target_category: "Action".to_string(),
            params: [("genre".to_string(), "Action".to_string())]
                .into_iter()
                .collect(),
            enabled: true,
        },
        AutoCatRule {
            id: "3".to_string(),
            name: "Strategy".to_string(),
            filter_type: crate::steam::models::AutoCatFilterType::Genre,
            target_category: "Strategy".to_string(),
            params: [("genre".to_string(), "Strategy".to_string())]
                .into_iter()
                .collect(),
            enabled: true,
        },
        AutoCatRule {
            id: "4".to_string(),
            name: "Adventure".to_string(),
            filter_type: crate::steam::models::AutoCatFilterType::Genre,
            target_category: "Adventure".to_string(),
            params: [("genre".to_string(), "Adventure".to_string())]
                .into_iter()
                .collect(),
            enabled: true,
        },
    ]
}
