use reqwest::Client;
use serde_json::Value;

/// Response struct for `IPlayerService/GetOwnedGames/v0001/`
#[derive(Debug, Clone)]
pub struct WebApiGame {
    pub app_id: u32,
    pub name: String,
    pub playtime_forever: u32,
    pub img_icon_url: Option<String>,
}

/// Official Steam Web API client.
pub struct SteamApiClient {
    client: Client,
    api_key: Option<String>,
}

impl SteamApiClient {
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self { client, api_key }
    }

    /// Fetches all owned games for a SteamID64 via `IPlayerService/GetOwnedGames/v0001/`.
    pub async fn get_owned_games(&self, steam_id64: u64) -> Result<Vec<WebApiGame>, String> {
        let Some(ref key) = self.api_key else {
            return Err("No Steam Web API key configured".to_string());
        };

        if key.trim().is_empty() {
            return Err("Steam Web API key is empty".to_string());
        }

        let url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1&format=json",
            key.trim(),
            steam_id64
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Steam API network error: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Steam Web API returned HTTP status {}",
                resp.status()
            ));
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Steam API response: {e}"))?;

        let mut results = Vec::new();
        if let Some(games) = body["response"]["games"].as_array() {
            for g in games {
                let app_id = g["appid"].as_u64().unwrap_or(0) as u32;
                let name = g["name"].as_str().unwrap_or("").to_string();
                let playtime = g["playtime_forever"].as_u64().unwrap_or(0) as u32;
                let icon = g["img_icon_url"].as_str().map(|s| s.to_string());

                if app_id > 0 && !name.is_empty() {
                    results.push(WebApiGame {
                        app_id,
                        name,
                        playtime_forever: playtime,
                        img_icon_url: icon,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Fetches store metadata (genres and category flags) for an app from the official Store API.
    pub async fn get_store_details(
        &self,
        app_id: u32,
    ) -> Result<
        (
            Vec<String>,
            Vec<String>,
            Option<String>,
            Option<String>,
            Option<u32>,
            Vec<String>,
        ),
        String,
    > {
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}&filters=genres,categories,developers,publishers,release_date,platforms",
            app_id
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Store API request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Store API HTTP {}", resp.status()));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("Store API JSON error: {e}"))?;

        let app_key = app_id.to_string();
        if !data[&app_key]["success"].as_bool().unwrap_or(false) {
            return Err("Store API reported no data".to_string());
        }

        let app_data = &data[&app_key]["data"];

        let mut genres = Vec::new();
        if let Some(arr) = app_data["genres"].as_array() {
            for item in arr {
                if let Some(desc) = item["description"].as_str() {
                    genres.push(desc.to_string());
                }
            }
        }

        let mut tags = Vec::new();
        if let Some(arr) = app_data["categories"].as_array() {
            for item in arr {
                if let Some(desc) = item["description"].as_str() {
                    tags.push(desc.to_string());
                }
            }
        }

        let developer = app_data["developers"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let publisher = app_data["publishers"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let release_year = app_data["release_date"]["date"]
            .as_str()
            .and_then(|d| d.split(',').next_back())
            .and_then(|y| y.trim().parse::<u32>().ok());

        let mut platforms = Vec::new();
        if app_data["platforms"]["windows"].as_bool().unwrap_or(false) {
            platforms.push("Windows".to_string());
        }
        if app_data["platforms"]["mac"].as_bool().unwrap_or(false) {
            platforms.push("macOS".to_string());
        }
        if app_data["platforms"]["linux"].as_bool().unwrap_or(false) {
            platforms.push("Linux".to_string());
        }

        Ok((genres, tags, developer, publisher, release_year, platforms))
    }
}
