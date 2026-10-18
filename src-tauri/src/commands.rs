use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::steam::api::SteamApiClient;
use crate::steam::autocat::apply_autocat_to_game;
use crate::steam::models::{
    AppSettings, AutoCatConfig, AutoCatProgress, AutoCatRule, SteamAccount, SteamCategory,
    SteamGame,
};
use crate::steam::paths::{detect_steam_path, get_local_accounts};
use crate::steam::vdf::{
    load_user_categories, save_sharedconfig,
};
use crate::worker::BackgroundWorker;

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub cached_games: Arc<Mutex<Vec<SteamGame>>>,
    pub worker: BackgroundWorker,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().await;
    let stored = crate::steam::storage::load_settings();
    *settings = stored.clone();
    Ok(stored)
}

#[tauri::command]
pub async fn save_settings(
    new_settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;
    crate::steam::storage::save_settings(&new_settings)?;
    *settings = new_settings;
    Ok(())
}

#[tauri::command]
pub async fn detect_steam() -> Result<Option<String>, String> {
    let path = detect_steam_path();
    Ok(path.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<SteamAccount>, String> {
    let settings = state.settings.lock().await;
    let steam_path = settings
        .steam_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(detect_steam_path)
        .ok_or_else(|| "Steam directory not found".to_string())?;

    Ok(get_local_accounts(&steam_path))
}

/// Loads the user's complete library of ALL owned games (via Steam Web API if key is present,
/// merged with local manifests and VDF configs).
#[tauri::command]
pub async fn load_library(
    steam_id3: String,
    state: State<'_, AppState>,
) -> Result<Vec<SteamGame>, String> {
    let settings = state.settings.lock().await.clone();
    // Steam client path is optional: it is only needed to read/write library collections
    // (sharedconfig.vdf). Owned games come from the Steam Web API.
    let steam_path = settings
        .steam_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(detect_steam_path);

    let worker = state.worker.clone();
    worker
        .emit_progress(
            "load_library",
            10,
            100,
            "Scanning local Steam files...",
            false,
        )
        .await;

    // 2. Load existing categories & hidden flags from sharedconfig.vdf (only if a Steam
    //    client path is available; collections are a client-side concern).
    let (categories_map, hidden_map) = match &steam_path {
        Some(p) => load_user_categories(p, &steam_id3),
        None => (std::collections::HashMap::new(), std::collections::HashMap::new()),
    };

    worker
        .emit_progress(
            "load_library",
            35,
            100,
            "Querying owned games catalog...",
            false,
        )
        .await;

    // 3. Populate initially from persistent SQLite cache
    let mut library_map: HashMap<u32, SteamGame> = HashMap::new();
    for cached in crate::steam::storage::load_cached_games() {
        library_map.insert(cached.app_id, cached);
    }

    let id3_num = steam_id3.parse::<u64>().unwrap_or(0);
    let id64_num = id3_num + 76561197960265728;

    let api_client = SteamApiClient::new(settings.api_key.clone());
    if let Ok(owned_games) = api_client.get_owned_games(id64_num).await {
        for og in owned_games {
            let cats = categories_map.get(&og.app_id).cloned().unwrap_or_else(|| {
                library_map
                    .get(&og.app_id)
                    .map(|g| g.categories.clone())
                    .unwrap_or_default()
            });
            let is_hid = hidden_map.get(&og.app_id).copied().unwrap_or_else(|| {
                library_map
                    .get(&og.app_id)
                    .map(|g| g.is_hidden)
                    .unwrap_or(false)
            });

            library_map.insert(
                og.app_id,
                SteamGame {
                    app_id: og.app_id,
                    name: og.name,
                    categories: cats,
                    is_hidden: is_hid,
                    is_favorite: false,
                    playtime_minutes: og.playtime_forever,
                    last_played: 0,
                    icon_hash: og.img_icon_url,
                    genres: Vec::new(),
                    tags: Vec::new(),
                    developer: None,
                    publisher: None,
                    release_year: None,
                    review_score: None,
                    platforms: vec!["Windows".to_string()],
                    vr_support: None,
                    hltb_main_story: None,
                    hltb_main_extras: None,
                    hltb_completionist: None,
                },
            );
        }
    }

    // 5. Merge any games that already have categories in sharedconfig.vdf
    for (&app_id, cats) in &categories_map {
        library_map.entry(app_id).or_insert_with(|| {
            let is_hid = hidden_map.get(&app_id).copied().unwrap_or(false);
            SteamGame {
                app_id,
                name: format!("App {app_id}"),
                categories: cats.clone(),
                is_hidden: is_hid,
                is_favorite: false,
                playtime_minutes: 0,
                last_played: 0,
                icon_hash: None,
                genres: Vec::new(),
                tags: Vec::new(),
                developer: None,
                publisher: None,
                release_year: None,
                review_score: None,
                platforms: vec!["Windows".to_string()],
                vr_support: None,
                hltb_main_story: None,
                hltb_main_extras: None,
                hltb_completionist: None,
            }
        });
    }

    let mut result: Vec<SteamGame> = library_map.into_values().collect();
    result.sort_by_key(|a| a.name.to_lowercase());

    // Persist into SQLite cache
    let _ = crate::steam::storage::save_cached_games(&result);
    *state.cached_games.lock().await = result.clone();

    let mut cache = state.cached_games.lock().await;
    *cache = result.clone();

    worker
        .emit_progress(
            "load_library",
            100,
            100,
            &format!("Loaded {} games", result.len()),
            true,
        )
        .await;

    Ok(result)
}

/// Applies an AutoCat configuration across the library, streaming live per-game progress
/// to the frontend so the workbench rows update as each title is classified.
#[tauri::command]
pub async fn run_autocat(
    config: AutoCatConfig,
    state: State<'_, AppState>,
) -> Result<Vec<SteamGame>, String> {
    let worker = state.worker.clone();
    let cache = state.cached_games.clone();

    // Clone the library out so the cache mutex is never held while processing.
    let mut games = {
        let guard = cache.lock().await;
        guard.clone()
    };
    let total = games.len();

    worker
        .emit_progress(
            "run_autocat",
            0,
            total.max(1),
            "Applying AutoCat rules...",
            false,
        )
        .await;

    for (index, game) in games.iter_mut().enumerate() {
        apply_autocat_to_game(game, &config);

        worker.emit_event(
            "autocat://progress",
            AutoCatProgress {
                current: index + 1,
                total,
                done: index + 1 == total,
                game: game.clone(),
            },
        );

        // Yield periodically so the async runtime can flush IPC events to the webview.
        if (index + 1) % 25 == 0 {
            tokio::task::yield_now().await;
        }
    }

    {
        let mut guard = cache.lock().await;
        *guard = games.clone();
    }

    worker
        .emit_progress(
            "run_autocat",
            total,
            total.max(1),
            &format!("AutoCat complete — processed {total} games"),
            true,
        )
        .await;

    Ok(games)
}

/// Saves categories and hidden flags to Steam's `sharedconfig.vdf`.
#[tauri::command]
pub async fn save_to_steam(
    steam_id3: String,
    games: Vec<SteamGame>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().await.clone();
    let steam_path = settings
        .steam_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(detect_steam_path)
        .ok_or_else(|| "Steam directory not found".to_string())?;

    let saved_path = save_sharedconfig(&steam_path, &steam_id3, &games, settings.backup_on_save)?;

    let mut cache = state.cached_games.lock().await;
    *cache = games;

    Ok(saved_path.to_string_lossy().to_string())
}

/// Summarizes distinct categories and their counts from the current library.
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<SteamCategory>, String> {
    let cache = state.cached_games.lock().await;
    let mut counts: HashMap<String, usize> = HashMap::new();

    for game in cache.iter() {
        for cat in &game.categories {
            *counts.entry(cat.clone()).or_insert(0) += 1;
        }
    }

    let mut categories: Vec<SteamCategory> = counts
        .into_iter()
        .map(|(name, count)| SteamCategory {
            name,
            count,
            is_favorite: false,
        })
        .collect();

    categories.sort_by_key(|a| a.name.to_lowercase());
    Ok(categories)
}

/// Fetches HowLongToBeat times for a Steam app id via the community REST wrapper.
#[tauri::command]
pub async fn fetch_hltb(app_id: u32) -> Result<crate::steam::models::HltbTimes, String> {
    use crate::steam::models::HltbTimes;
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // 1. Try the community REST wrapper (direct Steam appId lookup).
    let url = format!("https://hltbapi.codepotatoes.de/steam/{app_id}");
    if let Ok(resp) = client.get(&url).send().await {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let main_story = json.get("mainStory").and_then(|v| v.as_f64()).map(|v| v as f32);
                let main_extras = json.get("mainStoryWithExtras").and_then(|v| v.as_f64()).map(|v| v as f32);
                let completionist = json.get("completionist").and_then(|v| v.as_f64()).map(|v| v as f32);
                if main_story.is_some() || main_extras.is_some() || completionist.is_some() {
                    return Ok(HltbTimes { main_story, main_extras, completionist });
                }
            }
        }
    }

    // 2. Fallback: token-gated HLTB internal endpoint.
    // Discover the init token, then POST the find query.
    let init_url = "https://howlongtobeat.com/api/find/init?t=";
    if let Ok(resp) = client.get(init_url).send().await {
        if let Ok(init_json) = resp.json::<serde_json::Value>().await {
            let token = init_json
                .get("token")
                .and_then(|v| v.as_str())
                .or_else(|| init_json.get("data").and_then(|v| v.as_str()))
                .map(|s| s.to_string());

            if let Some(tok) = token {
                let body = serde_json::json!({
                    "searchType": "games",
                    "searchTerms": [],
                    "searchPage": 1,
                    "size": 1,
                    "searchOptions": {
                        "games": {
                            "userId": 0,
                            "platform": "",
                            "sortCategory": "name",
                            "rangeCategory": "main",
                            "rangeTime": { "min": 0, "max": 0 },
                            "gameplay": { "perspective": "", "flow": "", "genre": "" },
                            "modifier": "hide_dlc"
                        },
                        "users": {},
                        "filter": "",
                        "sort": 0,
                        "randomizer": 0
                    }
                });

                let find_resp = client
                    .post("https://howlongtobeat.com/api/find")
                    .header("Content-Type", "application/json")
                    .header("Referer", "https://howlongtobeat.com/")
                    .header("x-auth-token", tok)
                    .json(&body)
                    .send()
                    .await;

                if let Ok(r) = find_resp {
                    if r.status().is_success() {
                        if let Ok(j) = r.json::<serde_json::Value>().await {
                            // Try to find a game whose profile_steam matches app_id
                            if let Some(data) = j.get("data").and_then(|v| v.as_array()) {
                                for g in data {
                                    let steam_id = g.get("profile_steam").and_then(|v| v.as_u64());
                                    if steam_id == Some(app_id as u64) {
                                        let ms = g.get("comp_main").and_then(|v| v.as_f64()).map(|v| (v / 3600.0) as f32);
                                        let me = g.get("comp_plus").and_then(|v| v.as_f64()).map(|v| (v / 3600.0) as f32);
                                        let c = g.get("comp_100").and_then(|v| v.as_f64()).map(|v| (v / 3600.0) as f32);
                                        return Ok(HltbTimes { main_story: ms, main_extras: me, completionist: c });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(HltbTimes { main_story: None, main_extras: None, completionist: None })
}
/// Persists HowLongToBeat times for a single game.
#[tauri::command]
pub async fn save_hltb_times(app_id: u32, main_story: Option<f32>, main_extras: Option<f32>, completionist: Option<f32>) -> Result<(), String> {
    crate::steam::storage::save_hltb_times(app_id, main_story, main_extras, completionist)
}

#[tauri::command]
pub async fn verify_api_key(api_key: String) -> Result<bool, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url =
        format!("https://api.steampowered.com/ISteamWebAPIUtil/GetServerInfo/v0001/?key={key}");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {e}"))?;

    if resp.status().is_success() {
        Ok(true)
    } else {
        Err(format!(
            "Steam Web API rejected key (HTTP {})",
            resp.status()
        ))
    }
}

#[tauri::command]
pub async fn get_autocat_rules() -> Result<Vec<AutoCatRule>, String> {
    Ok(crate::steam::storage::load_autocat_rules())
}

#[tauri::command]
pub async fn save_autocat_rules(rules: Vec<AutoCatRule>) -> Result<(), String> {
    crate::steam::storage::save_autocat_rules(&rules)
}
