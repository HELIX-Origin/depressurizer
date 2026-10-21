# HANDOFF.md — depressurizer Session Handoff

**Last updated:** 2026-10-18
**Current session status:** Stable · Quality gate green (4/4 tests passed, svelte-check 0 errors/0 warnings)
**Project:** depressurizer
**Root:** projects/depressurizer

> Permanent, git-tracked session-state ledger for `depressurizer`. Maintained **incrementally** — refresh at every
> workstream state change, fully rewrite at session end. New session: read this file right after `AGENTS.md`,
> migrate every item into `ROADMAP.md` / `PLAN.md` / `TODO.md` / `BUGS.md`, then **refresh this file**.
> **Never delete it.**

---

## 1) [depressurizer] W.04: Bottom Dock Dropdown & UI Refinement (Delivered)
- **State:** Bottom action dock dropdown, dynamic filter execution, multiselect workbench table, and accessible modals delivered.
- **Delivered:**
  - `src/routes/+page.svelte`: bottom `<select id="autocat-filter-select">` with prebuilt filters (HowLongToBeat, Genres, Tags, Years, Scores, VR, Dev/Pub, Platforms) and custom rules.
  - `src/routes/+page.svelte`: `Apply AutoCat` button linked directly to filter selection; dynamic `Only selected` scoping checkbox; `⚙️ AutoCat Studio…` button.
  - `src/styles/depressurizer.scss` and `src/styles/_components.scss`: docked action bar styling, preset buttons, and guide cards.
- **Ledgers:** `TODO.md` (W.04), `PLAN.md` (Sprint 4)

---

## 2) [depressurizer] W.03: AutoCat Studio, Dynamic Filters & Regex Engine (Delivered)
- **State:** Custom filter creation, dynamic category auto-naming, and regex capture group extraction delivered.
- **Delivered:**
  - `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`: added `regex = "1"`.
  - `src-tauri/src/steam/models.rs`: `AutoCatFilterType::Regex` and `.env` loader candidate paths.
  - `src-tauri/src/steam/storage.rs`: SQLite persistence and `.env` fallback.
  - `src-tauri/src/steam/autocat.rs`: dynamic target category derivation (`⚡ Dynamic (Auto)`) and regex matching with capture group auto-naming.
  - `src-tauri/src/steam/autocat.rs`: unit tests `test_autocat_dynamic_categories_and_prefix` and `test_autocat_regex_filter`.
- **Ledgers:** `TODO.md` (W.03), `PLAN.md` (Sprint 3)

---

## 3) [depressurizer] W.05: Multi-Platform Packaging & GitHub CI Pipeline (Active / Next)
- **State:** Ready for CI workflow setup and installer packaging validation.
- **Next tasks:**
  - [ ] Validate NSIS multi-user installer on Windows runner.
  - [ ] Create `.github/workflows/ci.yml` verifying Rust and frontend across Windows, macOS, and Linux runners.
  - [ ] Verify zero-warning gate in CI build matrix.
- **Ledgers:** `TODO.md` (W.05), `PLAN.md` (Sprint 5)

---

## 4) Tooling & Quality Gate
- Rust backend: `cargo test` in `src-tauri/` (4 passed, 0 failed, 0 warnings).
- Frontend: `npm run check` (`svelte-check`) in repository root (0 errors, 0 warnings).
- Development `.env`: Supported locally for testing; never committed.

---

## Quick Resume Checklist (New Session)
1. Read `AGENTS.md`
2. Read this file immediately, then `ROADMAP.md` → `PLAN.md` → `TODO.md` → `BUGS.md`
3. Migrate this file's open items into the ledgers, then refresh it
4. Resume at W.05 packaging and CI workflow in `projects/depressurizer`
