# Release Notes: PhotoCraft Companion v1.2.3

**Release Tag:** `v1.2.3`  
**Date:** 2026-10-22  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.2.3** — bringing critical Linux system tray stability fixes (resolving KDE Plasma/Wayland crashes), a comprehensive suite of security and resilience enhancements from our technical audit (Zip-Slip prevention, weights preservation, IPC timeouts, ONNX session LRU cache, non-blocking clipboard I/O), and intelligent multi-version token conflict diagnostics.

---

### 🌟 Key Highlights & Major Additions

#### 1. Linux System Tray Resiliency & Early GTK Initialization (BUG-078 / GitHub Issue #3)
* **Early Main-Thread GTK Init:** Added targeted Linux dependencies (`gtk = { workspace = true }`) and invoked `gtk::init()` at the very beginning of `main()` on the main OS thread. This ensures GTK widgets wrapped by `tray-icon` and `muda` are properly initialized.
* **Safe Fallback Guard:** Tray creation is strictly conditioned on `gtk_initialized`. In headless, containerized, or restrictive Wayland environments where GTK cannot initialize, Companion logs a warning and continues running smoothly without crashing, keeping the floating toolbar and Plugin Manager fully functional.

#### 2. Comprehensive Security & Audit Remediation (F-01..F-09)
* **Zip-Slip & Zip-Bomb Protection (F-03):** Implemented strict path sanitization via `entry.enclosed_name()`, root prefix stripping (`<sanitized_id>/`), component normalization, and a 512 MB uncompressed size limit on `.pcpl` archives.
* **Model Weights Preservation (F-03):** When updating a `.pcpl` plugin, existing neural network weights in `target_dir/models` are automatically preserved, eliminating redundant multi-hundred-megabyte re-downloads.
* **IPC Bridge Reliability & Timeouts (F-01, F-02):** Added JSON-RPC response ID verification (`resp.id == Some(id)`) and robust operation timeouts (`DEFAULT_CONNECT_TIMEOUT = 5s`, `DEFAULT_RPC_TIMEOUT = 30s`) with automatic deadlock-free guard resets.
* **Asynchronous Clipboard I/O (F-06):** Offloaded system clipboard reads and writes to `tokio::task::spawn_blocking`, eliminating UI thread stalls during large image transfers.
* **ONNX Session LRU Caching (F-04, F-09):** Implemented an LRU cache (capacity of 3 models) using `Arc<Mutex<Session>>` to speed up repeated AI inference invocations. Faulted DirectML sessions are safely invalidated and purged before falling back to CPU. Mutex locks are poison-tolerant.
* **Rhai Sandbox Limits (F-05):** Constrained custom Rhai plugin scripts to 50,000 operations and expression depth of 64, preventing accidental infinite loops or UI lockups.
* **Model SHA-256 Verification (F-07):** Added streaming SHA-256 checksum validation during background weight downloads with localized error notifications.
* **Automated Linux CI (F-08):** Added Ubuntu Linux runner to GitHub Actions with GTK3, AppIndicator, Wayland, and headless Xvfb test execution.

#### 3. Contextual Multi-Version Token Diagnostics & UX (Phase 77)
* **Intelligent Version Conflict Detection:** The offline modal now detects when PhotoCraft is running on port 18492 but rejected authorization, clearly explaining that multiple portable installations on the system may have caused Companion to read an older `control.token`.
* **Refined Action Hierarchy:** The primary button now leads straight to **"Enter Token in Settings"**, while the secondary button offers **"> Relaunch PhotoCraft"**, accompanied by actionable tips to browse directly to the active editor's token file.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion обновлён до версии **1.2.3** — масштабный выпуск стабильности и безопасности: устранение падения системного трея на Linux (KDE Plasma/Wayland), полный комплекс инженерных улучшений по результатам технического аудита (защита от Zip-Slip, сохранение весов моделей при обновлениях, таймауты IPC-сокета, LRU-кэш ONNX-сессий, неблокирующий буфер обмена) и понятная диагностика конфликтов при наличии нескольких версий PhotoCraft на компьютере.

---

### 🌟 Ключевые нововведения

#### 1. Стабильность системного трея на Linux и инициализация GTK (BUG-078 / GitHub Issue #3)
* **Ранняя инициализация GTK:** Внедрена платформенная зависимость `gtk` под Linux и вызов `gtk::init()` на главном потоке ОС при старте `main()`. Это устранило панику `assert_initialized_main_thread!()` в библиотеках `tray-icon` и `muda`.
* **Защитный фоллбэк:** Создание трея обернуто в проверку `gtk_initialized`. Если графическая подсистема недоступна (например, в Wayland без XWayland), компаньон мягко продолжает работу без трея, гарантируя запуск плавающей панели и менеджера плагинов.

#### 2. Инженерные исправления по итогам технического аудита (F-01..F-09)
* **Безопасность пакетов `.pcpl` и защита весов (F-03):** Строгая проверка путей через `entry.enclosed_name()`, отсечение корневого префикса, проверка `Component::Normal` и лимит распаковки 512 МБ (защита от Zip-Slip и Zip-бомб).
* **Сохранение моделей пользователя (F-03):** При установке новой версии плагина `.pcpl` папка `models/` с тяжелыми весами (.onnx) переносится автоматически, предотвращая повторное скачивание сотен мегабайт.
* **Надежность сетевого моста и таймауты (F-01, F-02):** Внедрена сверка `resp.id == req.id` в ответах JSON-RPC и константы таймаутов (подключение 5с, RPC 30с) с автоматическим сбросом гардов сокета без риска взаимных блокировок.
* **Неблокирующий буфер обмена (F-06):** Операции чтения и записи больших изображений в буфер обмена переведены в `tokio::task::spawn_blocking`, полностью устранив фризы интерфейса.
* **LRU-кэш ONNX-сессий и DirectML (F-04, F-09):** Реализован кэш на 3 сессии инференса на базе `Arc<Mutex<Session>>`. Сбойные сессии DirectML автоматически инвалидируются перед переключением на CPU. Обращения к мьютексам защищены от паники.
* **Ограничения песочницы Rhai (F-05):** Лимит операций (50 000) и глубина стека (64), исключающие зависание интерфейса из-за бесконечных циклов в пользовательских скриптах.
* **Проверка SHA-256 моделей (F-07):** Потоковая валидация контрольной суммы скачиваемых весов с локализованными уведомлениями в UI.
* **Автоматические тесты Linux в CI (F-08):** Добавлен шаг сборки и тестов на Ubuntu в GitHub Actions под headless Xvfb.

#### 3. Контекстное информирование при конфликте версий PhotoCraft (Фаза 77)
* **Понятная диагностика токена:** Модальное окно отсутствия связи теперь точно определяет ситуацию, когда PhotoCraft запущен, но токен отклонен: объясняет причину (наличие нескольких версий на ПК и подтягивание старого токена) и дает понятную инструкцию.
* **Новая иерархия кнопок:** Главной кнопкой стало действие **«Указать токен в настройках»**, а второстепенной — **«> Перезапустить PhotoCraft»**. Добавлена контекстная подсказка в окно настроек.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.2.3-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.2.3_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.2.3-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.2.3-macos-portable.tar.gz` | Портабельный архив для macOS |

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
