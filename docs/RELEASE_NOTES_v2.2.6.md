# PhotoCraft Companion v2.2.6

[![GitHub Release](https://img.shields.io/badge/Release-v2.2.6-green.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.2.6)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

> [!NOTE]
> **Bundled Plugin Auto-Sync & Inpaint v1.0.1 CPU Fix / Автоматическая синхронизация бандловых плагинов и обновление Inpaint v1.0.1:**
> This release resolves an issue where bundled plugin updates (specifically AI Generative Fill / Inpaint v1.0.1 with forced CPU inference to prevent DirectML `0xC0000005` crash) were not automatically propagated to user installations created via the Windows setup installer ([Issue #21](https://github.com/kobaltgit/photocraft-companion/issues/21) / BUG-131). The companion app now automatically detects newer versions of bundled plugins by SemVer and manifest ID upon startup, updating manifests and scripts while strictly preserving all downloaded neural network weights (`models/`) without requiring redownloads.
> 
> *Этот релиз устраняет проблему, из-за которой обновления встроенных в дистрибутив плагинов (в частности, исправление AI Generative Fill / Inpaint v1.0.1 с принудительным CPU-инференсом для предотвращения сбоя DirectML `0xC0000005`) не доходили до пользователей, установивших приложение через инсталлятор Windows ([Issue #21](https://github.com/kobaltgit/photocraft-companion/issues/21) / BUG-131). При запуске компаньон теперь автоматически выполняет интеллектуальное сравнение версий бандловых плагинов по SemVer и `manifest.id`, обновляя манифесты и скрипты с гарантированным сохранением всех скачанных весов нейросетей в `models/` без повторной загрузки.*

---

## 🇬🇧 English: What's New in v2.2.6

### 1. Inpaint Plugin v1.0.1 CPU Inference Fix ([Issue #21](https://github.com/kobaltgit/photocraft-companion/issues/21) / BUG-131)
* **The Problem:** In v2.2.4, the AI Inpainting plugin was updated with `"execution_provider": "cpu"` to eliminate the fatal `0xC0000005` access violation in DirectML Fast Fourier Convolutions (FFC). However, the plugin version in `plugin.json` was kept at `1.0.0`. When users installed PhotoCraft Companion via the Windows installer (`C:\Program Files\PhotoCraft Companion\`), the initial seeding mechanism skipped plugins already present in `%APPDATA%\PhotoCraftData\plugins\inpaint\`, leaving users with an outdated manifest without the CPU provider directive.
* **The Solution:**
  * Bumped `plugins/inpaint/plugin.json` version to `1.0.1`.
  * Rebuilt the plugin registry (`dist/registry.json`), repackaged `dist/inpaint.pcpl` (v1.0.1), and synchronized markdown catalog tables.
  * In the companion core, enhanced `discovery::sync_missing_bundled_plugins()` to intelligently compare plugin SemVer versions upon startup.

### 2. Intelligent Bundled Plugin Synchronization with Model Weight Preservation
* **SemVer Comparison in Core Contracts:** Extracted the centralized positional SemVer comparison function `is_version_newer()` into `core_contracts::version`, eliminating code duplication between the in-app updater and discovery engine.
* **Manifest ID Matching:** When scanning plugins in the user's directory, existing folders are indexed by `manifest.id`. If a bundled plugin has a newer version (`bundled.version > installed.version`), files (`plugin.json`, `icon.svg`, `README.md`, scripts) are seamlessly updated in place.
* **Preservation of Downloaded Weights:** The synchronization mechanism never deletes files in the destination directory, guaranteeing that heavy downloaded ONNX models (such as `models/lama_inpaint.onnx`, 208 MB) remain completely intact. Users do not need to download models again.
* **Duplicate Folder Prevention:** Matches existing plugins by `manifest.id` even if the target folder has a custom or namespaced folder name, preventing folder duplication (e.g., `inpaint` vs `ai.photocraft.plugins.inpaint`).

### 3. Automated Regression Tests
* Added unit tests in `crates/companion_app/src/tests.rs`:
  * `test_sync_bundled_plugins_updates_older_version_without_deleting_models`: Verifies manifest and asset updates while ensuring `models/*.onnx` are preserved.
  * `test_sync_bundled_plugins_matches_by_id_avoiding_duplicate_folders`: Verifies that custom folder names are matched by `manifest.id` and no duplicate folders are created.

---

## 🇷🇺 Русский: Что нового в версии 2.2.6

### 1. Исправление плагина Inpaint v1.0.1 (CPU-инференс) ([Issue #21](https://github.com/kobaltgit/photocraft-companion/issues/21) / BUG-131)
* **В чём была проблема:** В версии 2.2.4 в плагин Inpaint было добавлено правило `"execution_provider": "cpu"` для предотвращения аппаратного сбоя DirectML FFC (`0xC0000005`). Однако версия плагина в `plugin.json` осталась `1.0.0`. У пользователей, установивших приложение через инсталлятор Windows в `C:\Program Files\PhotoCraft Companion\`, первичный механизм сидинга плагинов пропускал папку `%APPDATA%\PhotoCraftData\plugins\inpaint\`, из-за чего устаревший манифест без директивы CPU оставался на диске и приводил к повторению ошибки.
* **Решение:**
  * Версия плагина в `plugins/inpaint/plugin.json` поднята до `1.0.1`.
  * Пересобран реестр (`dist/registry.json`), упакован обновленный пакет `dist/inpaint.pcpl` (v1.0.1), синхронизированы каталоги в README.
  * В ядре компаньона модернизирована функция `discovery::sync_missing_bundled_plugins()`: теперь она выполняет интеллектуальное сравнение версий при каждом запуске.

### 2. Автоматическая синхронизация бандловых плагинов с сохранением весов нейросетей
* **Сравнение SemVer в core_contracts:** Функция позиционного сравнения версий `is_version_newer()` вынесена в чистые контракты ядра `core_contracts::version`, исключая дублирование логики между автоапдейтером и поисковым движком.
* **Сопоставление по manifest.id:** При сканировании плагинов папки индексируются по уникальному `manifest.id`. Если версия в бандле программы новее установленной (`bundled.version > installed.version`), файлы манифеста, иконок и скриптов автоматически обновляются.
* **Гарантированное сохранение моделей:** Синхронизация не удаляет существующие файлы в целевой папке, благодаря чему скачанные тяжелые веса моделей (`models/lama_inpaint.onnx`, 208 МБ) остаются нетронутыми. Повторная загрузка моделей не требуется.
* **Предотвращение дубликатов папок:** Сопоставление по `manifest.id` гарантирует, что плагин обновится даже в случае нестандартного имени папки, не создавая копий на диске пользователя.

### 3. Автоматические тесты регрессии
* В `crates/companion_app/src/tests.rs` внедрены тесты:
  * `test_sync_bundled_plugins_updates_older_version_without_deleting_models`: проверка обновления манифеста с сохранением весов `models/`;
  * `test_sync_bundled_plugins_matches_by_id_avoiding_duplicate_folders`: проверка точного сопоставления по `manifest.id` без дублирования папок.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.2.6-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `PhotoCraft-Companion-v2.2.6-linux-amd64.deb` | Установочный пакет для Ubuntu 22.04+, Mint 21/22, Debian 12+ |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.2.6-linux-x86_64-portable.tar.gz` | Универсальный портабельный архив для любых дистрибутивов Linux |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-v2.2.6-macos.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
