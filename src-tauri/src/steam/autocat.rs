use crate::steam::models::{AutoCatConfig, AutoCatFilterType, SteamGame};
use regex::RegexBuilder;

/// Applies an AutoCat configuration to a single game, mutating it in place.
pub fn apply_autocat_to_game(game: &mut SteamGame, config: &AutoCatConfig) {
    if config.clear_existing {
        game.categories.clear();
    }

    for rule in &config.rules {
        if !rule.enabled {
            continue;
        }

        let is_dynamic_target = rule.target_category.trim().is_empty()
            || rule.target_category.trim() == "__dynamic__";

        let mut matched_categories: Vec<String> = Vec::new();

        match rule.filter_type {
            AutoCatFilterType::Genre => {
                let target = rule.params["genre"].as_str().unwrap_or("").trim().to_lowercase();
                if target.is_empty() || target == "*" {
                    if is_dynamic_target {
                        for g in &game.genres {
                            if !g.trim().is_empty() {
                                matched_categories.push(g.clone());
                            }
                        }
                    } else if !game.genres.is_empty() {
                        matched_categories.push(rule.target_category.clone());
                    }
                } else {
                    for g in &game.genres {
                        if g.to_lowercase().contains(&target) {
                            if is_dynamic_target {
                                matched_categories.push(g.clone());
                            } else {
                                matched_categories.push(rule.target_category.clone());
                                break;
                            }
                        }
                    }
                }
            }
            AutoCatFilterType::Tags => {
                let target = rule.params["tag"].as_str().unwrap_or("").trim().to_lowercase();
                if target.is_empty() || target == "*" {
                    if is_dynamic_target {
                        for t in &game.tags {
                            if !t.trim().is_empty() {
                                matched_categories.push(t.clone());
                            }
                        }
                    } else if !game.tags.is_empty() {
                        matched_categories.push(rule.target_category.clone());
                    }
                } else {
                    for t in &game.tags {
                        if t.to_lowercase().contains(&target) {
                            if is_dynamic_target {
                                matched_categories.push(t.clone());
                            } else {
                                matched_categories.push(rule.target_category.clone());
                                break;
                            }
                        }
                    }
                }
            }
            AutoCatFilterType::DevPub => {
                let target = rule.params["name"].as_str().unwrap_or("").trim().to_lowercase();
                let dev_match = game
                    .developer
                    .as_ref()
                    .filter(|d| target.is_empty() || target == "*" || d.to_lowercase().contains(&target));
                let pub_match = game
                    .publisher
                    .as_ref()
                    .filter(|p| target.is_empty() || target == "*" || p.to_lowercase().contains(&target));

                if is_dynamic_target {
                    if let Some(dev) = dev_match {
                        matched_categories.push(dev.clone());
                    }
                    if let Some(publ) = pub_match {
                        if !matched_categories.contains(publ) {
                            matched_categories.push(publ.clone());
                        }
                    }
                } else if dev_match.is_some() || pub_match.is_some() {
                    matched_categories.push(rule.target_category.clone());
                }
            }
            AutoCatFilterType::Year => {
                let min_year = rule.params["min_year"].as_u64().unwrap_or(1970) as u32;
                let max_year = rule.params["max_year"].as_u64().unwrap_or(2100) as u32;
                if let Some(year) = game.release_year {
                    if year >= min_year && year <= max_year {
                        if is_dynamic_target {
                            if min_year == max_year {
                                matched_categories.push(format!("{year}"));
                            } else if min_year % 10 == 0 && max_year == min_year + 9 {
                                matched_categories.push(format!("{min_year}s"));
                            } else {
                                matched_categories.push(format!("{min_year}-{max_year}"));
                            }
                        } else {
                            matched_categories.push(rule.target_category.clone());
                        }
                    }
                }
            }
            AutoCatFilterType::Playtime => {
                let min_hours = rule.params["min_hours"].as_f64().unwrap_or(0.0);
                let max_hours = rule.params["max_hours"].as_f64().unwrap_or(100000.0);
                let hours = (game.playtime_minutes as f64) / 60.0;
                if hours >= min_hours && hours <= max_hours {
                    if is_dynamic_target {
                        let label = if min_hours <= 0.0 && max_hours <= 0.05 {
                            "Unplayed (0 hrs)".to_string()
                        } else if max_hours >= 10000.0 {
                            format!("{min_hours:.0}+ hrs")
                        } else {
                            format!("{min_hours:.0}-{max_hours:.0} hrs")
                        };
                        matched_categories.push(label);
                    } else {
                        matched_categories.push(rule.target_category.clone());
                    }
                }
            }
            AutoCatFilterType::Score => {
                let min_score = rule.params["min_score"].as_u64().unwrap_or(0) as u32;
                let max_score = rule.params["max_score"].as_u64().unwrap_or(100) as u32;
                if let Some(score) = game.review_score {
                    if score >= min_score && score <= max_score {
                        if is_dynamic_target {
                            let label = if min_score >= 90 {
                                "90%+ Overwhelmingly Positive".to_string()
                            } else if min_score >= 80 {
                                "80-89% Very Positive".to_string()
                            } else if min_score >= 70 {
                                "70-79% Mostly Positive".to_string()
                            } else if min_score >= 40 {
                                "40-69% Mixed".to_string()
                            } else {
                                "<40% Negative".to_string()
                            };
                            matched_categories.push(label);
                        } else {
                            matched_categories.push(rule.target_category.clone());
                        }
                    }
                }
            }
            AutoCatFilterType::Platform => {
                let target = rule.params["platform"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
                if target.is_empty() || target == "*" {
                    if is_dynamic_target {
                        for p in &game.platforms {
                            if !p.trim().is_empty() {
                                matched_categories.push(p.clone());
                            }
                        }
                    } else if !game.platforms.is_empty() {
                        matched_categories.push(rule.target_category.clone());
                    }
                } else {
                    for p in &game.platforms {
                        if p.to_lowercase().contains(&target) {
                            if is_dynamic_target {
                                matched_categories.push(p.clone());
                            } else {
                                matched_categories.push(rule.target_category.clone());
                                break;
                            }
                        }
                    }
                }
            }
            AutoCatFilterType::VrSupport => {
                let vr_only = rule.params["vr_only"].as_bool().unwrap_or(false);
                let matches = if vr_only {
                    game.vr_support.as_deref() == Some("VR Only")
                } else {
                    game.vr_support.is_some()
                };

                if matches {
                    if is_dynamic_target {
                        matched_categories.push(
                            game.vr_support
                                .clone()
                                .unwrap_or_else(|| "VR Supported".to_string()),
                        );
                    } else {
                        matched_categories.push(rule.target_category.clone());
                    }
                }
            }
            AutoCatFilterType::Manual => {
                if let Some(app_ids) = rule.params["app_ids"].as_array() {
                    if app_ids.iter().any(|id| id.as_u64() == Some(game.app_id as u64)) {
                        if is_dynamic_target {
                            matched_categories.push("Manual Collection".to_string());
                        } else {
                            matched_categories.push(rule.target_category.clone());
                        }
                    }
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
                if let Some(hours) = value {
                    let h = hours as f64;
                    if h >= min_hours && h <= max_hours {
                        if is_dynamic_target {
                            let label = if max_hours >= 10000.0 {
                                format!("HLTB: {min_hours:.0}+ hrs")
                            } else {
                                format!("HLTB: {min_hours:.0}-{max_hours:.0} hrs")
                            };
                            matched_categories.push(label);
                        } else {
                            matched_categories.push(rule.target_category.clone());
                        }
                    }
                }
            }
            AutoCatFilterType::Regex => {
                let pattern = rule.params["pattern"].as_str().unwrap_or("").trim();
                let field = rule.params["field"].as_str().unwrap_or("name");
                if !pattern.is_empty() {
                    if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                        let target_text = match field {
                            "developer" => game.developer.as_deref().unwrap_or(""),
                            "publisher" => game.publisher.as_deref().unwrap_or(""),
                            _ => &game.name,
                        };

                        if let Some(caps) = re.captures(target_text) {
                            if is_dynamic_target {
                                // If capture group 1 exists, use it; otherwise use full match
                                let matched_text = if caps.len() > 1 {
                                    caps.get(1).map(|m| m.as_str().trim()).unwrap_or("")
                                } else {
                                    caps.get(0).map(|m| m.as_str().trim()).unwrap_or("")
                                };
                                if !matched_text.is_empty() {
                                    matched_categories.push(matched_text.to_string());
                                }
                            } else {
                                matched_categories.push(rule.target_category.clone());
                            }
                        }
                    }
                }
            }
        }

        for cat in matched_categories {
            let category_name = if let Some(ref p) = config.prefix {
                if !p.is_empty() {
                    format!("{p}{cat}")
                } else {
                    cat
                }
            } else {
                cat
            };

            if !category_name.trim().is_empty() && !game.categories.contains(&category_name) {
                game.categories.push(category_name);
            }
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

    #[test]
    fn test_autocat_dynamic_categories_and_prefix() {
        let games = vec![
            make_test_game(10, "Final Fantasy VII", vec!["RPG", "Adventure"], 120, Some(1997)),
            make_test_game(20, "Half-Life 2", vec!["Action", "FPS"], 300, Some(2004)),
        ];

        let config = AutoCatConfig {
            rules: vec![
                AutoCatRule {
                    id: "dyn-genre".to_string(),
                    name: "Dynamic Genres".to_string(),
                    filter_type: AutoCatFilterType::Genre,
                    target_category: "".to_string(), // Dynamic!
                    params: serde_json::json!({ "genre": "" }),
                    enabled: true,
                },
                AutoCatRule {
                    id: "dyn-year".to_string(),
                    name: "Dynamic Decade".to_string(),
                    filter_type: AutoCatFilterType::Year,
                    target_category: "__dynamic__".to_string(), // Dynamic!
                    params: serde_json::json!({ "min_year": 1990, "max_year": 1999 }),
                    enabled: true,
                },
            ],
            clear_existing: false,
            prefix: Some("Genre: ".to_string()),
        };

        let result = apply_autocat(games, &config);
        assert!(result[0].categories.contains(&"Genre: RPG".to_string()));
        assert!(result[0].categories.contains(&"Genre: Adventure".to_string()));
        assert!(result[0].categories.contains(&"Genre: 1990s".to_string()));
        assert!(result[1].categories.contains(&"Genre: Action".to_string()));
        assert!(result[1].categories.contains(&"Genre: FPS".to_string()));
        assert!(!result[1].categories.contains(&"Genre: 1990s".to_string()));
    }

    #[test]
    fn test_autocat_regex_filter() {
        let games = vec![
            make_test_game(1, "Final Fantasy VII", vec![], 0, None),
            make_test_game(2, "Final Fantasy X", vec![], 0, None),
            make_test_game(3, "Half-Life: Alyx", vec![], 0, None),
        ];

        let config = AutoCatConfig {
            rules: vec![
                AutoCatRule {
                    id: "re-franchise".to_string(),
                    name: "Franchise Prefix".to_string(),
                    filter_type: AutoCatFilterType::Regex,
                    target_category: "".to_string(), // dynamic from capture group 1
                    params: serde_json::json!({
                        "pattern": r"^([A-Za-z\- ]+?)(?:\s+[IVXLCDM0-9]+|:)",
                        "field": "name"
                    }),
                    enabled: true,
                },
            ],
            clear_existing: false,
            prefix: None,
        };

        let result = apply_autocat(games, &config);
        assert!(result[0].categories.contains(&"Final Fantasy".to_string()));
        assert!(result[1].categories.contains(&"Final Fantasy".to_string()));
        assert!(result[2].categories.contains(&"Half-Life".to_string()));
    }
}
