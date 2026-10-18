<div align="center">

<img src="./static/icon.svg" alt="Depressurizer Logo" width="128" height="128" />

# Depressurizer

<p align="center">
  <a href="https://github.com/HELIX-Origin/depressurizer/releases"><img src="https://img.shields.io/github/v/release/HELIX-Origin/depressurizer?include_prereleases&label=Release&color=blue" alt="Release" height="20" /></a>
  <a href="https://docs.helix-origin.club/depressurizer/"><img src="https://img.shields.io/badge/Docs-docs.helix--origin.club-brightgreen" alt="Documentation" height="20" /></a>
  <a href="https://wiki.helix-origin.club/depressurizer/"><img src="https://img.shields.io/badge/Wiki-wiki.helix--origin.club-brightgreen" alt="Technical Wiki" height="20" /></a>
  <a href="https://www.gnu.org/licenses/gpl-3.0"><img src="https://img.shields.io/badge/License-GPL--3.0-blue.svg" alt="License: GPL-3.0" height="20" /></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/Tauri-v2-blue?logo=tauri" alt="Tauri v2" height="20" /></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/Svelte-5-orange?logo=svelte" alt="Svelte 5" height="20" /></a>
</p>

**A fast, modern, cross-platform Steam library categorizer and AutoCat manager — powered by Tauri v2.**

Auto-categorize, filter, backup, and organize your entire Steam game library (installed and uninstalled). Owned games come from the official **Steam Web API** (Web API key + SteamID64); collections are read and written only through the desktop Steam client's `sharedconfig.vdf`, with multi-rule AutoCat filters and zero UI hangs.

**Platforms:** Windows, macOS, and Linux desktops. Android and iOS are not supported, because Steam collections are managed only by the desktop Steam client.

*Depressurizer is an active revival fork of the original [rallion/depressurizer](https://github.com/rallion/depressurizer) and [mvegter/Depressurizer](https://github.com/mvegter/Depressurizer) codebases, re-architected from legacy .NET WinForms into a cross-platform desktop application.*

</div>

---

## 📖 Primary Gateways

- 📖 **Documentation**: [docs.helix-origin.club/depressurizer](https://docs.helix-origin.club/depressurizer/) — setup guides, AutoCat rules, Steam Web API keys, and backup management.
- 🧠 **Technical Wiki**: [wiki.helix-origin.club/depressurizer](https://wiki.helix-origin.club/depressurizer/) — Steam Web API integration, Tokio worker architecture, and CLI automation.
- 🌐 **Organization Portal**: [www.helix-origin.club](https://www.helix-origin.club/) — HELIX Origin organization hub.

---

## ⚡ Quick Start

### Prerequisites
- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://www.rust-lang.org/) (1.80+)

### Running Locally
```bash
npm install
npm run tauri dev
```

### Headless CLI
```bash
# List all games discovered via the Steam Web API
depressurizer --cli list-games

# Execute AutoCat categorization rules headlessly
depressurizer --cli autocat --rules rules.json
```

### Verification & Quality Gate
```bash
npm run verify
```

> ⚠️ **Steam client note:** because Steam's desktop client can overwrite library changes made
> through the Web API, fully exit the Steam client before applying changes and start it again
> afterwards. This is a known quirk on Valve's side, not a defect in Depressurizer.

---

## 🤝 Community

- [Contributing](./CONTRIBUTING.md) · [Code of Conduct](./CODE_OF_CONDUCT.md) · [Support](./SUPPORT.md)
- [Security Policy](./SECURITY.md) · [Privacy Policy](./PRIVACY.md) · [Terms of Service](./TOS.md)
- [Changelog](./CHANGELOG.md) · [Governance](./GOVERNANCE.md) · [License](./LICENSE)

---

## ⚖️ License

[GPL-3.0-or-later](LICENSE) — Copyright (c) 2012-2020 rallion & mvegter; Copyright (c) 2026, HELIX Origin.
