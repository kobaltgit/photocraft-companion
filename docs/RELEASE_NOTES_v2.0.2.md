# Release Notes: PhotoCraft Companion v2.0.2

**Release Tag:** `v2.0.2`  
**Date:** 2026-11-02  
**Status:** Stable Production Bugfix & Security Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v2.0.2** is a critical bugfix and security release addressing filesystem permission issues, Windows NTFS file locks, and package installation failures when installing plugins via the **Store** tab in system-protected directories (`C:\Program Files`).

---

### 🌟 Key Highlights & Fixes in v2.0.2

#### 1. Zero Access Denied on Plugin Store Installations (BUG-104 / Issue #12)
* **User Data Fallback for Protected Installations:**
  - Resolved `IO error: Access is denied. (os error 5)` occurring when users installed or updated plugins (*Depth Studio*, *Harmonic Color Studio*, *Selection Modifiers*) via the **Store** tab while Companion was installed under `C:\Program Files`.
  - In `core_contracts::install_pcpl`, introduced automatic write-permission validation (`is_dir_writable`): when the target directory is read-only, unpacking transparently falls back to the user data directory `%LOCALAPPDATA%\PhotoCraft Companion\plugins`.
  - In `overlay_ui`, `PluginManagerApp` and `CompanionOverlayApp` now receive the guaranteed writable directory via `.with_plugins_dir(...)` and use user-space fallback by default.

#### 2. Duplicate Directory Prevention (Zero Duplicate Directories)
* **Intelligent Folder Matching:**
  - Eliminated parallel duplicate directories (e.g. `plugins/harmonic_studio` vs `plugins/ui.photocraft.plugins.harmonic_studio`).
  - Prior to extraction, `install_pcpl` inspects existing plugin subdirectories by parsing `plugin.json` for matching `manifest.id` or matching trailing identifier segments (`short_id`). Existing folders are cleanly updated in-place.

#### 3. Windows NTFS Locks & Readonly File Handling
* **Safe Directory Replacement (`remove_dir_all_safe`):**
  - Standard `std::fs::remove_dir_all` fails on Windows NTFS if files carry the `readonly` attribute. Companion now recursively strips readonly permissions (`clear_readonly_recursive`) before deletion.
  - Added retry logic with micro-delays (`rename_with_retry`) to withstand transient file descriptor locks held by Windows Defender and Windows Search Indexer.

#### 4. Incremental Seeding of Bundled Plugins
* **Seamless Updates Over Previous Versions:**
  - Replaced the coarse `is_empty` check in `companion_app` with `sync_missing_bundled_plugins`.
  - When users update Companion via installer, newly added bundled plugins from `Program Files\plugins` are automatically copied to their `%LOCALAPPDATA%` environment without overwriting existing customized configs.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v2.0.2** — это срочный стабилизационный релиз, устраняющий ошибку отказа в доступе (`Access is denied / os error 5`) при установке и обновлении плагинов через вкладку «Магазин» (Store) на Windows 11 при стандартной установке программы в `C:\Program Files`.

---

### 🌟 Ключевые исправления и улучшения

#### 1. Устранение ошибки доступа при установке плагинов (BUG-104 / Issue #12)
* **Автоматический переход в пользовательский каталог:**
  - Исправлен дефект `Installation failed: Install package error: IO error: Access is denied. (os error 5)`, возникавший при установке плагинов (*Depth Studio*, *Harmonic Color Studio*, *Selection Modifiers*) через вкладку **Store**, если приложение было установлено в системную папку `C:\Program Files`.
  - В распаковщике `core_contracts::install_pcpl` внедрена проверка прав записи `is_dir_writable`: если целевой каталог недоступен для записи, установка прозрачно и безопасно перенаправляется в `%LOCALAPPDATA%\PhotoCraft Companion\plugins`.
  - В интерфейсе `overlay_ui` Менеджер плагинов теперь получает гарантированно доступный для записи путь через метод `.with_plugins_dir(...)`.

#### 2. Защита от создания папок-дубликатов (Zero Duplicate Directories)
* **Интеллектуальное сопоставление директорий:**
  - Ликвидировано появление папок-двойников (например, `plugins/harmonic_studio` и `plugins/ui.photocraft.plugins.harmonic_studio`).
  - Перед распаковкой `install_pcpl` сканирует существующие подпапки и проверяет манифесты: если плагин уже был установлен под коротким именем, обновление накатывается строго в существующую директорию.

#### 3. Защита от блокировок Windows NTFS и безопасное удаление
* **Сброс атрибута `readonly` и retry-цикл:**
  - Стандартный вызов `std::fs::remove_dir_all` в Windows аварийно завершался с `os error 5`, если файлы имели атрибут «Только для чтения». Внедрена функция `remove_dir_all_safe`, рекурсивно снимающая атрибут `readonly` перед удалением.
  - Добавлен retry-цикл `rename_with_retry` с микропаузами против асинхронных блокировок открытых дескрипторов антивирусом Windows Defender и службой индексирования.

#### 4. Инкрементальный сидинг бандловых плагинов
* **Автодоставка новых плагинов при обновлении:**
  - В `companion_app` грубая проверка на полную пустоту каталога заменена на функцию `sync_missing_bundled_plugins`.
  - При установке новой версии программы поверх старой все новые плагины из директории инсталлятора автоматически копируются пользователю, не затирая существующие настройки.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.0.2-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_2.0.2_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.0.2-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v2.0.2-macos-portable.tar.gz` | Портабельный архив для macOS |
