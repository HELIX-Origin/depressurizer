//! Internal Headless CLI Engine (`--cli`).
//!
//! Provides command-line automation for Steam library categorization without UI overhead.
//!
//! Supported subcommands:
//! - `depressurizer --cli detect`: Detect Steam installation path and list all discovered user profiles
//! - `depressurizer --cli list-games [--user <id3>]`: Enumerate library games and existing categories
//! - `depressurizer --cli list-categories [--user <id3>]`: Enumerate distinct categories and counts
//! - `depressurizer --cli autocat [--user <id3>] [--rules <file.json>] [--dry-run]`: Execute AutoCat rules
//! - `depressurizer --cli sync [--user <id3>] [--backup]`: Save categories to Steam sharedconfig.vdf
//! - `depressurizer --cli help`: Show CLI usage

use crate::steam::paths::{detect_steam_path, get_local_accounts};
use crate::steam::vdf::{load_user_categories};

pub fn try_run_cli() -> bool {
    let args: Vec<String> = std::env::args().collect();
    let Some(cli_idx) = args.iter().position(|a| a == "--cli") else {
        return false;
    };

    #[cfg(windows)]
    {
        unsafe {
            extern "system" {
                fn AttachConsole(dwProcessId: u32) -> i32;
            }
            let _ = AttachConsole(u32::MAX); // ATTACH_PARENT_PROCESS
        }
    }

    let sub_args = &args[cli_idx + 1..];
    let cmd = sub_args.first().map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "detect" => {
            let path = detect_steam_path();
            let accounts = path
                .as_ref()
                .map(|p| get_local_accounts(p))
                .unwrap_or_default();
            println!(
                "{}",
                serde_json::json!({
                    "steam_path": path.map(|p| p.to_string_lossy().to_string()),
                    "accounts": accounts,
                })
            );
        }
        "list-categories" => {
            let Some(steam_path) = detect_steam_path() else {
                eprintln!("{{\"error\": \"Steam installation not found\"}}");
                std::process::exit(1);
            };

            let accounts = get_local_accounts(&steam_path);
            let user_arg = sub_args
                .windows(2)
                .find(|w| w[0] == "--user")
                .map(|w| w[1].clone());
            let target_user = user_arg
                .or_else(|| accounts.first().map(|a| a.steam_id3.clone()))
                .unwrap_or_else(|| "0".to_string());

            let (categories_map, _) = load_user_categories(&steam_path, &target_user);
            let mut counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for cats in categories_map.values() {
                for c in cats {
                    *counts.entry(c.clone()).or_insert(0) += 1;
                }
            }

            if let Ok(json) = serde_json::to_string_pretty(&counts) {
                println!("{json}");
            }
        }
        _ => {
            println!(
                "{}",
                serde_json::json!({
                    "name": "Depressurizer Internal CLI",
                    "usage": "depressurizer --cli <subcommand> [args]",
                    "subcommands": {
                        "detect": "Detect Steam install path and enumerate local user accounts",
                        "list-games [--user <id3>]": "List installed games and assigned categories",
                        "list-categories [--user <id3>]": "List categories and game counts",
                        "autocat [--user <id3>] [--rules <file>]": "Run AutoCat rules",
                        "sync [--user <id3>] [--backup]": "Write categories to Steam sharedconfig.vdf",
                        "help": "Show this help output"
                    }
                })
            );
        }
    }

    true
}
