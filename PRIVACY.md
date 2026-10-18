# Privacy Policy

**Last updated:** 2026-10-18 · **Version 0.8.0**

**Depressurizer** ("the App") is built around one principle: **your data stays on your machine.** This
policy explains what the App stores, what it transmits, and what it never collects. Short version:
**no account with us, no telemetry, no publisher servers, no third-party trackers.**

---

## 💾 1. What the App stores (locally)

All persistent data is stored **only on your own device**:

| Data | Where | Purpose |
| --- | --- | --- |
| Steam Web API key & SteamID64 | Local app configuration / local SQLite database | Authenticate calls to Valve's official Steam Web API |
| Game library cache | Local SQLite database | Offline browsing and fast startup |
| AutoCat profiles and rules | Local SQLite database | Categorization automation |
| Category assignments, favorites, hidden flags | Local SQLite database (synced to Steam only when you choose to save) | Library organization |

Nothing listed here is uploaded to the publisher. There is no account on any server operated by us.

## 🌐 2. What the App transmits

The App contacts only the hosts required for its documented features:

- **Valve / Steam** — the official **Steam Web API** (`api.steampowered.com`) and Steam
  store/community endpoints, authenticated with **your** Steam Web API key and SteamID64. This is how
  the App reads your owned games and writes categories/collections. Your key is sent only to Valve.
- **HowLongToBeat** (optional) — playtime enrichment for games in your library. Only non-personal
  game titles are sent; no account data.
- **Steam images/CDN** — game artwork and metadata thumbnails.

The App contacts **no other hosts** and contains no advertising, analytics, or crash-reporting SDKs.

## 🚫 3. What the App never does

- No telemetry or analytics by default.
- No accounts created on any publisher server.
- No selling or sharing of personal data.
- No tracking across other applications or websites.
- No access to your Steam password or Steam Guard codes — authentication uses a Web API key you
  generate yourself.

## 🔑 4. Your Steam Web API key

You provide your own official Steam Web API key. It is stored locally, sent **only** to Valve's
endpoints, and is never transmitted to the publisher or any third party. You can revoke it at any
time from your [Steam account page](https://steamcommunity.com/dev/apikey).

## 🗑️ 5. Removing your data

- Delete your library cache, AutoCat profiles, and settings from within the App's settings.
- Uninstalling the App removes its local data directory; you may also delete the local database file
  manually.

## 📬 6. Contact

Questions? Open an issue or discussion on the
[GitHub repository](https://github.com/HELIX-Origin/depressurizer).

## 🔄 7. Changes

We may update this policy; the current version always lives in this repository as `PRIVACY.md`.
