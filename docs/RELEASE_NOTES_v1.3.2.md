# Release Notes: PhotoCraft Companion v1.3.2

**Release Tag:** `v1.3.2`  
**Date:** 2026-10-30  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v1.3.2** introduces the **Selection Modifier Stack Studio (`selection_mod`)** — a non-destructive parametric selection modifier panel allowing artists to smoothly expand, contract, feather, smooth, and border-outline selections in real time. This release also establishes generalized architectural invariants across the core runtime, eliminates mouse wheel inertial drift (BUG-089), adds `IntList` dynamic parameter marshaling, and delivers strict adaptive layout guarantees.

---

### 🌟 Key Highlights & Major Additions

#### 1. Selection Modifier Stack Studio (`plugins/selection_mod`)
* **Parametric Non-Destructive Modifiers:**
  - **Expand / Contract:** Dynamic dilation and erosion of the active selection mask with sub-pixel precision ($0 \dots 100\text{ px}$).
  - **Feather:** Gaussian-like edge softening ($0 \dots 100\text{ px}$) for seamless layer masking and blending.
  - **Smooth:** Edge anti-aliasing and corner rounding ($0 \dots 30\text{ px}$) eliminating pixel staircase artifacts.
  - **Border:** Automatic dual-stroke border selection generation ($0 \dots 50\text{ px}$).
* **Single Dedicated Base Channel Architecture:**
  - Fully eliminated temporary channel clutter: all operations rely on a single temporary channel `__pc_companion_base_sel`.
  - Automatic channel cleanup on dialog close, leaving the user's document pristine.
  - Instant, non-destructive **Reset** restoring the original user contour with zero quality degradation.
* **Reactive Selection Tracking:**
  - Real-time synchronization with PhotoCraft's canvas via `document.inspect`: automatically monitors `selection_bounds`.
  - If the artist makes a new selection in PhotoCraft while the panel is open, the stack dynamically captures the new contour as the baseline.

#### 2. Zero-Core Hardcoding & Pure Rhai Pipeline
* **Zero Composite IPC in Core:**
  - Adheres strictly to Rule 3.2: Companion core remains a 100% transparent protocol relayer.
  - The multi-stage selection modification pipeline is executed declaratively via Rhai scripting (`script.rhai`) without a single hardcoded plugin branch in `companion_app`.
* **Dynamic Value Marshaling (`IntList`):**
  - Extended `DynamicValue` in `core_contracts` with `IntList(Vec<i64>)`, allowing seamless array passing (e.g., bounding boxes `[x0, y0, x1, y1]`) between host events and plugin scripting.

#### 3. Adaptive UI & Discrete Mouse Wheel Stepping (BUG-089)
* **Elimination of Inertial Wheel Drift:**
  - Fixed an interface flaw where `egui`'s `smooth_scroll_delta` caused runaway slider oscillations (jumping 10–12 units per single physical notch).
  - Implemented exact single-notch discrete steps: strictly $\pm 1.0$ for integer parameters and $\pm 0.1$ for floating-point controls.
* **Strict Column Alignment & Zero Magic Numbers:**
  - Interactive controls align to unified layout guides without hardcoded width constants.
  - Fully adaptive localized text rendering without text clipping or truncated labels.

#### 4. Generalized Architectural Invariants & Linter Guard
* **Generalized Core Protection:**
  - Replaced fragile keyword blacklists with generalized structural checks in `crates/ai_engine/tests/architecture_integrity_test.rs`.
* **Layout Guard Linter (`layout_guard_lint_test.rs`):**
  - Integrated automated AST/token linter verifying mouse wheel handling, discrete stepping, and absence of fixed-width magic allocations across all UI components.

#### 5. Community Plugins Marketplace
* **12 Official Plugins Validated & Packaged:**
  - Re-packaged all plugins into portable `.pcpl` format in `dist/`.
  - Updated registry index `registry.json` and synchronized catalogs in `README.md` and `README.ru.md`.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v1.3.2** представляет **Студию модификаторов выделения (Selection Modifier Stack, `selection_mod`)** — интерактивную неразрушающую панель для расширения, сжатия, растушевки, сглаживания и создания контурных рамок выделения в реальном времени. В этом релизе также внедрены обобщенные архитектурные инварианты ядра, устранена инерционная раскачка колеса мыши (BUG-089), добавлена поддержка числовых массивов `IntList` в динамических контрактах и обеспечен строгий адаптивный версточный контроль.

---

### 🌟 Ключевые нововведения

#### 1. Студия модификаторов выделения (`plugins/selection_mod`)
* **Параметрические неразрушающие модификаторы:**
  - **Расширение / Сжатие (Expand / Contract):** Динамическое расширение и сужение маски активного выделения с точностью до пикселя ($0 \dots 100\text{ px}$).
  - **Растушевка (Feather):** Мягкое размытие краев ($0 \dots 100\text{ px}$) для аккуратного вырезания и композитинга.
  - **Сглаживание (Smooth):** Устранение ступенчатых пиксельных артефактов и скругление углов ($0 \dots 30\text{ px}$).
  - **Рамка (Border):** Автоматическое построение контурной рамки ($0 \dots 50\text{ px}$) вокруг выделенного объекта.
* **Архитектура единого базового канала:**
  - Полная ликвидация захламления каналов документа: вся работа ведется через один прозрачный рабочий канал `__pc_companion_base_sel`.
  - Автоматическое корректное удаление канала при закрытии окна плагина.
  - Мгновенный неразрушающий **Сброс (Reset)**, возвращающий исходный контур художника без накопления искажений.
* **Двусторонняя реактивность выделения:**
  - Отслеживание изменений холста в реальном времени через IPC-протокол `document.inspect`: плагин непрерывно считывает `selection_bounds`.
  - Если художник перевыделил область в PhotoCraft при открытом окне плагина, модификатор автоматически подхватывает свежий контур как новую базу.

#### 2. Чистая архитектура (Zero-Core Hardcoding) и Rhai-пайплайн
* **Запрет составных команд в ядре:**
  - Строгое соблюдение правила 3.2: ядро компаньона остается 100% прозрачным транслятором команд в IPC-сокет PhotoCraft.
  - Вся логика модификации выделения реализована декларативно через Rhai-сценарий (`script.rhai`) без единой специфичной ветки в `companion_app`.
* **Поддержка числовых массивов (`IntList`):**
  - Структура `DynamicValue` в `core_contracts` расширена вариантом `IntList(Vec<i64>)`, обеспечивающим передачу числовых списков (включая координаты `[x0, y0, x1, y1]`) между ядром, хостом и скриптами.

#### 3. Адаптивная верстка и дискретный шаг колеса мыши (BUG-089)
* **Ликвидация инерционного дрифта колеса:**
  - Устранен дефект интерфейса, при котором `smooth_scroll_delta` в `egui` генерировал затухающие колебания и приводил к бесконтрольным скачкам значений на 10–12 единиц за один физический щелчок.
  - Реализован дискретный шаг: строго $\pm 1.0$ для целочисленных параметров и $\pm 0.1$ для дробных.
* **Строгое выравнивание колонок без магических чисел:**
  - Элементы управления выравниваются по единой вертикальной направляющей с динамическим расчетом ширины текста.
  - Полная адаптивность к длине локализованных строк во всех языковых локалях.

#### 4. Обобщенные архитектурные инварианты и линтеры
* **Обобщенная защита ядра:**
  - Замена хрупких черных списков на проверку обобщенных инвариантов системы в `crates/ai_engine/tests/architecture_integrity_test.rs`.
* **Линтер верстки (`layout_guard_lint_test.rs`):**
  - Добавлен автоматический линтер, проверяющий обработку событий прокрутки колеса мыши, шаг слайдеров и отсутствие магических констант ширины.

#### 5. Каталог официальных плагинов
* **12 проверенных плагинов в формате `.pcpl`:**
  - Все плагины собраны в директорию `dist/`.
  - Обновлен индекс реестра `dist/registry.json`, синхронизированы каталоги в `README.md` и `README.ru.md`.

---

### 📦 Validation & Quality Assurance
* **Full Workspace Tests:** `cargo test --workspace -j 1` — **200+ passed**, 0 failed.
* **Compiler & Linter:** `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — **0 warnings**.
* **Integrity Linters:** Passed `architecture_integrity_test`, `dev_regulations_lint_test`, `layout_guard_lint_test`, `i18n_lint_test`, `vector_first_lint_test`.
* **Plugin Ecosystem:** 12/12 plugins validated by `plugin_validator`.
* **Formatting:** `cargo fmt --all -- --check` verified clean.
