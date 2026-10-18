# Governance

This document describes how **Depressurizer** (a
[HELIX Origin](https://github.com/HELIX-Origin) project) is governed.

## Roles

| Role | Responsibilities |
| --- | --- |
| **Maintainers** | Set direction, review and merge pull requests, manage releases and tags, triage issues, and uphold the Code of Conduct. |
| **Contributors** | Submit issues, discussions, documentation, and pull requests under the project's license and contribution guidelines. |
| **Users** | File bug reports and feature requests and follow the project's [Code of Conduct](./CODE_OF_CONDUCT.md). |

## Decision-making

- Day-to-day decisions happen in issues and pull requests through discussion and review.
- Significant changes (architecture, data handling, or the Steam Web API integration model) should be
  raised as an issue or discussion before implementation.
- Maintainers seek rough consensus; where consensus cannot be reached, maintainers decide by majority
  and record the rationale on the relevant issue or pull request.

## Releases

- Releases follow [Semantic Versioning](https://semver.org/) (`vMAJOR.MINOR.PATCH`) and receive
  human-written release notes.
- Only maintainers cut releases and push tags.
- All notable changes are recorded in [CHANGELOG.md](./CHANGELOG.md).

## Contributions

Contributions are governed by [CONTRIBUTING.md](./CONTRIBUTING.md) and the
[Code of Conduct](./CODE_OF_CONDUCT.md). By contributing, you agree that your contributions are
licensed under the project's license (see `LICENSE`).

## Security

Security reports are handled privately; see [SECURITY.md](./SECURITY.md).

## Changes to this document

Changes require maintainer approval and are made through a pull request.
