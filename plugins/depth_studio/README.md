# Depth Studio (Atmospheric Fog & Lens Bokeh)

[![Plugin: Depth Studio](https://img.shields.io/badge/Plugin-Depth_Studio-blue.svg)](plugin.json)
[![Model: Depth Anything V2](https://img.shields.io/badge/Model-Depth_Anything_V2_Small-purple.svg)](https://huggingface.co/onnx-community/depth-anything-v2-small)
[![Type: Interactive UI](https://img.shields.io/badge/Type-Interactive_UI-orange.svg)](plugin.json)

**English:**
**Depth Studio** is an interactive, multi-tab AI studio for PhotoCraft that brings cinematic depth-of-field, realistic lens bokeh defocus, and atmospheric aerial perspective into a single live window.
Powered by **Depth Anything V2 Small** running locally via ONNX Runtime (DirectML / CoreML / CPU), it computes an accurate 3D scene depth map upon opening and enables intuitive click-to-focus directly on the real-time split preview.

### Features & Capabilities:
- **Atmospheric Fog & Mist:** Add aerial perspective, morning hazes, or dense cinematic fog with custom tint colors, exponential distance falloff, and realistic blend modes (`Normal`, `Screen`, `Soft Light`).
- **Lens Blur & Bokeh:** Simulate professional fast camera lenses (f/1.2 - f/2.8) with adjustable aperture radius, depth-of-field (DOF) width, and specular highlight bloom.
- **Interactive Click-to-Focus:** Click anywhere on the live preview image to instantly set the focal plane to that exact subject or landmark.
- **Non-Destructive Layer Export:** Export cleanly as separate layers (defocused lens layer + tinted fog mask layer) or as a merged composite, leaving the original artwork intact.

### Use Cases:
- **Portrait Photography Retouching:** Turn flat smartphone or studio portraits into shallow depth-of-field captures with creamy background bokeh while keeping eyes and facial features pin-sharp.
- **Landscape & Environment Art:** Inject atmospheric depth and sense of scale into digital paintings, photos, and 3D renders using realistic aerial hazes and mist roll-offs.
- **Product & Miniature Showcase:** Direct viewer attention to key product details using macro tilt-shift and tight focal planes.

### Model Weights:
- **Direct Download:** [depth-anything-v2-small.onnx](https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx) (99 MB)
- **Hugging Face Repository:** [onnx-community/depth-anything-v2-small](https://huggingface.co/onnx-community/depth-anything-v2-small)

---

**Русский:**
**Depth Studio** — интерактивная студия для PhotoCraft, объединяющая кинематографичную глубину резкости, оптическое боке объектива и атмосферную воздушную перспективу в одном удобном окне со сплит-превью.
На базе нейросети **Depth Anything V2 Small**, работающей локально через ONNX Runtime с аппаратным ускорением DirectML / CoreML / CPU, инструмент автоматически оценивает 3D-глубину сцены и позволяет переводить фокус одним кликом мыши прямо по превью.

### Возможности и особенности:
- **Атмосферный туман и дымка:** Создание воздушной перспективы, утренней дымки или плотного кинематографичного тумана с настраиваемым цветом, дистанцией старта, спадом и режимами наложения (`Normal`, `Screen`, `Soft Light`).
- **Оптическое боке объектива:** Имитация светосильной оптики (f/1.2 - f/2.8) с управляемым радиусом апертуры, шириной ГРИП (зоны резкости) и усилением зеркальных бликов.
- **Интерактивная клик-фокусировка:** Клик в любую точку правого превью моментально перемещает плоскость резкости на выбранный объект.
- **Неразрушающий экспорт:** Вывод результата отдельными слоями (слой с оптическим боке + слой тумана с альфа-маской) или единым сведённым слоем поверх нетронутого оригинала.

### Практическое применение:
- **Портретная ретушь:** Превращение снимков с плоским фоном в выразительные кадры с мягким размытием заднего плана и идеальной резкостью глаз и лица.
- **Пейзажная фотография и концепт-арт:** Добавление глубины пространства, масштаба и погодных эффектов в иллюстрации, 3D-рендеры и фотографии природы.
- **Предметная и коммерческая съёмка:** Акцентирование внимания покупателя на ключевых элементах товара при помощи эффекта макросъёмки.

### Ссылки на веса модели:
- **Прямое скачивание:** [depth-anything-v2-small.onnx](https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx) (99 МБ)
- **Репозиторий Hugging Face:** [onnx-community/depth-anything-v2-small](https://huggingface.co/onnx-community/depth-anything-v2-small)
