# Release Notes: PhotoCraft Companion v2.0.0 — Platform Milestone

**Release Tag:** `v2.0.0`  
**Date:** 2026-11-01  
**Status:** Major Production Milestone Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v2.0.0** represents a milestone generational transition from a lightweight companion toolbar into an open, extensible **Interactive Plugin Platform and Runtime** for the PhotoCraft graphics editor. 

This major milestone delivers the **Photoshop-style Multi-Column Grid Toolstrip**, the **Native Win32 Dual-Slot Crisp Icons architecture**, a collection of advanced interactive plugin studios (**Depth Studio**, **Harmonic Studio**, **PopCraft 3D**, and **Selection Modifiers**), comprehensive **Plugin Manager UI Polish**, and strict **Contract-First Zero-Core Hardcoding guarantees** across all 14 ecosystem plugins.

---

### 🌟 Key Highlights & Major Additions in v2.0.0

#### 1. Multi-Column Grid Toolstrip (Photoshop-Style Toolbar)
* **Automatic Multi-Tier Grid (`MAX_BUTTONS_PER_TIER = 8`):**
  - Up to 8 active plugins: toolstrip remains a compact, monolithic single-column or single-row palette.
  - Automatically expands into 2 columns (or 2 rows) when the 9th plugin is enabled, and 3 columns at 17 plugins.
* **Column-Major Flow Order:**
  - Vertical mode organizes tools into top-to-bottom columns (Column 1: tools 1..8; Column 2: tools 9..16), preserving the classic muscle memory of digital artists accustomed to Photoshop.
  - Horizontal mode flows left-to-right in balanced rows.
* **Adaptive Dynamic Geometry (`calculate_size`):**
  - Fully adaptive window sizing with no visual clipping or hidden buttons, automatically centering the top 6-dot drag handle and orientation/settings controls across expanded widths.

#### 2. Native Win32 Dual-Slot Crisp Icons (BUG-099)
* **Elimination of Titlebar Blurring and Taskbar Pixelation:**
  - Solved the classic cross-platform window toolkit limitation by implementing a native Win32 window message hook (`WM_SETICON`) separating `ICON_SMALL` from `ICON_BIG`.
  - **Titlebar Caption (`ICON_SMALL`):** Renders pixel-perfect 1:1 at physical display metrics (16 px @ 100% DPI, 20 px @ 125%, 24 px @ 150%) from an optimized square 1:1 `assets/icon_small.svg` without vertical aspect-ratio squashing.
  - **Taskbar & Alt+Tab (`ICON_BIG`):** Renders high-resolution 64 px rasters with sub-pixel anti-aliasing via `resvg`.
* **Zero Resource Leaks & Zero 60 FPS Overhead:**
  - HICON handles are cached in a static `OnceLock` singleton created strictly once per process lifetime.
  - One-time window property marking (`PC_CRISP_ICONS`) guarantees $O(1)$ zero runtime overhead during rendering frames.

#### 3. Advanced Multi-Window Interactive Plugin Studios
* 🌫️ **Depth Studio (`plugins/depth_studio`):**
  - AI monocular depth estimation via Depth Anything V2.
  - Click-to-focus optical lens bokeh defocus with non-destructive depth-map scaling.
  - Atmospheric fog simulation with customizable aerial density and independent blend-mode layer export (`Screen` / `Soft Light`).
* 🎨 **Harmonic Studio (`plugins/harmonic_studio`):**
  - Interactive chromatic wheel with 5 classical harmony models (Monochromatic, Analogous, Complementary, Triadic, Tetradic).
  - Real-time palette generation and seamless session color synchronization with PhotoCraft brushes.
  - WCAG 2.1 AAA accessibility contrast grading and CVD color blindness simulation (Deuteranopia, Protanopia, Tritanopia).
* 📐 **PopCraft 3D (`plugins/popcraft_3d`):**
  - Parametric 2.5D extrusion for text, logos, and silhouette layers with directional light shading and drop shadows.
* 🪄 **Selection Modifier Stack (`plugins/selection_mod`):**
  - Non-destructive parametric selection dilation, erosion, edge feathering, smoothing, and border generation.

#### 4. Plugin Manager UI Polish & Adaptive Layout (BUG-100)
* **Vector SVG Icons in Installed Plugins Tab:**
  - 24×24 px vector SVG icons for all installed plugins with dark/light theme tinting and caching in `store_icon_cache`.
  - Clean vector fallback badges (`AI`, `FX`, `ALGO`, `UI`, `RH`, `CLI`) for custom scripts without icons.
* **Unified 3-Column Card Layout:**
  - Restructured cards into Icon, central vertical content stack, and right-aligned action toggle/button.
* **Store Layout Guard (Zero Text Collision):**
  - Enforced adaptive text bounding (`ui.set_max_width`) in catalog cards, ensuring long multilingual descriptions wrap gracefully before action buttons (`Installed`, `Install`, `Update`).

#### 5. Contract-First Zero-Core Hardcoding Architecture
* Centralized `resolve_icon_path` in `core_contracts::PluginManifest`.
* Strict zero-core modification invariant: all 14 official plugins function as declarative packages (`.pcpl`) with zero hardcoded core branches.
* 100% test coverage with automated architectural, localization, and layout guard linters.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v2.0.0** знаменует масштабный переход проекта от вспомогательной плавающей панели к полноценной открытой **платформе и среде выполнения интерактивных плагинов** для графического редактора PhotoCraft.

Этот мажорный релиз приносит **многоколоночный адаптивный тулбар в стиле Photoshop**, **нативную Win32-архитектуру попиксельной чёткости значков Dual-Slot**, целую линейку интерактивных плагин-студий (**Depth Studio**, **Harmonic Studio**, **PopCraft 3D** и **Selection Modifiers**), **полировку интерфейса Менеджера плагинов** и железные гарантии архитектуры **Zero-Core Hardcoding** для всех 14 плагинов экосистемы.

---

### 🌟 Ключевые нововведения в v2.0.0

#### 1. Многоколоночный адаптивный тулбар (Photoshop-Style Grid)
* **Автоматическая сетка с порогом 8 кнопок (`MAX_BUTTONS_PER_TIER = 8`):**
  - До 8 активных плагинов тулбар сохраняет компактный ультратонкий вид в 1 колонку или 1 ряд.
  - При подключении 9-го плагина панель автоматически перестраивается в 2 колонки (или 2 ряда), а при 17 плагинах — в 3 колонки без необходимости ручного переключения.
* **Порядок заполнения по стандарту графических редакторов:**
  - В вертикальном режиме кнопки заполняются столбцами сверху вниз (столбец 1: кнопки 1..8; столбец 2: кнопки 9..16), сохраняя мышечную память художников, привыкших к палитре инструментов Photoshop.
  - В горизонтальном режиме кнопки выстраиваются слева направо рядами.
* **Динамический расчёт геометрии окна (`calculate_size`):**
  - Адаптивный пересчёт габаритов без выхода за границы экрана и без обрезания кнопок с автоматическим центрированием ручки захвата и служебных кнопок.

#### 2. Нативная попиксельная чёткость иконок Win32 Dual-Slot (BUG-099)
* **Победа над замыливанием заголовков и пикселизацией таскбара:**
  - Устранена фундаментальная дилемма кроссплатформенных оконных фреймворков внедрением нативного Win32-хука `WM_SETICON`, разделяющего слоты `ICON_SMALL` и `ICON_BIG`.
  - **Заголовок окна (`ICON_SMALL`):** рендерится точно пиксель-в-пиксель под системный масштаб (16 px при 100% DPI, 20 px при 125%, 24 px при 150%) из специализированного векторного `assets/icon_small.svg` с квадратным viewBox `0 90 612 612` без вертикального сплющивания.
  - **Панель задач Windows и Alt+Tab (`ICON_BIG`):** получают полноразмерный значок 64 px с субпиксельным сглаживанием `resvg`.
* **Нулевой оверхед на 60 FPS и отсутствие утечек GDI:**
  - Дескрипторы `HICON` кэшируются в статическом синглтоне `OnceLock` ровно один раз за жизнь процесса.
  - Проверка свойства окна `PC_CRISP_ICONS` отрабатывает при создании окна за $O(1)$, исключая спам сообщений при отрисовке кадров.

#### 3. Новая линейка интерактивных студий
* 🌫️ **Студия глубины (Depth Studio, `plugins/depth_studio`):**
  - Оценка карты глубины сцены на базе Depth Anything V2.
  - Клик-фокусировка и реалистичное оптическое боке с неразрушающим масштабированием карты.
  - Моделирование атмосферного тумана с настраиваемой воздушной перспективой и экспортом отдельным слоем в режимах `Screen` или `Soft Light`.
* 🎨 **Студия гармонии цвета (Harmonic Studio, `plugins/harmonic_studio`):**
  - Интерактивный цветовой круг с 5 классическими моделями гармонии (монохромная, аналогичная, комплементарная, триада, тетрада).
  - Генерация палитр в реальном времени и горячая синхронизация цвета кисти с холстом PhotoCraft.
  - Проверка доступности контраста текста по стандарту WCAG 2.1 AAA и аппаратная симуляция дальтонизма (дейтеранопия, протанопия, тританопия).
* 📐 **PopCraft 3D (`plugins/popcraft_3d`):**
  - Параметрическая 2.5D экструзия текста, логотипов и силуэтов с расчётом направленного освещения, скосов и падающих теней.
* 🪄 **Стек модификаторов выделения (Selection Modifiers, `plugins/selection_mod`):**
  - Неразрушающее параметрическое расширение, сжатие, растушевка, сглаживание и построение рамок выделения.

#### 4. Полировка интерфейса Менеджера плагинов (BUG-100)
* **Векторные иконки установленных плагинов:**
  - Вывод иконок 24×24 px во вкладке «Плагины» с поддержкой светлой и тёмной темы и кэшированием в `store_icon_cache`.
  - Векторные бейджи-фолбэки (`AI`, `FX`, `ALGO`, `UI`, `RH`, `CLI`) для плагинов без собственных иконок.
* **Единый гармоничный макет карточки:**
  - Трехчастная компоновка: иконка слева, вертикальный блок описания по центру, выровненный чекбокс активности справа.
* **Устранение наплыва текста в карточках Магазина:**
  - Ограничение ширины текстового контейнера (`ui.set_max_width`), гарантирующее перенос строк описания до зоны кнопок `Installed`, `Install` и `Update`.

#### 5. Архитектурная чистота Zero-Core Hardcoding
* Централизация логики разрешения иконок `resolve_icon_path` в `core_contracts::PluginManifest`.
* Все 14 плагинов функционируют строго декларативно через автономные пакеты `.pcpl` без модификации и пересборки ядра Rust.
* 100% покрытие тестами воркспейса с валидацией линтерами архитектуры, локализации и верстки.

---

## 📦 Distribution Packages / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Описание |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Инсталлятор InnoSetup с автообновлением и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.0.0-windows-x64-portable.zip` | Портативный ZIP-архив без установки: распакуйте и запускайте |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_2.0.0_amd64.deb` | Установочный пакет для Ubuntu, Debian, Linux Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.0.0-linux-x64-portable.tar.gz` | Портативный архив для любых дистрибутивов Linux |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска DMG для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v2.0.0-macos-portable.tar.gz` | Портативный архив для macOS |

---

## 🧩 Community Plugins Marketplace / Каталог плагинов v2.0.0

Все 14 официальных плагинов автоматически доступны для установки в один клик через вкладку **«Магазин плагинов»** в менеджере:
* 🤖 **AI Remove Background** (`remove_bg`): Удаление фона на базе RMBG-1.4 с режимами Cutout и Layer Mask.
* 🤖 **AI Depth Map** (`depth_map`): Оценка глубины сцены на базе Depth Anything V2.
* 🤖 **AI Generative Fill** (`inpaint`): Бесшовное заполнение выделенной области текстурой окружающего фона (LaMa).
* 🤖 **AI Super-Resolution** (`upscale`): 2x супер-разрешение на базе Real-ESRGAN с тайлингом.
* 🌫️ **AI Depth Studio** (`depth_studio`): Интерактивная студия атмосферного тумана и оптического размытия боке.
* 🎨 **ChromaSense A11y** (`chromasense`): Проверка контраста WCAG 2.1 и симуляция дальтонизма.
* 🎨 **Harmonic Studio** (`harmonic_studio`): Интерактивный цветовой круг и гармонии палитр в реальном времени.
* 📐 **PopCraft 3D** (`popcraft_3d`): 2.5D экструзия, освещение и падающие тени.
* 🪄 **Selection Modifiers** (`selection_mod`): Неразрушающий стек параметрической модификации выделения.
* 📐 **Vector Tracing** (`vectorize`): Автономная векторизация силуэтов в чистый векторный SVG (vtracer).
* ⚡ **B&W Film Pop** (`black_and_white`): Драматичная ч/б конвертация с контрастной S-кривой.
* ⚡ **Beauty Frequency Separation** (`frequency_separation`): Частотное разложение на текстуру и тон для портретной ретуши.
* ⚡ **Pro Dodge & Burn** (`dodge_and_burn`): Неразрушающее осветление и затемнение на 50% сером слое.
* ⚡ **Cinematic Teal & Orange** (`teal_orange`): Кинематографическая цветокоррекция Teal & Orange.
