# Release Notes: PhotoCraft Companion v1.3.1

**Release Tag:** `v1.3.1`  
**Date:** 2026-10-26  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion **v1.3.1** introduces **PopCraft 3D Studio** — a high-performance 2.5D and 3D extrusion studio inspired by the canonical Japanese plugin **TOBIDASE (トビダセ)**. This release delivers true 3D perspective tapering, full-canvas document projection with zero layer clipping, live reactive preview with instant layer refresh, interactive native color pickers, and precision drop shadow controls.

---

### 🌟 Key Highlights & Major Additions

#### 1. PopCraft 3D Studio & Raster Engine (`crates/raster_engine`)
* **Dedicated High-Performance 2.5D Engine:**
  - Fast slice-based volumetric extrusion with directional normals and cosine shading.
  - Smooth two-tone depth gradients (`Front Depth` to `Back Depth`) along the extrusion vector.
  - Multi-pass inner bevel with angle-controlled highlight reflections (`Light Angle` $0^\circ \dots 360^\circ$ and `Highlight` intensity $0.0 \dots 1.0$).
* **Zero-Core Hardcoding Architecture:**
  - Implemented as a clean, portable standalone crate [`crates/raster_engine`](file:///d:/Projects/active/photocraft-plugins/crates/raster_engine/) without any hardcoded plugin IDs or rigid dependencies.
  - Fully controllable via declarative JSON manifests (`plugin.json`) and reactive Rhai scripts (`script.rhai`).

#### 2. True 3D Perspective Projection (Vanishing Point Convergence)
* **Beyond Isometric Extrusion:**
  - Moves beyond flat parallel/orthographic extrusion to realistic 3D perspective depth.
  - Slices smoothly taper inward towards the content centroid $(cx, cy)$ as depth increases:
    $$\text{scale} = 1.0 - t \cdot \text{perspective}$$
    $$\text{dest\_x} = cx + (x - cx) \cdot \text{scale} + ox$$
    $$\text{dest\_y} = cy + (y - cy) \cdot \text{scale} + oy$$
  - Controllable via the new **«Perspective»** slider ($0.0$ for pure isometric, up to $0.8$ for dramatic perspective pop-out).

#### 3. Full-Canvas Placement & Zero Clipping on Layers
* **Active Layer Bounds Resolution:**
  - Extended `DocumentInfo` with `active_layer_bounds: Option<[i32; 4]>` (`[x0, y0, x1, y1]`) parsed directly from PhotoCraft's `document.inspect` protocol.
* **Canvas-Wide Projection:**
  - Active text or graphic layers are projected onto a transparent full-canvas image $(doc.width \times doc.height)$ at their exact canvas coordinates $(x_0, y_0)$.
  - Extrusion volume and drop shadows can expand in any direction across the entire canvas with zero clipping at the edges of the original bounding box.
  - When pasted via `paste_image_in_place`, the result matches the document canvas at $(0, 0)$, aligning pixel-perfect with the original artwork.

#### 4. Precision Drop Shadow & Toggle Controls
* **Accurate Sliding Window Box Blur:**
  - Resolved accumulator bleed in `separable_box_blur`, ensuring all transparent canvas borders strictly maintain $\alpha = 0$.
* **One-Click Drop Shadow Toggle:**
  - Added declarative toggle `"shadow_enabled"`: when disabled, shadow computation and compositing are completely bypassed.

#### 5. Interactive UI, Native Color Pickers & Live Preview
* **Native Color Slots:**
  - Replaced hex text fields with interactive `egui::color_picker::color_edit_button_srgba` controls opening native RGB/HSV/Hex palettes on click.
* **Live Vector Refresh Button:**
  - Interactive top-right refresh button (circular arc vector icon) instantly re-fetches the active layer from PhotoCraft without closing the dialog.
  - Automatic cache invalidation upon window reopen.
* **One-Click Creative Presets:**
  - Includes 4 tuned presets: **YouTube Pop**, **Retro Comic**, **Cyber Neon**, and **Gold Metal**.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion **v1.3.1** представляет **PopCraft 3D Studio** — высокопроизводительную студию 2.5D и 3D-экструзии текста и графики, созданную по референсу легендарного японского плагина **TOBIDASE (トビダセ)**. В этом релизе реализована полноценная трёхмерная перспектива, вынос слоёв на полный холст без обрезки по границам, реактивный интерактивный предпросмотр с кнопкой мгновенного обновления слоя, нативные колорпикеры и точное управление мягкой тенью.

---

### 🌟 Ключевые нововведения

#### 1. Студия PopCraft 3D и крейт `crates/raster_engine`
* **Выделенный быстрый 2.5D растровый движок:**
  - Послойная объемная экструзия со сглаженными нормалями и направленным косинусным затенением.
  - Плавный двухцветный градиент глубины (от цвета фасада `Front Depth` к цвету глубины `Back Depth`).
  - Регулируемая внутренняя фаска с направленным световым бликом (`Light Angle` $0^\circ \dots 360^\circ$ и `Highlight` $0.0 \dots 1.0$).
* **Чистая архитектура (Zero-Core Hardcoding):**
  - Движок оформлен в виде независимого крейта [`crates/raster_engine`](file:///d:/Projects/active/photocraft-plugins/crates/raster_engine/) без хардкода моделей, ID плагинов или костылей ядра.
  - Полное декларативное управление через `plugin.json` и реактивный скрипт `script.rhai`.

#### 2. Трёхмерная перспектива (схождение к точке схода)
* **Больше, чем изометрия:**
  - Переход от плоской параллельной экструзии к честному 3D-объёму с точкой схода (vanishing point).
  - Срезы по мере удаления в глубину плавно сужаются к центру текста $(cx, cy)$:
    $$\text{scale} = 1.0 - t \cdot \text{perspective}$$
    $$\text{dest\_x} = cx + (x - cx) \cdot \text{scale} + ox$$
    $$\text{dest\_y} = cy + (y - cy) \cdot \text{scale} + oy$$
  - Новый слайдер **«Перспектива»** позволяет плавно варьировать глубину от строгой изометрии ($0.0$) до выразительного 3D-выноса ($0.8$).

#### 3. Полноэкранный рендеринг без обрезки слоёв
* **Считывание границ активного слоя:**
  - Структура `DocumentInfo` расширена полем `active_layer_bounds: Option<[i32; 4]>` (`[x0, y0, x1, y1]`), получаемым из штатного IPC-протокола `document.inspect`.
* **Проекция на холст документа:**
  - Слой перед экструзией проецируется на полный холст документа $(doc.width \times doc.height)$ со своими точными координатами $(x_0, y_0)$.
  - Объемные грани и тени могут свободно выходить за пределы текста во всех направлениях без каких-либо срезов.
  - При вставке через `paste_image_in_place` слой позиционируется точно в $(0, 0)$ холста, идеально совпадая с исходным текстом.

#### 4. Точная мягкая тень и тумблер отключения
* **Математически точный Box Blur:**
  - Устранена утечка аккумулятора в алгоритме скользящего окна `separable_box_blur`: прозрачные поля холста гарантированно сохраняют нулевую альфу.
* **Тумблер «Включить тень»:**
  - При отключении тумблера расчет и вставка слоя тени полностью исключаются, экономя время и ресурсы.

#### 5. Интерактивный UI, нативный выбор цвета и предпросмотр
* **Нативный колорпикер:**
  - Замена текстовых hex-полей на интерактивные кнопки цвета `egui::color_picker::color_edit_button_srgba` со всплывающей палитрой (RGB/HSV/Hex).
* **Векторная кнопка обновления слоя:**
  - Кнопка в правом верхнем углу превью (круглая векторная стрелка) позволяет мгновенно подтянуть актуальный слой из PhotoCraft без перезапуска окна.
* **4 готовых пресета в один клик:**
  - **YouTube Pop**, **Retro Comic**, **Cyber Neon** и **Gold Metal** для мгновенной стилизации заголовков.

---

### 📦 Validation & Quality Assurance
* **Full Workspace Tests:** `cargo test --workspace -j 1` — **196 passed**, 0 failed.
* **Integrity & Linters:** Passed `architecture_integrity_test`, `dev_regulations_lint_test`, `i18n_lint_test`, `vector_first_lint_test`, `multi_viewport_lint_test`, `plugin_i18n_lint_test`.
* **Plugin Validator:** 11/11 plugins validated, `dist/popcraft_3d.pcpl` built.
* **Style:** `cargo fmt --all -- --check` verified clean.
