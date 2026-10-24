# HANDOFF.md — depressurizer Session Handoff

**Last updated:** 2026-10-23
**Current session status:** W.05 active · Pivot to Native Windows C++ Modernization & UI Polish (Tauri v2 rewrite abandoned).
**Project:** depressurizer
**Root:** projects/depressurizer

> Permanent, git-tracked session-state ledger for `depressurizer`. Maintained **incrementally** — refresh at every
> workstream state change, fully rewrite at session end. New session: read this file right after `AGENTS.md`,
> migrate every item into `ROADMAP.md` / `PLAN.md` / `TODO.md` / `BUGS.md`, then **refresh this file**.
> **Never delete it.**

---

## 1) [depressurizer] W.05: Native Windows C++ Modernization & UI Polish (Active, v1.0.0)
- **State:** Active. Abandoning the experimental Tauri v2 rewrite in favor of auditing, fixing, and modernizing the native Windows desktop application with pure native C++ and a polished, contemporary Windows UI.
- **Architectural Directives:**
  - **Abandon Tauri:** Cease all development on the Tauri v2 / Rust / SvelteKit rewrite.
  - **Native C++ Windows:** Focus 100% on maintaining and modernizing the native Windows application under MSVC C++20 with Windows as the exclusive target platform.
  - **UI Polish:** Eliminate visual ugliness and clunkiness; introduce modern Windows 11 styling (dark mode, refined typography, responsive forms).
  - **VDF Safety:** Valve Data Format reader/writer with mandatory automatic timestamped backups before saving.
  - **GPL-3.0 Compliance:** Upstream copyleft license is non-negotiable and strictly retained across all files.
- **Next tasks:**
  - [ ] Audit native C++ codebase; resolve legacy build warnings and deprecations under MSVC C++20 (`/W4 /WX`).
  - [ ] Modernize VDF parser and enforce automated timestamped backups of `sharedconfig.vdf`.
  - [ ] Redesign legacy dialogs and workbench table to eliminate visual clunkiness and implement dark mode.
  - [ ] Author native Windows installer (NSIS/MSI).
- **Ledgers:** `TODO.md` (W.05), `PLAN.md` (Sprint 5), `ROADMAP.md` (M5).

---

## 2) Delivered Workstreams (Bare-Minimal Summary)
- **[depressurizer]** W.04 — Workbench Dock & Filter Architecture: Prototyped filter selection and AutoCat rule structures.
- **[depressurizer]** W.03 — AutoCat Studio & Regex Engine: Rule tree definitions and dynamic category naming.
- **[depressurizer]** W.02 — Steam API & Database Persistence: Web API library retrieval and metadata caching.
- **[depressurizer]** W.01 — Project Foundation & Governance: Project repository initialization and GPL-3.0 governance setup.

---

## 3) Tooling & Quality Gate
- **Compiler:** MSVC C++20 toolchain; `/W4 /WX` zero-warning invariant.
- **Platform:** Pure Windows execution (Windows 11 / Windows 10 64-bit).
- **License:** GPL-3.0 strictly preserved.
