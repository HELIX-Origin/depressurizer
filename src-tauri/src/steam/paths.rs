use crate::steam::models::SteamAccount;
use std::path::{Path, PathBuf};

/// Detects the Steam installation directory across Windows, macOS, and Linux.
pub fn detect_steam_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        // Try 64-bit / 32-bit registry keys first
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(steam_key) = hkcu.open_subkey(r"Software\Valve\Steam") {
            if let Ok(path_str) = steam_key.get_value::<String, _>("SteamPath") {
                let p = PathBuf::from(path_str.replace('/', "\\"));
                if p.is_dir() {
                    return Some(p);
                }
            }
        }

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(steam_key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam") {
            if let Ok(path_str) = steam_key.get_value::<String, _>("InstallPath") {
                let p = PathBuf::from(path_str);
                if p.is_dir() {
                    return Some(p);
                }
            }
        }

        // Standard default directories
        let common = [
            r"C:\Program Files (x86)\Steam",
            r"C:\Program Files\Steam",
            r"D:\Steam",
        ];
        for path in common {
            let p = PathBuf::from(path);
            if p.is_dir() {
                return Some(p);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let p = home.join("Library/Application Support/Steam");
            if p.is_dir() {
                return Some(p);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data) = dirs::data_local_dir() {
            let p = data.join("Steam");
            if p.is_dir() {
                return Some(p);
            }
        }
        if let Some(home) = dirs::home_dir() {
            let candidates = [
                home.join(".local/share/Steam"),
                home.join(".steam/steam"),
                home.join(".steam/root"),
            ];
            for p in candidates {
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
    }

    None
}

/// Enumerate local Steam accounts from `<steam_path>/userdata` (the only location holding `sharedconfig.vdf`).
pub fn get_local_accounts(steam_path: &Path) -> Vec<SteamAccount> {
    let mut accounts = Vec::new();
    let userdata_path = steam_path.join("userdata");

    // Scan userdata folders
    let mut user_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&userdata_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.chars().all(|c| c.is_ascii_digit()) && name != "0" {
                            user_dirs.push(name);
                        }
                    }
                }
            }
        }
    }

    for id3 in user_dirs {
        let id3_num = id3.parse::<u64>().unwrap_or(0);
        // SteamID64 = SteamID3 + 76561197960265728
        let id64_num = id3_num + 76561197960265728;

        accounts.push(SteamAccount {
            steam_id3: id3.clone(),
            steam_id64: Some(id64_num),
            account_name: format!("user_{id3}"),
            persona_name: format!("Steam User {id3}"),
            most_recent: false,
        });
    }

    // Most recently modified `sharedconfig.vdf` is treated as the most recent account.
    let mtime = |a: &SteamAccount| {
        std::fs::metadata(
            userdata_path
                .join(&a.steam_id3)
                .join("7")
                .join("remote")
                .join("sharedconfig.vdf"),
        )
        .and_then(|m| m.modified())
        .ok()
    };
    if let Some(best) = accounts.iter().enumerate().max_by_key(|(_, a)| mtime(a)).map(|(i, _)| i) {
        accounts[best].most_recent = true;
    }

    accounts.sort_by_key(|a| std::cmp::Reverse(a.most_recent));
    accounts
}

