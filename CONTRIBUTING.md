# Contributing to Depressurizer

Thanks for your interest in improving **Depressurizer**! This guide covers development setup,
conventions, the quality gate, and how to submit changes.

> By participating you agree to our [Code of Conduct](./CODE_OF_CONDUCT.md). For security issues,
> follow [SECURITY.md](./SECURITY.md) — **do not** open a public issue.

Depressurizer is a cross-platform [Tauri v2](https://tauri.app) desktop application (Rust backend +
Svelte 5 frontend) that organizes a Steam library through the **official Steam Web API**. It is a
revival fork of the original Depressurizer codebases, re-architected away from the legacy .NET
WinForms application.

---

## Table of Contents

1. [Ways to Contribute](#ways-to-contribute)
2. [Development Setup](#development-setup)
3. [Project Layout](#project-layout)
4. [Quality Gate](#quality-gate)
5. [Coding Conventions](#conventions)
6. [Commit Style](#commit-style)
7. [Pull Request Process](#pull-request-process)
8. [Getting Help](#getting-help)

---

## Ways to contribute

- 🐛 **Report bugs** via [GitHub Issues](https://github.com/HELIX-Origin/depressurizer/issues) (bug form).
- ✨ **Propose features** via [GitHub Issues](https://github.com/HELIX-Origin/depressurizer/issues) (feature form).
- 🧩 **Request AutoCat rules** via the dedicated AutoCat request form.
- 📝 **Improve documentation** in the centralized docs/wiki hubs.
- 🔧 **Submit code** via pull request.
- 👀 **Review pull requests** from other contributors.

Open an issue or comment on an existing one before starting significant work, so the approach can be
discussed first.

## Development setup

Prerequisites:

- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://rustup.rs/) (stable, 1.80+)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform

```bash
npm install
npm run tauri:dev      # launches the Tauri desktop app in dev mode
```

The Rust build embeds the compiled frontend, so `npm run build` must run before a fresh `cargo`
command (otherwise Tauri's `frontendDist` (`build/`) does not exist yet).

### Architecture notes

- **Client-free**: the app talks to the official **Steam Web API** using a user-provided Web API key
  and SteamID64. It never parses local VDF files and never interacts with the Steam client process.
- The app's state is held in a local SQLite database; long-running work (library sync, AutoCat) runs
  on a Tokio worker and streams progress to the Svelte frontend over Tauri events.
- Never introduce a hard dependency on local Steam files or the desktop Steam client.

## Project layout

```
src/                 Svelte 5 frontend (routes, components, lib)
src-tauri/           Rust backend (Tauri v2)
  src/               commands, steam API client, models, worker
  capabilities/      Tauri v2 ACL capability files
  tauri.conf.json    Tauri configuration
scripts/             build / test / sign helpers
legacy/              original .NET/C# codebase (reference only — not built)
```

The `legacy/` directory is retained **for historical reference only**; it is not part of the build
and must not be wired into the app.

## Quality gate

Everything below must pass with **zero errors and zero warnings** before a pull request is merged:

```bash
npm run verify         # svelte-check + cargo check/clippy/test + vite build
```

For the full gate (formatting and a fresh production build too):

```bash
npm run test:all
```

Warnings are treated as errors — fix the code rather than suppressing the warning.

## Conventions

- **Rust**: `cargo fmt` is mandatory; `cargo clippy -- -D warnings` must be clean. No `dead_code`
  or unused imports.
- **Frontend**: TypeScript everywhere; `svelte-check` must report 0 errors and 0 warnings.
- **Styling**: SCSS-first — put styles in the SCSS partials under `src/`, not inline.
- Keep changes minimal and focused; match the surrounding style.
- Update `CHANGELOG.md` when behavior changes.

## Commit style

We use [Conventional Commits](https://www.conventionalcommits.org/) with concise, imperative
subjects:

```
<type>(<scope>): <short summary in imperative mood>
```

Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `security`, `build`.

## Pull request process

1. **Open an issue first** for non-trivial changes.
2. **Create a branch** from the default branch.
3. **Make focused commits** following the commit style above.
4. **Run the quality gate** (zero errors, zero warnings).
5. **Update docs/changelog** if behavior changed.
6. **Open a PR** and complete the template.
7. **Respond to review feedback** and keep the branch up to date.

Do not push directly to the default branch. Releases are cut by maintainers (see
[GOVERNANCE.md](./GOVERNANCE.md)).

## Getting help

See [SUPPORT.md](./SUPPORT.md).
