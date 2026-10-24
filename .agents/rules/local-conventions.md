# Local Conventions — depressurizer (Native Windows C++)

> Authoritative conventions for `depressurizer` (GPL-3.0).
> Native C++ Windows desktop application targeting Windows exclusively (Tauri v2 abandoned).

---

## 1. Architecture & Platform

- **Platform Target:** Windows exclusively with progressive enhancement from **Windows 2000 (or XP)** up to **Windows 11+**. No cross-platform abstractions.
- **Language Standard:** C++20 standard; MSVC compiler.
- **Warning Level:** MSVC `/W4` with warnings treated as errors (`/WX`). Zero warnings invariant strictly enforced.

---

## 2. License Invariant (GPL-3.0)

- **GPL-3.0 strictly preserved:** Upstream revival fork. Every source file header must retain the GPL-3.0 notice. Never relicense any file.
- Any dependency added must be compatible with GPL-3.0.

---

## 3. Steam VDF & File Safety

- **Mandatory Backup:** Always create timestamped backup of `sharedconfig.vdf` before saving modifications.
- **Parser Robustness:** Ensure safe, robust VDF parsing handling current Steam formats without memory leaks.
- **Key-Only Web API:** Never call Steam Web API with user credentials — key-only, read-only endpoints only.

---

## 4. UI Modernization Standards

- Eliminate visual ugliness, clunky borders, and cramped legacy Win32 layouts.
- Modern Windows 11 Fluent design: dark mode support (`DwmSetWindowAttribute`), clean typography, refined margins, intuitive table multiselect.
