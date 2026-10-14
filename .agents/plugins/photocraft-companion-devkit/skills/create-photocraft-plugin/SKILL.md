---
name: create-photocraft-plugin
description: Пошаговое создание нового автономного плагина для PhotoCraft Companion без модификации и пересборки ядра Rust (Zero-Core Hardcoding).
---

# Создание нового плагина для PhotoCraft Companion

Используйте этот сценарий при добавлении нового плагина или инструмента для PhotoCraft.

## Шаг 1. Создание папки плагина
Создайте директорию в `plugins/<id_плагина>/`.

## Шаг 2. Создание манифеста `plugin.json`
Создайте `plugins/<id_плагина>/plugin.json`:
```json
{
  "id": "ai.photocraft.plugins.my_tool",
  "name": "My Tool Name",
  "version": "1.0.0",
  "author": "kobaltGIT",
  "description": "Описание назначения плагина",
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
*(Для плагинов без нейросетей укажите `plugin_type: "native_algorithm"` и секцию `algorithm: { "task": "vector_trace", ... }`)*

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
