# PhotoCraft Companion v2.2.2

[![GitHub Release](https://img.shields.io/badge/Release-v2.2.2-green.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.2.2)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

> [!NOTE]
> **Universal Compatibility & Lightweight Binaries / Универсальная совместимость и легковесные бинарники:**
> This release brings full universal CPU compatibility, eliminating `0xC000001D` (`STATUS_ILLEGAL_INSTRUCTION`) crashes on older processors without AVX2 (Intel Sandy Bridge Core i7-2600, Ivy Bridge, legacy AMD), and reduces executable size from 65 MB to 14 MB! Successfully verified on real Sandy Bridge hardware in [Issue #16](https://github.com/kobaltgit/photocraft-companion/issues/16).
> 
> *Этот релиз обеспечивает полную аппаратную совместимость, устраняет краш `0xC000001D` на старых процессорах без AVX2 (Intel Sandy Bridge Core i7-2600, Ivy Bridge, старые AMD) и уменьшает размер исполняемого файла с 65 МБ до 14 МБ! Успешно протестировано на реальном оборудовании в [Issue #16](https://github.com/kobaltgit/photocraft-companion/issues/16).*

---

## 🇬🇧 English: What's New in v2.2.2

### 1. Fix for Illegal Instruction Crash on Older CPUs ([Issue #16](https://github.com/kobaltgit/photocraft-companion/issues/16))
* **The Problem:** Previously, PhotoCraft Companion crashed immediately within 1–2 seconds of launch with exit code `0xC000001D` (`STATUS_ILLEGAL_INSTRUCTION`) on processors lacking AVX2 instructions (such as 2nd-gen Intel Core Sandy Bridge i7-2600). This occurred because ONNX Runtime was statically compiled and linked using an `x86-64-v3` baseline (mandatory AVX2).
* **The Solution (SPEC-005):**
  * Switched the `ort` engine from static linkage (`download-binaries`) to dynamic loading (`load-dynamic`).
  * `photocraft-companion.exe` is now compiled as a clean, portable baseline `x86-64` binary that launches reliably on any 64-bit CPU.
  * Executable binary size decreased dramatically from 65 MB down to **14 MB**!
  * ONNX Runtime is now loaded dynamically via official Microsoft `onnxruntime.dll` equipped with MLAS CPUID runtime dispatch, automatically choosing AVX1/SSE4.2 math routines on older CPUs and AVX2/AVX-512 on modern hardware.
  * Added `crates/ai_engine/src/loader.rs` with hardware instruction detection (`is_avx_supported()`, `is_avx2_supported()`).
  * Verified and confirmed working on real Intel Core i7-2600 hardware by the community!

---

## 🇷🇺 Русский: Что нового в версии 2.2.2

### 1. Ликвидация аппаратного краша 0xC000001D на старых процессорах ([Issue #16](https://github.com/kobaltgit/photocraft-companion/issues/16))
* **В чём была проблема:** Ранее при запуске Компаньона на процессорах без инструкций AVX2 (в частности Intel Core i7-2600 Sandy Bridge 2011 года) процесс мгновенно падал через 1–2 секунды с кодом ошибки `0xC000001D` (`STATUS_ILLEGAL_INSTRUCTION`). Причиной была статическая запечка C++ монолита ONNX Runtime, скомпилированного под микроархитектуру `x86-64-v3` (принудительный AVX2).
* **Решение (SPEC-005):**
  * Движок переведен со статической линковки на динамическую загрузку `load-dynamic`.
  * `photocraft-companion.exe` теперь скомпилирован как чистый x86-64 бинарник без AVX2-инструкций в основном коде.
  * **Размер исполняемого файла снижен с 65 МБ до 14 МБ!**
  * Библиотека `onnxruntime.dll` теперь подгружается динамически: на современных ПК задействуются AVX2 и DirectML, а на Sandy Bridge — инструкции AVX1 и SSE4.2 через внутренний диспетчер Microsoft MLAS.
  * Создан компактный загрузчик `crates/ai_engine/src/loader.rs` с определением инструкций процессора.
  * Успешно протестировано и подтверждено на реальном компьютере с Intel Core i7-2600 сообществом!
