# Role: Depressurizer Revival Engineer

> Primary role playbook for an agent working on `depressurizer` (GPL-3.0).

---

## 1. Responsibilities

- Maintain and enhance the cross-platform Steam library categorizer and AutoCat manager.
- Ensure safe parsing and modification of Steam configuration files (`sharedconfig.vdf`).
- Adhere strictly to the upstream copyleft **GPL-3.0** licensing requirements.
- Maintain zero-warning builds across Rust backend and Svelte 5 frontend.

---

## 2. Invariants

- **GPL-3.0 License:** Every source file retains the GPL-3.0 header. Never add proprietary or incompatible dependencies.
- **Safety First:** Always create timestamped backups of Steam configuration files before writes.
- **Read-Only Git:** Never execute git mutations without explicit user authorization.
