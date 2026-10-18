# Security Policy

We take the security of **Depressurizer** seriously — for the application itself and for the privacy
of your Steam data and credentials.

## 🛡️ Supported versions

| Version | Supported |
| --- | --- |
| 0.8.x | ✅ Active |
| < 0.8 | ❌ No longer supported |

Only the latest release receives security fixes. See the
[Releases page](https://github.com/HELIX-Origin/depressurizer/releases) for the current version.

## 🚨 Reporting a vulnerability

**Do not open a public issue** for a security vulnerability.

Report privately using GitHub's **private vulnerability reporting**:

- Repository → **Security** → **Report a vulnerability**, or
- Directly: `https://github.com/HELIX-Origin/depressurizer/security`

Please include:

1. The affected version(s) and platform.
2. A clear description of the vulnerability.
3. Minimal steps to reproduce.
4. Impact assessment and — if you have one — a suggested fix.

We aim to acknowledge reports within **5 business days** and to ship a fix for confirmed issues in a
reasonable window depending on severity. Security fixes are noted in
[CHANGELOG.md](./CHANGELOG.md).

## 🎯 Scope

In scope (examples):

- Code execution, path traversal, or arbitrary file writes via the app, installer, or uninstaller.
- Injection or data corruption via the Rust backend, the local database, or the WebView bridge.
- Unsafe handling of untrusted input (remote metadata, local file paths) that breaks the app or
  leaks data.
- Leakage or improper storage of the user's Steam Web API key or SteamID64.
- Insecure release pipeline (tampered binaries, malicious installers).

Out of scope (examples):

- Issues that fundamentally stem from Valve's Steam Web API, the Steam client, or another third-party
  service the app consumes.
- The Steam client overwriting library changes made by the app (a known desktop-client limitation —
  see the troubleshooting note below).
- Reports about Steam's own API or website.

## 🔐 Security design notes

- **Local-first**: no telemetry; your library and settings live only on your machine.
- **API key handling**: your Steam Web API key and SteamID64 are stored **only** in the local
  configuration/database on your machine and are sent **only** to Valve's official endpoints. Never
  commit keys, `.env` files, or credential material; the repository `.gitignore` excludes them.
- **Client-free & read/write via API**: the app does not parse local VDF files and never drives the
  Steam client process.
- **No remote code paths**: the UI renders in a CSP-restricted WebView (Tauri v2).

## ⚠️ Known desktop flakiness (not a vulnerability)

Steam's desktop client can overwrite library/collection changes made through the Web API while it is
running — a known quirk on Valve's side, not an app defect. If categorization does not appear to
stick, fully exit the Steam client, apply changes, then start it again. This is a troubleshooting
note only; the app performs no Steam-client detection.
