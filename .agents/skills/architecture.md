# Architecture — depressurizer

> Authoritative architectural specification for `depressurizer` (GPL-3.0).
> Tauri v2 cross-platform Steam library categorizer and AutoCat manager.

---

## 1. Directory Structure

```
depressurizer/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs              # Tauri builder & command registration
│       ├── commands.rs         # IPC command handlers
│       ├── error.rs            # AppError & Result types
│       ├── steam/              # Steam library parsing & AutoCat engine
│       │   ├── mod.rs
│       │   ├── vdf.rs          # Valve Data Format (VDF) parser
│       │   ├── autocat.rs      # Automated category rule engine
│       │   └── api.rs          # Steam Web API (key-only client)
│       ├── categorizer/        # Category definitions and rule trees
│       └── platform/           # OS-specific Steam detection (Windows/macOS/Linux)
├── src/                        # SvelteKit SPA (Svelte 5 runes)
│   ├── lib/
│   │   ├── components/         # Categorizer UI components
│   │   ├── stores/             # Svelte 5 rune state
│   │   ├── styles/             # SCSS tokens and design system
│   │   └── api.ts              # Tauri invoke wrapper types
│   └── routes/                 # Pages (library, autocat rules, settings)
```

---

## 2. Key Subsystems

1. **AutoCat Engine (`src-tauri/src/steam/autocat.rs`):** Pattern-matching and tag-based automated category generator for massive Steam libraries.
2. **VDF Reader/Writer (`src-tauri/src/steam/vdf.rs`):** Parses and writes Steam's proprietary `sharedconfig.vdf` and `localconfig.vdf` safely, creating timestamped backups before modification.
3. **Steam Web API Client (`src-tauri/src/steam/api.rs`):** Key-only endpoint consumer for fetching game tags, genres, and store metadata without user credentials.
