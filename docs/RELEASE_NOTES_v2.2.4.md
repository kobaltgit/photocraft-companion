# PhotoCraft Companion v2.2.4

[![GitHub Release](https://img.shields.io/badge/Release-v2.2.4-green.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.2.4)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

> [!NOTE]
> **Generative AI Fill Crash Fix & Stability / Исправление вылета генеративной заливки и стабильность:**
> This release resolves a crash when running Generative AI Fill (Inpaint / LaMa) on Windows GPU via DirectML ([Issue #19](https://github.com/kobaltgit/photocraft-companion/issues/19)), eliminates canvas selection clipping during in-place pasting, fixes layer coordinate alignment, and bundles full DirectML runtime libraries in Windows distributions.
> 
> *Этот релиз устраняет аварийное завершение приложения при выполнении «Генеративной заливки» (Inpaint / LaMa) на Windows через DirectML ([Issue #19](https://github.com/kobaltgit/photocraft-companion/issues/19)), устраняет обрезку результата выделением при вставке, исправляет выравнивание координат смещённых слоёв и включает необходимые библиотеки DirectML в дистрибутив Windows.*

---

## 🇬🇧 English: What's New in v2.2.4

### 1. Generative AI Fill Crash Resolution ([Issue #19](https://github.com/kobaltgit/photocraft-companion/issues/19) / BUG-129)
* **The Problem:** The LaMa inpainting model relies on Fast Fourier Convolutions (FFC) containing complex spectral matrix operations (`convg2g / rfftn / MatMul_5`). When executed via the GPU `DmlExecutionProvider` (`DirectML.dll`), graphics drivers reject the operator configuration with `E_INVALIDARG (0x80070057)` / `0xC0000005` inside C++ driver code. Because this exception occurs outside Rust, it terminated the process instantly without triggering Rust error handling or recovery.
* **The Solution (Zero-Core Hardcoding):**
  * Added declarative `ExecutionProviderPreference` (`Auto`, `Cpu`, `DirectMl`) in `AiPipelineConfig` (`crates/core_contracts`).
  * In `plugins/inpaint/plugin.json`, declared `"execution_provider": "cpu"` under the `"ai"` section.
  * In `ai_engine`, implemented `enforce_execution_provider_preference`, directing LaMa models to highly optimized CPU AVX/AVX2 execution (~1.5s per fill), while preserving full GPU DirectML acceleration for all other AI models (Background Removal, Depth Estimation, Super-Resolution).

### 2. Selection Clipping Elimination on In-Place Pasting
* **The Problem:** When inpainting a designated area, PhotoCraft's `pasteSpecial.pasteInPlace` command clipped pasted content strictly to the boundaries of the active selection marquee.
* **The Solution:** Added an automated `client.deselect()` step in `plugin_adapter.rs` right before in-place pasting, allowing the newly synthesized inpainting patch to blend seamlessly with surrounding content without hard edge cutoffs.

### 3. Signed Coordinate Alignment for Shifted Layers
* **The Problem:** When source layers were shifted relative to canvas origin, coordinate math in selection mask generation could wrap around or misalign bounding boxes.
* **The Solution:** Migrated `copy_active_layer_and_selection_mask()` in `crates/core_contracts/src/plugin/manifest.rs` to signed arithmetic (`canvas_x = bx0 + cx as i64`, `layer_x = canvas_x - lx`), ensuring pixel-accurate sampling even for negative or shifted layer origins.

### 4. DirectML & ONNX Runtime Distribution Packaging
* Updated `scripts/package_release.ps1` to ensure both `onnxruntime.dll` and `DirectML.dll` are included in the portable `.zip` and Inno Setup Windows installer packages.

---

## 🇷🇺 Русский: Что нового в версии 2.2.4

### 1. Устранение падения «Генеративной заливки» ([Issue #19](https://github.com/kobaltgit/photocraft-companion/issues/19) / BUG-129)
* **В чём была проблема:** Модель LaMa использует быстрые преобразования Фурье (FFC) со спектральными матричными операциями (`convg2g / rfftn / MatMul_5`). При выполнении на GPU через `DmlExecutionProvider` (`DirectML.dll`) видеодрайвер возвращал `E_INVALIDARG (0x80070057)` / `STATUS_ACCESS_VIOLATION (0xC0000005)` внутри C++ кода. Так как исключение возникало на уровне нативного драйвера в обход рантайма Rust, процесс аварийно завершался без возможности перехвата ошибки.
* **Решение (без хардкода в ядре):**
  * В `crates/core_contracts` в структуру `AiPipelineConfig` добавлено декларативное перечисление `ExecutionProviderPreference` (`Auto`, `Cpu`, `DirectMl`).
  * В манифесте `plugins/inpaint/plugin.json` в секции `"ai"` объявлен параметр `"execution_provider": "cpu"`.
  * В `ai_engine` реализован метод `enforce_execution_provider_preference`, который направляет LaMa на оптимизированный CPU AVX/AVX2 инференс (~1.5–2 секунды), сохраняя аппаратное GPU-ускорение через DirectML для всех остальных моделей (RMBG, Depth Anything, RealESRGAN).

### 2. Защита от обрезки сгенерированного фрагмента границами выделения
* **В чём была проблема:** При вставке сгенерированного фрагмента через команду `pasteSpecial.pasteInPlace` PhotoCraft жестко обрезал вставляемое изображение по контуру активного выделения.
* **Решение:** В `plugin_adapter.rs` перед вставкой слоя добавлен автоматический сброс активного выделения (`client.deselect()`), гарантирующий гладкое наложение заплат без артефактов жестких границ.

### 3. Точное выравнивание координат смещённых слоёв
* **В чём была проблема:** Если слой был сдвинут относительно холста, в расчете маски выделения могли возникать смещения из-за беззнаковой арифметики.
* **Решение:** В `crates/core_contracts/src/plugin/manifest.rs` метод `copy_active_layer_and_selection_mask()` переведен на знаковую арифметику со строгой валидацией границ слоя и холста.

### 4. Комплектация библиотек DirectML и ONNX Runtime в дистрибутиве
* Скрипт `scripts/package_release.ps1` обновлен: портативный архив `.zip` и установщик Inno Setup теперь гарантированно содержат обе библиотеки `onnxruntime.dll` и `DirectML.dll`.
