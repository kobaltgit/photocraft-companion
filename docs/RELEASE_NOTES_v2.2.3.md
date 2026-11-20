# PhotoCraft Companion v2.2.3

[![GitHub Release](https://img.shields.io/badge/Release-v2.2.3-green.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.2.3)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

> [!NOTE]
> **Plugin Manager Selection Persistence / Персистентность выбора плагинов в Менеджере:**
> This release resolves an issue where disabled plugins were restored to an enabled state upon application restart ([Issue #18](https://github.com/kobaltgit/photocraft-companion/issues/18)).
> 
> *Этот релиз устраняет сброс настроек списка активных плагинов: отключенные пользователем плагины больше не активируются повторно при перезапуске приложения ([Issue #18](https://github.com/kobaltgit/photocraft-companion/issues/18)).*

---

## 🇬🇧 English: What's New in v2.2.3

### 1. Restoring Plugin Enabled Selection on Startup ([Issue #18](https://github.com/kobaltgit/photocraft-companion/issues/18) / BUG-128 / SPEC-006)
* **The Problem:** In previous releases, when users disabled plugins in the Plugin Manager and clicked "Save Settings", the choice was saved into `companion_config.json` (`enabled_plugins`), but was never read back during application startup. All plugin manifests defaulted to `enabled: true`, restoring all 17 buttons on the floating toolstrip upon restart and overwriting the saved configuration on any subsequent save.
* **The Solution (SPEC-006):**
  * Added `PluginRegistry::reconcile_enabled_state(&mut self, enabled_ids: &[String])` in `crates/core_contracts/src/registry.rs`.
  * If `enabled_ids` is empty (such as on a fresh installation), all installed plugins remain enabled by default.
  * In `crates/companion_app/src/main.rs`, the loaded `companion_cfg.enabled_plugins` is reconciled immediately upon loading config, before toolbar buttons are constructed.
  * Added defense-in-depth reconciliation in `PluginManagerApp::new` in `crates/overlay_ui/src/manager/mod.rs`.
  * Covered by unit tests in both `core_contracts` and `overlay_ui`.

---

## 🇷🇺 Русский: Что нового в версии 2.2.3

### 1. Восстановление сохраненного выбора активных плагинов при старте ([Issue #18](https://github.com/kobaltgit/photocraft-companion/issues/18) / BUG-128 / SPEC-006)
* **В чём была проблема:** Ранее при отключении плагинов в Менеджере плагинов и нажатии кнопки «Сохранить настройки» список активных расширений сохранялся в `companion_config.json` (`enabled_plugins`), однако при повторном запуске приложения этот параметр игнорировался. Все манифесты по умолчанию загружались со статусом `enabled: true`, из-за чего все 17 кнопок снова появлялись на плавающей панели, а повторное сохранение затирало пользовательскую настройку.
* **Решение (SPEC-006):**
  * В структуру `PluginRegistry` (`crates/core_contracts/src/registry.rs`) добавлен метод `reconcile_enabled_state(&mut self, enabled_ids: &[String])`.
  * Если `enabled_ids` пуст (первый запуск на чистой системе), все установленные плагины остаются активными по умолчанию.
  * В `crates/companion_app/src/main.rs` вызов `reconcile_enabled_state` выполняется сразу после чтения конфигурации до сборки кнопок тулбара.
  * В конструктор `PluginManagerApp::new` (`crates/overlay_ui/src/manager/mod.rs`) внедрена эшелонированная сверка на случай автономного запуска окна настроек.
  * Добавлены модульные тесты в `core_contracts` и `overlay_ui`.
