# depressurizer

Native Windows Steam library categorizer and AutoCat manager — revival fork.
Pure native C++ desktop application targeting Windows exclusively. MIT/GPL-3.0 copyleft license strictly preserved.

## Architecture at a glance

- **Native Windows C++ Application:** Tauri v2 rewrite abandoned in favor of maintaining, fixing, and modernizing the native Windows desktop application with backwards compatibility spanning Windows 2000 / XP to Windows 11+.
- **UI Modernization:** Polishing native UI design, eliminating clunkiness, and introducing modern Windows 11 styling (dark mode, clean typography, refined layout).
- **AutoCat Engine:** Pattern-matching and tag-based automated category generator for Steam libraries.
- **Steam Integration:** Robust Valve Data Format (VDF) reader/writer and key-only Steam Web API integration.
- **Copyleft Invariant:** GPL-3.0 strictly preserved across all source files and documentation.

## Build & test

| Command / Tool | Purpose |
|---|---|
| MSVC `msbuild` / `cmake` | Native Windows compilation (`/W4 /WX`) |
| Native test runner | Unit tests for AutoCat rule evaluation and VDF parsing |

## Conventions (summary)

- **Platform:** Pure Windows desktop execution; no cross-platform overhead.
- **C++:** C++20 standard; MSVC warning level `/W4` with warnings treated as errors (`/WX`).
- **GPL-3.0:** Upstream copyleft license is non-negotiable and strictly retained.
- **Safety:** Always create timestamped backups of Steam config files (`sharedconfig.vdf`) before writes.

See `.agents/rules/local-conventions.md` and `.agents/rules/verification.md` for complete guidelines.

## Invariants

GPL-3.0 license · SemVer releases · zero-warning builds · community-health files · safe VDF handling.
