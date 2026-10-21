# Release Notes: PhotoCraft Companion v1.2.2

**Release Tag:** `v1.2.2`  
**Date:** 2026-10-21  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.2.2** — delivering major cross-platform stability enhancements, native macOS `.app` bundle execution, strict Win32 process isolation with zero handle leaks, DPI-aware screen boundary protection, and persistent Plugin Manager geometry.

---

### 🌟 Key Highlights & Major Additions

#### 1. Native macOS Bundle Execution & CodeSeal Protection (BUG-076)
* **LaunchServices Integration:** macOS `.app` bundles now launch natively via Apple LaunchServices (`/usr/bin/open -n -a "<bundle.app>" --args ...`), ensuring clean WindowServer initialization and eliminating Gatekeeper terminations (`Killed: 9`).
* **Semantic CodeSeal Protection:** Built-in safeguard prevents writing `control.token` inside `.app` bundles (which previously invalidated macOS Gatekeeper code signatures for apps located in `~/Downloads`). The token file safely redirects to standard user application support paths.
* **Accurate Process Identification:** Replaced basic process searching with `/bin/ps -A -o comm` and strict name matching to eliminate false self-detection where Companion misidentified itself as the host editor.
* **Expanded Candidate Autodetection:** Extended search locations in `find_photocraft_executable` to check standard system `/Applications` and user `~/Downloads` directories.

#### 2. Win32 Process Identification & Zero Handle Leaks (BUG-077)
* **Precise Win32 Verification:** Upgraded window enumeration from fragile title heuristics to verified process ID matching, executable image verification via `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` and `QueryFullProcessImageNameW`.
* **Zero Handle Leaks:** Implemented strict handle lifecycle management with guaranteed `CloseHandle(h_process)`, completely eliminating OS descriptor leaks during repeated status checks.

#### 3. DPI-Aware Screen Centering & Boundary Guard (BUG-077)
* **High-DPI Screen Centering:** Added `calculate_primary_screen_center(w, h)`, properly scaling physical desktop metrics via `(GetDpiForSystem() / 96.0)` into logical egui coordinates. The toolbar and offline modal now spawn squarely in the center of the primary display at 125%, 150%, and higher DPI settings.
* **Multi-Monitor Boundary Guard:** Added `is_position_visible_on_screen(pos, size)` using `MonitorFromRect(..., MONITOR_DEFAULTTONULL)`. If an external display is disconnected or coordinates drift off-screen, windows automatically snap back safely to the primary screen center.

#### 4. Plugin Manager Geometry Persistence (BUG-077)
* **Persistent Window State:** Added `manager_window_position` and `manager_window_size` to `CompanionConfig` with `#[serde(default)]`.
* **Anti-Bloat Protection:** Window position is tracked via `outer_rect.min` while window dimensions track strictly through `inner_rect.size()`. This prevents the progressive window expansion defect caused by adding OS title bar and frame padding on every reopen.

#### 5. Offline Modal UX & First-Run Platform Guidance (BUG-076, BUG-077)
* **Enlarged Layout:** Expanded the offline connection window from 280×210 px to 300×280 px across all entry points, eliminating scrollbars.
* **Contextual First-Run Guidance:** Added OS-tailored guidance for launching unsigned open-source applications (macOS Gatekeeper *"Right Click -> Open"*, Windows SmartScreen *"More info -> Run anyway"*, Linux executable permissions).
* **Clear Socket Status:** Added intuitive feedback when the PhotoCraft editor process is running but the control socket (port 18492) is not yet accessible, advising a clean restart.
* **Settings Auto-Save:** Clicking `> Launch` in Settings now saves configuration to disk immediately before spawning the editor.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion обновлён до версии **1.2.2** — ключевое обновление стабильности и межплатформенной интеграции: нативный запуск бандлов macOS `.app`, строгая Win32-идентификация хост-редактора с гарантией отсутствия утечек дескрипторов, адаптивное DPI-центрирование, персистентность геометрии Менеджера плагинов и подсказки первого запуска.

---

### 🌟 Ключевые нововведения

#### 1. Нативный запуск бандлов macOS и защита CodeSeal (BUG-076)
* **Интеграция с Apple LaunchServices:** Графические бандлы macOS `.app` теперь запускаются через системный сервис `/usr/bin/open -n -a "<bundle.app>" --args ...`. Это устранило сбои инициализации WindowServer и аварийное завершение процесса защитой Gatekeeper (`Killed: 9`).
* **Семантическая защита цифровой подписи (CodeSeal):** Внедрена защита от записи `control.token` внутрь тела бандла `.app` (что нарушало подпись приложений, находящихся в `~/Downloads`). Токен надёжно перенаправляется в стандартный каталог пользователя `~/Library/Application Support/PhotoCraft/`.
* **Точная идентификация процессов:** Поиск процессов переведён на `/bin/ps -A -o comm` со строгой фильтрацией имени, исключая ложное самообнаружение, когда Компаньон принимал себя за хост-редактор.
* **Расширенный автопоиск:** В поиск исполняемого файла добавлены стандартные каталоги `/Applications` и пользовательские `~/Downloads`.

#### 2. Строгая Win32-идентификация и предотвращение утечек дескрипторов (BUG-077)
* **Точная верификация процессов Windows:** Отказ от ненадежного сканирования заголовков окон. Внедрена проверка PID процесса, опрос исполняемого файла через `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` и `QueryFullProcessImageNameW`.
* **Полное отсутствие утечек дескрипторов (Zero Handle Leaks):** Гарантированное освобождение дескрипторов через `CloseHandle(h_process)` при каждом опросе системы, исключающее исчерпание хэндлов Windows.

#### 3. DPI-Aware центрирование и защита от улета окон (BUG-077)
* **Адаптивное центрирование:** Функция `calculate_primary_screen_center(w, h)` корректно учитывает системный масштаб дисплея `(GetDpiForSystem() / 96.0)`. Плавающий тулбар и окна всегда появляются точно по центру основного экрана при масштабировании 125%, 150% и выше.
* **Валидатор экранных границ:** Функция `is_position_visible_on_screen(pos, size)` через `MonitorFromRect(..., MONITOR_DEFAULTTONULL)` отслеживает положение окон. При отключении монитора или сбросе координат окна автоматически возвращаются в центр экрана.

#### 4. Персистентность геометрии Plugin Manager с защитой от раздувания (BUG-077)
* **Сохранение положения и размеров:** В `CompanionConfig` добавлены поля `manager_window_position` и `manager_window_size`.
* **Защита от «раздувания» окон:** Координаты окна отслеживаются через `outer_rect.min`, а размеры — строго по клиентской области `inner_rect.size()`. Окно больше не увеличивается на толщину системных рамок при каждом повторном открытии.

#### 5. Улучшение оффлайн-онбординга и подсказки первого запуска (BUG-076, BUG-077)
* **Увеличенные габариты модалки:** Окно оффлайн-подключения расширено до 300×280 px во всех точках входа, устраняя появление нежелательных полос прокрутки.
* **Подсказки первого запуска:** В оффлайн-режим добавлены контекстные платформенные инструкции для неподписанного ПО (macOS Gatekeeper «Правый клик -> Открыть», Windows SmartScreen «Подробнее -> Выполнить в любом случае», права Linux `chmod +x`).
* **Понятный статус сокета:** Индикация состояния, когда процесс редактора обнаружен в системе, но порт управления 18492 ещё недоступен.
* **Автосохранение настроек:** Клик по кнопке `> Launch` в General Settings теперь немедленно сохраняет настройки на диск перед запуском.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.2.2-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.2.2_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.2.2-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.2.2-macos-portable.tar.gz` | Портабельный архив для macOS |

---

## 🧩 Community Plugins Marketplace / Каталог плагинов

Все официальные плагины (`.pcpl`) автоматически доступны для установки в один клик через вкладку **«Магазин плагинов»** в менеджере:
* **Background Removal** (`remove_bg`): Качественное удаление фона на базе нейросети RMBG-1.4 с выбором режимов Cutout и Layer Mask.
* **ChromaSense A11y** (`chromasense`): Интерактивная палитра, проверка контраста WCAG 2.1 и симуляция дальтонизма (Brettel LMS).
* **Depth Map** (`depth_map`): Оценка глубины сцены на базе Depth Anything v2.
* **Image Upscale** (`upscale`): 2x супер-разрешение на базе Real-ESRGAN с ускоренным кэшированным тайлингом.
* **Generative Inpaint** (`inpaint`): Умное заполнение выделенных областей текстурой окружающего фона (LaMa).
* **Vector Tracing** (`vectorize`): Алгоритмическая векторизация силуэтов и штриховых рисунков.
* **Black & White Pro** (`black_and_white`): Высококонтрастная черно-белая конвертация No-Code.

---
