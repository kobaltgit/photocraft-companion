# Release Notes: PhotoCraft Companion v2.0.3

**Release Tag:** `v2.0.3`  
**Date:** 2026-11-04  
**Status:** Stable Production Bugfix & Platform Compatibility Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v2.0.3** is a stability and platform compatibility release resolving clipboard access failures in modern Linux **Wayland** sessions (KDE Plasma 6, Hyprland, Sway), fixing extensionless binary discovery in Zenity file dialogs, and expanding portable installation discovery heuristics.

---

### 🌟 Key Highlights & Fixes in v2.0.3

#### 1. Native Wayland Clipboard Support & Multi-Tier Fallback (BUG-105 / Issue #13)
* **Target-Gated Wayland Protocol Feature:**
  - In `crates/photocraft_bridge` and `crates/companion_app`, enabled the `wayland-data-control` feature for `arboard` under Linux targets (`cfg(target_os = "linux")`).
  - Windows and macOS builds remain lightweight with zero Wayland dependencies.
* **Multi-Tier Robust Clipboard Fallback (`photocraft_bridge`):**
  - If direct in-memory `arboard` reading fails, Companion automatically checks `$WAYLAND_DISPLAY` and invokes `wl-paste --type image/png`, decoding the PNG stream into `RgbaImage` via `image::load_from_memory`. If running on X11 (`$DISPLAY`), it falls back to `xclip -selection clipboard -t image/png -o`.
  - For writing images to clipboard, Companion encodes the raster to PNG in memory and pipes it via stdin to `wl-copy --type image/png` or `xclip`, explicitly closing stdin (`drop(stdin)`) to allow proper daemonization.
  - Increased clipboard retry policy from 6 to 8 attempts with a 40 ms settling delay to accommodate asynchronous compositor frame composition latency on Wayland.

#### 2. Extensionless Executable Visibility in Linux File Choosers (BUG-105)
* **Zenity / GTK File Chooser Filter Enhancement:**
  - In `companion_app` and `overlay_ui`, added an `"All Files (*)"` filter (`&["*"]`) to `rfd::FileDialog` on Linux.
  - Users can now effortlessly select extensionless Unix executables (`photocraft`) without the file chooser hiding them.

#### 3. Expanded Discovery for Portable PhotoCraft Installations (BUG-105)
* **Portable Directory Heuristics:**
  - Expanded `find_photocraft_executable` to dynamically scan `~/Programs` and `~/Applications` for directories matching `photocraft*` (e.g. `~/Programs/photocraft-0.20.1-linux-x86_64/bin/photocraft`).
  - Added traversal of sibling directories in the parent folder of `current_exe` (`current_exe.parent().parent()`).

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v2.0.3** — это обновление совместимости и стабильности, полностью устраняющее сбои работы с буфером обмена в сессиях Linux **Wayland** (CachyOS, Arch Linux, KDE Plasma 6, Hyprland, Sway), снимающее ограничения фильтра файлов Zenity и расширяющее эвристики поиска переносимых сборок редактора.

---

### 🌟 Ключевые исправления и улучшения

#### 1. Нативная поддержка буфера Wayland и многоуровневый фоллбэк (BUG-105 / Issue #13)
* **Подключение протокола Wayland в `arboard`:**
  - Для сборки под Linux (`cfg(target_os = "linux")`) в крейтах `photocraft_bridge` и `companion_app` активирована фича `wayland-data-control`. Сборки для Windows и macOS изолированы и не затягивают зависимости Wayland.
* **Многоуровневая отказоустойчивость буфера обмена (`photocraft_bridge`):**
  - При ошибке прямого чтения через `arboard` мост автоматически проверяет переменную `$WAYLAND_DISPLAY` и считывает растр через системную утилиту `wl-paste --type image/png`, декодируя поток байтов в `RgbaImage` через `image::load_from_memory`. В сессиях X11 (`$DISPLAY`) мост обращается к `xclip -selection clipboard -t image/png -o`.
  - При записи растра в буфер растр кодируется в PNG в памяти и передается в stdin `wl-copy --type image/png` или `xclip` с явным закрытием входного потока (`drop(stdin)`).
  - Число попыток чтения буфера увеличено до 8 с микропаузами 40 мс для защиты от рассинхронизации композитора на Wayland.

#### 2. Отображение исполняемых файлов в диалоге выбора (BUG-105)
* **Улучшение фильтра Zenity / GTK:**
  - В `companion_app` и `overlay_ui` в диалог `rfd::FileDialog` добавлен фильтр `All Files (*)` (`&["*"]`), благодаря чему диалоги выбора больше не скрывают бинарники без расширения (`photocraft`).

#### 3. Автообнаружение переносимых версий PhotoCraft (BUG-105)
* **Динамический поиск переносимых архивов:**
  - В `find_photocraft_executable` добавлено сканирование каталогов `~/Programs` и `~/Applications` на наличие подпапок вида `photocraft*` (например, `~/Programs/photocraft-0.20.1-linux-x86_64/bin/photocraft`).
  - Добавлен поиск по соседним каталогам родительской папки компаньона (`current_exe.parent().parent()`).

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.0.3-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_2.0.3_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.0.3-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v2.0.3-macos-portable.tar.gz` | Портабельный архив для macOS |
