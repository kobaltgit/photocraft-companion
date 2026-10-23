# Release Notes: PhotoCraft Companion v1.2.4

**Release Tag:** `v1.2.4`  
**Date:** 2026-10-23  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.2.4** — delivering critical Linux Desktop Entry (`.desktop`) and AppImage launch improvements (resolving GitHub Issue #4 `[Bug]: Unable to locate Photocraft`), introducing the professional **Frequency Separation** portrait beauty retouching plugin (No-Code macro), and updating community plugin catalog maintenance.

---

### 🌟 Key Highlights & Major Additions

#### 1. Linux Desktop Entry (.desktop) & AppImage Autodiscovery (BUG-079 / GitHub Issue #4)
* **FreeDesktop Desktop Entry Specification Parser:**
  - Engineered an in-house, zero-dependency parser for FreeDesktop Desktop Entry (`.desktop`) files (`crates/photocraft_bridge/src/desktop_entry.rs`).
  - Implemented shlex-compliant `Exec` command tokenization with support for single/double quotes, escaped spaces, environment wrappers (`env FOO=BAR`), and sanitization of XDG field codes (`%f`, `%F`, `%u`, `%U`, `%d`, `%n`, `%i`, `%k`, `%v`) and Flatpak markers (`@@u %U @@`).
  - Resolves target executables on disk or via system `PATH`.
* **Prevention of `ENOEXEC` (Exec format error):**
  - In `PhotoCraftClient::launch_photocraft`, `.desktop` shortcuts are resolved directly to physical executable binaries before passing them to `std::process::Command::new()`. Unresolvable entries return explicit diagnostics instead of triggering low-level OS execution format crashes.
* **AppImage Process Tracking & Read-Only Squashfs Guard:**
  - Updated `find_running_photocraft_dir` to perform case-insensitive process scanning in `/proc/<pid>/exe` and fallback checking in `/proc/<pid>/cmdline`, correctly recognizing versioned AppImage binaries (e.g., `PhotoCraft-1.2.3.AppImage`).
  - When the process runs inside an AppImage squashfs mount point (`/tmp/.mount_*`), Companion inspects `/proc/<pid>/environ` for `APPIMAGE=` to determine the host directory containing user data (`PhotoCraftData`).
  - Marked `/tmp/.mount_*` paths as read-only in `is_token_dir_writable`, preventing token generation failures (`os error 30`) and directing storage to standard XDG data paths (`~/.local/share/photocraft/`).
* **Desktop Application Directory Scanning & File Dialogs:**
  - Added autodiscovery of standard Linux application directories (`~/.local/share/applications`, `/usr/share/applications`, `/usr/local/share/applications`, and Flatpak export directories).
  - Extended file dialog filters in `companion_app` and `overlay_ui` to account for Linux case sensitivity: `&["AppImage", "appimage", "desktop", "bin", ""]`.

#### 2. New Plugin: Frequency Separation Beauty Retouch (Phase 79)
* **Deterministic Studio Retouching:**
  - Added `plugins/frequency_separation/`: a deterministic No-Code macro executing an 8-step split-frequency pipeline using native PhotoCraft commands (`layer.duplicate`, `layer.renameLayer`, `filter.other.highPass`, `layer.setProps` with Linear Light blend, `layer.selectBelow`, `filter.blur.gaussianBlur`).
  - Isolates skin microtexture into a dedicated High Frequency layer while preserving smooth shading, tone, and makeup on the Low Frequency layer.
  - Zero-Core Hardcoding: 100% declarative workflow requiring 0 lines of core code changes.
  - Features a custom pure-vector SVG icon (`icon.svg`) with 0% Unicode tofu glyphs and comprehensive bilingual documentation (`README.md`).

#### 3. AI Colorize (DeOldify) Plugin Status Update (Quarantine)
* **Real-World Evaluation & Placement into Quarantine:**
  - Evaluation of the experimental DeOldify colorization model on high-resolution photographs revealed tiling boundary seams and erratic chromatic hallucinations.
  - The plugin has been officially withdrawn from the public marketplace registry and placed into quarantine (`Quarantine`) pending future non-tiled or global-context architectures.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion обновлён до версии **1.2.4** — ключевое обновление совместимости с Linux: полноценная поддержка ярлыков FreeDesktop Desktop Entry (`.desktop`) и пакетов AppImage (решение GitHub Issue #4 `[Bug]: Unable to locate Photocraft`), новый профессиональный плагин частотного разложения **Frequency Separation** для бьюти-ретуши кожи (No-Code макрос) и актуализация каталога плагинов.

---

### 🌟 Ключевые нововведения

#### 1. Поддержка Linux Desktop Entry (.desktop), AppImage и надежный запуск PhotoCraft (BUG-079 / GitHub Issue #4)
* **Кроссплатформенный парсер FreeDesktop Desktop Entry:**
  - В `crates/photocraft_bridge/src/desktop_entry.rs` разработан независимый парсер спецификации FreeDesktop Desktop Entry Specification без внешних библиотек.
  - Реализован shlex-подобный разбор поля `Exec` с поддержкой кавычек, экранированных пробелов, префиксов `env` (`env FOO=BAR "/opt/PhotoCraft.AppImage" %F`) и очистки спецификаторов XDG (`%F`, `%U`, `@@u`).
  - Парсер валидирует секцию `[Desktop Entry]`, тип `Type=Application` и находит реальный исполняемый файл на диске или в системном `PATH`.
* **Устранение системной ошибки `ENOEXEC` (`Exec format error`):**
  - Метод `PhotoCraftClient::launch_photocraft` теперь автоматически разрешает путь `.desktop` в реальный исполняемый бинарник перед передачей в `std::process::Command::new()`. Если ярлык поврежден или указывает на отсутствующий файл, компаньон выдает понятную диагностическую ошибку, предотвращая аварийный отказ ядра Linux.
* **Отслеживание процессов AppImage и защита от `Read-Only` squashfs:**
  - В `find_running_photocraft_dir` внедрено регистронезависимое распознавание имени процесса в `/proc/<pid>/exe` и резервный опрос `/proc/<pid>/cmdline`, благодаря чему компаньон видит запущенный PhotoCraft даже при наличии номера версии в имени файла (`PhotoCraft-1.2.3.AppImage`).
  - При запуске из точки монтирования AppImage squashfs (`/tmp/.mount_*`) компаньон читает `/proc/<pid>/environ` и извлекает оригинальный путь из `APPIMAGE=...`, находя физическую директорию с данными пользователя `PhotoCraftData`.
  - В `is_token_dir_writable` точки `/tmp/.mount_*` исключены из проверки токена, что предотвращает сбои записи в файловую систему read-only (`os error 30`) и гарантирует сохранение токена в `~/.local/share/photocraft/`.
* **Автопоиск ярлыков и фильтры диалогов:**
  - Добавлено сканирование стандартных каталогов ярлыков XDG: `~/.local/share/applications`, `/usr/share/applications`, `/usr/local/share/applications`, Flatpak exports.
  - В диалогах выбора файлов расширен фильтр расширений с учетом регистра на Linux: `&["AppImage", "appimage", "desktop", "bin", ""]`.

#### 2. Новый плагин: Frequency Separation (Частотное разложение) для бьюти-ретуши (Фаза 79)
* **Студийный стандарт портретной ретуши:**
  - Добавлен макро-плагин `plugins/frequency_separation/`: детерминированное частотное разложение в 8 шагов через нативные команды PhotoCraft (`layer.duplicate`, `layer.renameLayer`, `filter.other.highPass`, `layer.setProps` с режимом Linear Light, `layer.selectBelow`, `filter.blur.gaussianBlur`).
  - Изолирует микротекстуру кожи на отдельный слой High Frequency, сохраняя светотеневой рисунок, тон и макияж на слое Low Frequency.
  - 100% Zero-Core Hardcoding: полностью декларативный макрос без изменения ядра Rust.
  - Векторная иконка `icon.svg` без глифов «тофу» и подробный двуязычный `README.md` с 3 практическими сценариями ретуши.

#### 3. Статус плагина AI Colorize (DeOldify) (Карантин / Quarantine)
* **Результаты полевых испытаний:**
  - По итогам тестирования DeOldify на фотографиях высокого разрешения выявлены артефакты тайлинга 512×512 (швы и скачки цветовой температуры) и цветовые пятна на нейтральных участках.
  - Плагин снят с публикации в публичном реестре и переведен в карантин (`Quarantine`) до появления моделей с глобальным контекстом.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.2.4-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.2.4_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.2.4-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.2.4-macos-portable.tar.gz` | Портабельный архив для macOS |

---

## 🧩 Community Plugins Marketplace / Каталог плагинов

Все официальные плагины (`.pcpl`) автоматически доступны для установки в один клик через вкладку **«Магазин плагинов»** в менеджере:
* **Frequency Separation** (`frequency_separation`): Студийная бьюти-ретушь кожи методом частотного разложения на текстуру и тон (No-Code макрос).
* **Background Removal** (`remove_bg`): Качественное удаление фона на базе нейросети RMBG-1.4 с выбором режимов Cutout и Layer Mask.
* **ChromaSense A11y** (`chromasense`): Интерактивная палитра, проверка контраста WCAG 2.1 и симуляция дальтонизма (Brettel LMS).
* **Depth Map** (`depth_map`): Оценка глубины сцены на базе Depth Anything v2.
* **Image Upscale** (`upscale`): 2x супер-разрешение на базе Real-ESRGAN с ускоренным кэшированным тайлингом.
* **Generative Inpaint** (`inpaint`): Умное заполнение выделенных областей текстурой окружающего фона (LaMa).
* **Vector Tracing** (`vectorize`): Алгоритмическая векторизация силуэтов и штриховых рисунков.
* **Black & White Pro** (`black_and_white`): Высококонтрастная черно-белая конвертация No-Code.
