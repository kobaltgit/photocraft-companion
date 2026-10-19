# Руководство разработчика плагинов PhotoCraft (Plugin Development Guide)

Добро пожаловать в экосистему расширений графического редактора **PhotoCraft**!

PhotoCraft Companion предоставляет мощный механизм создания и распространения плагинов четырёх основных категорий:
1. **No-Code Macro (`no_code`)**: цепочки декларативных команд PhotoCraft без написания кода;
2. **Native AI (`native_ai`)**: локальный инференс нейросетей ONNX (DirectML / CPU fallback) с автоматическим тайлингом;
3. **Native Algorithm (`native_algorithm`)**: скомпилированные высокопроизводительные алгоритмы (векторизация, геометрия);
4. **Interactive UI (`interactive_ui`)**: плавающие панели инструментов, палитры и ассистенты на базе декларативных виджетов и безопасного скриптового рантайма Rhai с двунаправленной синхронизацией с холстом PhotoCraft.

Ядро компаньона построено по принципу **Zero-Core Hardcoding**: рантайм полностью нейтрален к логике плагинов и декларативно интерпретирует манифесты `plugin.json`.

---

## 1. Структура пакета плагина (`.pcpl`)

Пакет плагина `.pcpl` представляет собой стандартный архив (ZIP-контейнер) со следующей структурой:

```text
my_plugin/
├── plugin.json         # Главный декларативный манифест (обязательно)
├── icon.svg            # Авторская векторная SVG-иконка (обязательно)
├── README.md           # Описание возможностей плагина и документация (обязательно)
├── logic.rhai          # Скрипт реактивной логики (обязательно для interactive_ui)
└── LICENSE             # Лицензия (MIT, Apache-2.0 и т.д.)
```

> [!IMPORTANT]
> **Тяжелые модели (`*.onnx`, веса) СТРОГО ЗАПРЕЩЕНО включать в `.pcpl` архив и коммитить в Git!**  
> Пакет плагина должен быть легковесным (< 50 КБ). Веса моделей скачиваются автоматически менеджером плагинов по прямым ссылкам `url`, указанным в секции `"models"` манифеста.

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

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="9"/>
  <path d="M12 3a9 9 0 0 1 0 18z" fill="currentColor"/>
</svg>
```

---

## 3. Манифест плагина (`plugin.json`)

Манифест определяет тип плагина, его метаданные, входные данные, шаги инференса или макрокоманд, а также способ возврата результата в холст PhotoCraft.

### 3.1. No-Code Macro плагин (`plugin_type: "no_code"`)
Чисто декларативная цепочка команд PhotoCraft. Выполняется через сокет управления без компиляции:

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

### 3.2. Native AI плагин (`plugin_type: "native_ai"`)
Локальный запуск нейросетей через встроенный в ядро ONNX Runtime (DirectML с автоматическим fallback на CPU и тайлингом):

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

### 3.3. Native Algorithm плагин (`plugin_type: "native_algorithm"`)
Высокоскоростная обработка нативными скомпилированными алгоритмами (например, векторная трассировка растра в SVG через VTracer):

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

### 3.4. Interactive UI плагин (`plugin_type: "interactive_ui"`)
Интерактивные окна, ассистенты, палитры и диагностические инструменты. Позволяют строить богатый пользовательский интерфейс из декларативных виджетов и программировать реактивную логику на языке Rhai:

```json
{
  "id": "ui.photocraft.plugins.chromasense",
  "name": "ChromaSense",
  "version": "1.0.0",
  "description": "Accessibility Color Assistant for artists with color vision deficiencies",
  "author": "PhotoCraft Community",
  "plugin_type": "interactive_ui",
  "icon": "icon.svg",
  "commands": [],
  "interactive_ui": {
    "ipc_subscriptions": [
      "session.colors"
    ],
    "window_title": "ChromaSense A11y",
    "window_title_ru": "ChromaSense Доступность",
    "default_width": 350.0,
    "default_height": 570.0,
    "script": "logic.rhai",
    "widgets": [
      {
        "type": "color_slot",
        "id": "color_fg",
        "label": "Active Color",
        "label_ru": "Выбранный цвет",
        "default_hex": "#2563eb",
        "selectable": true,
        "target_binding": "edit_target"
      },
      {
        "type": "button",
        "id": "swap_colors",
        "label": "Swap Colors",
        "label_ru": "Поменять цвета",
        "icon": "swap"
      },
      {
        "type": "color_slot",
        "id": "color_sim",
        "label": "Vision Simulation",
        "label_ru": "Как видит зритель",
        "default_hex": "#2563eb",
        "read_only": true
      },
      {
        "type": "color_slot",
        "id": "color_bg",
        "label": "Background",
        "label_ru": "Цвет фона",
        "default_hex": "#ffffff",
        "selectable": true,
        "target_binding": "edit_target"
      },
      {
        "type": "dropdown",
        "id": "vision_profile",
        "label": "Vision Profile",
        "label_ru": "Профиль зрения",
        "options": [
          "Normal Vision",
          "Deuteranopia (Green-blind)",
          "Protanopia (Red-blind)",
          "Tritanopia (Blue-blind)"
        ],
        "options_ru": [
          "Обычное зрение",
          "Дейтеранопия (зелёная слепота)",
          "Протанопия (красная слепота)",
          "Тританопия (синяя слепота)"
        ],
        "default_index": 0
      },
      {
        "type": "hue_wheel",
        "id": "compass",
        "source_color_id": "edit_target",
        "show_pointer": true
      },
      {
        "type": "banner",
        "id": "wcag_status",
        "message_binding": "collision_warning",
        "style_binding": "collision_style"
      },
      {
        "type": "slider",
        "id": "red",
        "label": "Red",
        "label_ru": "Красный",
        "min": 0.0,
        "max": 255.0,
        "step": 1.0,
        "default": 37.0
      }
    ]
  }
}
```

---

## 4. Спецификация декларативных виджетов (`widgets`)

Все элементы окна объявляются в массиве `"widgets"`. Каждый виджет имеет обязательное поле `"type"`:

| Тип (`type`) | Параметры манифеста | Описание |
| :--- | :--- | :--- |
| **`color_slot`** | `id`, `label`, `label_ru`, `default_hex`, `read_only`, `selectable`, `target_binding` | Интерактивная плашка цвета. Если `selectable: true`, клик по плашке переключает фокус редактирования (записывает `id` слота в ключ `target_binding`). При `read_only: true` служит для отображения результата симуляции. |
| **`slider`** | `id`, `label`, `label_ru`, `min`, `max`, `step`, `default` | Ползунок числового значения с плавной векторной отрисовкой. Значение автоматически доступно в состоянии по ключу `id`. |
| **`dropdown`** | `id`, `label`, `label_ru`, `options`, `options_ru`, `default_index` | Выпадающий список с поддержкой локализованных вариантов `options_ru`. Выбранный индекс хранится в `state[id]`. |
| **`toggle`** | `id`, `label`, `label_ru`, `default` | Логический переключатель-чекбокс (хранит `true`/`false`). |
| **`button`** | `id`, `label`, `label_ru`, `icon` | Кнопка действия. При клике выставляет флаг `action_<id> = true` в состоянии, доступный для обработки в скрипте. Поддерживает векторные иконки (например, `"swap"` для смены мест). |
| **`hue_wheel`** | `id`, `source_color_id`, `show_pointer` | Векторный круговой цветовой спектр HSL (цветовой круг). Поддерживает динамическое связывание `source_color_id`: если указан ключ `edit_target`, круг автоматически читает hex-код текущего активного слота. |
| **`banner`** | `id`, `message_binding`, `style_binding` | Информационный баннер статуса с автопереносом длинного текста. `style_binding` принимает стили: `"info"`, `"warning"`, `"error"`, `"success"`. Текст читается из ключа состояния `message_binding`. |
| **`label`** | `text_binding` | Текстовая строка, связанная со строковым значением в состоянии плагина. |
| **`separator`** | *(без параметров)* | Тонкая визуальная горизонтальная черта для разделения логических блоков. |
| **`row`** | `items: [ WidgetNode, ... ]` | Горизонтальный контейнер для размещения нескольких виджетов в одну строку. |

---

## 5. Скриптовая логика Rhai (`logic.rhai`)

Плагины типа `interactive_ui` управляются легковесным, безопасным скриптом на языке **Rhai** (песочница без доступа к произвольным файлам ОС).

### 5.1. Хуки жизненного цикла

#### 1. `on_state_changed(state) -> state`
Вызывается ядром при любом взаимодействии пользователя с виджетами (передвижение слайдера, клик по кнопке, переключение слота, выбор пункта меню):
```rhai
fn on_state_changed(state) {
    // 1. Проверяем, была ли нажата кнопка swap_colors
    if state["action_swap_colors"] == true {
        state["action_swap_colors"] = false; // сбрасываем флаг действия
        let temp = state["color_fg"];
        state["color_fg"] = state["color_bg"];
        state["color_bg"] = temp;
    }

    // 2. Выполняем пересчет логики
    state = recalculate_diagnostics(state);

    // 3. Отправляем обновленный цвет кисти в PhotoCraft
    execute_command("tools.setColors", #{
        foreground: state["color_fg"],
        background: state["color_bg"]
    });

    return state;
}
```

#### 2. `on_session_colors(fg_hex, bg_hex, state) -> state`
Вызывается автоматически, когда художник в PhotoCraft меняет цвет внешними инструментами (**Пипетка `I`**, сброс на **`D`**, инверсия на **`X`**):
```rhai
fn on_session_colors(fg_hex, bg_hex, state) {
    state["color_fg"] = fg_hex;
    state["color_bg"] = bg_hex;

    // Синхронизируем слайдеры активного слота
    let active_hex = state[state["edit_target"]];
    let rgb = hex_to_rgb(active_hex);
    state["red"] = rgb["r"];
    state["green"] = rgb["g"];
    state["blue"] = rgb["b"];

    state = recalculate_diagnostics(state);

    // ВНИМАНИЕ: Внутри этого хука СТРОГО ЗАПРЕЩЕНО вызывать set_colors или tools.setColors,
    // так как эти цвета уже пришли из PhotoCraft!
    return state;
}
```

### 5.2. Встроенные нативные функции Rhai

В рантайм Rhai встроены быстрые скомпилированные функции:

* **Управление редактором:**
  - `execute_command(command, params_map)` — отправить команду в сокет PhotoCraft;
  - `execute_command(command)` — отправить команду без параметров;
  - `set_colors(fg_hex, bg_hex)` — мгновенно выставить цвет переднего плана и фона в PhotoCraft.
* **Локализация (i18n):**
  - `t("English text", "Русский текст")` — возвращает перевод в зависимости от активного языка приложения;
  - `current_language()` — возвращает `"ru"` или `"en"`.
* **Работа с цветом и математика:**
  - `rgb_to_hex(r, g, b) -> String` — конвертирует числа $0..255$ в `#rrggbb`;
  - `hex_to_rgb(hex) -> Map` — возвращает ассоциативный массив `#{ r: 255, g: 0, b: 0 }`;
  - `get_named_color(r, g, b) -> String` — возвращает художественное название оттенка из базы 152 цветов (CSS3 + классические пигменты) на активном языке;
  - `simulate_cvd(r, g, b, profile_idx) -> String` — колориметрическая симуляция восприятия цвета дальтоником по матрицам Бреттла (0: Normal, 1: Deuteranopia, 2: Protanopia, 3: Tritanopia);
  - `calculate_contrast(r1, g1, b1, r2, g2, b2) -> Float` — вычисляет строгий коэффициент контрастности по международному стандарту W3C WCAG 2.1. Контраст 100% симметричен.

---

## 6. Двунаправленная синхронизация и защита от эхо (`ipc_subscriptions`)

Если плагин подписан на топик `"session.colors"`:
```json
"ipc_subscriptions": [
  "session.colors"
]
```
Ядро PhotoCraft Companion активирует умный двухэшелонный механизм:
1. **Zero-Waste поллер:** сокет PhotoCraft опрашивается с частотой 180 мс **только тогда, когда открыто хотя бы одно окно плагина**. Когда все окна закрыты, нагрузка на CPU и сеть равна 0%.
2. **Echo Guard (Подавление эхо):** когда плагин отправляет цвет через `set_colors` или `execute_command("tools.setColors")`, значение запоминается в `echo_guard`. Ответное эхо от редактора отсекается ядром, предотвращая зацикливание и мерцание слайдеров.

---

## 7. Спецификация секции ИИ (`ai` / `AiPipelineConfig`)

Для плагинов типа `native_ai`:

* **`task`** (обязательно): тип задачи инференса:
  - `"segmentation_mask"`: вырезание объектов или создание карт глубины/нормалей (RMBG, Depth Anything);
  - `"image_to_image"`: преобразование пикселей 1:1 (шумоподавление, реставрация лиц GFPGAN);
  - `"super_resolution"`: апскейл разрешения (Real-ESRGAN);
  - `"inpainting"`: контекстное удаление объектов и генеративная заливка по выделению (LaMa).
* **`input_size`** (опционально): разрешение входа модели в пикселях (например, `518` для Depth Anything V2, `1024` для RMBG-1.4). По умолчанию `1024`.
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

---

## 8. Спецификация секции алгоритмов (`algorithm` / `AlgorithmConfig`)

Для плагинов типа `native_algorithm`:

* **`task`**: тип встроенного алгоритма ядра (`"vector_trace"`).
* **`mode`**: режим аппроксимации кривых (`"spline"`, `"polygon"`).
* **`color_mode`**: цветовая модель (`"color"`, `"binary"`).
* **`filter_speckle`**: фильтрация мелкого шума в пикселях (например, `4`).
* **`corner_threshold`**: угол сглаживания угловых точек (например, `60`).
* **`copy_clipboard`**: автокопирование готового SVG-кода в системный буфер обмена (для вставки в Figma / Illustrator).
* **`export_file`**: сохранение `.svg` файла во временную папку экспорта.

---

## 9. Обязательная документация плагина (`README.md`)

Каждый плагин в каталоге `plugins/<id>/` **обязан содержать файл `README.md`**. Без качественной документации плагин не принимается в экосистему и отклоняется автоматическим валидатором Definition of Done (`plugin_validator`).

> [!TIP]
> **Интерактивное отображение в Компаньоне:** Содержимое `README.md` отображается напрямую в интерфейсе **PhotoCraft Companion Manager**! В карточке плагина пользователь может нажать иконку документа и открыть стилизованное окно с переключателем языков `[ Русский | English ]` и кликабельными ссылками на веса моделей.

### Требования к `README.md`:
1. **Двуязычность:** Обязательное синхронное описание на **английском** и **русском** языках.
2. **Практические сценарии использования (Use Cases / Практическое применение):**
   * Минимум 2 конкретных примера того, как дизайнер или художник применяет результат в PhotoCraft (комбинация с фильтрами, масками, режимами наложения слоев).
3. **Ссылки на веса моделей:** Прямые ссылки на Hugging Face / GitHub Releases для воспроизводимости.

---

## 10. Валидация и упаковка плагина (`.pcpl`)

Для проверки плагина на соответствие стандартам экосистемы, компиляции каталога и генерации пакетов `.pcpl` запустите системный валидатор:

```powershell
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
```

Валидатор автоматически:
- Проверит синтаксис `plugin.json` и наличие обязательных полей;
- Проверит чистоту векторной иконки `icon.svg` (отсутствие внедренного растра);
- Проверит структуру и двуязычность `README.md`;
- Упакует плагин в легковесный архив `.pcpl` в папке `dist/`;
- Обновит общие таблицы каталога плагинов в корневых файлах проекта.
