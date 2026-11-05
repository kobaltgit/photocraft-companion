# PhotoCraft Companion v2.1.0-beta.1 (Public Beta)

[![GitHub Release](https://img.shields.io/badge/Release-v2.1.0--beta.1-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.1.0-beta.1)
[![Pre-release](https://img.shields.io/badge/Status-Public_Beta-blueviolet.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.1.0-beta.1)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Companion For](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)
[![Discord](https://img.shields.io/badge/Discord-Companion_Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

---

> [!WARNING]
> ### ⚠️ PUBLIC BETA NOTICE / ВНИМАНИЕ: ПУБЛИЧНАЯ БЕТА-ВЕРСИЯ
> **English:** This is a **Public Beta** pre-release featuring a fundamental re-engineering of the multi-viewport graphics pipeline, DWM desktop stall elimination, and the brand-new standalone **Timelapse Studio** plugin. While all 240+ automated workspace tests and linters pass on clean environments, we invite the community to test this release across diverse Windows, macOS, and Linux hardware configurations. Please report any defects or observations in [GitHub Issues](https://github.com/kobaltgit/photocraft-companion/issues)!
>
> **Русский:** Это **публичная бета-версия**, включающая фундаментальную переработку графического конвейера и управления окнами, полную ликвидацию системных фризов Windows DWM, а также новый инструмент **Студия таймлапсов (Timelapse Studio)**. Хотя все 240+ автоматических тестов и 7 линтеров пройдены со 100% успехом, мы приглашаем пользователей протестировать сборку на разных конфигурациях Windows, macOS и Linux перед финальным релизом 2.1.0. О любых замеченных неполадках сообщайте в [GitHub Issues](https://github.com/kobaltgit/photocraft-companion/issues)!

---

## 🌟 What's New in v2.1.0-beta.1 / Что нового в версии 2.1.0-beta.1

### 🇬🇧 English

#### 🎬 1. New Plugin: Timelapse Studio (SYS-013 Protocol)
* **Background Speedpaint Capture with Zero Brush Freeze**: Automatically records drawing progress in the active PhotoCraft document without freezing or lagging the brush tool, utilizing native non-blocking canvas render RPC.
* **State-Driven Session Machine**: Dedicated controls for **Record**, **Pause**, **Stop**, **Snapshot**, and **Reset**. Seamlessly pause and resume painting without losing captured frames; frame counter and cache reset cleanly upon export.
* **Social Media Presets (9:16 Blur-Fill & 16:9 Widescreen)**: Automatic background blur-fill padding for vertical TikTok / Reels / Shorts videos, eliminating black bars on square or horizontal artwork.
* **Final Artwork Hold Duration**: Adjustable pause (0–5 seconds) on the finished piece via FFmpeg `tpad` filter before the video loops.
* **Dynamic Pulsating `REC` Badge**: Reactive toolstrip badge featuring smooth sine-wave alpha pulsing, notifying the artist that recording is active even with the tool window closed.

#### ⚡ 2. Graphics Pipeline Overhaul & Zero-Lag Desktop (BUG-110, BUG-111)
* **Elimination of Windows DWM / GDI Starvation**: Solved computer-wide stuttering and mouse cursor freezes by removing per-frame Win32 `WM_SETICON` calls in secondary viewports (`dynamic_tool`), replacing them with crisp single-shot Win32 caption calibration.
* **Single-Active Viewport Architecture**: Enforced strict mutual exclusivity between auxiliary windows. Opening an interactive tool automatically collapses other tool windows and hides the Plugins Manager; opening the Manager cleanly closes active plugin windows. Eliminates multi-viewport OpenGL `SwapBuffers` VSync queue stacking.
* **Thread-Safe OnceLock Icon Caching**: SVG XML parsing and `resvg` rasterization run exactly once per requested size across the application lifetime.
* **Quantized Animation Repaints**: Pulsing badges and busy spinners quantized from unbounded 1000+ FPS loops to smooth ~30 FPS (`request_repaint_after(33ms)`), preventing Windows event queue exhaustion.
* **Event-Driven Idle Rest (0.0% CPU)**: Relaxed fallback idle redraw loop from 50ms to 500ms. In quiet state, CPU and GPU utilization drops to 0%, while mouse interactions and PhotoCraft IPC trigger instantaneous 60+ FPS redraws.
* **TTL Disk I/O Caching**: Replaced per-frame filesystem scans in the Plugins Manager with thread-safe `OnceLock` TTL caches for system `PATH` tools (10s) and plugin neural models (5s).

#### 🛠️ 3. Multi-OS Stability & Ergonomics (BUG-108, BUG-109)
* **Windows Explorer Reveal Fix**: Repaired `reveal_in_file_manager` by passing raw unquoted flag arguments (`/select,"..."`) and stripping UNC `\\?\` prefixes, ensuring exported MP4 videos are correctly highlighted in Windows Explorer.
* **Tray Restore Viewport Cooldown**: Added a 2-frame stabilization buffer (`tray_restore_cooldown_frames: 2`) when restoring the floating toolbar from the system tray, completely eliminating eframe `the user callback was never called` panics when auxiliary windows are open.

---

### 🇷🇺 Русский

#### 🎬 1. Новый инструмент: Студия таймлапсов (Timelapse Studio, протокол SYS-013)
* **Фоновая запись процесса рисования с нулевой задержкой кисти**: Автоматический захват холста PhotoCraft в фоновом режиме без блокировки инструмента «Кисть» и без использования буфера обмена.
* **Эргономичная машина состояний сессии**: Четкое разделение на действия **«Запись»**, **«Пауза»**, **«Стоп»**, **«Снимок»** и **«Сброс»**. Запись можно ставить на паузу и возобновлять в любой момент без риска затереть ранее сохраненные кадры.
* **Пресеты для соцсетей (9:16 Blur-Fill и 16:9)**: Автоматическое размытие фона (Blur-Fill) под формат вертикальных видео TikTok, Reels и Shorts, устраняющее черные полосы по краям рисунка.
* **Задержка на финальном рисунке**: Настраиваемая пауза (0–5 секунд) на готовом арте перед зацикливанием видео (через фильтр FFmpeg `tpad`).
* **Пульсирующий бейдж `REC` на панели**: Реактивный векторный индикатор с мягким альфа-дыханием на кнопке плагина, информирующий художника об активной записи даже при свернутом окне студии.

#### ⚡ 2. Переработка графического конвейера и ликвидация фризов DWM (BUG-110, BUG-111)
* **Ликвидация системных зависаний Windows DWM и GDI**: Полностью устранены лаги курсора мыши и рабочего стола за счет снятия ежекадрового спама Win32 `WM_SETICON` во вторичных вьюпортах и перехода на однократную аппаратную калибровку заголовков.
* **Политика единого активного окна (Single-Active Viewport)**: Вторичное окно в системе теперь строго одно. Открытие любого плагина автоматически сворачивает другие плагины и закрывает Менеджер; открытие Менеджера закрывает плагины. Это навсегда исключает VSync-сталлинг OpenGL Glow бэкенда при параллельной работе нескольких окон.
* **Потокобезопасный `OnceLock` кэш иконок**: Растеризация векторной SVG-иконки через `resvg` выполняется ровно 1 раз за все время работы приложения.
* **Квантование частоты перерисовки**: Анимации бейджей и спиннеров переведены с бесконечного цикла 1000+ FPS на стабильные ~30 кадров/с (`request_repaint_after(33ms)`), разгружая очередь сообщений Windows.
* **Истинный покой в режиме ожидания (0.0% CPU)**: Холостой таймер расслаблен с 50 мс до 500 мс. В покое нагрузка на процессор и видеокарту падает до нуля, а любое движение мыши или сигнал сокета PhotoCraft мгновенно пробуждают интерфейс на 60 FPS.
* **TTL-кэширование дисковых операций**: Заменен ежекадровый опрос системного `PATH` и файлов моделей в Менеджере плагинов на быстрый TTL-кэш (10 сек для утилит, 5 сек для нейросетей).

#### 🛠️ 3. Стабильность жизненного цикла и мультиплатформенность (BUG-108, BUG-109)
* **Исправление показа в Проводнике Windows**: В `reveal_in_file_manager` устранены проблемы с экранированием флага `/select,` и снят UNC-префикс `\\?\`, благодаря чему экспортированное видео всегда выделяется в Проводнике.
* **Защитный кулдаун при разворачивании из трея**: Внедрен 2-кадровый буфер ожидания (`tray_restore_cooldown_frames: 2`), полностью исключающий падение eframe (`the user callback was never called`) при наличии открытых плавающих окон.

---

## 📦 Packages & Downloads / Файлы для загрузки

| Platform / ОС | Package Type / Тип пакета | File / Имя файла | Description / Описание |
|---|---|---|---|
| 🪟 **Windows (x64)** | Setup Installer (`.exe`) | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Standard Windows Inno Setup installer |
| 🪟 **Windows (x64)** | Portable (`.zip`) | `PhotoCraft-Companion-v2.1.0-beta.1-windows-x64-portable.zip` | Zero-install standalone archive with launcher |
| 🍏 **macOS (Universal)** | Disk Image (`.dmg`) | `PhotoCraft-Companion-v2.1.0-beta.1-macOS-universal.dmg` | Drag-and-drop bundle for Apple Silicon (M1/M2/M3/M4) & Intel |
| 🐧 **Linux (x64)** | Debian / Ubuntu (`.deb`) | `photocraft-companion_2.1.0-beta.1_amd64.deb` | Native DEB package with desktop shortcuts and app indicator |
| 🐧 **Linux (x64)** | Portable (`.tar.gz`) | `photocraft-companion-v2.1.0-beta.1-linux-x64.tar.gz` | Standalone portable archive for Arch, Fedora, openSUSE |
| 🧩 **Plugins** | Registry & Packages | `dist/*.pcpl`, `registry.json` | 15 pre-packaged offline plugins including Timelapse Studio |
