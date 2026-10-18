use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};


/// Finds the target `sharedconfig.vdf` path for a given user.
pub fn get_sharedconfig_path(steam_path: &Path, steam_id3: &str) -> PathBuf {
    let remote = steam_path
        .join("userdata")
        .join(steam_id3)
        .join("7")
        .join("remote")
        .join("sharedconfig.vdf");
    if remote.is_file() {
        return remote;
    }

    let config = steam_path
        .join("userdata")
        .join(steam_id3)
        .join("config")
        .join("sharedconfig.vdf");
    if config.is_file() {
        return config;
    }

    remote
}

/// Loads existing game categories and hidden status from `sharedconfig.vdf` for a specific Steam user.
pub fn load_user_categories(
    steam_path: &Path,
    steam_id3: &str,
) -> (HashMap<u32, Vec<String>>, HashMap<u32, bool>) {
    let mut categories_map = HashMap::new();
    let mut hidden_map = HashMap::new();

    let shared_vdf = get_sharedconfig_path(steam_path, steam_id3);

    if !shared_vdf.is_file() {
        return (categories_map, hidden_map);
    }

    if let Ok(content) = fs::read_to_string(&shared_vdf) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut inside_apps = false;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.eq_ignore_ascii_case("\"apps\"") || line.contains("\"apps\"") {
                inside_apps = true;
                i += 1;
                continue;
            }

            if inside_apps && line.starts_with('"') && !line.contains(' ') {
                let app_id_str = line.trim_matches('"');
                if let Ok(app_id) = app_id_str.parse::<u32>() {
                    i += 1;
                    let mut app_depth = 0;
                    let mut inside_tags = false;
                    let mut game_categories = Vec::new();

                    while i < lines.len() {
                        let app_line = lines[i].trim();
                        if app_line == "{" {
                            app_depth += 1;
                        } else if app_line == "}" {
                            app_depth -= 1;
                            if app_depth == 0 {
                                break;
                            }
                            if inside_tags {
                                inside_tags = false;
                            }
                        } else if app_line.eq_ignore_ascii_case("\"tags\"") {
                            inside_tags = true;
                        } else if inside_tags {
                            let parts: Vec<&str> = app_line
                                .split('"')
                                .filter(|s| !s.trim().is_empty())
                                .collect();
                            if parts.len() >= 2 {
                                let tag = parts[1].to_string();
                                if !tag.is_empty() && !game_categories.contains(&tag) {
                                    game_categories.push(tag);
                                }
                            }
                        } else if app_line.starts_with("\"hidden\"")
                            || app_line.starts_with("\"IsHidden\"")
                        {
                            let parts: Vec<&str> = app_line
                                .split('"')
                                .filter(|s| !s.trim().is_empty())
                                .collect();
                            if parts.len() >= 2 {
                                let is_hid = parts[1] == "1";
                                hidden_map.insert(app_id, is_hid);
                            }
                        }
                        i += 1;
                    }

                    if !game_categories.is_empty() {
                        categories_map.insert(app_id, game_categories);
                    }
                }
            }

            i += 1;
        }
    }

    (categories_map, hidden_map)
}

/// Creates a timestamped backup of `sharedconfig.vdf` before making any modifications.
pub fn backup_sharedconfig(steam_path: &Path, steam_id3: &str) -> Result<PathBuf, String> {
    let target = steam_path
        .join("userdata")
        .join(steam_id3)
        .join("7")
        .join("remote")
        .join("sharedconfig.vdf");

    if !target.is_file() {
        return Ok(target);
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = target.with_file_name(format!("sharedconfig_backup_{timestamp}.vdf"));
    fs::copy(&target, &backup_path)
        .map_err(|e| format!("Failed to create backup at {}: {e}", backup_path.display()))?;

    Ok(backup_path)
}

/// Saves user categories and hidden flags to `sharedconfig.vdf`.
pub fn save_sharedconfig(
    steam_path: &Path,
    steam_id3: &str,
    games: &[crate::steam::models::SteamGame],
    backup_first: bool,
) -> Result<PathBuf, String> {
    let remote_dir = steam_path
        .join("userdata")
        .join(steam_id3)
        .join("7")
        .join("remote");

    fs::create_dir_all(&remote_dir).map_err(|e| {
        format!(
            "Failed to create remote directory {}: {e}",
            remote_dir.display()
        )
    })?;

    let file_path = remote_dir.join("sharedconfig.vdf");

    if backup_first && file_path.is_file() {
        let _ = backup_sharedconfig(steam_path, steam_id3);
    }

    // Build standard Valve KeyValues text block
    let mut out = String::with_capacity(32768);
    out.push_str("\"UserLocalConfigStore\"\n{\n");
    out.push_str("\t\"Software\"\n\t{\n");
    out.push_str("\t\t\"Valve\"\n\t\t{\n");
    out.push_str("\t\t\t\"Steam\"\n\t\t\t{\n");
    out.push_str("\t\t\t\t\"apps\"\n\t\t\t\t{\n");

    for game in games {
        if game.categories.is_empty() && !game.is_hidden {
            continue;
        }

        out.push_str(&format!("\t\t\t\t\t\"{}\"\n\t\t\t\t\t{{\n", game.app_id));

        if game.is_hidden {
            out.push_str("\t\t\t\t\t\t\"hidden\"\t\t\"1\"\n");
        }

        if !game.categories.is_empty() {
            out.push_str("\t\t\t\t\t\t\"tags\"\n\t\t\t\t\t\t{\n");
            for (idx, cat) in game.categories.iter().enumerate() {
                out.push_str(&format!("\t\t\t\t\t\t\t\"{idx}\"\t\t\"{cat}\"\n"));
            }
            out.push_str("\t\t\t\t\t\t}\n");
        }

        out.push_str("\t\t\t\t\t}\n");
    }

    out.push_str("\t\t\t\t}\n");
    out.push_str("\t\t\t}\n");
    out.push_str("\t\t}\n");
    out.push_str("\t}\n");
    out.push_str("}\n");

    fs::write(&file_path, out).map_err(|e| format!("Failed to write sharedconfig.vdf: {e}"))?;

    Ok(file_path)
}
