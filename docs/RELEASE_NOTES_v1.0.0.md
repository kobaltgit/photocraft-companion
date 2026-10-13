# Release Notes: PhotoCraft Companion v1.0.0

**Release Tag:** `v1.0.0`  
**Date:** 2026-10-13  
**Status:** Stable Production Release  
**Author:** kobaltGIT  

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.0.0** — an extensible, open-source companion application and autonomous plugin runtime designed specifically for the **PhotoCraft** graphics editor. 

This release marks the transition from early prototypes to a production-grade, zero-compromise ecosystem: an In-App Marketplace, local GPU-accelerated neural tools (Inpainting, Super-Resolution, Subject Cutout), declarative Zero-Core contracts, and cross-platform installation packages for Windows, macOS, and Linux.

---

### 🌟 Key Highlights & Major Additions

#### 1. In-App Community Plugin Marketplace (Store)
* **One-Click Installation:** Discover and install community plugins directly inside the Companion without manual file downloads or folder extraction.
* **Instant Reactive Hot-Reload:** Installed plugins appear immediately on the floating toolbar without restarting the application or editor.
* **Lightweight Packages (< 1 KB):** Plugins are distributed as `.pcpl` archives containing manifests and vector icons, while multi-hundred-megabyte neural weights can be downloaded on demand.
* **Offline-Resilient Caching:** Automatic local caching of the registry and icons ensures the store remains accessible even during network drops.

#### 2. Zero-Core AI Runtime & 4 Computer Vision Paradigms
* **Strict Zero-Core Standard:** Zero plugin-specific logic hardcoded into the companion core (`companion_app`, `ai_engine`, `photocraft_bridge`). Any developer can create a plugin folder with a `plugin.json` and `.onnx` model, and it runs immediately.
* **4 Universal AI Paradigms:**
  - `segmentation_mask` — High-precision object isolation (Bria AI RMBG-1.4).
  - `super_resolution` — Nx upscale with seam-blended tiling (Real-ESRGAN Compact 2x).
  - `inpainting` — Context-aware generative fill and object erasure based on active lasso contours (LaMa).
  - `image_to_image` — Direct pixel filters, denoise, and enhancement.
* **Resilient Hardware Acceleration:** DirectML GPU execution (NVIDIA RTX, AMD Radeon, Intel Arc) with automatic fallback to CPU upon unsupported operator errors.
* **Seam-Blended Tiling:** High-resolution canvases (up to 8K+) are processed via tiled chunks with linear seam fading, preventing VRAM overflow.

#### 3. Floating Two-Tier Toolstrip & Overlay UI
* **Two-Tier Layout:** Utility controls (orientation toggle, settings, minimize) sit neatly above dynamic plugin buttons.
* **Orientation Toggle:** One-click instant switch between Horizontal (78 px) and Vertical (50 px) layouts with zero clipping for any button count.
* **Pure Vector SVG Rendering:** Strict vector rendering via `resvg` with automatic luminance adaptation for Dark and Light themes.
* **Mini-HUD Status Badge:** Floating live timer badge under the toolbar showing processing time and preventing duplicate clicks.
* **Persistent Position:** Screen coordinates and orientation are remembered across sessions via drag-and-drop detection.

#### 4. Plugin Manager & In-App Self-Updater
* **Comprehensive Tabs:** `[ General ]`, `[ Plugins ]`, `[ Store ]`, `[ About ]`.
* **Download Progress Feedback:** Visual progress bar in plugin cards displaying download percentage, transferred size, and real-time network speed in MB/s.
* **In-App Updater:** Automatically checks GitHub Releases, compares SemVer, and triggers one-click platform updates.
* **Connection Diagnostics:** 3-state real-time socket indicator (`Connected`, `Auth Required`, `Offline`) with native file token picker (`rfd`).

#### 5. Ecosystem Validation & Publishing Automation
* **`plugin_validator` CLI:** Rust-based automated validation utility checking schema integrity, enforcing pure vector SVG icons (rejecting embedded raster base64), verifying model SHA-256 hashes, and assembling packages.
* **Marketplace CI:** Automated GitHub Actions pipeline publishing plugins and the index to `releases/tag/plugins`.

---

### 📦 Platform Packages & Installation

| Platform | Package / Installer | Description |
| :--- | :--- | :--- |
| **Windows (x64)** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Inno Setup installer with tray autostart and Start Menu shortcuts |
| **Windows (x64)** | `PhotoCraft-Companion-v1.0.0-windows-x64-portable.zip` | Standalone portable archive (no registry modification) |
| **macOS (Universal)** | `PhotoCraft-Companion-macos.dmg` | Drag-and-drop DMG image for `/Applications` |
| **macOS (Universal)** | `PhotoCraft-Companion-v1.0.0-macos-portable.tar.gz` | Standalone `.app` bundle archive |
| **Linux (x86_64)** | `PhotoCraft-Companion-v1.0.0-linux-amd64.deb` | Debian package with desktop entry and application menu integration |
| **Linux (x86_64)** | `PhotoCraft-Companion-v1.0.0-linux-x86_64-portable.tar.gz` | Standalone binary archive |

---

### 🧪 Verification & Quality Gate
* **100% Pass Rate:** All **78 workspace tests** passed (`cargo test --workspace -j 1`).
* **Strict Resource Guard:** Compiles with `-j 1` and `codegen-units = 1`.
* **Zero Linter Warnings:** `cargo clippy --workspace --all-targets -j 1 -- -D warnings` passed with 0 warnings.
* **Format Compliance:** Fully formatted according to `cargo fmt --all -- --check`.
* **Localization Guard:** `i18n_lint_test` verifies 100% localization parity across English and Russian.

---
---

# 🇷🇺 Описание релиза на русском языке

Выпущена стабильная версия **PhotoCraft Companion v1.0.0** — автономного компаньона и рантайма плагинов для графического редактора **PhotoCraft**.

Версия 1.0.0 знаменует завершение перехода от экспериментального инструмента к зрелой, полнофункциональной экосистеме: встроенный магазин расширений, локальные нейросети без облака (Generative Fill, Super-Resolution, вырезание фона), 100% декларативные контракты без хардкода ядра и нативные установщики для Windows, macOS и Linux.

---

### 🌟 Ключевые нововведения релиза

#### 1. Встроенный магазин плагинов (In-App Marketplace)
* **Установка в 1 клик:** Витрина расширений прямо во вкладке «Магазин» Менеджера плагинов без ручного поиска файлов в браузере.
* **Мгновенный Hot-Reload:** Установленный плагин сразу появляется на плавающей панели инструментов без перезапуска программы.
* **Легковесные пакеты (.pcpl):** Архив плагина весит менее 1 КБ; тяжелые веса нейросетей загружаются по запросу с живым прогресс-баром и расчетом скорости (МБ/с).
* **Отказоустойчивое кэширование:** Каталог сохраняется локально, позволяя открывать магазин и просматривать плагины даже при отсутствии интернета.

#### 2. Универсальное Zero-Core ИИ-ядро и 4 парадигмы нейросетей
* **Железный запрет хардкода:** В коде ядра нет ни строчки под конкретные плагины. Любой разработчик может положить в папку плагина файл `plugin.json` и веса `.onnx`, и расширение заработает сразу.
* **4 парадигмы компьютерного зрения:**
  - `segmentation_mask` — Интеллектуальное вырезание объектов (Bria AI RMBG-1.4).
  - `super_resolution` — Сверхразрешение 2x с бесшовным тайлингом (Real-ESRGAN Compact).
  - `inpainting` — Контекстно-зависимая генеративная заливка без промптов по выделению лассо (LaMa).
  - `image_to_image` — Прямые фильтры, денойз и стилизация.
* **Адаптивный DirectML / CPU Fallback:** Аппаратное ускорение на видеокартах NVIDIA RTX, AMD Radeon и Intel Arc с автоматическим прозрачным переключением на CPU при ошибках специфичных слоев нейросетей.
* **Бесшовный тайлинг:** Обработка холстов вплоть до 8K блоками с линейным сглаживанием швов, исключающим переполнение видеопамяти (VRAM).

#### 3. Двухъярусный плавающий тулстрип (Two-Tier Toolstrip)
* **Продуманная эргономика:** Сервисные кнопки (переключение ориентации, настройки, сворачивание в трей) размещены строго над кнопками инструментов.
* **Переключение ориентации:** Мгновенный переход между горизонтальным (высота 78 px) и вертикальным (ширина 50 px) режимами без обрезания кнопок при любом числе плагинов.
* **Векторные авторские SVG-иконки:** Строгий векторный рендеринг через `resvg` с адаптацией контраста под тёмную и светлую темы.
* **Плавающий мини-HUD:** Всплывающий бейдж с живым таймером операции и защитой от повторных нажатий во время вычислений.
* **Память позиции:** Панель не отображается на панели задач Windows и запоминает координаты на экране после перетаскивания.

#### 4. Менеджер плагинов и автоапдейтер
* **Вкладки интерфейса:** `[ Общие ]`, `[ Плагины ]`, `[ Магазин ]`, `[ О программе ]`.
* **Прогресс-бар загрузки моделей:** Индикация процентов, переданных мегабайт и скорости скачивания в реальном времени.
* **Встроенный автоапдейтер:** Проверка свежих релизов на GitHub, сравнение версий по SemVer и скачивание установщика в один клик.
* **Диагностика подключения:** 3-позиционный статус сокета (`Подключено`, `Требуется авторизация`, `Офлайн`) и диалог выбора токена через проводник (`rfd`).

#### 5. CI/CD экосистемы и валидатор плагинов
* **Утилита `plugin_validator`:** Проверка манифестов, запрет внедренного растра в SVG, валидация контрольных сумм SHA-256 моделей и автоматическая сборка пакетов `.pcpl`.
* **Автоматическая публикация:** GitHub Actions публикует проверенные плагины в специальный релиз `plugins` и обновляет таблицы витрины в `README`.

---

### 📦 Дистрибутивы и установка

| Платформа | Пакет / Инсталлятор | Описание |
| :--- | :--- | :--- |
| **Windows (x64)** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Inno Setup инсталлятор с автозапуском в трее и меню «Пуск» |
| **Windows (x64)** | `PhotoCraft-Companion-v1.0.0-windows-x64-portable.zip` | Автономный портабельный архив без записей в реестр |
| **macOS (Universal)** | `PhotoCraft-Companion-macos.dmg` | DMG-образ с перетаскиванием в папку `/Applications` |
| **macOS (Universal)** | `PhotoCraft-Companion-v1.0.0-macos-portable.tar.gz` | Автономный `.app` архив |
| **Linux (x86_64)** | `PhotoCraft-Companion-v1.0.0-linux-amd64.deb` | Debian-пакет с интеграцией в системное меню приложений |
| **Linux (x86_64)** | `PhotoCraft-Companion-v1.0.0-linux-x86_64-portable.tar.gz` | Автономный исполняемый архив |

---

### 🧪 Контроль качества (Quality Gate)
* **100% покрытие:** Все **78 тестов воркспейса** завершились со 100% успехом (`cargo test --workspace -j 1`).
* **Контроль ресурсов ПК:** Сборка со строгим ограничением (`-j 1`, `codegen-units = 1`).
* **Чистый код:** 0 замечаний Clippy (`cargo clippy --workspace --all-targets -j 1 -- -D warnings`).
* **Стандарт форматирования:** Полное соответствие `cargo fmt --all -- --check`.
* **Линтер локализации:** Модуль `i18n_lint_test` гарантирует полную синхронизацию переводов интерфейса на русском и английском языках.
