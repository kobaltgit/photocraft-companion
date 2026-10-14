# Smart Vectorizer / SVG Trace (`vectorize`)

[![Plugin: Smart Vectorizer](https://img.shields.io/badge/Plugin-Smart_Vectorizer-blue.svg)](plugin.json)
[![Engine: VTracer 0.6.5](https://img.shields.io/badge/Engine-VTracer_0.6.5-teal.svg)](https://github.com/visioncortex/vtracer)
[![Type: Native Algorithm](https://img.shields.io/badge/Type-Native_Algorithm-17a2b8.svg)](plugin.json)
[![Weights: 0 MB](https://img.shields.io/badge/Weights-0_MB_(Algorithmic)-success.svg)](plugin.json)

**English:**
High-fidelity raster-to-vector tracing plugin for PhotoCraft based on the native **VTracer 0.6.5** geometric spline-fitting engine.
Converts bitmap layers, scanned sketches, black & white artwork, and logos into clean, scalable SVG vector curves with Bezier fitting.

### Features & Triple Output:
1. **PhotoCraft Layer In-Place:** Rasterizes the fitted SVG curves via `resvg` / `tiny-skia` matching canvas dimensions and pastes it directly as a new layer (`Layer (Vectorized)`).
2. **OS Clipboard Copy:** Automatically copies clean XML SVG markup to the system clipboard, ready for instant `Ctrl + V` into Figma, Adobe Illustrator, or Inkscape.
3. **Standalone File Export:** Automatically writes a standalone `.svg` file into `exports/{doc}_{layer}_vector.svg`.

### Best Results Tips:
- **Clean Input:** For sharp logos, prefer high-contrast rasters or remove the background first using `AI Remove Background`.
- **CAD Geometry Finishing:** Like all automatic spline-tracing tools (Illustrator Image Trace, Potrace), complex geometric primitives (perfect circles, exact straight lines) serve as a rapid 80% foundation and can be polished in a vector editor.

---

**Русский:**
Высокоточный плагин трассировки растра в вектор для PhotoCraft на базе нативного геометрического движка **VTracer 0.6.5**.
Преобразует растровые слои, сканы скетчей, контрастную графику и логотипы в чистые масштабируемые векторные кривые Безье формата SVG.

### Возможности и тройной результат (Triple Output):
1. **Новый слой в PhotoCraft:** Рендерит полученные SVG-кривые через быстрый растеризатор `resvg` / `tiny-skia` в точный размер холста и вставляет новым слоем (`Слой (Векторизовано)`).
2. **Буфер обмена ОС:** Автоматически копирует чистый SVG-код в системный буфер обмена для мгновенной вставки через `Ctrl + V` в Figma, Illustrator или Inkscape.
3. **Физический файл на диске:** Сохраняет готовый векторный файл в папку `exports/{doc}_{layer}_vector.svg`.

### Советы по использованию:
- **Чистота исходника:** Для идеального результата используйте контрастные изображения или предварительно удалите фон плагином `AI Remove Background`.
- **Доводка чертежей:** Как и в любом автотрассировщике (Illustrator Image Trace, Potrace), автовекторизация выполняет 80% черновой работы по сложным формам за доли секунды, а идеальные строгие окружности или выравнивание стрелок при необходимости калибруются за пару минут в векторном редакторе.
