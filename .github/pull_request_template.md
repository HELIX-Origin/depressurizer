## Description

Provide a clear and concise summary of what this change achieves, the motivation behind it, and the problems it solves.

---

## Type of Change

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📝 Documentation update
- [ ] 🎨 Style / refactor / performance improvement
- [ ] 🔧 Toolchain, build, or CI configuration

---

## Related Issues

Closes #<issue-number>

---

## Verification & Quality Gates

- [ ] `npm run verify` passed (0 errors, 0 warnings)
- [ ] `npm run test:all` passed (Rust + frontend + build), if touching Rust/build
- [ ] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean, if touching Rust
- [ ] Manual desktop verification of the changed behavior
- [ ] `CHANGELOG.md` updated if behavior changed

---

## Checklist

- [ ] My changes follow the project's [contributing guidelines](../CONTRIBUTING.md).
- [ ] My changes follow the [Code of Conduct](../CODE_OF_CONDUCT.md).
- [ ] No Steam Web API keys, tokens, `.env` files, or machine paths are included.
- [ ] No new dependency on local Steam client files or the Steam client process was introduced.
- [ ] Documentation has been updated if needed.
