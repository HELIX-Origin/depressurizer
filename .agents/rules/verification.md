# Verification Gate — depressurizer (Native Windows C++)

> Authoritative verification gate for `depressurizer` (GPL-3.0).
> All compilers and linters must pass with zero errors and zero warnings (`/W4 /WX`).

---

## 1. Native Build & Test Verification

```powershell
# 1. Native C++ build — MSVC warning level /W4 with warnings treated as errors (/WX)
cmake -B build -S . -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release

# 2. Native unit test suite (AutoCat evaluation & VDF parser)
ctest --test-dir build -C Release --output-on-failure
```

---

## 2. Quality Gate Invariants

- Every compilation step must produce **0 errors and 0 warnings** under MSVC `/W4 /WX`.
- All AutoCat rule parsing and VDF read/write tests must pass cleanly.
- License compliance: Every modified/new source file must carry the GPL-3.0 header.
