---
name: create-photocraft-plugin
description: Пошаговое создание нового автономного плагина для PhotoCraft Companion без модификации и пересборки ядра Rust (Zero-Core Hardcoding).
---

# Создание нового плагина для PhotoCraft Companion

Используйте этот сценарий при добавлении нового плагина или инструмента для PhotoCraft.

## Шаг 1. Создание папки плагина
Создайте директорию в `plugins/<id_плагина>/`.

## Шаг 2. Создание манифеста `plugin.json`
В зависимости от типа инструмента выберите подходящий `plugin_type`:

### Вариант А: ИИ-плагин (`native_ai`)
Создайте `plugins/<id_плагина>/plugin.json`:
```json
{
  "id": "ai.photocraft.plugins.my_tool",
  "name": "My Tool Name",
  "name_ru": "Мой инструмент",
  "version": "1.0.0",
  "author": "kobaltGIT",
  "description": "Описание назначения плагина",
  "description_ru": "Русское описание",
  "plugin_type": "native_ai",
  "action_id": "plugin:my_tool",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "new_layer",
  "post_commands": [],
  "ai": {
    "task": "segmentation_mask",
    "input_size": 1024,
    "normalize": "image_net",
    "output_type": "grayscale_layer"
  },
  "models": [
    {
      "name": "model.onnx",
      "required": true,
      "size_mb": 99,
      "url": "https://huggingface.co/...",
      "sha256": "afb6a5c28f3b6bf1618c6e43f02073ef9dfdc70e937502d51603e57b0a1df10c"
    }
  ]
}
```

### Вариант Б: Алгоритмический плагин (`native_algorithm`)
```json
{
  "id": "ai.photocraft.plugins.vector_tracer",
  "name": "Vector Tracer",
  "name_ru": "Векторизация",
  "version": "1.0.0",
  "author": "kobaltGIT",
  "description": "Алгоритмическая векторизация растра",
  "description_ru": "Преобразование в вектор без ИИ",
  "plugin_type": "native_algorithm",
  "action_id": "plugin:vector_tracer",
  "icon": "icon.svg",
  "input": "active_layer",
  "output_mode": "in_place",
  "algorithm": {
    "task": "vector_trace"
  }
}
```

### Вариант В: Макрос / No-Code плагин (`no_code`)
```json
{
  "id": "ai.photocraft.plugins.quick_flatten",
  "name": "Quick Flatten",
  "name_ru": "Быстрое сведение",
  "version": "1.0.0",
  "author": "kobaltGIT",
  "description": "Цепочка команд PhotoCraft без ИИ",
  "description_ru": "Объединяет слои и подгоняет холст",
  "plugin_type": "no_code",
  "action_id": "plugin:quick_flatten",
  "icon": "icon.svg",
  "commands": [
    "image.revealAll",
    "layer.mergeDown"
  ]
}
```

### Вариант Г: Интерактивный UI-виджет / Панель (`interactive_ui`)
Манифест `plugin.json` задает декларативную верстку окна (`WidgetNode`), подписки на события IPC и имя скрипта:
```json
{
  "id": "ai.photocraft.plugins.color_helper",
  "name": "Color Helper",
  "name_ru": "Цветовой помощник",
  "version": "1.0.0",
  "author": "kobaltGIT",
  "description": "Интерактивная панель работы с цветом",
  "description_ru": "Палитра и диагностика контраста",
  "plugin_type": "interactive_ui",
  "action_id": "plugin:color_helper",
  "icon": "icon.svg",
  "interactive_ui": {
    "title": "Color Helper",
    "title_ru": "Цветовой помощник",
    "width": 320.0,
    "height": 420.0,
    "resizable": false,
    "script": "logic.rhai",
    "ipc_subscriptions": [
      "session.colors"
    ],
    "root": {
      "type": "row",
      "children": [
        {
          "type": "color_slot",
          "id": "slot_fg",
          "label": "FG Color",
          "label_ru": "Основной цвет",
          "color_key": "color_fg",
          "action": "open_fg"
        }
      ]
    }
  }
}
```
Рядом создается скрипт `plugins/<id_плагина>/logic.rhai` с хуками `on_state_changed(id, action, state)` и `on_session_colors(fg_hex, bg_hex, state)`.

## Шаг 3. Векторная иконка и веса модели
1. Создайте адаптивную векторную иконку `icon.svg` (`viewBox="0 0 24 24"`, `stroke="currentColor"`).
2. Веса модели `.onnx` никогда не коммитятся в Git (строго в `.gitignore`). Проверьте доступность ссылки `download_url`.

## Шаг 4. Обязательное создание `README.md` плагина (Mandatory)
Каждый плагин **обязан** содержать `plugins/<id_плагина>/README.md` со следующей структурой:

```markdown
# <Название Плагина>

[![Plugin: <Название>](https://img.shields.io/badge/Plugin-<Название>-blue.svg)](plugin.json)
[![Model: <Модель>](https://img.shields.io/badge/Model-<Модель>-purple.svg)](<Ссылка на модель>)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
<Краткое и точное описание плагина, что он делает и на базе какой модели/алгоритма работает.>

### Use Cases:
- **<Сценарий 1>:** <Пошаговое применение в PhotoCraft, слои, эффекты, назначение.>
- **<Сценарий 2>:** <Применение в дизайне/ретуши.>

---

**Русский:**
<Русскоязычное подробное описание назначения плагина и алгоритма.>

### Практическое применение:
- **<Сценарий 1>:** <Пошаговое описание использования в PhotoCraft, как комбинировать с инструментами редактора.>
- **<Сценарий 2>:** <Второй пример практической пользы.>
```

## Шаг 5. Локальная валидация и упаковка
```powershell
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
```

## Шаг 6. Завершение и коммит
Закоммитьте манифест `plugin.json`, иконку `icon.svg`, документацию `README.md` и ОБНОВЛЕННЫЕ каталоги `README.md` / `README.ru.md` вместе, чтобы избежать конфликтов с GitHub Actions.
Код ядра приложения при этом **НЕ ТРОГАЕТСЯ**.
