# Release Notes: PhotoCraft Companion v1.3.0

**Release Tag:** `v1.3.0`  
**Date:** 2026-10-25  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion advances to milestone version **1.3.0** — introducing **Clean Glyph Internationalization (15 UI Languages, 22 Host Parameter Languages)**, the automated **Smart Parameter Retry Engine**, anti-tofu safeguards with safe English font fallback, native operating system locale auto-detection with dialect normalization, modernized multi-monitor Offline UX, and multilingual community plugin manifests.

---

### 🌟 Key Highlights & Major Additions

#### 1. Host Internationalization (22 Languages) & Smart Parameter Retry (BUG-080)
* **Direct Alignment with Upstream PhotoCraft BCP 47 Languages:**
  - Derived from PhotoCraft's localization catalog, `core_contracts::host_i18n` establishes exhaustive mappings for all 22 official host languages: `en`, `ja`, `zh-hans`, `zh-hant`, `es`, `ru`, `uk`, `sr`, `cs`, `fr`, `id`, `ko`, `pl`, `de`, `pt-br`, `el`, `nl`, `it`, `hu`, `tr`, `vi`, `hi`.
* **Smart Parameter Retry Engine:**
  - Solves the localized parameters dilemma (e.g. blend modes: `"Soft Light"` in English vs `"Weiches Licht"` in German vs `"Мягкий свет"` in Russian).
  - Workflows first dispatch canonical English parameters. If PhotoCraft rejects the command with a parameter error under a localized edition, Companion automatically translates parameters on the fly and retries seamlessly in milliseconds.
  - Zero-Core Hardcoding: 0 plugin-specific branches; fully neutral declarative runtime.

#### 2. Clean Glyph UI Localization & Anti-Tofu Safeguards (BUG-081)
* **15 Guaranteed UI Languages (`Language::UI_SUPPORTED`):**
  - Confined UI rendering strictly to Latin and Cyrillic character sets (`en`, `ru`, `fr`, `it`, `de`, `es`, `pt-br`, `pl`, `cs`, `nl`, `tr`, `hu`, `uk`, `sr`, `id`) that render with 100% fidelity in egui.
* **Safe Font Fallback (`safe_ui_language()`):**
  - If PhotoCraft runs in an environment with complex or East Asian scripts (CJK, Hindi, Greek, Vietnamese), Companion UI automatically and safely falls back to English, eliminating Unicode tofu squares (`□□□`) across buttons and menus (Rule 6: Vector-First UI).
* **Industrial UI Dictionary (113 Unique Keys, 1,469 Translations):**
  - Full translation coverage across all tabs, status badges, action buttons, onboarding alerts, and modal dialogs in `crates/overlay_ui/src/i18n/mod.rs`.
  - Backed by an automated test suite (`i18n_lint_test.rs`) enforcing font atlas cleanliness.

#### 3. Native Operating System Locale Auto-Detection (BUG-083)
* **Seamless Dialect Normalization (`sys-locale`):**
  - When PhotoCraft is set to "Auto" (`"language": "auto"` in `preferences.json`), Companion queries the OS locale hierarchy via `sys_locale::get_locales()`.
  - Automatically strips regional and dialect subtags (`ru-KZ` -> `ru`, `cs-CZ` -> `cs`, `de-AT` -> `de`, `fr-CA` -> `fr`), encoding suffixes (`.UTF-8`), and modifiers (`@latin`).
  - Guarantees instant synchronization between PhotoCraft and Companion upon initial startup on Windows, macOS, and Linux.

#### 4. Modernized Standby / Offline UX
* **Free Window Dragging & Position Persistence:**
  - Waiting window can be freely dragged across multi-monitor setups using a centered vector grip handle. Coordinates persist in `CompanionConfig::offline_window_position`.
* **Always-on-Top (Pin) & Minimize-to-Tray:**
  - Vector pushpin button toggles Always-on-Top floating state. Dedicated minimize-to-tray button parks the window while background socket polling continues on port 18492.

#### 5. Multilingual Plugin Manifests & RMBG-1.4 v1.1.1 Release (BUG-082)
* **`locales` Dictionary in `plugin.json`:**
  - All 10 official plugins updated with native translations for major world languages.
* **Layer Mask Deselect Resilience:**
  - Added `"ignore_errors": true` to the final `select.deselect` step in the `mask` sub-action of AI Remove Background (`remove_bg` v1.1.1), eliminating false-positive bridge error toasts when PhotoCraft automatically consumes active selections.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion достигает знаковой версии **1.3.0** — масштабное обновление экосистемы: **интернационализация (15 языков интерфейса, 22 языка параметров)**, интеллектуальный механизм повтора параметров **Smart Parameter Retry**, чистая локализация интерфейса с защитой от квадратов «тофу», нативное автоопределение системной локали ОС с отсечением диалектов, модернизированный мультимониторный Offline UX и мультиязычные манифесты плагинов.

---

### 🌟 Ключевые нововведения

#### 1. Интернационализация параметров (22 языка PhotoCraft) и Smart Parameter Retry (BUG-080)
* **Синхронизация со стандартами BCP 47 PhotoCraft:**
  - Модуль `core_contracts::host_i18n` содержит исчерпывающие таблицы соответствия параметров на всех 22 официальных языках графического редактора: `en`, `ja`, `zh-hans`, `zh-hant`, `es`, `ru`, `uk`, `sr`, `cs`, `fr`, `id`, `ko`, `pl`, `de`, `pt-br`, `el`, `nl`, `it`, `hu`, `tr`, `vi`, `hi`.
* **Движок Smart Parameter Retry:**
  - Устраняет проблему языковых параметров в неанглийских сборках редактора (например, режимы наложения: `"Soft Light"` в английском, `"Weiches Licht"` в немецком, `"Мягкий свет"` в русском).
  - Компаньон сначала отправляет канонический английский параметр. Если локализованный редактор возвращает ошибку параметра, рантайм на лету переводит значение на активный язык PhotoCraft и мгновенно повторяет команду.
  - 100% Zero-Core Hardcoding: разработчикам плагинов не нужно писать языковые развилки.

#### 2. Чистая локализация интерфейса и защита от квадратов «тофу» (BUG-081)
* **15 проверенных языков интерфейса (`Language::UI_SUPPORTED`):**
  - Отображение UI ограничено наборами латиницы и кириллицы (`en`, `ru`, `fr`, `it`, `de`, `es`, `pt-br`, `pl`, `cs`, `nl`, `tr`, `hu`, `uk`, `sr`, `id`), гарантированно поддерживаемыми шрифтовым атласом egui.
* **Безопасный шрифтовой фоллбэк (`safe_ui_language()`):**
  - При запуске PhotoCraft в средах со сложной письменностью (CJK, хинди, греческий, вьетнамский) интерфейс компаньона безопасно переключается на английский, исключая появление нечитаемых символов «тофу» (`□□□`) на кнопках и в меню (Правило 6: Vector-First UI).
* **Словарь на 113 уникальных UI-ключей (1469 переводов):**
  - Полный перевод вкладок Менеджера, кнопок действий, статус-баров, онбординга и модальных окон в `crates/overlay_ui/src/i18n/mod.rs`.
  - Автоматический линтер `i18n_lint_test.rs` гарантирует чистоту шрифтового атласа при любых правках.

#### 3. Нативное автоопределение системной локали ОС (BUG-083)
* **Нормализация региональных диалектов (`sys-locale`):**
  - При установке языка в режим «Авто» (`"language": "auto"` в `preferences.json`) компаньон опрашивает ОС через `sys_locale::get_locales()`.
  - Автоматически отсекает региональные суффиксы (`ru-KZ` -> `ru`, `cs-CZ` -> `cs`, `de-AT` -> `de`, `fr-CA` -> `fr`), кодировки (`.UTF-8`) и модификаторы (`@latin`).
  - Обеспечивает мгновенную синхронизацию языка компаньона с языком PhotoCraft из коробки на Windows, macOS и Linux.

#### 4. Модернизированный режим ожидания (Offline UX)
* **Свободное перемещение и сохранение координат:**
  - Окно ожидания плавно перетаскивается за центральную рукоятку по любым мониторам. Координаты автоматически сохраняются в `CompanionConfig::offline_window_position`.
* **Закрепление поверх окон (Pin) и сворачивание в трей:**
  - Векторная булавка переключает режим Always-on-Top. Кнопка сворачивания паркует окно в системный трей без остановки фонового мониторинга сокета 18492.

#### 5. Мультиязычные манифесты плагинов и релиз RMBG-1.4 v1.1.1 (BUG-082)
* **Словарь `locales` в `plugin.json`:**
  - Все 10 плагинов каталога получили переводы описаний и суб-действий на ведущие языки.
* **Отказоустойчивость масок слоя:**
  - В шаге `select.deselect` суб-действия `mask` плагина AI Remove Background выставлен `"ignore_errors": true`, исключающий ложные уведомления об ошибках при автоматическом снятии выделения движком PhotoCraft.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.3.0-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.3.0_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.3.0-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.3.0-macos-portable.tar.gz` | Портабельный архив для macOS |

---

## 🧩 Community Plugins Marketplace / Каталог плагинов

Все официальные плагины (`.pcpl`) доступны для установки в один клик через вкладку **«Магазин плагинов»** в менеджере:

1. **AI Remove Background** (`v1.1.1`) — Нейросетевое вырезание фона (RMBG-1.4) с поддержкой вырезанного слоя и маски слоя.
2. **AI Depth Estimation** (`v1.1.0`) — 16-битная карта глубины (Depth Anything V2 Small) с нормализацией и инверсией.
3. **AI Generative Fill & Inpaint** (`v1.0.0`) — Бесшовное удаление объектов и умная заливка по маске (LaMa).
4. **AI Super-Resolution 2x** (`v1.0.0`) — 2-кратный апскейл холста с шовным тайлингом (Real-ESRGAN Compact).
5. **Vectorize Selection & Layer** (`v1.0.0`) — Алгоритмическая векторизация пикселей в кривые Bezier SVG (VTracer).
6. **ChromaSense Palette Lab** (`v1.0.0`) — Двунаправленная палитра гармоний, WCAG контрастность и CVD симуляция зрения.
7. **Pro Dodge & Burn 50% Gray Suite** (`v1.0.0`) — Студийный стек светотеневой лепки Soft Light и вспомогательного Ч/Б слоя.
8. **Cinematic Teal & Orange Suite** (`v1.0.0`) — Голливудская цветокоррекция (3D LUT + S-кривая + защита тона кожи).
9. **Frequency Separation Beauty Retouch** (`v1.0.0`) — Частотное разложение текстуры и тона для бьюти-ретуши.
10. **B&W Studio Monochrome Master** (`v1.0.0`) — Студийная монохромная конвертация с селективным управлением спектром.
