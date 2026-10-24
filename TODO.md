# depressurizer — Task Checklist & Workstream Tracking

> 📋 **Living Source of Truth**: Actionable task ledger for **depressurizer**.
> Statuses: ⬜ backlog · 🚧 in progress · ✅ done (tracked as `- [ ]` and `- [x]` checkboxes).
> High-level direction lives in [`ROADMAP.md`](./ROADMAP.md), active sprints in [`PLAN.md`](./PLAN.md), issues in [`BUGS.md`](./BUGS.md).

---

## 📜 Tracking Rules

- **100% Active Direction Accuracy**: Task items and checklists must reflect the current state of Depressurizer with zero contradictions.
- **Normalized Directives**: Rewrite user directives into clean, self-contained engineering requirements.
- **Done Moves Down**: Completed items are marked with `- [x]` and collapsed to minimal one-line entries under completed workstreams.
- **Zero-Warning Gate**: All builds and tests must pass with 0 errors and 0 warnings (`MSVC /W4 /WX`).

---

## 🔥 Active Workstreams

### 🎯 [depressurizer] Workstream W.05: Native Windows C++ Modernization & UI Polish (v1.0.0)
- **Project Identifier:** `[depressurizer]`
- **Repository Path:** `projects/depressurizer`
- **Status:** 🚧 In progress. Abandon Tauri v2 rewrite; focus 100% on fixing the native C++ app and polishing UI.
- **Target Component:** Native C++ Core & Win32 UI (GPL-3.0)

**Locked user directives (normalized by domain):**
- **Abandon Tauri Rewrite**: Cease development on the experimental Tauri v2 / Rust / SvelteKit rewrite.
- **Native C++ Windows Core**: Focus on maintaining, auditing, and fixing the already existing native C++ application for Windows.
- **UI Modernization**: Polish up the visual ugliness and clunkiness of the legacy interface. Implement modern Windows 11 styling (dark mode, refined typography, responsive proportions, clean controls).
- **Steam Integration Safety**: Ensure safe VDF reading and writing with mandatory automatic timestamped backups before saving.
- **Licensing Invariant**: Strictly preserve GPL-3.0 across all files.

**Implementation checklist:**
- [ ] **[depressurizer] 1. Native C++ Core Audit & Modernization**:
  - [ ] `[depressurizer]` Audit native codebase; resolve legacy build warnings and deprecations under MSVC C++20.
  - [ ] `[depressurizer]` Fix existing bugs in AutoCat rule evaluation and Steam profile detection.
  - [ ] `[depressurizer]` Ensure memory safety and RAII handle management across all modules.
- [ ] **[depressurizer] 2. Steam VDF & Web API Hardening**:
  - [ ] `[depressurizer]` Validate and modernize Valve Data Format parser for modern Steam config layouts.
  - [ ] `[depressurizer]` Enforce automatic timestamped backup of `sharedconfig.vdf` before writes.
  - [ ] `[depressurizer]` Ensure key-only Steam Web API integration operates cleanly with proper timeouts.
- [ ] **[depressurizer] 3. Native UI Redesign & Polish**:
  - [ ] `[depressurizer]` Redesign legacy forms, tables, and dialogs to eliminate visual clunkiness and ugliness.
  - [ ] `[depressurizer]` Implement modern Windows 11 dark mode theme support.
  - [ ] `[depressurizer]` Refine category workbench, batch actions, and rule editor ergonomics.
- [ ] **[depressurizer] 4. Quality Gate & Packaging**:
  - [ ] `[depressurizer]` Enforce MSVC `/W4 /WX` (zero warnings treated as errors).
  - [ ] `[depressurizer]` Author native Windows installer (NSIS/MSI) and release packaging.

---

## ✅ Completed Workstreams

- **[depressurizer]** W.04 — Workbench Dock & Filter Architecture: Prototyped filter selection and AutoCat rule structures.
- **[depressurizer]** W.03 — AutoCat Studio & Regex Engine: Rule tree definitions and dynamic category naming.
- **[depressurizer]** W.02 — Steam API & Database Persistence: Web API library retrieval and metadata caching.
- **[depressurizer]** W.01 — Project Foundation & Governance: Project repository initialization and GPL-3.0 governance setup.
