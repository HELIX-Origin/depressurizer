# depressurizer — 🐛 Bug Tracker

> 🐛 **Living Source of Truth**: Active tracking ledger for open bugs, quirks, external limitations, and resolution history for **depressurizer**.
> Pair with [`ROADMAP.md`](./ROADMAP.md) (direction), [`PLAN.md`](./PLAN.md) (sprints), and [`TODO.md`](./TODO.md) (tasks).

---

## 📜 Tracking Rules

- **100% Active Direction Accuracy**: Reflect the current, active state of Depressurizer with zero contradictions.
- **Explicit Project Tagging**: Every entry is tagged with `[depressurizer]`.
- **Zero-Warning Gate**: Compiler warnings are treated as errors and resolved before release.

---

## 🚫 Known Quirks & External Limitations

- 🔒 **[depressurizer] Steam Store API Rate Limiting**: The public Steam Store API (`store.steampowered.com/api/appdetails`) aggressively throttles requests when querying tags and genres in bulk. Depressurizer enforces concurrency limits and caching in SQLite to prevent HTTP 429 errors.
- 🔒 **[depressurizer] Web API Collections Capability**: Valve's official public Web API provides `IPlayerService/GetOwnedGames` for full library reads, but lacks a published REST endpoint for writing custom collections. "Save to Steam" operates via the Steam Cloud storage synchronization pipeline.

---

## 💡 Explicitly Not Bugs

- 💡 **[depressurizer] Client-Free Operation**: Depressurizer deliberately does not attempt to read or modify running Steam client processes or local Windows VDF files. This is an intentional architectural decision to ensure cross-platform stability across Windows, macOS, and Linux.

---

## ⚠️ Active & Open Bugs

*(No critical open bugs currently blocking releases. All quality gate suites pass.)*

---

## ✅ Resolved Bugs & Historical Fixes

### 2026-10-18 — [depressurizer]: AutoCat Studio Required Mandatory Category Names
- **Severity**: 🟠 High
- **Status**: ✅ resolved
- **Scope**: `src-tauri/src/steam/autocat.rs`, `src/routes/+page.svelte`
- **Root Cause**: Custom rule creation previously required users to manually type a target category name, preventing dynamic auto-naming.
- **Fix**: Implemented `⚡ Dynamic (Auto)` mode deriving category names directly from matched genres, tags, years, scores, playtime brackets, platforms, or regex capture groups.

### 2026-10-18 — [depressurizer]: Bottom Dock Action Confusion
- **Severity**: 🟡 Medium
- **Status**: ✅ resolved
- **Scope**: `src/routes/+page.svelte`, `src/styles/depressurizer.scss`
- **Root Cause**: The bottom bar lacked an immediate, intuitive selector for executing prebuilt and configured filters without opening dialogs.
- **Fix**: Replaced popover with a clean `<select id="autocat-filter-select">` listing all prebuilt filters and custom rules, paired with `Apply AutoCat` and `⚙️ AutoCat Studio…` buttons.

### 2026-10-18 — [depressurizer]: Development / Testing `.env` Loading
- **Severity**: 🟡 Medium
- **Status**: ✅ resolved
- **Scope**: `src-tauri/src/steam/models.rs`, `src-tauri/src/steam/storage.rs`
- **Root Cause**: App settings strictly relied on SQLite and ignored `.env` keys, requiring manual re-entry during development sessions.
- **Fix**: Added `AppSettings::load_from_env_if_present()` checking candidate `.env` paths and process environment for `STEAM_API_KEY`, `STEAMID64`, and `STEAM_PATH`.
