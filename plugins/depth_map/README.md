# AI Depth Map (Depth Anything V2)

[![Plugin: AI Depth Map](https://img.shields.io/badge/Plugin-AI_Depth_Map-blue.svg)](plugin.json)
[![Model: Depth Anything V2](https://img.shields.io/badge/Model-Depth_Anything_V2_Small-purple.svg)](https://huggingface.co/onnx-community/depth-anything-v2-small)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
State-of-the-art monocular 3D depth estimation plugin for PhotoCraft based on **Depth Anything V2 (Small)**.
Generates an 8-bit grayscale depth map layer (`Layer (Depth Map)`) where white/light pixels represent objects closest to the camera, and dark/black pixels represent far background.

### Use Cases:
- **Lens Blur / Bokeh:** Use the generated depth map in PhotoCraft (`Filter > Blur > Lens Blur`) to create natural optical bokeh with custom aperture shape and focal point.
- **Atmospheric Fog & Mist:** Create distance-based volumetric fog and depth fade effects.
- **Relighting:** Add directional light and ambient rim lighting based on spatial 3D surface depth.

---

**Русский:**
Передовой плагин монокулярной оценки 3D-глубины для PhotoCraft на базе модели **Depth Anything V2 (Small)**.
Создает отдельный 8-битный полутоновый слой карты глубины (`Слой (Карта глубины)`), где белые/светлые пиксели соответствуют объектам на переднем плане перед камерой, а темные/черные — удаленному фону.

### Практическое применение:
- **Размытие диафрагмы (Lens Blur / Боке):** Используйте карту глубины в PhotoCraft (`Фильтр > Размытие > Размытие при малой глубине резкости`) для создания реалистичного оптического боке с выбором точки фокуса кликом мыши.
- **Объёмный туман и воздушная перспектива:** Наложение реалистичной атмосферной дымки с мягким растворением вдали.
- **Релайтинг и светотень:** Расстановка виртуальных источников света с физически корректным затенением объектов по глубине.
