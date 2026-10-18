use crate::steam::models::{AutoCatConfig, AutoCatFilterType, SteamGame};

/// Applies an AutoCat configuration to a single game, mutating it in place.
pub fn apply_autocat_to_game(game: &mut SteamGame, config: &AutoCatConfig) {
    if config.clear_existing {
        game.categories.clear();
    }

    for rule in &config.rules {
        if !rule.enabled || rule.target_category.trim().is_empty() {
            continue;
        }

        let category_name = if let Some(ref p) = config.prefix {
            if !p.is_empty() {
                format!("{p}{}", rule.target_category)
            } else {
                rule.target_category.clone()
            }
        } else {
            rule.target_category.clone()
        };

        let matches = match rule.filter_type {
            AutoCatFilterType::Genre => {
                let target = rule.params["genre"].as_str().unwrap_or("").to_lowercase();
                game.genres
                    .iter()
                    .any(|g| g.to_lowercase().contains(&target))
            }
            AutoCatFilterType::Tags => {
                let target = rule.params["tag"].as_str().unwrap_or("").to_lowercase();
                game.tags.iter().any(|t| t.to_lowercase().contains(&target))
            }
            AutoCatFilterType::DevPub => {
                let target = rule.params["name"].as_str().unwrap_or("").to_lowercase();
                let dev_match = game
                    .developer
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&target))
                    .unwrap_or(false);
                let pub_match = game
                    .publisher
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&target))
                    .unwrap_or(false);
                dev_match || pub_match
            }
            AutoCatFilterType::Year => {
                let min_year = rule.params["min_year"].as_u64().unwrap_or(1970) as u32;
                let max_year = rule.params["max_year"].as_u64().unwrap_or(2100) as u32;
                game.release_year
                    .map(|y| y >= min_year && y <= max_year)
                    .unwrap_or(false)
            }
            AutoCatFilterType::Playtime => {
                let min_hours = rule.params["min_hours"].as_f64().unwrap_or(0.0);
                let max_hours = rule.params["max_hours"].as_f64().unwrap_or(100000.0);
                let hours = (game.playtime_minutes as f64) / 60.0;
                hours >= min_hours && hours <= max_hours
            }
            AutoCatFilterType::Score => {
                let min_score = rule.params["min_score"].as_u64().unwrap_or(0) as u32;
                game.review_score.map(|s| s >= min_score).unwrap_or(false)
            }
            AutoCatFilterType::Platform => {
                let target = rule.params["platform"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase();
                game.platforms
                    .iter()
                    .any(|p| p.to_lowercase().contains(&target))
            }
            AutoCatFilterType::VrSupport => {
                rule.params["vr_only"].as_bool().unwrap_or(false)
                    && game.vr_support.as_deref() == Some("VR Only")
            }
            AutoCatFilterType::Manual => {
                if let Some(app_ids) = rule.params["app_ids"].as_array() {
                    app_ids
                        .iter()
                        .any(|id| id.as_u64() == Some(game.app_id as u64))
                } else {
                    false
                }
            }
            AutoCatFilterType::Hltb => {
                let min_hours = rule.params["min_hours"].as_f64().unwrap_or(0.0);
                let max_hours = rule.params["max_hours"].as_f64().unwrap_or(100000.0);
                let value = match rule.params["field"].as_str().unwrap_or("main_story") {
                    "main_extras" => game.hltb_main_extras,
                    "completionist" => game.hltb_completionist,
                    _ => game.hltb_main_story,
                };
                value
                    .map(|h| (h as f64) >= min_hours && (h as f64) <= max_hours)
                    .unwrap_or(false)
            }
        };

        if matches && !game.categories.contains(&category_name) {
            game.categories.push(category_name);
        }
    }
}

/// Evaluates a collection of games against an AutoCat configuration.
pub fn apply_autocat(mut games: Vec<SteamGame>, config: &AutoCatConfig) -> Vec<SteamGame> {
    for game in &mut games {
        apply_autocat_to_game(game, config);
    }

    games
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::models::{AutoCatFilterType, AutoCatRule};

    fn make_test_game(
        app_id: u32,
        name: &str,
        genres: Vec<&str>,
        playtime: u32,
        year: Option<u32>,
    ) -> SteamGame {
        SteamGame {
            app_id,
            name: name.to_string(),
            categories: Vec::new(),
            is_hidden: false,
            is_favorite: false,
            playtime_minutes: playtime,
            last_played: 0,
            icon_hash: None,
            genres: genres.into_iter().map(|s| s.to_string()).collect(),
            tags: Vec::new(),
            developer: None,
            publisher: None,
            release_year: year,
            review_score: None,
            platforms: vec!["Windows".to_string()],
            vr_support: None,
            hltb_main_story: None,
            hltb_main_extras: None,
            hltb_completionist: None,
        }
    }

    #[test]
    fn test_autocat_genre_matching() {
        let games = vec![
            make_test_game(10, "Counter-Strike", vec!["Action"], 120, Some(2000)),
            make_test_game(
                20,
                "Baldur's Gate 3",
                vec!["RPG", "Strategy"],
                3600,
                Some(2023),
            ),
        ];

        let config = AutoCatConfig {
            rules: vec![
                AutoCatRule {
                    id: "r1".to_string(),
                    name: "RPGs".to_string(),
                    filter_type: AutoCatFilterType::Genre,
                    target_category: "Role-Playing".to_string(),
                    params: serde_json::json!({ "genre": "rpg" }),
                    enabled: true,
                },
                AutoCatRule {
                    id: "r2".to_string(),
                    name: "High Playtime".to_string(),
                    filter_type: AutoCatFilterType::Playtime,
                    target_category: "Most Played".to_string(),
                    params: serde_json::json!({ "min_hours": 50.0 }),
                    enabled: true,
                },
            ],
            clear_existing: false,
            prefix: None,
        };

        let result = apply_autocat(games, &config);
        assert!(!result[0].categories.contains(&"Role-Playing".to_string()));
        assert!(result[1].categories.contains(&"Role-Playing".to_string()));
        assert!(result[1].categories.contains(&"Most Played".to_string()));
    }

    fn hltb_config(params: serde_json::Value) -> AutoCatConfig {
        AutoCatConfig {
            rules: vec![AutoCatRule {
                id: "h1".to_string(),
                name: "HLTB".to_string(),
                filter_type: AutoCatFilterType::Hltb,
                target_category: "Short".to_string(),
                params,
                enabled: true,
            }],
            clear_existing: false,
            prefix: None,
        }
    }

    #[test]
    fn test_autocat_hltb_range_and_fields() {
        let mut short = make_test_game(1, "Short Game", vec![], 0, None);
        short.hltb_main_story = Some(4.0);
        short.hltb_completionist = Some(12.0);
        let mut long = make_test_game(2, "Long Game", vec![], 0, None);
        long.hltb_main_story = Some(60.0);
        long.hltb_completionist = Some(150.0);
        let unknown = make_test_game(3, "No Data", vec![], 0, None);

        // Main story (default field) within 0..=10 hours.
        let cfg = hltb_config(serde_json::json!({ "min_hours": 0.0, "max_hours": 10.0 }));
        let r = apply_autocat(vec![short.clone(), long.clone(), unknown.clone()], &cfg);
        assert!(r[0].categories.contains(&"Short".to_string()));
        assert!(r[1].categories.is_empty());
        assert!(r[2].categories.is_empty(), "games without HLTB data never match");

        // Completionist field with only a minimum.
        let cfg = hltb_config(serde_json::json!({ "field": "completionist", "min_hours": 100.0 }));
        let r = apply_autocat(vec![short, long, unknown], &cfg);
        assert!(r[0].categories.is_empty());
        assert!(r[1].categories.contains(&"Short".to_string()));
        assert!(r[2].categories.is_empty());
    }
}
