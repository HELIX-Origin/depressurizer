# Workflows — depressurizer

> Authoritative runbooks and dev workflows for `depressurizer` (GPL-3.0).

---

## 1. Modifying AutoCat Rule Evaluator

1. Update rule structures in `src-tauri/src/categorizer/`.
2. Ensure backward compatibility with existing user-defined JSON rule exports.
3. Write unit tests covering rule precedence, regex edge cases, and negation rules.
4. Verify build with `RUSTFLAGS="-D warnings" cargo test`.

---

## 2. Handling Steam VDF File Updates

1. Always enforce backup generation before writing to `sharedconfig.vdf`.
2. Check that Steam is not actively running/writing to config files when writing.
3. Validate written VDF syntax against Steam's parser expectation.
