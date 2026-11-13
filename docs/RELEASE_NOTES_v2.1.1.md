# PhotoCraft Companion v2.1.1 (Official Stable Release)

[![GitHub Release](https://img.shields.io/badge/Release-v2.1.1-007acc.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.1.1)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Companion For](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)
[![Discord](https://img.shields.io/badge/Discord-Companion_Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

---

> [!IMPORTANT]
> ### 🚀 OFFICIAL STABLE RELEASE v2.1.1 / ОФИЦИАЛЬНЫЙ СТАБИЛЬНЫЙ РЕЛИЗ v2.1.1
> **English:** PhotoCraft Companion **v2.1.1** introduces the official flagship cloud AI suite **Flexy Studio** (powered by XpucT's [Flexy AI](https://flexy-ai.com/main)), industrial-grade **Context Crop & Seamless Patch Compositing** (preserving 100% native canvas resolution without server downscaling), declarative selection mask contracts, complete elimination of the Windows console window on launch (PE Subsystem 2), multi-monitor geometry persistence, and total modular codebase decomposition. All 270+ workspace tests and 8 architectural linters pass with 100% success under Resource Guard (`-j 1`).
>
> **Русский:** В версии **v2.1.1** представлен официальный облачный флагман **Студия Flexy AI** (на базе платформы [Flexy AI](https://flexy-ai.com/main) от XpucT), промышленный алгоритм **Context Crop и бесшовное In-Place слияние заплаток** (100% сохранение нативной четкости фото без серверного сжатия), декларативный протокол масок выделения, полное искоренение консольного окна при запуске на Windows (PE Subsystem 2), многомониторное сохранение геометрии окон и тотальная модульная декомпозиция кодовой базы. Все 270+ тестов воркспейса и 8 архитектурных линтеров пройдены со 100% успехом под Resource Guard (`-j 1`).

---

## 🌟 What's New in v2.1.1 / Что нового в версии 2.1.1

### 🇬🇧 English

#### ☁️ 1. Official Cloud AI Integration: Flexy Studio (by XpucT)
* **Flagship Cloud Suite ([flexy-ai.com/main](https://flexy-ai.com/main))**: Direct REST API integration with XpucT's advanced unconstrained neural platform.
* **In-App Features (PhotoCraft Canvas Optimized)**:
  - **Neural Generation & ControlNet Guidance**: Txt2Img with native Cyrillic text rendering in `"quotes"`, guided by active layer contours (*Line Art*, *Canny*, *Depth*, *Scribble*, *Pose*);
  - **Smart Eraser & Generative Fill**: Clean object removal and prompt-based regional synthesis;
  - **1-Click Studio Retouch**: *Natural* (authentic pore microtextures) and *Studio Glamour* portrait skin smoothing;
  - **Canvas Outpaint & Background Changer**: Panoramic canvas expansion and realistic background synthesis;
  - **B&W Colorization & Vintage Aging**: Deep learning archival photo color restoration and chronological age styling.
* **Full Multimodal Web Ecosystem ([flexy-ai.com/main](https://flexy-ai.com/main))**:
  - Virtual outfit try-on (*Clothes Swap*), *Face Swap*, cinematic *Video Generation* (Txt2Video/Img2Video), *SVG Vectorizer*, and *3D Mesh Synthesis* available on the web platform.
  - Transparent API billing: credits never expire.

#### 🎯 2. Industrial Context Crop & Seamless In-Place Patch Compositing (SPEC-004 / BUG-126)
* **100% Native Resolution Preservation**: Instead of sending the full 4K/8K canvas to the cloud (which triggers a forced 1536 px server downscale), Companion extracts the defect region with 35% context padding. Defects are processed at **100% native camera resolution** with zero blur.
* **Large Selection Fallback**: Selections larger than 1536 px downscale strictly the cropped patch via Catmull-Rom, upscaling the result back with precision before compositing.
* **Full-Canvas Transparent Patch Layer**: Composites generated pixels strictly within the defect mask onto an `orig_w x orig_h` transparent canvas, ensuring pixel-perfect alignment.
* **In-Place Layer Fusion**: The `Replace in Active Layer (In-Place)` toggle automatically fuses results into the active layer (`layer.mergeDown`) or preserves an aligned non-destructive patch layer.
* **Clipping Protection**: Dispatches `client.deselect()` before `pasteInPlace`, preventing PhotoCraft selection bounds from clipping the patch.

#### 🎭 3. Declarative Selection Mask Protocol (SPEC-003 / BUG-125)
* **Non-Destructive Mask Extraction**: `copy_active_layer_and_selection_mask()` in `BridgeOperations` captures both full layer and binary `GrayImage` selection mask without altering canvas selections.
* **HTTP 400 Prevention**: Solved missing `mask` field errors in cloud inpainting APIs.
* **Reactive Selection Validation**: Subscribed to `document.selection` (`IPC_SUB_SELECTION_STATE`), dynamically dismissing warning banners the moment an artist draws a selection.

#### 🔕 4. Windows Console Window Elimination (BUG-127)
* **PE Subsystem 2 GUI**: Restored `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in `companion_app/src/main.rs`.
* The application launches cleanly as a pure Windows GUI process with **zero console or terminal window flashes**.

#### 🧩 5. Comprehensive Modular Codebase Decomposition (SPEC-002 / Chapter 132)
* Decomposed all legacy monolith files exceeding 800 lines into structured modular packages with lightweight facades:
  - `companion_app/main.rs` (1,561 lines) -> `main.rs`, `discovery.rs`, `dispatcher.rs`, `raster.rs`, `session_sync.rs`, `cloud_engine.rs`;
  - `core_contracts/plugin.rs` (1,348 lines) -> `plugin/` package;
  - `overlay_ui/manager.rs` (3,469 lines) -> `manager/` package;
  - `overlay_ui/lib.rs` (2,035 lines) -> `overlay/` package;
  - `overlay_ui/toolstrip.rs` (1,151 lines) -> `toolstrip/` package;
  - `overlay_ui/i18n/` dictionary split into static data tables (`table_part1.rs`, `table_part2.rs`).
* Introduced automated `file_size_lint_test.rs` enforcing the strict 800-line Hard Cap.

#### 🖥️ 6. Multi-Monitor Window Geometry Persistence (SPEC-001 / BUG-123)
* Remembers window positions `(x, y)` and dimensions per plugin across launches.
* Multi-monitor clamping prevents windows from opening off-screen when external displays are disconnected.

---

### 🇷🇺 Русский

#### ☁️ 1. Официальная интеграция облачного партнёра: Студия Flexy AI (автор — XpucT)
* **Флагманская облачная студия ([flexy-ai.com/main](https://flexy-ai.com/main))**: Прямая интеграция с передовой нейросетевой платформой от создателя *Win 10 Tweaker* и моделей *Deliberate*.
* **Инструменты прямо в PhotoCraft (плагин Flexy Studio)**:
  - **Генерация артов и ControlNet**: Txt2Img с поддержкой текста на кириллице в `"кавычках"`, управление композицией по слою (*Line Art*, *Canny*, *Depth*, *Scribble*, *Pose*);
  - **Умный ластик и генеративная заливка**: Чистка проводов, мусора, лишних людей и дорисовка по описанию;
  - **Студийная ретушь в 1 клик**: *Естественная* (с сохранением микротекстуры пор) и *Глянцевая* ретушь кожи;
  - **Расширение полотна (Outpaint) и замена фона**: Достройка панорамы и фотореалистичная смена окружения;
  - **Колоризация ч/б и состаривание**: Нейросетевое восстановление цвета старых снимков и возрастная стилизация.
* **Полная веб-платформа Flexy ([flexy-ai.com/main](https://flexy-ai.com/main))**:
  - Примерка и смена одежды (*Clothes Swap*), перенос лиц (*Face Swap*), генерация видео (*Img2Video/Txt2Video*), векторизация в SVG и 3D-моделирование.
  - Честный биллинг: кредиты API никогда не сгорают в конце месяца.

#### 🎯 2. Промышленный контекстный кроп и бесшовное In-Place слияние (SPEC-004 / BUG-126)
* **100% нативная чёткость матрицы**: Вместо отправки всего 4K/8K холста (вызывавшего сжатие сервером до 1536 px), вырезается область дефекта с 35% контекстом. Обработка идёт в **100% нативном разрешении без потери резкости**.
* **Откат для больших дефектов**: Если выделение превышает 1536 px, локальный Catmull-Rom ресемплинг применяется только к вырезанному кропу с точным обратным восстановлением.
* **Полноразмерная прозрачная заплатка**: Сгенерированные пиксели переносятся на прозрачный холст `orig_w x orig_h` строго по маске, гарантируя идеальную соосность 1:1.
* **In-Place слияние**: Переключатель `Заменять на активном слое (In-Place)` автоматически запекает результат в исходный слой (`layer.mergeDown`) или сохраняет аккуратный слой-заплатку.
* **Защита от клиппинга**: Вызов `client.deselect()` перед вставкой исключает обрезку слоя контуром выделения PhotoCraft.

#### 🎭 3. Декларативный протокол масок выделения (SPEC-003 / BUG-125)
* **Неразрушающий захват**: Метод `copy_active_layer_and_selection_mask()` в `BridgeOperations` извлекает слой и ч/б маску `GrayImage` без сброса выделения пользователя.
* **Ликвидация HTTP 400**: Полное устранение сбоев из-за отсутствия поля `mask` в облачных API.
* **Реактивная валидация**: Мгновенный сброс предупреждающего баннера при выделении области на холсте (`document.selection`).

#### 🔕 4. Ликвидация консольного окна при запуске (BUG-127)
* **Подсистема PE Subsystem 2 GUI**: Возвращена директива `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` в `main.rs`.
* Приложение запускается как чистый GUI-процесс Windows **без мелькания и появления окон терминала**.

#### 🧩 5. Тотальная модульная декомпозиция монолитов (SPEC-002 / Глава 132)
* Все файлы свыше 800 строк декомпозированы в пакетные модули с лаконичными фасадами:
  - `companion_app/main.rs` (1,561 строк) -> `main.rs`, `discovery.rs`, `dispatcher.rs`, `raster.rs`, `session_sync.rs`, `cloud_engine.rs`;
  - `core_contracts/plugin.rs` (1,348 строк) -> пакет `plugin/`;
  - `overlay_ui/manager.rs` (3,469 строк) -> пакет `manager/`;
  - `overlay_ui/lib.rs` (2,035 строк) -> пакет `overlay/`;
  - `overlay_ui/toolstrip.rs` (1,151 строк) -> пакет `toolstrip/`;
  - Словарь i18n вынесен в статические таблицы данных (`table_part1.rs`, `table_part2.rs`).
* Внедрен автоматический линтер `file_size_lint_test.rs` (Hard Cap 800 строк).

#### 🖥️ 6. Многомониторное сохранение геометрии (SPEC-001 / BUG-123)
* Автоматическое сохранение координат `(x, y)` и размеров окон плагинов.
* Защита от вылета окон за пределы видимости при отключении мониторов.

---

## 📦 Downloads / Дистрибутивы

| Platform / Платформа | Package / Пакет | Description / Описание |
| :--- | :--- | :--- |
| **Windows x64 (Portable)** | `PhotoCraft-Companion-v2.1.1-windows-x64-portable.zip` | Портабельный архив (распаковать и запустить `photocraft-companion.exe`) |
| **Windows x64 (Installer)** | `PhotoCraft-Companion-v2.1.1-Setup.exe` | Инсталлятор Inno Setup с ярлыками и интеграцией в систему |
| **macOS (Universal)** | `PhotoCraft-Companion-v2.1.1-macOS-universal.zip` | Бандл приложения `PhotoCraft Companion.app` (Apple Silicon & Intel) |
| **Linux x64 (Tarball)** | `photocraft-companion-v2.1.1-linux-x86_64.tar.gz` | Бинарный архив для дистрибутивов Linux (X11 / Wayland) |

---

## ⚙️ Verification Results & Quality Gate (Resource Guard `-j 1`)

* `cargo fmt --all -- --check` — **0 diffs**.
* **100% workspace tests passing (270+ tests across all crates)**:
  - `companion_app`: 18 tests;
  - `ai_engine`: 11 tests + `architecture_integrity_test` + `dev_regulations_lint_test`;
  - `core_contracts`: 54 tests + `file_size_lint_test`;
  - `overlay_ui`: 85 unit + 24 integration tests + `i18n_lint_test`;
  - `photocraft_bridge`: 36 tests + `session_colors_test`;
  - `plugin_validator`: 15 tests;
  - `raster_engine`: 9 tests + `extrude_test`.
* Plugin ecosystem validator confirmed **16/16 plugins with 0 errors**.
