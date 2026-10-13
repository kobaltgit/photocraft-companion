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
    "task": "segmentation_mask"
  },
  "models": [
    {
      "name": "rmbg-1.4.onnx",
      "required": true,
      "size_mb": 168,
      "url": "https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx"
    }
  ]
}
```

### 3.3. Параметры конфигурации:
* **`input`**:
  - `"active_layer"`: отправляет пиксели текущего активного слоя;
  - `"selection"`: отправляет вырезанную область по активному выделению;
  - `"canvas"`: отправляет полный композитный холст документа.
* **`output_mode`**:
  - `"in_place"`: замещает пиксели текущего слоя;
  - `"new_layer"`: вставляет результат как новый слой над активным;
  - `"mask"`: создает маску слоя.
* **`post_commands`**: список команд графического редактора, автоматически выполняемых после инференса нейросети (например, `image.revealAll`, `layer.mergeDown`).

---

## 4. Сборка и распространение плагина

Для упаковки вашего плагина в распространяемый `.pcpl` архив используйте скрипт:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/package_plugins.ps1
```
Скрипт автоматически подготовит архив в папке `dist/packages/`, исключив любые локальные бинарные веса моделей.
