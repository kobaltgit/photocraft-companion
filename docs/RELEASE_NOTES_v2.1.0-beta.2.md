# PhotoCraft Companion v2.1.0-beta.2 (Public Beta 2)

[![GitHub Release](https://img.shields.io/badge/Release-v2.1.0--beta.2-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.1.0-beta.2)
[![Pre-release](https://img.shields.io/badge/Status-Public_Beta_2-blueviolet.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.1.0-beta.2)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Companion For](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)
[![Discord](https://img.shields.io/badge/Discord-Companion_Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

---

> [!IMPORTANT]
> ### ⚠️ PUBLIC BETA 2 NOTICE / ВНИМАНИЕ: ПУБЛИЧНАЯ БЕТА-ВЕРСИЯ 2
> **English:** PhotoCraft Companion **v2.1.0-beta.2** delivers a major architectural re-engineering of the speedpaint capture pipeline and multi-viewport lifecycle. We eliminated the main UI thread timer loop entirely, transitioned canvas recording to PhotoCraft's native reactive document revision protocol (`document.revision`), solved Windows DWM ghost-window stalls, and fixed FFmpeg export failures caused by empty session directories. All 252+ automated workspace tests and 7 architectural linters pass with 100% success.
>
> **Русский:** Версия **v2.1.0-beta.2** представляет фундаментальную переработку конвейера записи спидпеинта и управления окнами. Полностью ликвидирован циклический таймер в основном потоке интерфейса, запись переведена на нативный протокол ревизий документа PhotoCraft (`document.revision`), устранены окна-призраки в Windows DWM и исправлен сбой экспорта FFmpeg из-за пустых директорий сессий. Все 252+ автоматических теста воркспейса и 7 линтеров пройдены со 100% успехом.

---

## 🌟 What's New in v2.1.0-beta.2 / Что нового в версии 2.1.0-beta.2

### 🇬🇧 English

#### ⚡ 1. Event-Driven Reactive Speedpaint Capture (Zero-Timer Paradigm)
* **Canvas Revision Protocol (`document.revision`)**: Completely eliminated synthetic 1-second interval timers. Recording now subscribes directly to PhotoCraft's native document revision counter (`revision: u64`). 
* **Brush-Release Frame Capture**: While the artist paints across the tablet, zero frames are polled and zero CPU overhead is incurred, keeping canvas framerates silky-smooth at 60+ FPS. The instant the artist lifts the pen and PhotoCraft commits the stroke to history, a single clean frame is recorded.
* **Non-Blocking Headless Screenshot (`capture_canvas_frame`)**: Canvas captures use PhotoCraft's native `ui.screenshot` RPC with `focus: false`, preventing window focus theft or screen flickering. Base64 decoding and PNG disk writes run entirely off the main thread.
* **Obsolete Preview Protocol Purged**: Completely removed legacy screenshot mock methods (`CanvasPreviewParams`, `get_canvas_preview`) from `core_contracts` and `photocraft_bridge`.

#### 🖥️ 2. UI Thread Optimization & DWM Ghost Window Elimination (BUG-112, BUG-113)
* **Purge of `dynamic_tool/timer.rs`**: Removed the 16ms per-frame `process_timers` polling loop from the main UI thread, returning `overlay_ui` to a lightweight event-driven model (500ms in idle, 0.0% CPU).
* **Viewport Decomposition (`viewports.rs`)**: Window lifecycle and visibility methods are isolated in a dedicated submodule, keeping code architecture pristine and file sizes well below the 800-line limit.
* **Guaranteed DWM Viewport Closure Queue (`pending_close_viewports`)**: Closed window IDs are buffered and dispatched via `ctx.send_viewport_cmd_to(id, ViewportCommand::Close)` on the subsequent render pass, eliminating phantom window handles in Windows DWM (`dwm.exe`).
* **1 Hz Phase-Quantized `REC` Badge**: The toolbar pulsing recording badge repaints strictly at a 1 Hz phase interval (0.6s on / 0.4s pause), eliminating 30 FPS redraw cycles when idle.

#### 🎬 3. Robust Session I/O & Clean FFmpeg Export (BUG-114)
* **Guaranteed Session Directory Creation**: `clean_session_dir` now explicitly ensures directory presence via `std::fs::create_dir_all(&dir)`, fixing OS file write errors when recording new sessions.
* **Empty-Session Export Guard**: If no frames have been recorded (`frame_count == 0`), FFmpeg export is aborted early with a friendly message: *"No frames recorded yet. Draw on canvas first!"*.
* **Sanitized FFmpeg Error Logs**: Filtered compiler and build configuration banners (`ffmpeg version`, `configuration:`, `built with`) from `stderr`, ensuring only meaningful error messages appear in the UI banner.
* **Dynamic On-the-Fly Localization**: Added reactive language updates for session info and video length strings in Timelapse Studio when switching languages at runtime.

---

### 🇷🇺 Русский

#### ⚡ 1. Реактивный конвейер записи спидпеинта по отпусканию кисти (Zero-Timer)
* **Нативный протокол ревизий документа (`document.revision`)**: Полностью ликвидирован циклический 1-секундный таймер. Запись кадров переведена на подписку на системный счетчик правок документа PhotoCraft (`revision: u64`).
* **Захват кадра строго по завершению мазка**: Пока художник ведёт кисть или стилус по холсту — не делается никаких снимков и не тратится процессорное время. Холст работает с максимальной частотой 60+ кадров/с. Как только художник отпускает стилус и PhotoCraft коммитит мазок в историю, инкрементируя ревизию, Companion делает ровно один чистый снимок.
* **Неблокирующий фоновый снимок (`capture_canvas_frame`)**: Захват холста использует метод `ui.screenshot` сокета PhotoCraft с параметром `focus: false` (без мерцания и без воровства фокуса окна). Декодирование Base64 и сохранение PNG происходят асинхронно без подтормаживания интерфейса.
* **Очистка протокола ядра**: Из `core_contracts` и `photocraft_bridge` удалены устаревшие методы `CanvasPreviewParams` и `get_canvas_preview`.

#### 🖥️ 2. Ликвидация циклического таймера и окон-призраков DWM (BUG-112, BUG-113)
* **Полное удаление `timer.rs` из UI-потока**: Убран 16мс-цикл опроса таймеров из интерфейса. Приложение вернулось к энергоэффективной событийно-ориентированной модели (500 мс в покое, 0.0% CPU).
* **Декомпозиция управления окнами (`viewports.rs`)**: Методы жизненного цикла окон плагинов вынесены в чистый модуль, сохранив строгие лимиты размера файлов (< 800 строк).
* **Очередь гарантированного закрытия окон в DWM (`pending_close_viewports`)**: Свернутые окна накапливаются в очереди и закрываются системной командой `ViewportCommand::Close`, устраняя фантомные контексты в диспетчере Windows DWM (`dwm.exe`).
* **1 Гц фазовое квантование индикатора `REC`**: Пульсация красной точки записи переведена на секундный интервал (0.6с активно / 0.4с пауза), исключая перерисовку экрана в простое.

#### 🎬 3. Стабильный I/O сессий и чистый экспорт FFmpeg (BUG-114)
* **Гарантированное создание папки сессии**: В `clean_session_dir` внедрен обязательный вызов `create_dir_all(&dir)`, гарантирующий наличие директории `timelapse_current` на диске перед записью кадров.
* **Защита от пустого экспорта**: При 0 сохраненных кадров экспорт FFmpeg блокируется с понятным сообщением: *«Кадры еще не записаны. Сделайте несколько мазков на холсте!»*.
* **Фильтрация логов FFmpeg**: Из сообщений об ошибках отсекаются 20-строчные параметры сборки компилятора GCC, оставляя только суть проблемы.
* **Динамическая локализация на лету**: Статус сессии и суффикс длительности видео мгновенно обновляются при переключении языка приложения без перезапуска.

---

## 📦 Packages & Downloads / Файлы для загрузки

| Platform / ОС | Package Type / Тип пакета | File / Имя файла | Description / Описание |
|---|---|---|---|
| 🪟 **Windows (x64)** | Setup Installer (`.exe`) | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Standard Windows Inno Setup installer |
| 🪟 **Windows (x64)** | Portable (`.zip`) | `PhotoCraft-Companion-v2.1.0-beta.2-windows-x64-portable.zip` | Zero-install standalone archive with launcher |
| 🍏 **macOS (Universal)** | Disk Image (`.dmg`) | `PhotoCraft-Companion-v2.1.0-beta.2-macOS-universal.dmg` | Drag-and-drop bundle for Apple Silicon (M1/M2/M3/M4) & Intel |
| 🐧 **Linux (x64)** | Debian / Ubuntu (`.deb`) | `photocraft-companion_2.1.0-beta.2_amd64.deb` | Native DEB package with desktop shortcuts and app indicator |
| 🐧 **Linux (x64)** | Portable (`.tar.gz`) | `photocraft-companion-v2.1.0-beta.2-linux-x64.tar.gz` | Standalone portable archive for Arch, Fedora, openSUSE |
| 🧩 **Plugins** | Registry & Packages | `dist/*.pcpl`, `registry.json` | 15 pre-packaged offline plugins including Timelapse Studio |
