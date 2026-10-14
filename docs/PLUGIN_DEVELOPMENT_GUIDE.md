# Руководство разработчика плагинов PhotoCraft (Plugin Development Guide)

Добро пожаловать в экосистему расширений графического редактора **PhotoCraft**!

PhotoCraft Companion предоставляет мощный механизм создания и распространения плагинов (No-Code макросы, Native AI нейросети, скрипты автоматизации).
Ядро компаньона построено по принципу **Zero-Core Hardcoding**: рантайм полностью нейтрален к логике плагинов и декларативно интерпретирует манифесты `plugin.json`.

---

## 1. Структура пакета плагина (`.pcpl`)

Пакет плагина `.pcpl` представляет собой стандартный ZIP-архив со следующей структурой:

```text
my_plugin/
├── plugin.json         # Главный декларативный манифест (обязательно)
├── icon.svg            # Авторская векторная SVG-иконка (обязательно)
├── README.md           # Описание возможностей плагина и документация
└── LICENSE             # Лицензия (MIT, Apache-2.0 и т.д.)
```

> [!IMPORTANT]
> **Тяжелые модели (`*.onnx`, веса) СТРОГО ЗАПРЕЩЕНО включать в `.pcpl` архив!**
> Пакет плагина должен быть легковесным (< 50 КБ). Веса моделей скачиваются автоматически или по запросу пользователя из надежных репозиториев (HuggingFace / GitHub Releases) по прямым ссылкам `url`, указанным в секции `"models"` манифеста.

---

## 2. Требования к авторским векторным иконкам (`icon.svg`)

Для отображения кнопки вашего плагина на плавающей панели инструментов (`Toolstrip`) и в Менеджере плагинов требуется уникальная авторская векторная иконка.

1. **Формат:** Исключительно чистый векторный `.svg` (Strict SVG). Растровые изображения (`.png`, `.jpg`) не поддерживаются.
2. **Размер холста (ViewBox):** Рекомендуется стандартный размер `viewBox="0 0 24 24"` с пропорциями 1:1.
3. **Адаптивность к теме (Dark / Light):**
   * Используйте `stroke="currentColor"` или `fill="currentColor"` для основных линий и контуров.
   * Движок компаньона автоматически адаптирует `currentColor` под текущую тему оформления:
     - В темной теме (Dark Mode) отображается светлым контрастным оттенком (`#E6E6E6`).
     - В светлой теме (Light Mode) отображается темным контрастным оттенком (`#1E1E1E`).
   * Если вы используете монохромные оттенки (`#FFFFFF` или `#000000`), движок также безопасно инвертирует их для сохранения контраста.
   * Кастомные акцентные цвета (например, `#3186FF`, `#FF5555`) сохраняются неизменными.
4. **Стилистика линий:**
   * Рекомендуемая толщина обводки: `stroke-width="1.75"` или `stroke-width="2"`.
   * Закругления: `stroke-linecap="round"` и `stroke-linejoin="round"`.
   * Прозрачный фон (`fill="none"` для корневого тега `<svg>`).
5. **Запрет на внедренный растр:**
   * **СТРОГО ЗАПРЕЩЕНО** внедрять растровые изображения через base64 (`<image xlink:href="data:image/png;base64,...">`).

### Пример корректной иконки (`icon.svg`):
```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="9"/>
  <path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor"/>
</svg>
```

---

## 3. Манифест плагина (`plugin.json`)

Манифест определяет тип плагина, его метаданные, входные данные, шаги инференса или макрокоманд, а также способ возврата результата в холст PhotoCraft.

### 3.1. Пример No-Code Macro плагина:
```json
{
  "id": "macro.photocraft.plugins.black_and_white",
  "name": "B&W Film Pop (No-Code)",
  "version": "1.0.0",
  "description": "Duplicates active layer, converts to grayscale with auto-contrast adjustment.",
  "author": "kobaltGIT",
  "plugin_type": "no_code",
  "icon": "icon.svg",
  "commands": [
    {
      "command": "layer.duplicate",
      "params": {},
      "description": "Duplicate source layer"
    },
    {
      "command": "image.adjustments.desaturate",
      "params": {},
      "description": "Desaturate layer pixels"
    }
  ]
}
```

### 3.2. Пример Native AI плагина:
```json
{
  "id": "ai.photocraft.plugins.remove_bg",
  "name": "AI Remove Background (RMBG-1.4)",
  "version": "1.0.0",
  "description": "High-fidelity neural background removal for PhotoCraft using Bria AI RMBG-1.4 model via local ONNX Runtime.",
  "author": "kobaltGIT",
  "plugin_type": "native_ai",
  "action_id": "plugin:remove_bg",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "new_layer",
  "post_commands": [],
  "ai": {
    "task": "segmentation_mask",
    "input_size": 1024,
    "normalize": "centered",
    "output_type": "alpha_cutout"
  },
  "models": [
    {
      "name": "rmbg-1.4.onnx",
      "required": true,
      "size_mb": 168,
      "url": "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx",
      "sha256": "4733475c754d9c727ebc4c81b953d6ab1bf55eecdfca692289659341496a77d2"
    }
  ]
}
```

### 3.3. Пример Native Algorithm плагина:
Для плагинов, использующих высокопроизводительные скомпилированные алгоритмы без нейросетей (например, векторный трассировщик VTracer):
```json
{
  "id": "algorithm.photocraft.plugins.vectorize",
  "name": "Smart Vectorizer / SVG Trace",
  "version": "1.0.0",
  "description": "High-fidelity vectorization and SVG tracing of raster layers with Bezier curve fitting.",
  "author": "kobaltGIT",
  "plugin_type": "native_algorithm",
  "action_id": "plugin:vectorize",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "new_layer",
  "algorithm": {
    "task": "vector_trace",
    "mode": "spline",
    "color_mode": "color",
    "filter_speckle": 4,
    "corner_threshold": 60,
    "copy_clipboard": true,
    "export_file": true
  },
  "models": []
}
```

### 3.4. Общие параметры конфигурации:
* **`plugin_type`**:
  - `"no_code"`: чисто декларативный макрос из последовательности команд PhotoCraft (`commands`);
  - `"native_ai"`: локальный инференс нейросетей через ONNX Runtime (`ai`);
  - `"native_algorithm"`: встроенный нативный вычислительный алгоритм ядра (`algorithm`);
  - `"script"`: внешний скрипт автоматизации;
  - `"external_cli"`: консольный инструмент.
* **`input`**:
  - `"active_layer"`: отправляет пиксели текущего активного слоя;
  - `"selection"`: отправляет вырезанную область по активному выделению;
  - `"canvas"`: отправляет полный композитный холст документа.
* **`output_mode`**:
  - `"in_place"`: замещает пиксели текущего слоя;
  - `"new_layer"`: вставляет результат как новый слой над активным;
  - `"mask"`: создает маску слоя.
* **`post_commands`**: список команд графического редактора, автоматически выполняемых после инференса нейросети или алгоритма (например, `image.revealAll`, `layer.mergeDown`).

### 3.5. Спецификация секции ИИ (`ai` / `AiPipelineConfig`):
* **`task`** (обязательно): тип задачи инференса:
  - `"segmentation_mask"`: вырезание объектов или создание карт (RMBG, Depth Anything);
  - `"image_to_image"`: преобразование пикселей 1:1 (шумоподавление, цветокоррекция);
  - `"super_resolution"`: апскейл разрешения (Real-ESRGAN);
  - `"inpainting"`: заполнение удаленных областей (LaMa).
* **`input_size`** (опционально): квадратное разрешение входа модели в пикселях (например, `518` для Depth Anything V2, `1024` для RMBG-1.4). По умолчанию `1024`.
* **`normalize`** (опционально): режим нормализации входного тензора:
  - `"zero_to_one"`: приведение диапазона к `[0.0, 1.0]`;
  - `"centered"`: центрирование `(pixel / 255.0 - 0.5) / 1.0` (RMBG);
  - `"raw_255"`: "сырые" значения `[0.0, 255.0]` (LaMa);
  - `"image_net"`: стандартизация по ImageNet `(pixel/255.0 - mean) / std` (ViT, Depth Anything V2).
* **`output_type`** (опционально): формат интерпретации выхода нейросети:
  - `"alpha_cutout"` (по умолчанию): наложение вырезанного объекта с прозрачным альфа-каналом;
  - `"grayscale_layer"`: вывод результата в виде 8-битного полутонового слоя `Rgba([d, d, d, 255])` (для карт глубины, салиентности, карт нормалей).
* **`scale`** (опционально): коэффициент увеличения для `super_resolution` (например, `2` или `4`).
* **`tile_size`** и **`tile_pad`** (опционально): параметры тайлинга для обработки гигантских холстов без исчерпания видеопамяти GPU (например, размер тайла `512` и нахлест `32` для бесшовного блендинга).

### 3.6. Спецификация секции алгоритмов (`algorithm` / `AlgorithmConfig`):
* **`task`**: тип встроенного алгоритма ядра (`"vector_trace"`).
* **`mode`**: режим аппроксимации кривых (`"spline"`, `"polygon"`).
* **`color_mode`**: цветовая модель (`"color"`, `"binary"`).
* **`filter_speckle`**: фильтрация мелкого шума в пикселях (например, `4`).
* **`corner_threshold`**: угол сглаживания угловых точек (например, `60`).
* **`copy_clipboard`**: автокопирование готового SVG-кода в системный буфер обмена (для вставки в Figma / Illustrator).
* **`export_file`**: сохранение `.svg` файла во временную папку экспорта.

---

## 4. Обязательная документация плагина (`README.md`)

Каждый плагин в каталоге `plugins/<id>/` **обязан содержать файл `README.md`**. Без качественной документации плагин не принимается в экосистему и отклоняется автоматическим валидатором Definition of Done (`plugin_validator`).

> [!TIP]
> **Интерактивное отображение в Компаньоне:** Содержимое `README.md` отображается не только на GitHub, но и напрямую в интерфейсе **PhotoCraft Companion Manager**!
> В карточке плагина пользователь может нажать векторную иконку документа и открыть стилизованное окно с переключателем языков `[ Русский | English ]`, векторными маркерами списков, плашками параметров и кликабельными ссылками на веса моделей.

### Требования к `README.md`:
1. **Двуязычность:** Обязательное синхронное описание на **английском** и **русском** языках.
2. **Практические сценарии использования (Use Cases / Практическое применение):**
   * Минимум 2 конкретных примера того, как дизайнер или художник применяет результат в PhotoCraft (комбинация с фильтрами, масками, слоями).
3. **Ссылки на веса моделей:** Прямые ссылки на Hugging Face / GitHub Releases для воспроизводимости.

### Шаблон `README.md`:
```markdown
# <Plugin Name>

[![Plugin: <Name>](https://img.shields.io/badge/Plugin-<Name>-blue.svg)](plugin.json)
[![Model: <Model>](https://img.shields.io/badge/Model-<Model>-purple.svg)](<Model URL>)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
<Clear and concise description of what the plugin does, what input it expects, and the underlying model/algorithm.>

### Use Cases:
- **<Use Case 1>:** <Step-by-step application in PhotoCraft, layers, effects, purpose.>
- **<Use Case 2>:** <Integration with design workflows.>

---

**Русский:**
<Четкое и подробное русскоязычное описание возможностей плагина и формата входных данных.>

### Практическое применение:
- **<Сценарий 1>:** <Пошаговое описание использования в PhotoCraft, как комбинировать с инструментами редактора.>
- **<Сценарий 2>:** <Второй пример практической пользы.>
```

---

## 5. Сборка и распространение плагина

Для упаковки вашего плагина в распространяемый `.pcpl` архив используйте скрипт:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/package_plugins.ps1
```
Скрипт автоматически подготовит архив в папке `dist/packages/`, исключив любые локальные бинарные веса моделей.

