# Changelog

All notable changes to **Depressurizer** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Release history for this
revival fork begins at **v0.8.0**; no earlier tag exists because the project was re-architected from
the ground up.

## [Unreleased]

### Added

- Cross-platform desktop application built on **Tauri v2** (Rust backend) and **Svelte 5**.
- **Client-free Steam integration** using the official **Steam Web API** with a user-provided Web API
  key and SteamID64 — no local VDF parsing and no Steam client process interaction.
- Library sync for both installed and uninstalled games, with automatic paging.
- **AutoCat** rule engine with multi-rule filters and dry-run preview.
- **Live AutoCat execution feedback**: the Tokio worker streams per-game progress to the UI, with
  matching per-game category updates and a progress indicator while running.
- Collections read/write through the Steam Web API, including Steam Cloud sync.
- Workbench table multiselect with batch category assign/remove, favorite toggle, and hidden toggle.
- Backup and restore of category data.
- **HowLongToBeat** playtime enrichment via the current, active HLTB API.
- Tauri v2 **capabilities** (ACL) configuration scoping IPC to the main window.
- Headless CLI mode for library listing and AutoCat execution.

### Changed

- Re-architected the legacy .NET WinForms application into a cross-platform Tauri v2 desktop app.
- Library discovery and collection writes now go exclusively through Valve's official Web API.

### Removed

- Legacy local VDF file parsing and Steam client registration/launch methods (abandoned in favour of
  the client-free Web API architecture).

### Security

- Steam Web API key and SteamID64 are stored locally only and transmitted **only** to Valve's
  official endpoints.
- Added a Tauri v2 capability manifest to restrict IPC surface to the main window.

## Planned

- The first tagged release will be **`v0.8.0`**.

[Unreleased]: https://github.com/HELIX-Origin/depressurizer/commits/main
