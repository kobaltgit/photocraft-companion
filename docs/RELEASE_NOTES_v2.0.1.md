# Release Notes: PhotoCraft Companion v2.0.1

**Release Tag:** `v2.0.1`  
**Date:** 2026-11-01  
**Status:** Stable Production Bugfix & Reliability Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v2.0.1** is a focused maintenance and reliability release that resolves critical socket connection failures across Windows, macOS, and Linux, eliminates layer offset positioning bugs in Native AI output generation, and dramatically improves first-time setup onboarding.

---

### 🌟 Key Highlights & Fixes in v2.0.1

#### 1. Layer Bounds Offset Preservation for Native AI Output (BUG-101 / Issue #8)
* **Accurate Coordinate Mapping:**
  - Resolved an issue where Native AI plugins producing new layers (such as **AI Remove Background** and **AI Depth Map**) pasted output layers at `(0, 0)` in the top-left corner of the canvas, ignoring the source layer's physical bounding box.
  - Now correctly extracts and preserves `active_layer_bounds.xy`, translating pasted layers directly to their intended position matching the source artwork.

#### 2. Strict Unix Token Security & Pre-Launch Permission Repair (BUG-102 / Issue #10)
* **Mode 0600 Enforcement on macOS and Linux:**
  - PhotoCraft 0.18+ strictly rejects `control.token` files that are readable by group or others (`mode 0644`), causing PhotoCraft to exit immediately on launch.
  - Companion now writes `control.token` with atomic `0o600` permissions (`OpenOptionsExt::mode(0o600)`).
* **Pre-Launch Auto-Repair:**
  - Prior to launching PhotoCraft via `--control-token-file`, Companion automatically invokes `set_permissions` with `0o600`, self-healing any token files left in `0644` mode by older releases or external scripts.

#### 3. Multi-Version Isolation & Robust Connection Onboarding (BUG-103 / Issue #11)
* **Configured Path Priority Over Environment Variables:**
  - Fixed an issue where third-party launchers (such as ArtCraft Launcher) set a persistent global environment variable `PHOTOCRAFT_CONTROL_TOKEN_FILE` pointing to outdated or mismatched installations, causing silent token mismatch and infinite connection drops.
  - `detect_default_token` now grants highest priority to the user-configured `photocraft_path`, ensuring the companion strictly connects to the chosen PhotoCraft instance.
* **Smart Setup Diagnostics & Folder Input Guard:**
  - Added folder validation on the `> Launch` button: if a user selects a directory instead of an executable, the status bar displays `"Please specify the photocraft.exe binary, not a folder."`.
  - Added instant auto-saving (`save_settings()`) when using `Auto-Detect` or `Browse...`.
  - Upgraded offline modal and manager status displays with clear port instructions: `"PhotoCraft is running, but port 18492 is closed. Restart PhotoCraft or set Control port to 18492 in Edit -> Preferences -> Integrations"`.

#### 4. Clean Architecture & Crate Isolation
* Host process monitoring is exposed via `PhotoCraftClient::is_photocraft_process_running()` and supplied to `overlay_ui` strictly through Dependency Injection (`Option<Arc<dyn Fn() -> bool + Send + Sync>>`), maintaining 100% independence of the UI crate from IPC transport internals.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v2.0.1** — это стабилизационный релиз, устраняющий критические сбои сетевого подключения и авторизации токена на Windows, macOS и Linux, исправляющий смещение слоев при выводе результатов нейросетей и упрощающий первый запуск программы.

---

### 🌟 Ключевые исправления и улучшения

#### 1. Сохранение координат и смещения слоев в Native AI (BUG-101 / Issue #8)
* **Точное позиционирование результатов нейросетей:**
  - Исправлен дефект, из-за которого плагины с выводом в новый слой (**AI Remove Background**, **AI Depth Map**) вставляли результат в координаты `(0, 0)` левого верхнего угла холста вместо сохранения исходного положения слоя.
  - Реализован корректный учет физических границ `active_layer_bounds.xy`, благодаря чему новый слой бесшовно совпадает с исходным объектом.

#### 2. Безопасные права токена 0600 и автоматическое восстановление на Unix (BUG-102 / Issue #10)
* **Строгий режим 0600 на macOS и Linux:**
  - Редактор PhotoCraft 0.18+ принудительно завершает работу с ошибкой I/O, если файл `control.token` доступен для чтения группе или остальным пользователям (`mode 0644`).
  - Компаньон теперь создает токен строго с правами `0o600` через `OpenOptionsExt::mode(0o600)`.
* **Pre-Launch Auto-Repair:**
  - Перед запуском PhotoCraft компаньон проверяет и принудительно выставляет права `0o600` на существующий файл токена, автоматически исправляя старые конфигурации без ручного ввода команд `chmod` в терминале.

#### 3. Изоляция нескольких версий PhotoCraft и улучшенный онбординг (BUG-103 / Issue #11)
* **Приоритет выбранного пути над глобальными переменными:**
  - Устранена проблема сторонних лаунчеров (включая ArtCraft Launcher), которые прописывали глобальную переменную окружения `PHOTOCRAFT_CONTROL_TOKEN_FILE` на старые портативные папки, приводя к мгновенному разрыву связи при попытке подключения к новой версии редактора.
  - Настроенный путь `photocraft_path` получил наивысший приоритет при поиске токена.
* **Защита от выбора папки и мгновенное сохранение:**
  - Кнопка `> Launch` предупреждает пользователя, если вместо бинарника `photocraft.exe` указана директория: `"Please specify the photocraft.exe binary, not a folder."`.
  - Кнопки `Auto-Detect` и `Browse...` теперь сразу сохраняют настройки и обновляют статус сокета.
  - Добавлены четкие указания по настройке порта: `"PhotoCraft is running, but port 18492 is closed. Restart PhotoCraft or set Control port to 18492 in Edit -> Preferences -> Integrations"`.

#### 4. Архитектурная чистота и изоляция крейтов
* Проверка активности процесса PhotoCraft вынесена в `PhotoCraftClient::is_photocraft_process_running()` и передается в `overlay_ui` через Dependency Injection (`is_process_running`), строго соблюдая запрет прямой зависимости UI от транспортного крейта `photocraft_bridge`.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.0.1-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_2.0.1_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.0.1-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v2.0.1-macos-portable.tar.gz` | Портабельный архив для macOS |
