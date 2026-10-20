# Release Notes: PhotoCraft Companion v1.2.0

**Release Tag:** `v1.2.0`  
**Date:** 2026-10-20  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.2.0** — an extensible, open-source companion application and autonomous plugin runtime designed specifically for the **PhotoCraft** graphics editor. 

This release introduces the **Universal Workflow Engine**, **Photoshop-Style Tool Variations (Sub-Actions)**, up to **25x faster AI upscaling** via warm DirectML GPU session tiling, and critical bug fixes for macro pipelines.

---

### 🌟 Key Highlights & Major Additions

#### 1. Universal Declarative Workflow Engine (`workflow_engine`)
* **Dynamic Context Variables:** Multi-step command pipelines can now interpolate real-time document context: `$source_layer`, `$result_layer`, `$active_layer`, `$doc_width`, `$doc_height`, `$has_selection`, `$fg_color`, `$bg_color`, and string interpolation `{{...}}`.
* **Structured Steps:** Each step supports `save_as` to capture layer IDs for subsequent steps, `when` conditional branching, and `ignore_errors` for resilient execution.
* **Inline Rhai Scripting:** Complex geometric or mathematical expressions can be evaluated directly on pipeline steps without compiling Rust code.

#### 2. Photoshop-Style Sub-Tools & Flyout Viewports (`sub_actions`)
* **Multi-Tool Buttons:** Plugin buttons in the floating Toolstrip can now offer multiple operational variants.
* **Vector Chevron Indicator:** Buttons with declared `sub_actions` render an adaptive vector corner chevron.
* **Dedicated Floating Flyout:** Long-pressing (>350ms) or clicking the chevron opens a floating flyout window (`egui::ViewportBuilder`) where users can switch tool modes.
* **Persistent Preferences:** The chosen sub-action is remembered across sessions.

#### 3. AI Remove Background v1.1.0 with Sub-Tools
* **Remove Background (Cutout):** Isolates the foreground subject onto a transparent new layer and non-destructively hides the source layer.
* **Create Layer Mask:** Seamlessly transfers neural alpha transparency into an active selection and applies a non-destructive native layer mask directly onto the original layer.

#### 4. Up to 25x Faster AI Upscaling via Warm DirectML Session Tiling (BUG-075)
* Resolved a severe bottleneck in `run_tiled_rgb_inference`. By keeping the DirectML/CPU session instantiated in VRAM across all 512×512 tiles instead of destroying and reloading 71MB weights on each tile, per-tile processing dropped from 16s to ~1.9s (a 10–25x speedup!).

#### 5. No-Code Macro Execution Guard (BUG-074)
* Eliminated a duplicate execution issue where No-Code macros ran both as pre-commands and the plugin body, causing triple layers in PhotoCraft. Macro pipelines now execute strictly once.

#### 6. Documentation & Community Updates
* **Plugin Developer Guide v2.1:** Bilingual guide in `docs/PLUGIN_DEVELOPMENT_GUIDE.md` and `.en.md` covering the Workflow Engine and Sub-Actions schema.
* **Companion Discord Server:** Added direct links and badges to the official PhotoCraft Companion Discord community ([https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)).

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion обновлён до версии **1.2.0** — автономный компаньон и открытая экосистема расширений для графического редактора **PhotoCraft**.

В этом релизе представлены **декларативный движок Workflow Engine**, **суб-инструменты в стиле Photoshop с плавающими меню Flyout**, многократное ускорение апскейла на GPU DirectML (до 25 раз быстрее) и устранение дефектов макросов.

---

### 🌟 Ключевые нововведения

#### 1. Универсальный движок рабочих процессов (Workflow Engine)
* **Контекстные переменные рантайма:** Автоматическая интерполяция `$source_layer`, `$result_layer`, `$active_layer`, `$doc_width`, `$doc_height`, `$has_selection`, `$fg_color`, `$bg_color` и строкового шаблона `{{...}}`.
* **Структурированные шаги команд:** Поддержка сохранения слоев `save_as`, условного выполнения `when: "$has_selection == true"` и пропуска ошибок `ignore_errors: true`.
* **Встроенные скрипты Rhai:** Возможность выполнения песочных скриптов прямо внутри цепочки команд.

#### 2. Суб-инструменты в стиле Photoshop и выпадающие меню Flyout (`sub_actions`)
* **Несколько режимов на одной кнопке:** Плагины могут объединять родственные инструменты под одной кнопкой тулбара.
* **Векторный шеврон и Flyout Viewport:** Долгое удержание (>350 мс) или клик по шеврону открывает плавающее окно выбора варианта, а выбранный инструмент сохраняется в настройках пользователя.

#### 3. Обновление плагина AI Remove Background v1.1.0
* **Удаление фона (Вырезка):** Мгновенная вырезка объекта на чистый слой со скрытием исходника.
* **Создать маску слоя:** Автоматический перенос контура ИИ в выделение и наложение неразрушающей маски слоя прямо на исходный слой.

#### 4. Ускорение ИИ-апскейла до 25 раз (BUG-075)
* Сессия GPU DirectML теперь кэшируется на всё время нарезки холста, исключая повторную загрузку 71 МБ весов с диска на каждом тайле. Время обработки тайла уменьшено с 16с до ~1.9с.

#### 5. Устранение дублирования слоев в No-Code макросах (BUG-074)
* Исправлен дефект повторного запуска команд макроса как pre-команд, приводивший к созданию 3 одинаковых слоев. Пайплайн макроса теперь запускается строго один раз.

#### 6. Документация и сервер Discord
* Актуализировано руководство разработчика плагинов (`PLUGIN_DEVELOPMENT_GUIDE.md` и `.en.md`).
* Добавлены официальные ссылки на Discord-сервер PhotoCraft Companion ([https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)).

---

## 📦 Файлы дистрибутива

| ОС | Тип пакета | Имя файла | Описание |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.2.0-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.2.0_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.2.0-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.2.0-macos-portable.tar.gz` | Портабельный архив для macOS |
