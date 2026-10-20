# AI Remove Background (RMBG-1.4)

[![Plugin: AI Remove Background](https://img.shields.io/badge/Plugin-AI_Remove_Background-blue.svg)](plugin.json)
[![Model: Bria AI RMBG-1.4](https://img.shields.io/badge/Model-Bria_AI_RMBG--1.4-purple.svg)](https://huggingface.co/briaai/RMBG-1.4)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
High-fidelity neural background removal plugin for PhotoCraft powered by **Bria AI RMBG-1.4** running locally via ONNX Runtime (DirectML / CoreML / CPU).
Accurately segments foreground subjects (people, animals, vehicles, complex products) with soft hair strands and transparent edges preserved.

### Sub-Actions & Operating Modes:
- **Remove Background (Cutout):** Isolates the subject onto a transparent new layer and non-destructively hides the original source layer.
- **Create Layer Mask:** Loads the neural alpha boundary into a precise selection, applies a non-destructive native layer mask directly onto the source layer, and cleans up temporary layers.

### Use Cases:
- **Portrait & Commercial Cutouts:** Instant isolation of models and subjects with semi-transparent hair details preserved without tedious manual lasso work.
- **Non-Destructive Compositing:** Generate precise layer masks that can be fine-tuned or painted back with soft brushes anytime in PhotoCraft.
- **E-Commerce Product Photography:** Quick extraction of goods for marketplace listings with transparent or pure white backdrops.

### Model Weights:
- **Direct Download:** [rmbg-1.4.onnx](https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx) (168 MB)
- **Hugging Face Repository:** [briaai/RMBG-1.4](https://huggingface.co/briaai/RMBG-1.4)

---

**Русский:**
Плагин высокоточного нейросетевого удаления фона для PhotoCraft на базе модели **Bria AI RMBG-1.4**, работающей локально через ONNX Runtime с поддержкой DirectML / CoreML / CPU.
Бережно вырезает объекты переднего плана (люди, животные, предметы, автомобили) с сохранением полупрозрачных волосков и тонких краев.

### Поддействия и режимы работы:
- **Удаление фона (Вырезка):** Изолирует объект на чистый прозрачный слой и автоматически скрывает исходный слой во избежание дублирования.
- **Создать маску слоя:** Переносит контур альфа-канала ИИ в точное выделение и накладывает неразрушающую маску слоя прямо на исходник, удаляя временные слои.

### Практическое применение:
- **Портретная и коммерческая ретушь:** Мгновенное отделение моделей от фона с сохранением сложных прядей волос и полупрозрачных тканей без ручного выделения пером.
- **Неразрушающий монтаж (Layer Mask):** Создание маски слоя, которую можно в любой момент подправить мягкой кистью в PhotoCraft.
- **Карточки товаров для маркетплейсов:** Быстрая изоляция предметов для предметной фотосъемки и размещения на белом или фирменном фоне.

### Ссылки на веса модели:
- **Прямое скачивание:** [rmbg-1.4.onnx](https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx) (168 МБ)
- **Репозиторий Hugging Face:** [briaai/RMBG-1.4](https://huggingface.co/briaai/RMBG-1.4)
